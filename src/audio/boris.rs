use crate::{
    audio::{filters, playback::Playback, whisper::Whisper},
    config::Config,
    constants::{
        KOKORO_MODEL_CONFIG_PATH, KOKORO_MODEL_PATH, VAD_SILENCE_DURATION, VAD_SILENCE_THRESHOLD,
        VAD_SPEECH_THRESHOLD, WAKEWORD_THRESHOLD, WHISPER_MODEL_PATH,
    },
    utils::{f32_to_i16, write_wav},
};
use std::{
    sync::mpsc::{self, Receiver, RecvTimeoutError, Sender},
    thread,
    time::{Duration, Instant},
};

use cpal::traits::HostTrait;
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
    ProcessPlayback(Vec<f32>),
}

pub struct Boris {
    event_tx: Sender<BorisEvent>,
    event_rx: Receiver<BorisEvent>,
    adapter_tx: Sender<AdapterCommand>,

    state: BorisState,

    wakeword_model: WakeWordModel,
    /// Exponential moving average of the raw per-frame wakeword score.
    /// Accumulating across frames lets far-field audio (many weak hits)
    /// reliably trigger detection without lowering the instantaneous threshold.
    wakeword_score_ema: f32,
    vad_model: Detector,
    vad_state: VadState,
    whisper: Whisper,
    openai: openai::OpenAiService,
    tts: TtsService,
}

impl Boris {
    pub fn new(adapter_tx: Sender<AdapterCommand>, config: Config) -> Self {
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
            wakeword_score_ema: 0.0,
            vad_model: Detector::default(),
            vad_state,
            whisper: Whisper::new(WHISPER_MODEL_PATH),
            tts: TtsService::new(KOKORO_MODEL_PATH, KOKORO_MODEL_CONFIG_PATH),
            openai: openai::OpenAiService::new(&config.api_key, &config.model, &config.base_url),
        }
    }

    fn init_playback(&self) -> Playback {
        let host = cpal::default_host();
        let device = host.default_output_device().unwrap();
        Playback::new(device)
    }

    fn process_wakeword(&mut self, samples: Vec<f32>) {
        if self.state != BorisState::Listening {
            return;
        }

        // Apply filter chain: high-pass → pre-emphasis → RMS normalize.
        let hp_samples = filters::high_pass_filter(&samples);
        let mut processed = filters::pre_emphasis(&hp_samples);
        filters::rms_normalize(&mut processed, 0.25);

        let samples_i16 = f32_to_i16(&processed);

        let result = self.wakeword_model.predict(&samples_i16).unwrap();

        for (_name, raw_score) in result {
            // EMA smoothing: accumulate weak per-frame scores across time.
            // Far-field audio produces many frames of ~0.08–0.15 instead of
            // one frame of ~0.8. The EMA converts that stream of weak hits
            // into a rising smoothed score that crosses the threshold reliably.
            // Alpha=0.3 means ~5 frames of sustained signal to reach 0.5×peak.
            const EMA_ALPHA: f32 = 0.3;
            self.wakeword_score_ema =
                EMA_ALPHA * raw_score + (1.0 - EMA_ALPHA) * self.wakeword_score_ema;

            log::debug!(
                "[boris] wakeword raw={:.3} ema={:.3}",
                raw_score,
                self.wakeword_score_ema
            );

            if self.wakeword_score_ema >= WAKEWORD_THRESHOLD || raw_score >= WAKEWORD_THRESHOLD {
                log::info!(
                    "[boris] wakeword detected! raw={:.3} ema={:.3}",
                    raw_score,
                    self.wakeword_score_ema
                );
                // Reset EMA so it doesn't immediately re-trigger.
                self.wakeword_score_ema = 0.0;
                self.state = BorisState::Recording;
                self.vad_state.state = VadStateEnum::Speech;
                self.vad_state.timestamp = Instant::now() + Duration::from_millis(600);
                self.adapter_tx.send(AdapterCommand::StartCapture).unwrap();
                log::info!("[VAD] recording!");
                break;
            }
        }
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
        self.event_tx
            .send(BorisEvent::ProcessPlayback(samples))
            .ok();
    }

    fn process_playback(&mut self, samples: Vec<f32>) {
        log::info!("[PLAYBACK] playing audio.");
        let mut playback = self.init_playback();
        playback.play(samples);
        self.process_listening();
    }

    fn process_listening(&mut self) {
        self.state = BorisState::Listening;
        self.wakeword_score_ema = 0.0;

        // Drain every event that backed up while we were busy with
        // Whisper / OpenAI / TTS / playback. Those wakeword frames are
        // stale (up to ~15 s old) and, if processed in a tight burst,
        // would distort the EMA completely — making it seem like the
        // wakeword is being spoken for dozens of frames simultaneously.
        let mut drained = 0usize;
        while self.event_rx.try_recv().is_ok() {
            drained += 1;
        }
        if drained > 0 {
            log::debug!("[boris] drained {} stale events on wakeup", drained);
        }

        log::info!("[boris] listening...");
    }

    pub fn process(&mut self, mut adapter: AudioAdapter) {
        self.process_listening();

        let event_tx_clone = self.event_tx.clone();
        let _handle = thread::spawn(move || {
            adapter.process(event_tx_clone);
        });

        loop {
            // Block until an event arrives (or 20 ms elapses).
            // This replaces the previous busy-spin which consumed 100% of a
            // CPU core, gradually starving the audio adapter thread of
            // scheduling time and causing audio frames to queue up.
            match self.event_rx.recv_timeout(Duration::from_millis(20)) {
                Ok(event) => match event {
                    BorisEvent::ProcessWakeword(samples) => self.process_wakeword(samples),
                    BorisEvent::ProcessVAD(samples) => self.process_vad(samples),
                    BorisEvent::ProcessTranscribe(samples) => self.process_transcribe(samples),
                    BorisEvent::ProcessOpenAi(input) => self.process_openai(input),
                    BorisEvent::ProcessTTS(input) => self.process_tts(input),
                    BorisEvent::ProcessPlayback(samples) => self.process_playback(samples),
                },
                Err(RecvTimeoutError::Timeout) => {
                    // No events for 20 ms — gently decay the EMA so that
                    // stray ambient noise never builds up a false baseline
                    // over long idle periods.
                    self.wakeword_score_ema *= 0.85;
                }
                Err(RecvTimeoutError::Disconnected) => break,
            }
        }
    }
}
