// producer.rs
use common::entities::DicomObjectMeta;
use common::{server_config, DicomMessage};
use rdkafka::config::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use std::time::Duration;
use tracing::{debug, error, info};

pub struct KafkaProducer {
    producer: FutureProducer,
    pub(crate) topic: String,
}

impl KafkaProducer {}

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
            // --- 批量发送相关配置 ---
            .set(
                "queue.buffering.max.messages",
                kafka_config.queue_buffering_max_messages.to_string(),
            ) // producer 内部队列最大消息数
            .set(
                "queue.buffering.max.kbytes",
                kafka_config.queue_buffering_max_kbytes.to_string(),
            ) // 最大队列大小 (1GB)`
            .set(
                "queue.buffering.max.ms",
                kafka_config.queue_buffering_max_ms.to_string(),
            ) // 最大缓冲时间，100毫秒后强制发送
            .set(
                "batch.num.messages",
                kafka_config.batch_num_messages.to_string(),
            ) // 每个批次最大消息数
            .set("linger.ms", kafka_config.linger_ms.to_string()) // 等待更多消息以形成更大批次
            .set(
                "compression.codec",
                kafka_config.compression_codec.to_string(),
            ) // 启用压缩以减少网络开销
            // ---------------------------------------------
            .create()
            .expect("Failed to create Kafka producer");

        KafkaProducer {
            producer,
            topic: kafka_config.topic,
        }
    }

    pub(crate) async fn send_messages(
        &self,
        topic: &str,
        messages: &[DicomObjectMeta],
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Sending {} messages to topic {}", messages.len(), topic);
        for msg in messages {
            let payload = serde_json::to_vec(msg)?;
            let key = format!(
                "{}_{}",
                msg.patient_info.tenant_id, msg.image_info.sop_instance_uid
            ); // 使用 String 的引用
            let record: FutureRecord<String, Vec<u8>> =
                FutureRecord::to(topic).key(&key).payload(&payload);
            self.producer
                .send(record, Timeout::After(Duration::from_secs(1)))
                .await
                .map_err(|(e, _)| e)?;
        }
        self.producer.flush(Duration::from_secs(5))?;
        debug!("Flushed producer");
        Ok(())
    }

    // pub(crate) async fn send_message<T: Serialize>(
    //     &self,
    //     topic: &str,
    //     key: &str,
    //     payload: &T,
    // ) -> Result<(), Box<dyn std::error::Error>> {
    //     let payload_json = serde_json::to_string(payload)?;
    //     let delivery = self
    //         .producer
    //         .send(
    //             FutureRecord::to(topic).key(key).payload(&payload_json),
    //             Duration::from_secs(5),
    //         )
    //         .await;
    //
    //     match delivery {
    //         Ok(delivery) => println!("Sent: {:?}", delivery),
    //         Err((e, _)) => println!("Error: {:?}", e),
    //     }
    //     Ok(())
    // }
}
