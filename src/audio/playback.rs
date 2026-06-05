use std::sync::mpsc;

use cpal::{
    BufferSize, Device, Stream, StreamConfig,
    traits::{DeviceTrait, StreamTrait},
};

pub struct Playback {
    stream: Stream,
    sender: mpsc::Sender<Vec<f32>>,
}

// TODO: specify the output device
// TODO: work on the output audio model

impl Playback {
    pub fn new(device: Device) -> Self {
        let config = device.default_output_config().unwrap();
        let sample_format = config.sample_format();

        println!("sample_format: {:?}", sample_format);

        let stream_config = StreamConfig {
            channels: 1,
            sample_rate: 22050,
            buffer_size: BufferSize::Default,
        };

        let (sender, receiver) = mpsc::channel();
        let mut current: Vec<f32> = Vec::new();
        let mut cursor = 0;

        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _| {
                    if cursor >= current.len()
                        && let Ok(samples) = receiver.try_recv()
                    {
                        current = samples;
                        cursor = 0;
                    }

                    for sample in data.iter_mut() {
                        if cursor <= current.len() {
                            *sample = current[cursor];
                            cursor += 1;
                        } else {
                            *sample = 0.0;
                        }
                    }
                },
                |err| eprintln!("stream error: {err}"),
                None,
            )
            .unwrap();
        Self { stream, sender }
    }

    pub fn play(&mut self, samples: Vec<f32>) {
        self.stream.play().unwrap();
        self.sender.send(samples).ok();
    }
}
