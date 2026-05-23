use std::time::Duration;

pub const SAMPLE_RATE: u32 = 16_000;
pub const CHANNELS: usize = 1;

pub const WAKEWORD_THRESHOLD: f32 = 0.2;
pub const WAKEWORD_MODEL_PATH: &str = "models/livekit/boris.onnx";
pub const WHISPER_MODEL_PATH: &str = "models/whisper/ggml-base-q8_0.bin";

pub const WAKEWORD_INTERVAL: Duration = Duration::from_millis(80);
pub const VAD_INTERVAL: Duration = Duration::from_millis(40);
pub const VAD_SAMPLE_LEN: usize = 256;
pub const VAD_SILENCE_THRESHOLD: f32 = 0.3;
pub const VAD_SPEECH_THRESHOLD: f32 = 0.65;
pub const VAD_SILENCE_DURATION: Duration = Duration::from_millis(1000);
