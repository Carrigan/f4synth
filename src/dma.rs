const BUFFER_SIZE: usize = 64;
static mut DMA_BUFFER: [u16; BUFFER_SIZE] = [0; BUFFER_SIZE];

enum BufferHalf {
    FirstHalf,
    SecondHalf
}

pub struct DmaStream {
    dma: stm32f4xx_hal::stm32::DMA1,
    ping_pong: BufferHalf
}

impl DmaStream {
    pub fn new(dma: stm32f4xx_hal::stm32::DMA1) -> Self {
        let stream = &dma.st[5];

        stream.cr.write(|w| 
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

        DmaStream { dma, ping_pong: BufferHalf::FirstHalf }
    }

    pub fn begin<T: FnMut() -> u16>(&mut self, generator: &mut T) {
        self.fill_samples(generator);

        let stream = &self.dma.st[5];
        stream.ndtr.write(|w| unsafe { w.bits(BUFFER_SIZE as u32) });
        stream.par.write(|w| w.pa().bits(0x40003C00 + 0xC));
        stream.m0ar.write(|w| unsafe { w.m0a().bits(DMA_BUFFER.as_ptr() as usize as u32) });        
        stream.cr.modify(|_r, w| w.en().enabled());
    }

    pub fn block_and_fill<T: FnMut() -> u16>(&mut self, generator: &mut T) {      
        // Advance the ping pong
        self.ping_pong = match &self.ping_pong {
            BufferHalf::FirstHalf => BufferHalf::SecondHalf,
            BufferHalf::SecondHalf => BufferHalf::FirstHalf
        };

        // Fill that half while the DMA is churning
        self.fill_samples(generator);

        // Wait for the interrupt and clear it
        match &self.ping_pong {
            BufferHalf::SecondHalf => {
                while self.dma.hisr.read().htif5().bit_is_clear() {}
                self.dma.hifcr.write(|w| w.chtif5().set_bit());
            }

            BufferHalf::FirstHalf => {
                while self.dma.hisr.read().tcif5().bit_is_clear() {}
                self.dma.hifcr.write(|w| w.ctcif5().set_bit());
            }
        };
    }

    fn fill_samples<T: FnMut() -> u16>(&mut self, generator: &mut T) {
        for index in 0..(BUFFER_SIZE / 4) {
            let sample = generator();

            let shifted_index = match self.ping_pong {
                BufferHalf::FirstHalf => index * 2,
                BufferHalf::SecondHalf => index * 2 + (BUFFER_SIZE / 2)
            };

            unsafe { 
                DMA_BUFFER[shifted_index] = sample; 
                DMA_BUFFER[shifted_index + 1] = sample;
            };   
        }
    }
}
