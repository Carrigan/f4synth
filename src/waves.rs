#![allow(dead_code)]

pub trait WaveGenerator {
    fn next(&mut self) -> u16;
}

pub struct SquareWaveGenerator {
    current_position: u16,
    advancement: u16
}

impl SquareWaveGenerator {
    pub fn new(sample_frequency: usize, frequency: usize) -> Self {
        Self {
            current_position: 0,
            advancement: ((frequency as u32 * core::u16::MAX as u32) / sample_frequency as u32) as u16
        }
    }
}

impl WaveGenerator for SquareWaveGenerator {
    fn next(&mut self) -> u16 {
        let mut next_position: u32 = self.current_position as u32 + self.advancement as u32;
        if next_position > core::u16::MAX as u32 { next_position -= core::u16::MAX as u32 }
        self.current_position = next_position as u16;

        if self.current_position < (core::u16::MAX / 2) { 0 } else { 65534 }
    }
}

pub struct SawtoothWaveGenerator {
    current_position: u16,
    advancement: u16
}

impl SawtoothWaveGenerator {
    pub fn new(sample_frequency: usize, frequency: usize) -> Self {
        Self {
            current_position: 0,
            advancement: ((frequency as u32 * core::u16::MAX as u32) / sample_frequency as u32) as u16
        }
    }
}

impl WaveGenerator for SawtoothWaveGenerator {
    fn next(&mut self) -> u16 {
        let mut next_position: u32 = self.current_position as u32 + self.advancement as u32;
        if next_position > core::u16::MAX as u32 { next_position -= core::u16::MAX as u32 }
        self.current_position = next_position as u16;

        self.current_position
    }
}

#[allow(dead_code)]
pub enum WaveGenerable <'a> {
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

use stm32f4xx_hal::prelude::*;
pub struct HardwareWhiteNoiseGenerator {
    random_generator: stm32f4xx_hal::rng::Rng
}

impl WaveGenerator for HardwareWhiteNoiseGenerator {
    fn next(&mut self) -> u16 {
        let mut values: [u8; 2] = [0; 2];
        let _ = self.random_generator.read(&mut values);
        ((values[0] as u16) << 8) + values[1] as u16
    }
}
