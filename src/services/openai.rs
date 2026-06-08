use std::time::Instant;

use serde_json::json;

use crate::constants::SYSTEM_PROMPT;

pub struct OpenAiService {
    client: reqwest::blocking::Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiService {
    pub fn new(api_key: &str, model: &str, base_url: &str) -> Self {
        let client = reqwest::blocking::Client::new();

        Self {
            client,
            api_key: api_key.to_string(),
            model: model.to_string(),
            base_url: base_url.to_string(),
        }
    }

    pub fn get_completion(&self, prompt: &str) -> Option<String> {
        let instant = Instant::now();
        let payload = json!({
            "model": self.model,
            "messages": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user",   "content": prompt}
            ]
        });

        let url = format!("{}/chat/completions", self.base_url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .expect("[ERROR] failed to send request to OpenAI!");

        let json_res: serde_json::Value = response
            .json()
            .expect("[ERROR] failed to parse response from OpenAI!");

        log::debug!("[OPENAI] took {}ms", instant.elapsed().as_millis());

        json_res["choices"][0]["message"]["content"]
            .as_str()
            .map(|content| content.trim().to_string())
    }
}
