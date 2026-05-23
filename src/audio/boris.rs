use crate::{
    constants::{VAD_SILENCE_DURATION, VAD_SILENCE_THRESHOLD, VAD_SPEECH_THRESHOLD},
    utils::{f32_to_i16, write_wav},
};
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

use earshot::Detector;
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

#[derive(PartialEq)]
enum VadStateEnum {
    Speech,
    Silence,
}

struct VadState {
    state: VadStateEnum,
    timestamp: Instant,
}

pub enum BorisEvent {
    ProcessWakeword(Vec<f32>),
    ProcessVAD(Vec<f32>),
    ProcessTranscribe(Vec<f32>),
}

pub struct Boris {
    event_tx: Sender<BorisEvent>,
    event_rx: Receiver<BorisEvent>,
    adapter_tx: Sender<AdapterCommand>,

    state: BorisState,

    wakeword_model: WakeWordModel,
    vad_model: Detector,
    vad_state: VadState,
}

impl Boris {
    pub fn new(adapter_tx: Sender<AdapterCommand>) -> Self {
        let (event_tx, event_rx) = mpsc::channel::<BorisEvent>();
        let vad_state = VadState {
            state: VadStateEnum::Silence,
            timestamp: Instant::now(),
        };

        Self {
            event_tx,
            event_rx,
            adapter_tx,
            state: BorisState::Idle,
            wakeword_model: WakeWordModel::new(&[WAKEWORD_MODEL_PATH], SAMPLE_RATE).unwrap(),
            vad_model: Detector::default(),
            vad_state,
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
                self.vad_state.state = VadStateEnum::Speech;
                self.vad_state.timestamp = Instant::now();
                self.adapter_tx.send(AdapterCommand::StartCapture).unwrap();
                println!("[VAD] recording!");
                break;
            }
        }
        write_wav("models/audio/output.wav", &samples, SAMPLE_RATE);
    }

    fn process_vad(&mut self, samples: Vec<f32>) {
        if self.state != BorisState::Recording {
            return;
        }
        let result = self.vad_model.predict_f32(&samples);
        println!("[VAD] result: {}", result);
        if result > VAD_SPEECH_THRESHOLD {
            self.vad_state.state = VadStateEnum::Speech;
            self.vad_state.timestamp = Instant::now();
        } else if result < VAD_SILENCE_THRESHOLD {
            if self.vad_state.state == VadStateEnum::Speech
                && self.vad_state.timestamp.elapsed() >= VAD_SILENCE_DURATION
            {
                // reset
                println!("[VAD] silence detected!");
                self.vad_state.state = VadStateEnum::Silence;
                self.vad_state.timestamp = Instant::now();

                self.adapter_tx.send(AdapterCommand::StopCapture).unwrap();
            }
        }
    }

    fn process_transcribe(&mut self, samples: Vec<f32>) {}

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
                    BorisEvent::ProcessVAD(samples) => self.process_vad(samples),
                    BorisEvent::ProcessTranscribe(samples) => self.process_transcribe(samples),
                }
            }
        }
    }
}
