#![no_main]
#![no_std]

#[cfg(debug_assertions)]
extern crate panic_semihosting;

use cortex_m_rt::entry;

use stm32f4xx_hal::stm32::Peripherals;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::i2c;

mod waves;
mod melody;
mod i2s;
mod cs43l22;

use melody::{Melody, Note, Pitch};
use waves::{ SquareWaveGenerator, WaveGenerable, WaveGenerator };

const BUFFER_SIZE: usize = 64;
static mut DMA_BUFFER: [u16; BUFFER_SIZE] = [0; BUFFER_SIZE];

enum BufferHalf {
    FirstHalf,
    SecondHalf
}

fn generate_samples<'a>(buffer_half: BufferHalf, count: usize, wave_generator: WaveGenerable<'a>, melody: &mut Melody) -> WaveGenerable<'a> {
    let mut generable= wave_generator;
    
    for index in 0..count {
        generable = match melody.next_sample() {
            (true, Some(pitch)) => {
                let pitchf32: f32 = pitch.into();
                let square_gen = SquareWaveGenerator::new(48000, (pitchf32) as usize);
                
                WaveGenerable::Square(square_gen)
            },
            (true, None) => WaveGenerable::Silence,
            _ => generable
        };

        let sample = generable.next();

        let shifted_index = match buffer_half {
            BufferHalf::FirstHalf => index * 2,
            BufferHalf::SecondHalf => index * 2 + (BUFFER_SIZE / 2)
        };

        unsafe { 
            DMA_BUFFER[shifted_index] = sample; 
            DMA_BUFFER[shifted_index + 1] = sample;
        };   
    }

    generable
}


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

    // Set up the DMA
    let dma1 = periph.DMA1;
    let spi_stream = &dma1.st[5];

    spi_stream.cr.write(|w| 
        w
            .chsel().bits(0)
            .psize().bits16()
            .msize().bits16()
            .minc().incremented()
            .pinc().fixed()
            .dir().memory_to_peripheral()
            .pfctrl().dma()
            .pl().high()
            .circ().enabled()
    );

    let mut wave_generator = WaveGenerable::Square(SquareWaveGenerator::new(48000, 440));

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

    let mut melody = Melody::new(&notes, 194);

    // Fill the buffer initially and start the DMA
    wave_generator = generate_samples(BufferHalf::FirstHalf, BUFFER_SIZE / 4, wave_generator, &mut melody);

    spi_stream.ndtr.write(|w| unsafe { w.bits(64) });
    spi_stream.par.write(|w| w.pa().bits(0x40003C00 + 0xC));
    spi_stream.m0ar.write(|w| unsafe { w.m0a().bits(DMA_BUFFER.as_ptr() as usize as u32) });        
    spi_stream.cr.modify(|_r, w| w.en().enabled());

    loop {
        // Fill the second half while the first half runs
        wave_generator = generate_samples(BufferHalf::SecondHalf, BUFFER_SIZE / 4, wave_generator, &mut melody);

        // Wait for it to half-complete
        while dma1.hisr.read().htif5().bit_is_clear() {}

        // Clear the half-flag
        dma1.hifcr.write(|w| w.chtif5().set_bit());

        // Fill the first half while the second half runs
        wave_generator = generate_samples(BufferHalf::FirstHalf, BUFFER_SIZE / 4, wave_generator, &mut melody);

        // Wait for full-complete
        while dma1.hisr.read().tcif5().bit_is_clear() {}

        // Clear the flag
        dma1.hifcr.write(|w| w.ctcif5().set_bit());
    }
}
