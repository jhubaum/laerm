use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;
use std::{time, thread};

use device_query::{DeviceQuery, DeviceState, Keycode};

mod synthesis;
use synthesis::{Oscillator, WaveGenerator};

mod shared;
use shared::InputEvent;

#[derive(Default)]
struct Wave;
impl WaveGenerator for Wave {
    fn evaluate(&self, time: f32, freq: f32) -> f32 {
        0.2 * Oscillator::Square { freq }.evaluate(time)
    }
}

struct KeyboardState {
    device_state: DeviceState,
    new_pressed_keys: Vec<Keycode>,
    pressed_keys: Vec<Keycode>,
    released_keys: Vec<Keycode>
}

impl KeyboardState {
    fn new() -> Self {
        Self {
            device_state: DeviceState::new(),
            new_pressed_keys: Vec::new(),
            pressed_keys: Vec::new(),
            released_keys: Vec::new()
        }
    }

    fn update(&mut self) {
        let cur_keys = self.device_state.get_keys();

        self.released_keys.clear();
        self.new_pressed_keys.clear();

        for key in cur_keys.iter() {
            if !self.pressed_keys.contains(key) {
                self.new_pressed_keys.push(key.clone());
            }
        }

        for key in self.pressed_keys.iter() {
            if !cur_keys.contains(key) {
                self.released_keys.push(key.clone());
            }
        }
        self.pressed_keys = cur_keys;
    }
}

fn main() {
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("no output device available");

    let mut supported_configs_range = device
        .supported_output_configs()
        .expect("error while querying configs");

    let config = supported_configs_range
        .next()
        .expect("no supported config?!")
        .with_max_sample_rate();

    match config.sample_format() {
        cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
        cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
        cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
    };
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig)
where
    T: cpal::Sample,
{
    let sample_rate = config.sample_rate.0 as f32;
    let channels = config.channels as usize;

    let (tx, rx) = mpsc::channel();

    // Produce a sinusoid of maximum amplitude.
    let mut time = 0f32;
    let mut instrument = synthesis::Instrument::<synthesis::DummyEnvelope,
                                               Wave>::default();
    let mut next_value = move || {
        time += 1.0 / sample_rate;
        if let Ok(evt) = rx.try_recv() {
            match evt {
                InputEvent::StartNote => instrument.start_note(0),
                InputEvent::EndNote => instrument.end_note(0)
            }
        }

        instrument.remove_finished(instrument.finished_notes());
        instrument.evaluate(time)
    };

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_value)
            },
            |err| eprintln!("an error occurred on stream: {}", err),
        )
        .expect("Unable to build output stream");

    stream.play().expect("Unable to play stream");

    let mut keyboard = KeyboardState::new();

    loop {
        keyboard.update();
        if keyboard.new_pressed_keys.contains(&Keycode::Space) {
            tx.send(InputEvent::StartNote).unwrap();
        }

        if keyboard.released_keys.contains(&Keycode::Space) {
           tx.send(InputEvent::EndNote).unwrap();
        }

        if keyboard.pressed_keys.contains(&Keycode::Q) {
            break;
        }

        thread::sleep(time::Duration::from_millis(50))
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: cpal::Sample,
{
    for frame in output.chunks_mut(channels) {
        let value: T = cpal::Sample::from::<f32>(&next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}
