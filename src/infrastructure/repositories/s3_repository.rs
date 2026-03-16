use crate::domain::ports::S3Port;
use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_sdk_s3::Client;
use tracing::info;

pub struct S3Repository {
    client: Client,
    bucket: String,
}

impl S3Repository {
    pub fn new(client: Client, bucket: String) -> Self {
        Self { client, bucket }
    }
}

#[async_trait]
impl S3Port for S3Repository {
    async fn download(&self, key: &str) -> Result<Vec<u8>> {
        info!("📥 Downloading '{}' from bucket '{}'", key, self.bucket);
        
        let resp = self.client
            .get_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await
            .with_context(|| format!("Failed to get object '{}' from bucket '{}'", key, self.bucket))?;
            
        let data = resp.body.collect().await.with_context(|| format!("Failed to collect bytes for object '{}'", key))?;
        let bytes = data.into_bytes().to_vec();
        info!("✅ Downloaded {} bytes for {}", bytes.len(), key);
        Ok(bytes)
    }
}
