#![no_main]
#![no_std]

#[cfg(debug_assertions)]
extern crate panic_semihosting;

use cortex_m_rt::entry;

use stm32f4xx_hal::stm32::{Peripherals, RCC};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::{ i2c };
use stm32f4xx_hal::time::Hertz;

mod i2s;

#[entry]
fn main() -> ! {
    //  Get peripherals (clocks, flash memory, GPIO) for the STM32 Blue Pill microcontroller.
    let periph = Peripherals::take().unwrap();
    let _cortex_p = cortex_m::Peripherals::take().unwrap();

    let rcc = periph.RCC;

    // Enable I2S3
    rcc.apb1enr.modify(|_, w| w.spi3en().set_bit());
    
    let clocks = rcc.constrain().cfgr
        .use_hse(8.mhz())
        .sysclk(168.mhz())
        .freeze();

    // Setup i2s clock
    unsafe { &*RCC::ptr() }.plli2scfgr.write(|w| unsafe {
        w
            .plli2sn().bits(258)
            .plli2sr().bits(3)
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

    let mut i2s_periph = 
        i2s::I2s { spi: periph.SPI3, pins: (ck, ws, sd, i2s::NoSdExt) }
        .init(48000.hz(), 48000.hz());

    // Turn on the chip
    let _ = reset_line.set_high();

    // Test read: make sure chip is working
    let cs43l22_id = match config_i2c.write_read(address, &[0x01], &mut i2c_read_buffer[0..1]) {
        Ok(()) => Some(i2c_read_buffer[0]),
        _ => None
    };

    assert!(cs43l22_id == Some(0xE3));

    // Commence the boot sequence
    config_i2c.write(address, &[0x00, 0x99]).unwrap();
    config_i2c.write(address, &[0x47, 0x80]).unwrap();
    config_i2c.write(address, &[0x32, 0x80]).unwrap();
    config_i2c.write(address, &[0x32, 0x00]).unwrap();
    config_i2c.write(address, &[0x00, 0x00]).unwrap();

    // Set to headphones
    config_i2c.write(address, &[0x04, 0xAF]).unwrap();

    // BASS
    config_i2c.write(address, &[0x1F, 0xF0]);

    // Set volume max
    config_i2c.write(address, &[0x20, 0x90]).unwrap();
    config_i2c.write(address, &[0x21, 0x90]).unwrap();
    // config_i2c.write(address, &[0x22, 0]).unwrap();
    // config_i2c.write(address, &[0x23, 0]).unwrap();

    // Set the power control high
    config_i2c.write(address, &[0x02, 0x9E]).unwrap();

    let mut rando = periph.RNG.constrain(clocks);

    loop { 
        let mut values: [u8; 2] = [0; 2];
        let rand_val = rando.read(&mut values);
        let s: u16 = ((values[0] as u16) << 8) + values[1] as u16;
        i2s_periph.try_write(&[s], &[s]);
    }

    // let mut s = 0;
    // loop {
    //     if s >= 65000 { s = 0 };
    //     s += 500;
    //     i2s_periph.try_write(&[s], &[s]);
    // }
}
