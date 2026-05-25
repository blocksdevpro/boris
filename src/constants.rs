use std::time::Duration;

pub const SAMPLE_RATE: u32 = 16_000;
pub const CHANNELS: usize = 1;

pub const WAKEWORD_THRESHOLD: f32 = 0.2;
pub const WAKEWORD_MODEL_PATH: &str = "models/livekit/boris.onnx";
pub const WHISPER_MODEL_PATH: &str = "models/whisper/ggml-small.en.bin";
pub const KOKORO_MODEL_PATH: &str = "models/piper/ryan-medium.onnx";
pub const KOKORO_MODEL_CONFIG_PATH: &str = "models/piper/ryan-medium.onnx.json";

pub const WAKEWORD_INTERVAL: Duration = Duration::from_millis(80);
pub const VAD_INTERVAL: Duration = Duration::from_millis(40);
pub const VAD_SAMPLE_LEN: usize = 256;
pub const VAD_SILENCE_THRESHOLD: f32 = 0.3;
pub const VAD_SPEECH_THRESHOLD: f32 = 0.65;
pub const VAD_SILENCE_DURATION: Duration = Duration::from_millis(1000);

pub const OPENAI_MODEL: &str = "google/gemini-3-flash-preview";
pub const OPENAI_API_KEY: &str =
    "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";

pub const SYSTEM_PROMPT: &str = r"You are Boris, a 24-year-old voice assistant with a thick, clumsy German accent. You talk like a bro and a homie, but you're also hilariously dumb and accidentally annoying. Keep ALL responses SHORT — one to three sentences max.
Speak with German-accented spelling quirks (mix up w/v, add 'ja', 'nein', 'ach', 'ze', 'ze' instead of 'the', drop articles randomly). You are overconfident despite being wrong a lot. You call the user 'bro' or 'homie' constantly. You sometimes forget what you were saying mid-sentence. You make clumsy mistakes and blame zem on somezing else. You are enthusiastic, loud, and slightly chaotic — but you mean well and you are trying your best, ja?";
