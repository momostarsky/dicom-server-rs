use crate::database_entities::DicomObjectMeta;
use crate::server_config;
use futures::future::join_all;
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use std::collections::HashMap;
use std::time::Duration;
use tracing::{debug, error, info};

pub struct KafkaProducer {
    producer: FutureProducer,
    pub(crate) topic: String,
}

impl KafkaProducer {
    pub async fn send_message(
        &self,

        msg: &DicomObjectMeta,
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!("Sending message to topic {}", self.topic);
        let payload = serde_json::to_vec(msg)?;
        let key = format!(
            "{}_{}",
            msg.patient_info.tenant_id, msg.image_info.sop_instance_uid
        ); // 使用 String 的引用
        let record: FutureRecord<String, Vec<u8>> =
            FutureRecord::to(&*self.topic).key(&key).payload(&payload);
        self.producer
            .send(record, Timeout::After(Duration::from_secs(1)))
            .await
            .map_err(|(e, _)| e)?;
        self.producer.flush(Duration::from_micros(500))?;
        debug!("Flushed producer");
        Ok(())
    }

    pub async fn send_messages(
        &self,
        messages: &[DicomObjectMeta],
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Sending {} messages to topic {}",
            messages.len(),
            self.topic
        );

        for msg in messages {
            let payload = serde_json::to_vec(msg)?;
            let key = format!(
                "{}_{}",
                msg.patient_info.tenant_id, msg.image_info.sop_instance_uid
            ); // 使用 String 的引用
            let record: FutureRecord<String, Vec<u8>> =
                FutureRecord::to(&*self.topic).key(&key).payload(&payload);
            self.producer
                .send(record, Timeout::After(Duration::from_secs(1)))
                .await
                .map_err(|(e, _)| e)?;
        }
        self.producer.flush(Duration::from_micros(500))?;
        debug!("Flushed producer");
        Ok(())
    }

    pub  async fn send_batch_messages(
        &self,

        messages: &[DicomObjectMeta],
    ) -> Result<(), Box<dyn std::error::Error>> {
        info!(
            "Sending {} messages in batch to topic {}",
            messages.len(),
            self.topic
        );

        let mut wait_message = HashMap::new();
        // 预先创建所有消息的数据（Arc 包装）

        for msg in messages {
            match serde_json::to_vec(msg) {
                Ok(payload) => {
                    let key = format!(
                        "{}_{}",
                        msg.patient_info.tenant_id, msg.image_info.sop_instance_uid
                    );
                    wait_message.insert(key.clone(), payload.clone());
                }
                Err(e) => return Err(Box::new(e)),
            }
        }

        // 创建所有Future

        // 创建所有Future
        let futures: Vec<_> = wait_message
            .iter() // 注意这里要用 iter()，保证 Arc 存活
            .map(|(key, payload)| {
                let record = FutureRecord::to(&*self.topic)
                    .key(&key[..])
                    .payload(&payload[..]);
                self.producer
                    .send(record, Timeout::After(Duration::from_secs(1)))
            })
            .collect();

        // 并发等待所有消息发送完成
        let results = join_all(futures).await;

        // 检查发送结果
        let mut success_count = 0;
        let mut error_count = 0;

        for result in results {
            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    eprintln!("Failed to send message: {:?}", e);
                    error_count += 1;
                }
            }
        }

        println!(
            "✅ 批量发送完成: 成功 {} 条, 失败 {} 条",
            success_count, error_count
        );

        Ok(())
    }
}

fn get_kafka_config() -> server_config::KafkaConfig {
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
    config.kafka.unwrap()
}
pub fn create_main_kafka_producer() -> KafkaProducer {
    let kafka_config = get_kafka_config();
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
        topic: kafka_config.topic_main,
    }
}

pub fn create_extract_frames_kafka_producer() -> KafkaProducer {
    let kafka_config = get_kafka_config();
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
        topic: kafka_config.topic_extract_dicom,
    }
}

pub fn create_change_transfersyntax_kafka_producer() -> KafkaProducer {
    let kafka_config = get_kafka_config();
    let brokers = kafka_config.brokers; // 创建一个 Kafka 生产者
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
        topic: kafka_config.topic_change_transfer_syntax,
    }
}
