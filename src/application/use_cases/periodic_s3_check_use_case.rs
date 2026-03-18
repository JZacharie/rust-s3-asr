use crate::domain::ports::S3Port;
use crate::application::use_cases::process_video_use_case::ProcessVideoUseCase;
use anyhow::Result;
use std::sync::Arc;
use tracing::{info, error};

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
        let files = self.s3_port.list_files("").await?;
        info!("Found {} objects in S3 bucket", files.len());

        // 2. Filter for potential media files
        let media_files: Vec<String> = files.into_iter()
            .filter(|f| self.is_supported_media(f))
            .collect();

        info!("Identified {} potential media files to check", media_files.len());

        // 3. Process each media file
        // ProcessVideoUseCase already handles the "exists" check for .txt, 
        // but we can also do it here if we want to avoid redundant calls.
        // Let's rely on ProcessVideoUseCase's internal check for consistency.
        for file in media_files {
            let use_case = self.process_video_use_case.clone();
            let file_clone = file.clone();
            
            // We can spawn these or run them sequentially. 
            // For periodic check, sequential might be safer to avoid overloading, 
            // but let's do sequential for now.
            if let Err(e) = use_case.execute(&file_clone).await {
                error!("❌ Error during periodic processing of {}: {}", file_clone, e);
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
