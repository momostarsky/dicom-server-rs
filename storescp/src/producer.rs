// producer.rs
use common::server_config;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use std::time::Duration;
use tracing::error;

pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new() -> Self {
        let config = server_config::load_config();
        let config = match config {
            Ok(config) => config,
            Err(e) => {
                error!("{:?}", e);
                std::process::exit(-2);
            }
        };

        if config.kafka.is_none() {
            error!("kafka config is not found");
            std::process::exit(-2);
        }
        let kafka_config = config.kafka.unwrap();
        let brokers = kafka_config.brokers;
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "1000")
            .create()
            .expect("Failed to create Kafka producer");

        KafkaProducer { producer }
    }

    pub async fn send_message<T: Serialize>(
        &self,
        topic: &str,
        key: &str,
        payload: &T,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let payload_json = serde_json::to_string(payload)?;
        let delivery = self
            .producer
            .send(
                FutureRecord::to(topic).key(key).payload(&payload_json),
                Duration::from_secs(5),
            )
            .await;

        match delivery {
            Ok(delivery) => println!("Sent: {:?}", delivery),
            Err((e, _)) => println!("Error: {:?}", e),
        }
        Ok(())
    }
}
