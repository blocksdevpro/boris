use std::sync::mpsc;

use cpal::traits::HostTrait;

use crate::audio::{
    adapter::{AdapterCommand, AudioAdapter},
    boris::Boris,
    stream::AudioStream,
};

mod audio;
mod constants;
mod logger;
mod services;
mod utils;

fn main() {
    logger::setup_logger();
    let host = cpal::default_host();
    let input_device = host.default_input_device().unwrap();

    let (adapter_tx, adapter_rx) = mpsc::channel::<AdapterCommand>();

    let stream = AudioStream::from_device(input_device);
    let adapter = AudioAdapter::from_stream(stream, adapter_rx);
    let mut boris = Boris::new(adapter_tx);

    boris.process(adapter);
}
