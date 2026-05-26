use std::time::Instant;

use serde_json::json;

use crate::constants::{OPENAI_MODEL, SYSTEM_PROMPT};

pub struct OpenAiService {
    client: reqwest::blocking::Client,
    api_key: String,
}

impl OpenAiService {
    pub fn new(api_key: &str) -> Self {
        let client = reqwest::blocking::Client::new();

        Self {
            client,
            api_key: api_key.to_string(),
        }
    }

    pub fn get_completion(&self, prompt: &str) -> Option<String> {
        let instant = Instant::now();
        let payload = json!({
            "model": OPENAI_MODEL,
            "messages": [{"role": "system", "content": SYSTEM_PROMPT}, { "role": "user", "content": prompt }]
        });

        let response = self
            .client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .unwrap();

        let json_res: serde_json::Value = response.json().unwrap();

        log::debug!("[OPENAI] took {}ms", instant.elapsed().as_millis());

        json_res["choices"][0]["message"]["content"]
            .as_str()
            .map(|content| content.trim().to_string())
    }
}
