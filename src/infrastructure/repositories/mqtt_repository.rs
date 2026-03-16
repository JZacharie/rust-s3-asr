use crate::domain::ports::MqttPort;
use anyhow::{Context, Result};
use async_trait::async_trait;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tracing::info;

pub struct MqttRepository {
    client: AsyncClient,
}

impl MqttRepository {
    pub fn new(host: &str, port: u16, client_id: &str, username: Option<String>, password: Option<String>) -> (Self, rumqttc::EventLoop) {
        let mut mqttoptions = MqttOptions::new(client_id, host, port);
        mqttoptions.set_keep_alive(Duration::from_secs(5));
        
        if let (Some(u), Some(p)) = (username, password) {
            mqttoptions.set_credentials(u, p);
        }
        
        let (client, eventloop) = AsyncClient::new(mqttoptions, 10);
        (Self { client }, eventloop)
    }
}

#[async_trait]
impl MqttPort for MqttRepository {
    async fn publish(&self, topic: &str, payload: &str) -> Result<()> {
        self.client
            .publish(topic, QoS::AtLeastOnce, false, payload.as_bytes().to_vec())
            .await
            .context("Failed to publish MQTT message")
    }

    async fn subscribe(&self, topic: &str) -> Result<()> {
        self.client
            .subscribe(topic, QoS::AtLeastOnce)
            .await
            .context("Failed to subscribe to MQTT topic")?;
        
        info!("📡 Subscribed to topic: {}", topic);
        Ok(())
    }
}
