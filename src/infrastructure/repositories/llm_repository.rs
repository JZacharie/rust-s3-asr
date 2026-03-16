use crate::domain::ports::LlmPort;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use reqwest::multipart;
use serde_json::Value;
use tracing::{info, debug};

pub struct LlmRepository {
    client: reqwest::Client,
    base_url: String,
    api_key: Option<String>,
    model: String,
}

impl LlmRepository {
    pub fn new(base_url: String, api_key: Option<String>, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url,
            api_key,
            model,
        }
    }
}

#[async_trait]
impl LlmPort for LlmRepository {
    async fn asr(&self, audio_data: Vec<u8>, filename: &str) -> Result<String> {
        let url = format!("{}/audio/transcriptions", self.base_url);
        info!("🎙️ Calling LiteLLM ASR at {} with model {}", url, self.model);
        
        let part = multipart::Part::bytes(audio_data)
            .file_name(filename.to_string())
            .mime_str("video/mp4")
            .context("Failed to create multipart part")?;

        let form = multipart::Form::new()
            .text("model", self.model.clone())
            .part("file", part);

        let mut request = self.client.post(&url).multipart(form);

        if let Some(ref key) = self.api_key {
            request = request.header("Authorization", format!("Bearer {}", key));
        }

        let response = request.send().await.context("Failed to send ASR request")?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(anyhow!("ASR API error {}: {}", status, text));
        }

        let json: Value = response.json().await.context("Failed to parse ASR response JSON")?;
        
        let text = json["text"]
            .as_str()
            .ok_or_else(|| anyhow!("No 'text' field in ASR response"))?
            .to_string();

        debug!("ASR result: {}", text);
        Ok(text)
    }
}
