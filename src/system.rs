use device_query::{DeviceQuery, DeviceState};

pub type Keycode = device_query::Keycode;

pub enum KeyboardEvent {
    KeyPressed(Keycode),
    KeyReleased(Keycode),
}

pub struct KeyboardInput {
    device_state: DeviceState,
    pressed_keys: Vec<Keycode>,
}

impl KeyboardInput {
    pub fn new() -> Self {
        Self {
            device_state: DeviceState::new(),
            pressed_keys: Vec::new(),
        }
    }

    pub fn query_events(&mut self) -> Vec<KeyboardEvent> {
        let cur_keys = self.device_state.get_keys();

        let mut events = Vec::new();
        for key in cur_keys.iter() {
            if !self.pressed_keys.contains(key) {
                events.push(KeyboardEvent::KeyPressed(key.clone()));
            }
        }

        for key in self.pressed_keys.iter() {
            if !cur_keys.contains(key) {
                events.push(KeyboardEvent::KeyReleased(key.clone()));
            }
        }
        self.pressed_keys = cur_keys;
        events
    }
}
