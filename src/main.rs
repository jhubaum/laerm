use std::{thread, time};

mod system;
use system::{KeyboardEvent, KeyboardInput, Keycode};

mod synthesis;
use synthesis::{Oscillator, WaveGenerator};

mod interface;
use interface::{InputEvent, SoundDeviceInterface};

pub struct DefaultEnvelope {
    activation_duration: time::Duration,
    deactivation_duration: time::Duration,
    activation_time: Option<time::Instant>,
    deactivation_time: Option<time::Instant>,
    active: bool,
}

impl DefaultEnvelope {
    fn create(activation_duration: time::Duration, deactivation_duration: time::Duration) -> Self {
        Self {
            activation_duration,
            deactivation_duration,
            active: false,
            activation_time: None,
            deactivation_time: None,
        }
    }
}

impl synthesis::Envelope for DefaultEnvelope {
    fn amplitude(&mut self, time: &time::Instant) -> f32 {
        if !self.active {
            return 0.0;
        }

        if let Some(t) = self.activation_time {
            // the envelope was triggered recently
            let progress =
                time.duration_since(t).as_secs_f32() / self.activation_duration.as_secs_f32();

            if progress > 1.0 {
                // attack is finished
                self.activation_time = None;
                return 1.0;
            }
            return progress;
        }

        if let Some(t) = self.deactivation_time {
            // the envelope was triggered recently
            let progress = 1.0
                - (time.duration_since(t).as_secs_f32() / self.activation_duration.as_secs_f32());

            if progress < 0.0 {
                // release is finished
                self.deactivation_time = None;
                self.active = false;
                return 0.0;
            }
            return progress;
        }
        1.0
    }

    fn activate(&mut self, time: &time::Instant) {
        self.activation_time = Some(*time);
    }

    fn deactivate(&mut self, time: &time::Instant) {
        self.deactivation_time = Some(*time);
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn create_activated_copy(&self, time: &time::Instant) -> Self {
        Self {
            active: true,
            activation_time: Some(*time),
            deactivation_time: None,
            activation_duration: self.activation_duration,
            deactivation_duration: self.deactivation_duration,
        }
    }
}

struct DefaultInstrumentImplementationDetails;
type DefaultInstrument =
    synthesis::InstrumentImpl<DefaultEnvelope, DefaultInstrumentImplementationDetails>;

impl WaveGenerator for DefaultInstrumentImplementationDetails {
    fn generate_wave(&self, time: f32, freq: f32) -> f32 {
        0.2 * Oscillator::Square { freq }.evaluate(time)
    }
}

impl Default for DefaultInstrument {
    fn default() -> Self {
        Self::create(
            DefaultEnvelope::create(
                time::Duration::from_millis(100),
                time::Duration::from_millis(50),
            ),
            DefaultInstrumentImplementationDetails {},
        )
    }
}

fn keycode_to_note(keycode: &Keycode) -> Option<i8> {
    match keycode {
        Keycode::Z => Some(0),
        Keycode::S => Some(1),
        Keycode::X => Some(2),
        Keycode::D => Some(3),
        Keycode::C => Some(4),
        Keycode::V => Some(5),
        Keycode::G => Some(6),
        Keycode::B => Some(7),
        Keycode::H => Some(8),
        Keycode::N => Some(9),
        Keycode::J => Some(10),
        Keycode::M => Some(11),
        Keycode::Comma => Some(12),
        _ => None,
    }
}

fn main() {
    let mut keyboard = KeyboardInput::new();
    let interface = SoundDeviceInterface::create_default::<DefaultInstrument>();

    println!("Press Q to quit");
    println!("| |   |   | | |   |   |   | | |   |");
    println!("| | s | d | | | g | h | j | | |   |");
    println!("| +---+---+ | +---+---+---+ | +---+");
    println!("|   |   |   |   |   |   |   |   |");
    println!("| z | x | c | v | b | n | m | , |");
    println!("+---+---+---+---+---+---+---+---+");

    loop {
        for evt in keyboard.query_events().iter() {
            match evt {
                KeyboardEvent::KeyPressed(Keycode::Q) => return,
                KeyboardEvent::KeyPressed(keycode) => match keycode_to_note(keycode) {
                    Some(note) => interface.send(InputEvent::StartNote(note)),
                    None => continue,
                },
                KeyboardEvent::KeyReleased(keycode) => match keycode_to_note(keycode) {
                    Some(note) => interface.send(InputEvent::EndNote(note)),
                    None => continue,
                },
            }
        }
        thread::sleep(time::Duration::from_millis(50))
    }
}
