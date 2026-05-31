use crate::constants::OPENAI_MODEL_DEFAULT;

/// Runtime configuration loaded from environment variables (or a `.env` file).
///
/// Required env vars:
///   BORIS_API_KEY — your OpenRouter API key
///
/// Optional env vars (defaults shown):
///   BORIS_MODEL   — LLM model slug (default: google/gemini-2.5-flash-preview)
///   BORIS_BASE_URL — API base URL (default: https://openrouter.ai/api/v1)
pub struct Config {
    pub api_key: String,
    pub model: String,
    pub base_url: String,
}

impl Config {
    /// Load config from environment variables.
    /// Reads a `.env` file in the working directory if present (ignored if missing).
    pub fn load() -> Self {
        // Load .env file if it exists — silently skip if not found.
        let _ = dotenvy::dotenv();

        let api_key = std::env::var("BORIS_API_KEY").unwrap_or_else(|_| {
            eprintln!(
                "\n[boris] ERROR: BORIS_API_KEY is not set.\n\
                 Copy .env.example to .env and fill in your OpenRouter API key.\n\
                 Get one free at https://openrouter.ai\n"
            );
            std::process::exit(1);
        });

        let model = std::env::var("BORIS_MODEL")
            .unwrap_or_else(|_| OPENAI_MODEL_DEFAULT.to_string());

        let base_url = std::env::var("BORIS_BASE_URL")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());

        Self {
            api_key,
            model,
            base_url,
        }
    }
}
