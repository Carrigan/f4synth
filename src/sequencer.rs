use undosa::{melody::Melody, mixer::Mixer, waves::{sawtooth::SawtoothWaveGenerator, square::SquareWaveGenerator}};

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

    fn next(&mut self) -> Option<i16> {
        match self {
            Generator::Square(gen) => gen.next(),
            Generator::Sawtooth(gen) => gen.next()
        }
    }
}

pub enum GeneratorType {
    Square,
    Sawtooth
}

pub struct Sequencer<'a> {
    gen_type: GeneratorType,
    gen: Option<Generator>,
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
        let pitch_update = match self.melody.next_sample() {
            (true, Some(pitch)) => {
                let pitchf32: f32 = pitch.into();
                Some(Some(Generator::build(&self.gen_type, pitchf32)))
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
