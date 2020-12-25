use stm32f4xx_hal::stm32::RCC;

pub struct I2S where {
    _spi: stm32f4xx_hal::stm32::SPI3
}

impl I2S {
    // Setup clock to 86MHZ (2MHZ * 258 / 6)
    pub fn setup_clocks() {
        unsafe { &*RCC::ptr() }.plli2scfgr.write(|w| unsafe {
            w
                .plli2sn().bits(258)
                .plli2sr().bits(6)
        });

        unsafe { &*RCC::ptr() }.cr.write(|w| w.plli2son().set_bit());
    }

    pub fn init(spi: stm32f4xx_hal::stm32::SPI3) -> Self {
        spi.cr2.write(|w|
            w
                .ssoe().clear_bit()
                .txdmaen().enabled()
        );

        spi.i2spr.write(|w| unsafe {
            w
                .i2sdiv().bits(3)
                .odd().odd()
                .mckoe().enabled()
        });

        spi.i2scfgr.write(|w| {
            w
                .i2smod().i2smode()
                .i2sstd().philips()
                .datlen().sixteen_bit()
                .chlen().sixteen_bit()
                .ckpol().idle_high()
                .i2scfg().master_tx()
                .i2se().enabled()
        });

        I2S { _spi: spi }
    }
}
