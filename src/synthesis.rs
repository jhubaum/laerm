use rand::Rng;

// Convert frequency (Hz) to angular velocity
fn w(freq : f32) -> f32 {
    return freq * 2.0 * std::f32::consts::PI
}

pub enum Oscillator {
    Sine {freq: f32 },
    Square {freq: f32 },
    Noise,
}

impl Oscillator {
    pub fn evaluate(self, time: f32) -> f32 {
        match self {
            Oscillator::Sine{freq} => (time * w(freq)).sin(),
            Oscillator::Square{freq} => {
                if (time * w(freq)).sin() > 0.0 {1.0} else { -1.0 }
            },
            Oscillator::Noise => rand::thread_rng().gen_range(-1.0..1.0),
        }
    }
}
