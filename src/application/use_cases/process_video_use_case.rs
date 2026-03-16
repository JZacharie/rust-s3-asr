use crate::domain::ports::{LlmPort, MqttPort, S3Port};
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error};

pub struct ProcessVideoUseCase {
    mqtt_port: Arc<dyn MqttPort>,
    s3_port: Arc<dyn S3Port>,
    llm_port: Arc<dyn LlmPort>,
    output_topic: String,
}

impl ProcessVideoUseCase {
    pub fn new(
        mqtt_port: Arc<dyn MqttPort>,
        s3_port: Arc<dyn S3Port>,
        llm_port: Arc<dyn LlmPort>,
        output_topic: String,
    ) -> Self {
        Self {
            mqtt_port,
            s3_port,
            llm_port,
            output_topic,
        }
    }

    #[tracing::instrument(skip(self), fields(s3_key = %s3_key))]
    pub async fn execute(&self, s3_key: &str) -> Result<()> {
        info!("🎬 Starting processing for video: {}", s3_key);

        // 1. Download from S3
        let video_data = match self.s3_port.download(s3_key).await {
            Ok(data) => {
                info!("✅ Successfully downloaded video: {} ({} bytes)", s3_key, data.len());
                data
            }
            Err(e) => {
                error!("❌ Failed to download video {}: {}", s3_key, e);
                return Err(e);
            }
        };

        // 2. Perform ASR using LiteLLM
        let transcription = match self.llm_port.asr(video_data, s3_key).await {
            Ok(text) => {
                info!("✅ ASR successful for {}", s3_key);
                text
            }
            Err(e) => {
                error!("❌ ASR failed for {}: {}", s3_key, e);
                return Err(e);
            }
        };

        // 3. Publish to MQTT
        match self.mqtt_port.publish(&self.output_topic, &transcription).await {
            Ok(_) => {
                info!("✅ Successfully published transcription for {} to topic {}", s3_key, self.output_topic);
            }
            Err(e) => {
                error!("❌ Failed to publish transcription for {} to topic {}: {}", s3_key, self.output_topic, e);
                return Err(e);
            }
        }

        Ok(())
    }
}
