// producer.rs
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord};
use serde::Serialize;
use std::time::Duration;

pub struct KafkaProducer {
    producer: FutureProducer,
}

impl KafkaProducer {
    pub fn new(brokers: &str) -> Self {
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
                FutureRecord::to(topic)
                    .key(key)
                    .payload(&payload_json),
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