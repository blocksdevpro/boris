use std::sync::mpsc;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use cpal::{
    BufferSize, Device, Stream, StreamConfig,
    traits::{DeviceTrait, StreamTrait},
};

pub struct Playback {
    stream: Stream,
    sender: mpsc::Sender<Vec<f32>>,
    done_rx: mpsc::Receiver<()>,
}

// TODO: specify the output device
// TODO: work on the output audio model

impl Playback {
    pub fn new(device: Device) -> Self {
        log::debug!("device: {:?}", device.description());

        let stream_config = StreamConfig {
            channels: 1,
            sample_rate: 22050,
            buffer_size: BufferSize::Default,
        };

        let (sender, receiver) = mpsc::channel();
        let (done_tx, done_rx) = mpsc::channel();
        let mut current: Vec<f32> = Vec::new();
        let mut cursor = 0;
        let finished = Arc::new(AtomicBool::new(false));

        let stream = device
            .build_output_stream(
                &stream_config,
                move |data: &mut [f32], _| {
                    // If we've finished the current buffer, try to get the next one
                    if cursor >= current.len() {
                        match receiver.try_recv() {
                            Ok(samples) => {
                                current = samples;
                                cursor = 0;
                            }
                            Err(_) => {
                                // No new data — output silence
                                for sample in data.iter_mut() {
                                    *sample = 0.0;
                                }
                                // If we had data before and now we're done, signal completion
                                if !finished.load(Ordering::Relaxed) && !current.is_empty() {
                                    finished.store(true, Ordering::Relaxed);
                                    done_tx.send(()).ok();
                                }
                                return;
                            }
                        }
                    }

                    for sample in data.iter_mut() {
                        if cursor < current.len() {
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
        Self {
            stream,
            sender,
            done_rx,
        }
    }

    pub fn play(&mut self, samples: Vec<f32>) {
        self.stream.play().unwrap();
        self.sender.send(samples).ok();
    }

    /// Blocks until playback has finished.
    pub fn wait(&self) {
        self.done_rx.recv().ok();
    }
}
