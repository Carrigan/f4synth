#![no_main]
#![no_std]

#[cfg(debug_assertions)]
extern crate panic_semihosting;

#[cfg(not(debug_assertions))]
extern crate panic_halt;

use cortex_m_rt::entry;


#[cfg(debug_assertions)]
use cortex_m_semihosting::hprintln;

use stm32f4xx_hal::stm32::{Peripherals};
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::dma::{Transfer, config::DmaConfig, MemoryToMemory, StreamsTuple};
use stm32f4xx_hal::dma::traits::{Direction};

static mut FROM: [u32; 10] = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
static mut TO: [u32; 10] = [0; 10];

#[entry]
fn main() -> ! {
    #[cfg(debug_assertions)]
    hprintln!("Hello, world!").unwrap();

    //  Get peripherals (clocks, flash memory, GPIO) for the STM32 Blue Pill microcontroller.
    let periph = Peripherals::take().unwrap();
    let cortex_p = cortex_m::Peripherals::take().unwrap();

    //  Get the clocks from the STM32 Reset and Clock Control (RCC) and freeze the Flash Access Control Register (ACR).
    periph.RCC.ahb1enr.write(|w| w.dma2en().enabled());
    
    let rcc = periph.RCC.constrain();
    let clocks = rcc.cfgr.sysclk(168.mhz()).freeze();
    let gpiod = periph.GPIOD.split();

    //  Use Pin PC 13 of the Blue Pill for GPIO Port C. Select Output Push/Pull mode for the pin, which is connected to our LED.
    let mut led = gpiod.pd13.into_push_pull_output();

    let mut delay = Delay::new(cortex_p.SYST, clocks);

    let dma_config = DmaConfig::default()
        .fifo_enable(true)
        .memory_increment(true)
        .peripheral_increment(true);

    let mem_to_mem: MemoryToMemory<u32> = MemoryToMemory::new();
    let streams = StreamsTuple::new(periph.DMA2);

    unsafe {
        let mut transfer = Transfer::init(
            streams.0, 
            mem_to_mem,
            &mut TO,
            Some(&mut FROM),
            dma_config
        );

        transfer.start(|_dma_periph| ());
    }

    loop {
        led.set_high().unwrap();

        //  Wait 1,000 millisec (1 sec).
        delay.delay_ms(1000_u16);

        //  Output 0V on the LED Pin and show a message in OpenOCD console.
        led.set_low().unwrap();

        //  Wait 1,000 millisec (1 sec).
        delay.delay_ms(1000_u16);
    }
}
