use super::{ Melody };
use super::waves::{ WaveGenerable, WaveGenerator, SquareWaveGenerator };

pub struct Sequencer<'a> {
    gen: WaveGenerable<'a>,
    melody: Melody<'a>
}

impl <'a> Sequencer <'a> {
    pub fn new(melody: Melody<'a>) -> Self {
        Sequencer {
            gen: WaveGenerable::Square(SquareWaveGenerator::new(48000, 440)),
            melody
        }
    }

    pub fn update(&mut self) {
        // TODO
    }

    pub fn next(&mut self) -> u16 {
        let next_gen = match self.melody.next_sample() {
            (true, Some(pitch)) => {
                let pitchf32: f32 = pitch.into();
                let square_gen = SquareWaveGenerator::new(48000, (pitchf32) as usize);

                Some(WaveGenerable::Square(square_gen))
            },
            (true, None) => Some(WaveGenerable::Silence),
            _ => None
        };

        if let Some(generator) = next_gen { self.gen = generator };
        self.gen.next()
    }
}
