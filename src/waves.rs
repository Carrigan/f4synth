
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
