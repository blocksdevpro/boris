use std::time::Duration;

pub const SAMPLE_RATE: u32 = 16_000;
pub const CHANNELS: usize = 1;

pub const WAKEWORD_THRESHOLD: f32 = 0.2;
pub const WAKEWORD_MODEL_PATH: &str = "models/livekit/boris.onnx";

pub const WAKEWORD_INTERVAL: Duration = Duration::from_millis(80);
pub const VAD_INTERVAL: Duration = Duration::from_millis(20);
pub const VAD_SAMPLE_LEN: usize = 256;
pub const VAD_SILENCE_THRESHOLD: f32 = 0.35;
pub const VAD_SPEECH_THRESHOLD: f32 = 0.7;
pub const VAD_SILENCE_DURATION: Duration = Duration::from_millis(600);
