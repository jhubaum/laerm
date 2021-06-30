use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc;

use super::synthesis::Instrument;

pub enum InputEvent {
    StartNote(i8),
    EndNote(i8),
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

    pub fn create_default<TInstr: Instrument + Default + Send + 'static>() -> Self {
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
            cpal::SampleFormat::F32 => Self::create::<f32, TInstr>(&device, &config.into()),
            cpal::SampleFormat::I16 => Self::create::<i16, TInstr>(&device, &config.into()),
            cpal::SampleFormat::U16 => Self::create::<u16, TInstr>(&device, &config.into()),
        }
    }

    pub fn create<TSample: cpal::Sample, TInstr: Instrument + Default + Send + 'static>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
    ) -> Self {
        let sample_rate = config.sample_rate.0 as f32;
        let channels = config.channels as usize;

        let (tx, rx) = mpsc::channel();

        let mut sample_clock = 0f32;
        let mut instrument = TInstr::default();
        let mut next_value = move || {
            sample_clock = (sample_clock + 1.0) % sample_rate;
            if let Ok(evt) = rx.try_recv() {
                match evt {
                    InputEvent::StartNote(i) => instrument.start_note(i),
                    InputEvent::EndNote(i) => instrument.end_note(i),
                }
            }

            instrument.remove_finished(instrument.finished_notes());
            instrument.evaluate(sample_clock / sample_rate)
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
