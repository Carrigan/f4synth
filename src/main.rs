#![no_main]
#![cfg_attr(not(test), no_std)]

#[cfg(debug_assertions)]
extern crate panic_semihosting;

use cortex_m_rt::entry;

use sequencer::Sequencer;
use stm32f4xx_hal::stm32::Peripherals;
use stm32f4xx_hal::prelude::*;

mod i2s;
mod dma;
mod sequencer;
use undosa::{
    melody::{ Melody, Note },
    pitch::{ Pitch }
};

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
    let _clocks = rcc.constrain().cfgr
        .use_hse(8.mhz())
        .sysclk(168.mhz())
        .freeze();

    i2s::I2S::setup_clocks();

    // Split the GPIOs we will be using
    let gpioa = periph.GPIOA.split();
    let gpioc = periph.GPIOC.split();

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
        Note::Eighth(Pitch::A3),
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
        Note::Eighth(Pitch::A3),

        Note::Eighth(Pitch::E4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::G4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::Gb4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::G4),
        Note::Eighth(Pitch::B4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::E4),
        Note::Eighth(Pitch::G4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::Gb4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::G4),
        Note::Eighth(Pitch::E3),

        Note::Eighth(Pitch::E4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::G4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::Gb4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::G4),
        Note::Eighth(Pitch::B4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::E4),
        Note::Eighth(Pitch::G4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::Gb4),
        Note::Eighth(Pitch::E3),
        Note::Eighth(Pitch::G4),
        Note::Eighth(Pitch::E3),
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
