use crate::domain::ports::S3Port;
use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_sdk_s3::Client;
use aws_sdk_s3::primitives::ByteStream;
use tracing::{info, debug};

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
    #[tracing::instrument(skip(self))]
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

    #[tracing::instrument(skip(self, data))]
    async fn upload(&self, key: &str, data: Vec<u8>) -> Result<()> {
        info!("📤 Uploading '{}' to bucket '{}' ({} bytes)", key, self.bucket, data.len());
        
        let body = ByteStream::from(data);
        self.client
            .put_object()
            .bucket(&self.bucket)
            .key(key)
            .body(body)
            .send()
            .await
            .with_context(|| format!("Failed to upload object '{}' to bucket '{}'", key, self.bucket))?;
            
        info!("✅ Uploaded {} successfully", key);
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn list_files(&self, prefix: &str) -> Result<Vec<String>> {
        debug!("🔍 Listing files in bucket '{}' with prefix '{}'", self.bucket, prefix);
        
        let resp = self.client
            .list_objects_v2()
            .bucket(&self.bucket)
            .prefix(prefix)
            .send()
            .await
            .with_context(|| format!("Failed to list objects in bucket '{}' with prefix '{}'", self.bucket, prefix))?;
            
        let keys = resp.contents()
            .iter()
            .filter_map(|obj| obj.key().map(|s| s.to_string()))
            .collect();
            
        Ok(keys)
    }

    #[tracing::instrument(skip(self))]
    async fn exists(&self, key: &str) -> Result<bool> {
        debug!("❓ Checking if '{}' exists in bucket '{}'", key, self.bucket);
        
        let result = self.client
            .head_object()
            .bucket(&self.bucket)
            .key(key)
            .send()
            .await;
            
        match result {
            Ok(_) => Ok(true),
            Err(e) => {
                let service_error = e.into_service_error();
                if service_error.is_not_found() {
                    Ok(false)
                } else {
                    Err(anyhow::anyhow!("Error checking existence of '{}': {}", key, service_error))
                }
            }
        }
    }
}
