use cpal::traits::HostTrait;
use std::{
    sync::{Arc, Mutex, mpsc},
    thread,
};

use crate::audio::{
    adapter::{self, AudioAdapter},
    stream::AudioStream,
};

mod audio;
mod constants;

pub enum Event {
    Process(Vec<f32>),
}

fn main() {
    let host = cpal::default_host();
    let device = host.default_input_device().unwrap();

    let stream = AudioStream::from_device(device);
    let adapter = Arc::new(Mutex::new(AudioAdapter::from_stream(stream)));

    let (event_tx, event_rx) = mpsc::channel::<Event>();
    let adapter_clone = adapter.clone();
    let event_tx_clone = event_tx.clone();

    let model = thread::spawn(move || {
        let mut adapter = adapter_clone.lock().unwrap();
        adapter.process(event_tx_clone);
    });

    loop {
        let event = event_rx.recv().unwrap();
        match event {
            Event::Process(samples) => {
                println!("Process: {:?}", samples.len());
            }
        }
    }
}
