mod domain;
mod application;
mod infrastructure;

use anyhow::{Context, Result};
use application::use_cases::process_video_use_case::ProcessVideoUseCase;
use infrastructure::repositories::llm_repository::LlmRepository;
use infrastructure::repositories::mqtt_repository::MqttRepository;
use infrastructure::repositories::s3_repository::S3Repository;
use domain::ports::MqttPort;
use std::sync::Arc;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use rumqttc::{Event, Packet};


#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    info!("🚀 Starting Rust S3 ASR Application");

    // 2. Load configuration from environment
    dotenv::dotenv().ok();
    
    let mqtt_host = std::env::var("MQTT_HOST").unwrap_or_else(|_| "localhost".to_string());
    let mqtt_port = std::env::var("MQTT_PORT")
        .unwrap_or_else(|_| "1883".to_string())
        .parse::<u16>()?;
    let mqtt_input_topic = std::env::var("MQTT_INPUT_TOPIC").unwrap_or_else(|_| "input/video".to_string());
    let mqtt_output_topic = std::env::var("MQTT_OUTPUT_TOPIC").unwrap_or_else(|_| "output/transcription".to_string());
    
    let s3_bucket = std::env::var("S3_BUCKET").context("S3_BUCKET not set")?;
    let s3_endpoint = std::env::var("S3_ENDPOINT").unwrap_or_else(|_| "http://localhost:9000".to_string());
    let s3_region = std::env::var("S3_REGION").unwrap_or_else(|_| "us-east-1".to_string());
    let s3_access_key = std::env::var("S3_ACCESS_KEY").context("S3_ACCESS_KEY not set")?;
    let s3_secret_key = std::env::var("S3_SECRET_KEY").context("S3_SECRET_KEY not set")?;

    let llm_url = std::env::var("LLM_URL").unwrap_or_else(|_| "http://localhost:4000".to_string());
    let llm_api_key = std::env::var("LLM_API_KEY").ok();
    let llm_model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "whisper-1".to_string());

    // 3. Initialize S3 Client
    let credentials = aws_sdk_s3::config::Credentials::new(s3_access_key, s3_secret_key, None, None, "custom");
    let s3_config = aws_sdk_s3::config::Builder::new()
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::latest())
        .region(aws_sdk_s3::config::Region::new(s3_region))
        .endpoint_url(s3_endpoint)
        .credentials_provider(credentials)
        .force_path_style(true)
        .build();
    let s3_client = aws_sdk_s3::Client::from_conf(s3_config);
    let s3_repo = Arc::new(S3Repository::new(s3_client, s3_bucket));

    // 4. Initialize MQTT Repository
    let (mqtt_repo, mut eventloop) = MqttRepository::new(&mqtt_host, mqtt_port, "rust-s3-asr");
    let mqtt_repo = Arc::new(mqtt_repo);

    // 5. Initialize LLM Repository
    let llm_repo = Arc::new(LlmRepository::new(llm_url, llm_api_key, llm_model));

    // 6. Initialize Use Case
    let use_case = Arc::new(ProcessVideoUseCase::new(
        mqtt_repo.clone(),
        s3_repo,
        llm_repo,
        mqtt_output_topic,
    ));

    // 7. Subscribe to input topic
    mqtt_repo.subscribe(&mqtt_input_topic).await?;


    info!("📡 Listening for messages on: {}", mqtt_input_topic);

    // 8. Main Event Loop
    loop {
        match eventloop.poll().await {
            Ok(notification) => {
                if let Event::Incoming(Packet::Publish(publish)) = notification {
                    if publish.topic == mqtt_input_topic {
                        let payload = String::from_utf8_lossy(&publish.payload).to_string();
                        info!("🔔 Received trigger on topic {}: {}", publish.topic, payload);
                        
                        let use_case_clone = use_case.clone();
                        tokio::spawn(async move {
                            if let Err(e) = use_case_clone.execute(&payload).await {
                                error!("❌ Error processing video {}: {}", payload, e);
                            }
                        });
                    }
                }
            }
            Err(e) => {
                warn!("⚠️ MQTT connection error: {:?} — retrying in 5s", e);
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            }
        }
    }
}
