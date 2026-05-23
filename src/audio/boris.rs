use crate::{
    audio::{adapter, stream::AudioStream},
    utils::{f32_to_i16, write_wav},
};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread::{self, JoinHandle},
};

use livekit_wakeword::WakeWordModel;

use crate::{
    audio::adapter::{AdapterCommand, AudioAdapter},
    constants::{SAMPLE_RATE, WAKEWORD_MODEL_PATH},
};

#[derive(PartialEq)]
enum BorisState {
    Idle,
    Listening,
    Recording,
    Transcribing,
}

pub enum BorisEvent {
    ProcessWakeword(Vec<f32>),
}

pub struct Boris {
    event_tx: Sender<BorisEvent>,
    event_rx: Receiver<BorisEvent>,
    adapter_tx: Sender<AdapterCommand>,

    state: BorisState,

    wakeword_model: WakeWordModel,
}

impl Boris {
    pub fn new(adapter_tx: Sender<AdapterCommand>) -> Self {
        let (event_tx, event_rx) = mpsc::channel::<BorisEvent>();

        Self {
            event_tx,
            event_rx,
            adapter_tx,
            state: BorisState::Idle,
            wakeword_model: WakeWordModel::new(&[WAKEWORD_MODEL_PATH], SAMPLE_RATE).unwrap(),
        }
    }

    fn process_wakeword(&mut self, samples: Vec<f32>) {
        if self.state != BorisState::Listening {
            return;
        }
        let samples = f32_to_i16(&samples);
        let result = self.wakeword_model.predict(&samples).unwrap();

        for (_name, score) in result {
            if score > 0.2 {
                println!("[boris] wakeword detected!");
                self.state = BorisState::Recording;
                self.adapter_tx.send(AdapterCommand::StartCapture).unwrap();
                break;
            }
        }
        write_wav("models/audio/output.wav", &samples, SAMPLE_RATE);
    }

    pub fn process(&mut self, mut adapter: AudioAdapter) {
        self.state = BorisState::Listening;

        let event_tx_clone = self.event_tx.clone();

        let _handle = thread::spawn(move || {
            adapter.process(event_tx_clone);
        });

        loop {
            while let Ok(event) = self.event_rx.try_recv() {
                match event {
                    BorisEvent::ProcessWakeword(samples) => self.process_wakeword(samples),
                }
            }
        }
    }
}
