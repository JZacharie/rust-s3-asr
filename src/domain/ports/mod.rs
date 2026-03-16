use async_trait::async_trait;
use anyhow::Result;

#[async_trait]
pub trait MqttPort: Send + Sync {
    async fn publish(&self, topic: &str, payload: &str) -> Result<()>;
    async fn subscribe(&self, topic: &str) -> Result<()>;
}

#[async_trait]
pub trait S3Port: Send + Sync {
    async fn download(&self, key: &str) -> Result<Vec<u8>>;
}

#[async_trait]
pub trait LlmPort: Send + Sync {
    async fn asr(&self, audio_data: Vec<u8>, filename: &str) -> Result<String>;
}
