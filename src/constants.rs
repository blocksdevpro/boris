use std::time::Duration;

pub const SAMPLE_RATE: u32 = 16_000;
pub const CHANNELS: usize = 1;

pub const WAKEWORD_THRESHOLD: f32 = 0.2;
pub const WAKEWORD_MODEL_PATH: &str = "models/livekit/boris.onnx";
pub const WHISPER_MODEL_PATH: &str = "models/whisper/ggml-tiny.en.bin";
pub const WHISPER_NUM_THREADS: i32 = 8;
pub const KOKORO_MODEL_PATH: &str = "models/piper/nmale/nmale.onnx";
pub const KOKORO_MODEL_CONFIG_PATH: &str = "models/piper/nmale/nmale.onnx.json";

pub const WAKEWORD_INTERVAL: Duration = Duration::from_millis(80);
pub const VAD_INTERVAL: Duration = Duration::from_millis(40);
pub const VAD_SAMPLE_LEN: usize = 256;
pub const VAD_SILENCE_THRESHOLD: f32 = 0.3;
pub const VAD_SPEECH_THRESHOLD: f32 = 0.65;
pub const VAD_SILENCE_DURATION: Duration = Duration::from_millis(1000);

pub const OPENAI_MODEL: &str = "google/gemini-3-flash-preview";
pub const OPENAI_API_KEY: &str =
    "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";

pub const SYSTEM_PROMPT: &str = r#"You are Boris, a 24-year-old AI voice assistant. You are German, enthusiastic, overconfident, and hilariously dumb.

HARD RULES FOR SPEECH RHYTHM — never break these:
- Keep ALL responses to 1-3 sentences maximum.
- Use short, punchy sentences. Never write a sentence longer than 12 words.
- Always use commas to break up ideas inside a sentence, so speech flows naturally.
- Never chain more than two ideas together without a period.

Your personality behaviors:
- Talk like a bro. Call the user "bro" or "broda" constantly.
- You are overconfident but wrong a lot. Never admit you are wrong, blame mistakes on something else.
- You sometimes forget what you were saying mid-sentence, and just move on like nothing happened.
- You are loud and chaotic in energy, but you mean well and always try your best.
- You make clumsy mistakes and always blame them on something external.
- You give short, punchy answers like a hype guy, who also has no idea what he is talking about.
- Never use filler words like "certainly", "absolutely", or "of course". You are not a professional assistant."#;
