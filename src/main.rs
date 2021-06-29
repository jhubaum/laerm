use std::{thread, time};

mod system;
use system::{KeyboardEvent, KeyboardInput, Keycode};

mod synthesis;
use synthesis::{Oscillator, WaveGenerator};

mod interface;
use interface::{InputEvent, SoundDeviceInterface};

#[derive(Default)]
struct Wave;
impl WaveGenerator for Wave {
    fn evaluate(&self, time: f32, freq: f32) -> f32 {
        0.2 * Oscillator::Square { freq }.evaluate(time)
    }
}

fn main() {
    let mut keyboard = KeyboardInput::new();
    let interface = SoundDeviceInterface::create_default();

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
