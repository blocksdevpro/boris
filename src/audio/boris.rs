use crate::{
    audio::whisper::Whisper,
    constants::{
        self, KOKORO_MODEL_CONFIG_PATH, KOKORO_MODEL_PATH, VAD_SILENCE_DURATION,
        VAD_SILENCE_THRESHOLD, VAD_SPEECH_THRESHOLD, WAKEWORD_THRESHOLD, WHISPER_MODEL_PATH,
    },
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

use crate::services::openai;
use crate::services::tts::TtsService;

#[derive(PartialEq)]
enum BorisState {
    Idle,
    Listening,
    Recording,
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
    ProcessOpenAi(String),
    ProcessTTS(String),
}

pub struct Boris {
    event_tx: Sender<BorisEvent>,
    event_rx: Receiver<BorisEvent>,
    adapter_tx: Sender<AdapterCommand>,

    state: BorisState,

    wakeword_model: WakeWordModel,
    vad_model: Detector,
    vad_state: VadState,
    whisper: Whisper,
    openai: openai::OpenAiService,
    tts: TtsService,
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
            whisper: Whisper::new(WHISPER_MODEL_PATH),
            tts: TtsService::new(KOKORO_MODEL_PATH, KOKORO_MODEL_CONFIG_PATH),
            openai: openai::OpenAiService::new(constants::OPENAI_API_KEY),
        }
    }

    fn process_wakeword(&mut self, samples: Vec<f32>) {
        if self.state != BorisState::Listening {
            return;
        }
        let samples = f32_to_i16(&samples);
        let result = self.wakeword_model.predict(&samples).unwrap();

        for (_name, score) in result {
            log::debug!("[boris] wakeword score: {}", score);
            if score >= WAKEWORD_THRESHOLD {
                log::info!("[boris] wakeword detected!");
                self.state = BorisState::Recording;
                self.vad_state.state = VadStateEnum::Speech;
                self.vad_state.timestamp = Instant::now() + Duration::from_millis(600);
                self.adapter_tx.send(AdapterCommand::StartCapture).unwrap();
                log::info!("[VAD] recording!");
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
        if result > VAD_SPEECH_THRESHOLD {
            self.vad_state.state = VadStateEnum::Speech;
            self.vad_state.timestamp = Instant::now();
        } else if result < VAD_SILENCE_THRESHOLD
            && self.vad_state.state == VadStateEnum::Speech
            && self.vad_state.timestamp.elapsed() >= VAD_SILENCE_DURATION
        {
            // reset
            log::info!("[VAD] silence detected!");
            self.vad_state.state = VadStateEnum::Silence;
            self.vad_state.timestamp = Instant::now();

            self.adapter_tx.send(AdapterCommand::StopCapture).unwrap();
        }
    }

    fn process_transcribe(&mut self, samples: Vec<f32>) {
        let instant = Instant::now();
        let result = self.whisper.transcribe(&samples);
        log::debug!("[TRANSCRIBE] took {}ms", instant.elapsed().as_millis());
        log::info!("[TRANSCRIBE] result: {}", result);

        self.event_tx
            .send(BorisEvent::ProcessOpenAi(result))
            .unwrap();
        self.adapter_tx.send(AdapterCommand::Reset).unwrap();
    }

    fn process_openai(&mut self, text: String) {
        let result = self.openai.get_completion(&text);
        if let Some(result) = result {
            log::info!("[OPENAI] result: {}", result);
            self.event_tx.send(BorisEvent::ProcessTTS(result)).ok();
        };
    }

    fn process_tts(&mut self, text: String) {
        let instant = Instant::now();
        let (samples, sample_rate) = self.tts.synthesize(&text);
        log::debug!("[TTS] took {} ms", instant.elapsed().as_millis());
        log::info!("[TTS] result: {}, {}", sample_rate, samples.len());
        let samples = f32_to_i16(&samples);
        write_wav("output.wav", &samples, sample_rate);
        self.state = BorisState::Listening;
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
                    BorisEvent::ProcessVAD(samples) => self.process_vad(samples),
                    BorisEvent::ProcessTranscribe(samples) => self.process_transcribe(samples),
                    BorisEvent::ProcessOpenAi(input) => self.process_openai(input),
                    BorisEvent::ProcessTTS(input) => self.process_tts(input),
                }
            }
        }
    }
}
