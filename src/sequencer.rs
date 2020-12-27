use core::iter::Take;

use undosa::{melody::Melody, mixer::Mixer, pitch::Pitch, waves::{WaveGenerator, sawtooth::SawtoothWaveGenerator, square::SquareWaveGenerator}};

enum Generator {
    Square(SquareWaveGenerator),
    Sawtooth(SawtoothWaveGenerator)
}

impl Generator {
    fn build(gen_type: &GeneratorType, pitch: f32) -> Self {
        match gen_type {
            GeneratorType::Square => {
                Generator::Square(SquareWaveGenerator::new(
                    48000,
                    pitch as usize
                ))
            },
            GeneratorType::Sawtooth => {
                Generator::Sawtooth(SawtoothWaveGenerator::new(
                    48000,
                    pitch as usize
                ))
            }
        }

    }
}

impl Iterator for Generator {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Generator::Square(gen) => gen.next(),
            Generator::Sawtooth(gen) => gen.next()
        }
    }
}

impl ExactSizeIterator for Generator { }
impl WaveGenerator for Generator { }

pub enum GeneratorType {
    Square,
    Sawtooth
}

pub struct Sequencer<'a> {
    gen_type: GeneratorType,
    gen: Option<Take<Generator>>,
    melody: Melody<'a>
}

impl <'a> Sequencer <'a> {
    pub fn new(melody: Melody<'a>, gen_type: GeneratorType) -> Self {
        Sequencer {
            gen_type,
            gen: None,
            melody
        }
    }

    pub fn next(&mut self) -> i16 {
        let new_generator = match self.melody.next_note() {
            Some(note) => {
                let pitch_option: Option<Pitch> = note.into();

                match pitch_option {
                    Some(pitch) => {
                        let pitchf32: f32 = pitch.into();
                        let generator = Generator::build(&self.gen_type, pitchf32);
                        let quantized_generator = undosa::quantize::quantize(generator, self.melody.tempo, note.duration(), 127);

                        Some(quantized_generator)
                    }
                    None => None
                }
            },
            _ => None
        };

        if let Some(generator) = new_generator { self.gen = Some(generator) };

        let next_sample_raw = match &mut self.gen {
            Some(generator) => match generator.next() {
                Some(sample) => sample,
                None => 0
            }
            None => 0
        };

        Mixer::new()
            .add(next_sample_raw, 255)
            .finish(64)
    }
}
