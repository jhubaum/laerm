use rand::Rng;
use std::collections::HashMap;
use std::time::Instant;

// Convert frequency (Hz) to angular velocity
fn w(freq: f32) -> f32 {
    return freq * 2.0 * std::f32::consts::PI;
}

pub enum Oscillator {
    Sine { freq: f32 },
    Square { freq: f32 },
    Noise,
}

impl Oscillator {
    pub fn evaluate(self, time: f32) -> f32 {
        match self {
            Oscillator::Sine { freq } => (time * w(freq)).sin(),
            Oscillator::Square { freq } => {
                if (time * w(freq)).sin() > 0.0 {
                    1.0
                } else {
                    -1.0
                }
            }
            Oscillator::Noise => rand::thread_rng().gen_range(-1.0..1.0),
        }
    }
}

pub trait Envelope {
    fn amplitude(&mut self, time: &Instant) -> f32;
    fn activate(&mut self, time: &Instant);
    fn deactivate(&mut self, time: &Instant);

    fn is_active(&self) -> bool;
    fn create_activated_copy(&self, time: &Instant) -> Self;
}

pub trait WaveGenerator {
    fn generate_wave(&self, time: f32, freq: f32) -> f32;
}

pub trait Instrument {
    fn evaluate(&mut self, time: f32) -> f32;
    fn finished_notes(&self) -> Vec<i8>;
    fn remove_finished(&mut self, notes: Vec<i8>);
    fn start_note(&mut self, note: i8);
    fn end_note(&mut self, note: i8);
}

pub struct InstrumentImpl<T: Envelope, TOsc: WaveGenerator> {
    wave_generator: TOsc,
    envelope: T,
    notes: HashMap<i8, T>,
}

fn note_to_frequency(note: i8) -> f32 {
    // start from a C3
    let base = 130.81;
    base * 2.0_f32.powf(1.0 / 12.0).powi(note.into())
}

impl<T: Envelope, TOsc: WaveGenerator> InstrumentImpl<T, TOsc> {
    pub fn create(env: T, gen: TOsc) -> Self {
        Self {
            wave_generator: gen,
            envelope: env,
            notes: HashMap::new(),
        }
    }
}

impl<T: Envelope, TOsc: WaveGenerator> Instrument for InstrumentImpl<T, TOsc> {
    fn evaluate(&mut self, time: f32) -> f32 {
        let mut res = 0.0;
        for (note, envelope) in self.notes.iter_mut() {
            res += envelope.amplitude(&Instant::now())
                * self
                    .wave_generator
                    .generate_wave(time, note_to_frequency(*note));
        }
        res
    }

    fn finished_notes(&self) -> Vec<i8> {
        let mut res = Vec::new();
        for (note, envelope) in self.notes.iter() {
            if !envelope.is_active() {
                res.push(note.clone());
            }
        }
        res
    }

    fn remove_finished(&mut self, notes: Vec<i8>) {
        for note in notes.iter() {
            self.notes.remove(note);
        }
    }

    fn start_note(&mut self, note: i8) {
        if self.notes.contains_key(&note) {
            // Started to play note that's already played ??? ignore it
            return;
        }
        // copy given envelope
        let envelope = self.envelope.create_activated_copy(&Instant::now());
        self.notes.insert(note, envelope);
    }

    fn end_note(&mut self, note: i8) {
        if let Some(env) = self.notes.get_mut(&note) {
            env.deactivate(&Instant::now())
        }
    }
}
