use crate::domain::ports::S3Port;
use crate::application::use_cases::process_video_use_case::ProcessVideoUseCase;
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error, debug};

pub struct PeriodicS3CheckUseCase {
    s3_port: Arc<dyn S3Port>,
    process_video_use_case: Arc<ProcessVideoUseCase>,
}

impl PeriodicS3CheckUseCase {
    pub fn new(
        s3_port: Arc<dyn S3Port>,
        process_video_use_case: Arc<ProcessVideoUseCase>,
    ) -> Self {
        Self {
            s3_port,
            process_video_use_case,
        }
    }

    pub async fn execute(&self) -> Result<()> {
        info!("🔍 Starting periodic S3 bucket scan...");

        // 1. List all files in the bucket (empty prefix)
        debug!("Listing all files in bucket...");
        let files = self.s3_port.list_files("").await?;
        info!("📊 Found {} total objects in S3 bucket", files.len());

        // 2. Filter for potential media files
        let media_files: Vec<String> = files.into_iter()
            .filter(|f| self.is_supported_media(f))
            .collect();

        info!("🎯 Identified {} potential media files to check", media_files.len());

        // 3. Process each media file
        for (i, file) in media_files.iter().enumerate() {
            info!("[{}/{}] Checking media file: {}", i + 1, media_files.len(), file);
            
            let use_case = self.process_video_use_case.clone();
            if let Err(e) = use_case.execute(file).await {
                error!("❌ Error processing {}: {}", file, e);
            }
        }

        info!("✅ Periodic S3 bucket scan completed");
        Ok(())
    }

    fn is_supported_media(&self, filename: &str) -> bool {
        let lower = filename.to_lowercase();
        let extensions = [
            ".mp4", ".webm", ".mp3", ".wav", ".m4a", ".flac", ".ogg", ".mov", ".avi", ".mkv"
        ];
        
        // Ensure it's not a transcription file itself
        if lower.ends_with(".txt") {
            return false;
        }

        extensions.iter().any(|ext| lower.ends_with(ext))
    }
}
