use rand::Rng;
use std::default::Default;

use std::collections::HashMap;

// Convert frequency (Hz) to angular velocity
fn w(freq : f32) -> f32 {
    return freq * 2.0 * std::f32::consts::PI
}

pub trait Envelope {
    fn amplitude(&self, time: f32) -> f32;
    fn activate(&mut self);
    fn deactivate(&mut self);

    fn is_active(&self) -> bool;
}

pub trait WaveGenerator {
    fn evaluate(&self, time: f32, freq: f32) -> f32;
}

#[derive(Default)]
pub struct Instrument<T: Envelope+Default, U: WaveGenerator+Default> {
    wave_generator: U,
    notes: HashMap<i8, T>
}


fn note_to_frequency(note: &i8) -> f32 {
    110.0
}

impl<T: Envelope+Default,  U: WaveGenerator+Default> Instrument<T, U> {
    pub fn evaluate(&self, time: f32) -> f32 {
        let mut res = 0.0;
        for (note, envelope) in self.notes.iter() {
            res += envelope.amplitude(time) *
                self.wave_generator.evaluate(time, note_to_frequency(note));
        }
        res
    }

    pub fn finished_notes(&self) -> Vec<i8> {
        let mut res = Vec::new();
        for (note, envelope) in self.notes.iter() {
            if !envelope.is_active() {
                res.push(note.clone());
            }
        }
        res
    }

    pub fn remove_finished(&mut self, notes: Vec<i8>) {
        for note in notes.iter() {
            self.notes.remove(note);
        }
    }

    pub fn start_note(&mut self, note: i8) {
        if self.notes.contains_key(&note) {
            panic!("Started to play note that's already played");
        }
        let mut envelope = T::default();
        envelope.activate();
        self.notes.insert(note, envelope);
    }

    pub fn end_note(&mut self, note: i8) {
        if let Some(env) = self.notes.get_mut(&note) {
            env.deactivate()
        }
    }
}

#[derive(Default)]
pub struct DummyEnvelope {
    activated: bool
}

impl Envelope for DummyEnvelope {
    fn amplitude(&self, _: f32) -> f32 {
        if self.activated  { 1.0 } else { 0.0 }
    }

    fn activate(&mut self) {
        self.activated = true;
    }

    fn deactivate(&mut self) {
        self.activated = false;
    }

    fn is_active(&self) -> bool {
        self.activated
    }
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
