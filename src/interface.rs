use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;

use super::synthesis;

pub enum InputEvent {
    StartNote,
    EndNote,
}

// only declared temporarily – will be removed after implementing a proper instrument struct
#[derive(Default)]
struct Wave;
impl synthesis::WaveGenerator for Wave {
    fn evaluate(&self, time: f32, freq: f32) -> f32 {
        0.2 * synthesis::Oscillator::Square { freq }.evaluate(time)
    }
}

pub struct SoundDeviceInterface {
    sender: mpsc::Sender<InputEvent>,
    // only needed to keep the stream alive for now
    _stream: cpal::Stream,
}

impl SoundDeviceInterface {
    pub fn send(&self, evt: InputEvent) {
        self.sender.send(evt).unwrap()
    }

    pub fn create_default() -> Self {
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
            cpal::SampleFormat::F32 => Self::create::<f32>(&device, &config.into()),
            cpal::SampleFormat::I16 => Self::create::<i16>(&device, &config.into()),
            cpal::SampleFormat::U16 => Self::create::<u16>(&device, &config.into()),
        }
    }

    pub fn create<TSample: cpal::Sample>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> Self {
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        let (tx, rx) = mpsc::channel();

        // Produce a sinusoid of maximum amplitude.
        let mut time = 0f32;
        let mut instrument = synthesis::Instrument::<synthesis::DummyEnvelope, Wave>::default();
        let mut next_value = move || {
            time += 1.0 / sample_rate;
            if let Ok(evt) = rx.try_recv() {
                match evt {
                    InputEvent::StartNote => instrument.start_note(0),
                    InputEvent::EndNote => instrument.end_note(0),
                }
            }

            instrument.remove_finished(instrument.finished_notes());
            instrument.evaluate(time)
        };

        let stream = device
            .build_output_stream(
                config,
                move |data: &mut [TSample], _: &cpal::OutputCallbackInfo| {
                    write_data(data, channels, &mut next_value)
                },
                |err| eprintln!("an error occurred on stream: {}", err),
            )
            .expect("Unable to build output stream");

        stream.play().expect("Unable to play stream");

        SoundDeviceInterface {
            sender: tx,
            _stream: stream,
        }
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
