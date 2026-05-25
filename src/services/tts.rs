use std::path::PathBuf;

use piper_rs::Piper;

pub struct TtsService {
    model: Piper,
}

impl TtsService {
    pub fn new(model_path: &str, config_path: &str) -> Self {
        let model_path = PathBuf::from(model_path);
        let config_path = PathBuf::from(config_path);
        let piper = Piper::new(&model_path, &config_path).unwrap();
        Self { model: piper }
    }

    pub fn synthesize(&mut self, text: &str) -> (Vec<f32>, u32) {
        self.model
            .create(text, false, Some(1), None, None, None)
            .unwrap()
    }
}
