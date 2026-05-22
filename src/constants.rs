use std::time::Duration;

pub const SAMPLE_RATE: u32 = 16_000;
pub const CHANNELS: usize = 1;
pub const WAKEWORD_THRESHOLD: f32 = 0.5;

pub const WAKEWORD_INTERVAL: Duration = Duration::from_millis(70);
