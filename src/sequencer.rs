use super::{ Melody };
use undosa::waves::{WaveGenerator, sawtooth::SawtoothWaveGenerator};
use undosa::mixer::Mixer;
pub struct Sequencer<'a, T> where T: WaveGenerator {
    gen: Option<T>,
    melody: Melody<'a>
}

impl <'a> Sequencer <'a, SawtoothWaveGenerator> {
    pub fn new(melody: Melody<'a>) -> Self {
        Sequencer {
            gen: None,
            melody
        }
    }

    pub fn update(&mut self) {
        // TODO
    }

    pub fn next(&mut self) -> i16 {
        let pitch_update = match self.melody.next_sample() {
            (true, Some(pitch)) => {
                let pitchf32: f32 = pitch.into();
                let generator = SawtoothWaveGenerator::new(
                    48000,
                    (pitchf32) as usize
                );

                Some(Some(generator))
            },
            (true, None) => Some(None),
            _ => None
        };

        if let Some(generator) = pitch_update { self.gen = generator };

        let next_sample_raw = match &mut self.gen {
            Some(generator) => generator.next().unwrap(),
            None => 0
        };

        Mixer::new()
            .add(next_sample_raw, 255)
            .finish(64)
    }
}
