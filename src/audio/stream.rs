use cpal::{
    Device, Stream,
    traits::{DeviceTrait, StreamTrait},
};
use std::sync::mpsc;

pub struct AudioStream {
    rx: mpsc::Receiver<Vec<f32>>,
    rate: u32,
    stream: Stream,
}

impl AudioStream {
    pub fn from_device(device: Device) -> Self {
        let (tx, rx) = mpsc::channel();

        let config = device
            .default_input_config()
            .expect("[ERROR] failed to get default input config!");
        let samplerate = config.sample_rate();
        let channels = config.channels() as usize;

        log::info!(
            "Sample rate: {} channels: {}, format: {:?}",
            samplerate,
            channels,
            config.sample_format()
        );

        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &config.into(),
                move |samples: &[f32], _: &cpal::InputCallbackInfo| {
                    // convert to mono by averaging samples
                    let samples = samples
                        .chunks(channels)
                        .map(|sample| sample.iter().sum::<f32>() / sample.len() as f32)
                        .collect::<Vec<f32>>();
                    tx.send(samples).ok();
                },
                |_| {},
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &config.into(),
                move |samples: &[i16], _: &cpal::InputCallbackInfo| {
                    let samples = samples
                        .chunks(channels)
                        .map(|sample| {
                            let mono_sample = sample.iter().sum::<i16>() / sample.len() as i16;
                            mono_sample as f32 / i16::MAX as f32
                        })
                        .collect::<Vec<f32>>();
                    tx.send(samples).ok();
                },
                |_| {},
                None,
            ),

            format => panic!("[ERROR] unsupported sample format: {:?}", format),
        }
        .expect("[ERROR] failed to build input stream!");
        Self {
            rx,
            rate: samplerate,
            stream,
        }
    }

    pub fn rate(&self) -> u32 {
        self.rate
    }

    pub fn play(&mut self) {
        self.stream.play().expect("[ERROR] failed to play stream!")
    }

    pub fn read(&mut self) -> Vec<f32> {
        self.rx.recv().expect("[ERROR] failed to read stream!")
    }
}
