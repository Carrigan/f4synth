// Based on: https://github.com/maxekman/stm32f4xx-hal/blob/i2s_v0.2.x/src/i2s.rs

use stm32f4xx_hal::stm32::{ spi1, SPI3 };

use core::ops::Deref;
use core::ptr;

use super::Hertz;

use stm32f4xx_hal::gpio::{
    Alternate,
    gpioc::{PC10, PC12},
    gpioa::PA4,
    AF6
};

pub trait Pins<I2S> {}
pub trait PinCk<I2S> {}
pub trait PinWs<I2S> {}
pub trait PinSd<I2S> {}
pub trait PinSdExt<I2S> {}

impl<I2S, CK, WS, SD, SDEXT> Pins<I2S> for (CK, WS, SD, SDEXT)
where
    CK: PinCk<I2S>,
    WS: PinWs<I2S>,
    SD: PinSd<I2S>,
    SDEXT: PinSdExt<I2S>,
{
}

impl PinCk<SPI3> for PC10<Alternate<AF6>> {}
impl PinWs<SPI3> for PA4<Alternate<AF6>> {}
impl PinSd<SPI3> for PC12<Alternate<AF6>> {}
impl PinSdExt<SPI3> for NoSdExt {}

#[derive(Debug)]
pub struct I2s<SPI, PINS> {
    pub spi: SPI,
    pub pins: PINS,
}

pub struct NoSdExt;


impl<SPI, PINS> I2s<SPI, PINS>
where
    SPI: Deref<Target = spi1::RegisterBlock>,
{
    pub fn init(self, _freq: Hertz, _clock: Hertz) -> Self {
        // disable SS output
        self.spi.cr2.write(|w| w.ssoe().clear_bit());

        // TODO: Calculate baud rate.
        // let br = match clock.0 / freq.0 {
        //     0 => unreachable!(),
        //     1..=2 => 0b000,
        //     3..=5 => 0b001,
        //     6..=11 => 0b010,
        //     12..=23 => 0b011,
        //     24..=47 => 0b100,
        //     48..=95 => 0b101,
        //     96..=191 => 0b110,
        //     _ => 0b111,
        // };
        let br: u8 = 0;

        // Configure clock polarity.
        // self.spi.i2scfgr.write(|w| w.ckpol().idle_high());

        // Configure the I2S precsaler and enable MCKL output.
        // NOTE: Hardcoded for 48KHz audio sampling rate with PLLI2S at 86MHz.
        // I2S uses DIV=3, ODD=true to achive a 12.285714 MHz MCKL.
        // FS = I2SxCLK / [(16*2)*(((2*I2SDIV)+ODD)*8)] when the channel frame is 16-bit wide.
        // FS = 86MHz (from PLLI2S above) / [(16*2)*(((2*3)+1)*8)] = 48KHz
        // NOTE: Unsafe because the division can be set incorrectly.
        self.spi
            .i2spr
            .write(|w| unsafe { w.i2sdiv().bits(13).odd().odd().mckoe().enabled() });

        // Configure I2S.
        // TODO: Configurable I2S standard and data length from user input.
        self.spi.i2scfgr.write(|w| {
            w
                // SPI/I2S Mode.
                .i2smod()
                .i2smode()
                // I2S standard.
                .i2sstd()
                .philips()
                // Data and channel length.
                .datlen()
                .sixteen_bit()
                .chlen()
                .sixteen_bit()
                // Clock steady state polarity.
                .ckpol()
                .idle_high()
                // Master TX mode and enable.
                .i2scfg()
                .master_tx()
                .i2se()
                .enabled()
        });

        self
    }

    // TODO: Configure interrupts for I2S.
    // /// Enable interrupts for the given `event`:
    // ///  - Received data ready to be read (RXNE)
    // ///  - Transmit data register empty (TXE)
    // ///  - Transfer error
    // pub fn listen(&mut self, event: Event) {
    //     match event {
    //         Event::Rxne => self.spi.cr2.modify(|_, w| w.rxneie().set_bit()),
    //         Event::Txe => self.spi.cr2.modify(|_, w| w.txeie().set_bit()),
    //         Event::Error => self.spi.cr2.modify(|_, w| w.errie().set_bit()),
    //     }
    // }

    // /// Disable interrupts for the given `event`:
    // ///  - Received data ready to be read (RXNE)
    // ///  - Transmit data register empty (TXE)
    // ///  - Transfer error
    // pub fn unlisten(&mut self, event: Event) {
    //     match event {
    //         Event::Rxne => self.spi.cr2.modify(|_, w| w.rxneie().clear_bit()),
    //         Event::Txe => self.spi.cr2.modify(|_, w| w.txeie().clear_bit()),
    //         Event::Error => self.spi.cr2.modify(|_, w| w.errie().clear_bit()),
    //     }
    // }

    /// Return `true` if the BSY flag is set, indicating that the I2S is busy
    /// communicating.
    pub fn is_bsy(&self) -> bool {
        self.spi.sr.read().bsy().bit_is_set()
    }

    /// Return `true` if the TXE flag is set, i.e. new data to transmit
    /// can be written to the SPI.
    pub fn is_txe(&self) -> bool {
        self.spi.sr.read().txe().bit_is_set()
    }

    /// Return `true` if the RXNE flag is set, i.e. new data has been received
    /// and can be read from the SPI.
    pub fn is_rxne(&self) -> bool {
        self.spi.sr.read().rxne().bit_is_set()
    }

    /// Return the value of the CHSIDE flag, i.e. which channel to transmit next.
    pub fn ch_side(&self) -> bool {
        self.spi.sr.read().chside().bit_is_set()
    }

    /// Return `true` if the UDR flag is set, i.e. no new data was available
    /// for transmission while in slave mode.
    pub fn is_udr(&self) -> bool {
        self.spi.sr.read().udr().bit_is_set()
    }

    /// Return `true` if the OVR flag is set, i.e. new data has been received
    /// while the receive data register was already filled.
    pub fn is_ovr(&self) -> bool {
        self.spi.sr.read().ovr().bit_is_set()
    }

    /// Return `true` if the FRE flag is set, i.e. there was an unexpected change
    /// in the WS line by the master while in slave mode.
    pub fn is_fre(&self) -> bool {
        self.spi.sr.read().fre().bit_is_set()
    }

    pub fn free(self) -> (SPI, PINS) {
        (self.spi, self.pins)
    }

    pub fn try_write<'w, W>(&mut self, left_words: &'w [W], right_words: &'w [W],) -> Result<(), u8> where W: Copy {
        // TODO: Check preconditions, return errors.
        for (lw, rw) in left_words.iter().zip(right_words.iter()) {
            while !self.is_txe() {} // Wait for TX enable after the previous word.
            unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut W, *lw) }
            while !self.is_txe() {} // Wait for TX enable after the left word.
            unsafe { ptr::write_volatile(&self.spi.dr as *const _ as *mut W, *rw) }
        }
        Ok(())
    }
}
