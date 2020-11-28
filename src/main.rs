#![no_main]
#![no_std]

#[cfg(debug_assertions)]
extern crate panic_semihosting;

use cortex_m_rt::entry;

use stm32f4xx_hal::stm32::{Peripherals, RCC};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::{ i2c, dma::{Transfer, config::DmaConfig} };
use stm32f4xx_hal::time::Hertz;

mod i2s;
mod waves;
mod melody;

use melody::{Melody, Note, Pitch};
use waves::{ SquareWaveGenerator, SawtoothWaveGenerator, WaveGenerator };

enum WaveGenerable <'a> {
    Square(SquareWaveGenerator),
    Sawtooth(SawtoothWaveGenerator),
    Noise(&'a mut HardwareWhiteNoiseGenerator),
    Silence
}

impl <'a> WaveGenerator for WaveGenerable<'a> {
    fn next(&mut self) -> u16 {
        match self {
            WaveGenerable::Square(square) => square.next(),
            WaveGenerable::Sawtooth(sawtooth) => sawtooth.next(),
            WaveGenerable::Noise(noise) => noise.next(),
            WaveGenerable::Silence => (core::u16::MAX / 2)
        }
    }
}

struct HardwareWhiteNoiseGenerator {
    random_generator: stm32f4xx_hal::rng::Rng
}

impl WaveGenerator for HardwareWhiteNoiseGenerator {
    fn next(&mut self) -> u16 {
        let mut values: [u8; 2] = [0; 2];
        let _ = self.random_generator.read(&mut values);
        ((values[0] as u16) << 8) + values[1] as u16
    }
}

static mut DMA_BUFFER: [u16; 2000] = [0; 2000];

#[entry]
fn main() -> ! {
    //  Get peripherals (clocks, flash memory, GPIO) for the STM32 Blue Pill microcontroller.
    let periph = Peripherals::take().unwrap();
    let _cortex_p = cortex_m::Peripherals::take().unwrap();

    let rcc = periph.RCC;

    // Enable I2S3
    rcc.apb1enr.modify(|_, w| w.spi3en().set_bit());

    // Enable both DMAs
    rcc.ahb1enr.modify(|_, w| w.dma1en().set_bit());
    
    let clocks = rcc.constrain().cfgr
        .use_hse(8.mhz())
        .sysclk(168.mhz())
        .freeze();

    // Setup i2s clock
    // 2MHZ Vco
    // * 258 / 6 = 86 MHZ
    unsafe { &*RCC::ptr() }.plli2scfgr.write(|w| unsafe {
        w
            .plli2sn().bits(258)
            .plli2sr().bits(6)
    });

    unsafe { &*RCC::ptr() }.cr.write(|w| w.plli2son().set_bit());

    let gpioa = periph.GPIOA.split();
    let gpiob = periph.GPIOB.split();
    let gpioc = periph.GPIOC.split();
    let gpiod = periph.GPIOD.split();

    // Definitions for pins connected to the CS43L22
    let mut reset_line = gpiod.pd4.into_push_pull_output();
    let config_i2c_sda = gpiob.pb9.into_alternate_af4().set_open_drain();
    let config_i2c_scl = gpiob.pb6.into_alternate_af4().set_open_drain();

    // Set up our I2C config lines
    let mut config_i2c = i2c::I2c::i2c1(
        periph.I2C1, 
        (config_i2c_scl, config_i2c_sda), 
        400.khz(), 
        clocks
    );

    // Set up the I2C address and read buffer
    let address = 0x4A;
    let mut i2c_read_buffer = [0];

    // Set up I2S3.
    let _mck = gpioc.pc7.into_alternate_af6();
    let ck = gpioc.pc10.into_alternate_af6();
    let ws = gpioa.pa4.into_alternate_af6();
    let sd = gpioc.pc12.into_alternate_af6();

    let _i2s_periph = 
        i2s::I2s { spi: periph.SPI3, pins: (ck, ws, sd, i2s::NoSdExt) }
        .init(48000.hz(), 48000.hz());

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
    );

    // Turn on the chip
    let _ = reset_line.set_high();

    // Test read: make sure chip is working
    let cs43l22_id = match config_i2c.write_read(address, &[0x01], &mut i2c_read_buffer[0..1]) {
        Ok(()) => Some(i2c_read_buffer[0]),
        _ => None
    };

    assert!(cs43l22_id == Some(0xE3));

    config_i2c.write(address, &[0x02, 0x01]).unwrap();

    // Commence the boot sequence
    config_i2c.write(address, &[0x00, 0x99]).unwrap();
    config_i2c.write(address, &[0x47, 0x80]).unwrap();
    config_i2c.write(address, &[0x32, 0x80]).unwrap();
    config_i2c.write(address, &[0x32, 0x00]).unwrap();
    config_i2c.write(address, &[0x00, 0x00]).unwrap();

    // Set to headphones
    config_i2c.write(address, &[0x04, 0xAF]).unwrap();

    // Set the power control high
    config_i2c.write(address, &[0x02, 0x9E]).unwrap();

    // Set volume
    config_i2c.write(address, &[0x20, 0x90]).unwrap();
    config_i2c.write(address, &[0x21, 0x90]).unwrap();
    config_i2c.write(address, &[0x1a, 0]).unwrap();
    config_i2c.write(address, &[0x1b, 0]).unwrap();

    //config_i2c.write(address, &[0x05, 0x20]).unwrap();

    let _rng = periph.RNG.constrain(clocks);
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

    loop {
        // Fill the buffer
        for index in 0..1000 {
            wave_generator = match melody.next_sample() {
                (true, Some(pitch)) => {
                    let pitchf32: f32 = pitch.into();
                    let square_gen = SquareWaveGenerator::new(48000, (pitchf32) as usize);
                    
                    WaveGenerable::Square(square_gen)
                },
                (true, None) => WaveGenerable::Silence,
                _ => wave_generator
            };
    
            let sample = wave_generator.next();

            let shifted_index = index * 2;
            unsafe { 
                DMA_BUFFER[shifted_index] = sample; 
                DMA_BUFFER[shifted_index + 1] = sample;
            };
            
        }

        // Start the DMA
        spi_stream.ndtr.write(|w| unsafe { w.bits(2000) });
        spi_stream.par.write(|w| w.pa().bits(0x40003C00 + 0xC));
        spi_stream.m0ar.write(|w| unsafe { w.m0a().bits(DMA_BUFFER.as_ptr() as usize as u32) });        
        spi_stream.cr.modify(|r, w| w.en().enabled());

        // Wait for it to complete
        while dma1.hisr.read().tcif5().bit_is_clear() {}

        // Clear the flag
        dma1.hifcr.write(|w| w.ctcif5().set_bit());
    }
}
