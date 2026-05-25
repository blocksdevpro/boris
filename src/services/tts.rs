use std::path::PathBuf;

use piper_rs::Piper;

pub struct TtsService {
    model: Piper,
}

impl TtsService {
    pub fn new(model_path: &str, config_path: &str) -> Self {
        let model_path = PathBuf::from(model_path);
        let config_path = PathBuf::from(config_path);
        let piper = Piper::new(&model_path, &config_path).expect("failed to create piper model");
        Self { model: piper }
    }

    pub fn synthesize(&mut self, text: &str) -> (Vec<f32>, u32) {
        self.model
            .create(text, false, Some(1), Some(1.35), None, None)
            .unwrap()
    }
}

// tests
//
#[cfg(test)]
mod tests {
    use crate::{
        constants::{KOKORO_MODEL_CONFIG_PATH, KOKORO_MODEL_PATH},
        utils::{f32_to_i16, write_wav},
    };

    use super::*;

    #[test]
    fn test_synthesize() {
        let mut tts = TtsService::new(KOKORO_MODEL_PATH, KOKORO_MODEL_CONFIG_PATH);
        let (audio, sample_rate) = tts.synthesize("Broda, Patra is epic, you gotta go there. The turtles will love you, probably.

        June is super hot because, you know, the sun moves closer. My sensors are lagging, but you should packs some socks!");
        write_wav("output_test.wav", &f32_to_i16(&audio), sample_rate);

        assert_eq!(sample_rate, 22050);
    }
}
