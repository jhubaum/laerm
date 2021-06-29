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

fn main() {
    let mut keyboard = KeyboardInput::new();
    let interface = SoundDeviceInterface::create_default::<DefaultInstrument>();

    loop {
        for evt in keyboard.query_events().iter() {
            match evt {
                KeyboardEvent::KeyPressed(Keycode::Q) => return,
                KeyboardEvent::KeyPressed(Keycode::Space) => {
                    interface.send(InputEvent::StartNote);
                }
                KeyboardEvent::KeyReleased(Keycode::Space) => {
                    interface.send(InputEvent::EndNote);
                }
                _ => {}
            }
        }
        thread::sleep(time::Duration::from_millis(50))
    }
}
