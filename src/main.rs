use cpal::traits::HostTrait;
use hound::{SampleFormat, WavSpec, WavWriter};
use livekit_wakeword::WakeWordModel;
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

use crate::{
    audio::{adapter::AudioAdapter, stream::AudioStream},
    constants::{SAMPLE_RATE, WAKEWORD_MODEL_PATH, WAKEWORD_THRESHOLD},
};

mod audio;
mod constants;

pub enum Event {
    Process(Vec<f32>),
}

fn f32_to_i16(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|s| (*s * i16::MAX as f32) as i16)
        .collect()
}

fn write_wav(filename: &str, samples: &[i16], sample_rate: u32) {
    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };
    let mut writer = WavWriter::create(filename, spec).unwrap();
    for &sample in samples {
        writer.write_sample(sample).unwrap();
    }
}

fn main() {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let stream = AudioStream::from_device(device);
    let adapter = Arc::new(Mutex::new(AudioAdapter::from_stream(stream)));

    let (event_tx, event_rx) = mpsc::channel::<Event>();
    let adapter_clone = adapter.clone();
    let event_tx_clone = event_tx.clone();

    let mut model = WakeWordModel::new(&[WAKEWORD_MODEL_PATH], SAMPLE_RATE).unwrap();

    thread::spawn(move || {
        let mut adapter = adapter_clone.lock().unwrap();
        adapter.process(event_tx_clone);
    });

    println!("[boris] listening...");

    loop {
        let event = event_rx.recv().unwrap();
        match event {
            Event::Process(samples) => {
                let samples = &f32_to_i16(&samples);
                let result = model.predict(samples).unwrap();

                for (name, score) in result {
                    if score >= WAKEWORD_THRESHOLD {
                        println!("Wakeword detected: {} (score: {})", name, score);
                    }
                }

                write_wav("output.wav", samples, SAMPLE_RATE);
            }
        }
    }
}
