use std::{thread, time};

mod system;
use system::{KeyboardEvent, KeyboardInput, Keycode};

mod synthesis;
use synthesis::{Oscillator, WaveGenerator};

mod interface;
use interface::{InputEvent, SoundDeviceInterface};

pub struct DummyEnvelope {
    activated: bool,
}

impl synthesis::Envelope for DummyEnvelope {
    fn amplitude(&self, _: f32) -> f32 {
        if self.activated {
            1.0
        } else {
            0.0
        }
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

    fn create_activated_copy(&self) -> Self {
        DummyEnvelope { activated: true }
    }
}

struct DefaultInstrumentImplementationDetails;
type DefaultInstrument =
    synthesis::InstrumentImpl<DummyEnvelope, DefaultInstrumentImplementationDetails>;

impl WaveGenerator for DefaultInstrumentImplementationDetails {
    fn generate_wave(&self, time: f32, freq: f32) -> f32 {
        0.2 * Oscillator::Square { freq }.evaluate(time)
    }
}

impl Default for DefaultInstrument {
    fn default() -> Self {
        Self::create(
            DummyEnvelope { activated: false },
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
