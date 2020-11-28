#![no_main]
#![no_std]

#[cfg(debug_assertions)]
extern crate panic_semihosting;

use cortex_m_rt::entry;

use sequencer::Sequencer;
use stm32f4xx_hal::stm32::Peripherals;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::i2c;

mod waves;
mod melody;
mod i2s;
mod cs43l22;
mod dma;
mod sequencer;

use melody::{ Melody, Note, Pitch };

#[entry]
fn main() -> ! {
    //  Get peripherals (clocks, flash memory, GPIO) for the STM32 Blue Pill microcontroller.
    let periph = Peripherals::take().unwrap();
    let _cortex_p = cortex_m::Peripherals::take().unwrap();

    let rcc = periph.RCC;

    // Enable I2S3 and the DMA
    rcc.apb1enr.modify(|_, w| w.spi3en().set_bit());
    rcc.ahb1enr.modify(|_, w| w.dma1en().set_bit());

    // Set up and freeze the clocks
    let clocks = rcc.constrain().cfgr
        .use_hse(8.mhz())
        .sysclk(168.mhz())
        .freeze();

    i2s::I2S::setup_clocks();

    // Split the GPIOs we will be using
    let gpioa = periph.GPIOA.split();
    let gpiob = periph.GPIOB.split();
    let gpioc = periph.GPIOC.split();
    let gpiod = periph.GPIOD.split();

    // Allocate pins for the CS43L22 and set it up
    let reset_line = gpiod.pd4.into_push_pull_output();
    let config_i2c_sda = gpiob.pb9.into_alternate_af4().set_open_drain();
    let config_i2c_scl = gpiob.pb6.into_alternate_af4().set_open_drain();

    let config_i2c = i2c::I2c::i2c1(
        periph.I2C1, 
        (config_i2c_scl, config_i2c_sda), 
        400.khz(), 
        clocks
    );

    let mut dac = cs43l22::CS43L22::new(reset_line, config_i2c);
    dac.initialize();

    // Set up I2S3.
    let _mck = gpioc.pc7.into_alternate_af6();
    let _ck = gpioc.pc10.into_alternate_af6();
    let _ws = gpioa.pa4.into_alternate_af6();
    let _sd = gpioc.pc12.into_alternate_af6();
    let _i2s = i2s::I2S::init(periph.SPI3);

    // Get a note sequence going
    let notes = [
        Note::Eighth(Pitch::A4),
        Note::Eighth(Pitch::A3),
        Note::Eighth(Pitch::C5),
        Note::Eighth(Pitch::A3),
        Note::Eighth(Pitch::B4),
        Note::Eighth(Pitch::A3),
        Note::Eighth(Pitch::C5),
        Note::Eighth(Pitch::E5),
        Note::Eighth(Pitch::A3),
        Note::Eighth(Pitch::A4),
        Note::Eighth(Pitch::C5),
        Note::Eighth(Pitch::A3),
        Note::Eighth(Pitch::B4),
        Note::Eighth(Pitch::A3),
        Note::Eighth(Pitch::C5),
        Note::Eighth(Pitch::A3)
    ];

    let melody = Melody::new(&notes, 210);
    let mut sequencer = Sequencer::new(melody);

    // Set up and start the DMA
    let mut stream = dma::DmaStream::new(periph.DMA1);
    stream.begin(&mut || sequencer.next());
    
    loop {
        // Update the sequencer here with input
        sequencer.update();
        
        // Fill the stream and block
        stream.block_and_fill(&mut || sequencer.next());
    }
}
