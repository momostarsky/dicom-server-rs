 
pub use crate::message_sender::MessagePublisher;
use crate::server_config;
use async_trait::async_trait;
use futures_util::future::join_all;
use rdkafka::ClientConfig;
use rdkafka::producer::{FutureProducer, FutureRecord, Producer};
use rdkafka::util::Timeout;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tracing::{debug, error, info};
use database::dicom_meta::{DicomImageMeta, DicomStateMeta, DicomStoreMeta};
use crate::logevents::ApiLogEvent;

pub struct KafkaMessagePublisher {
    producer: Arc<FutureProducer>,
    topic: String,
}
impl KafkaMessagePublisher {
    pub fn topic(&self) -> &str {
        &self.topic
    }
    pub fn new(topic_name: String) -> Self {
        let app_config = server_config::load_config().expect("Failed to load config");
        let config = app_config.kafka;

        let brokers = config.brokers;
        let producer = ClientConfig::new()
            .set("bootstrap.servers", brokers)
            .set("message.timeout.ms", "30000") // 增加消息超时时间到30秒
            // --- 批量发送相关配置 ---
            .set(
                "queue.buffering.max.messages",
                config.queue_buffering_max_messages.to_string(),
            ) // producer 内部队列最大消息数
            .set(
                "queue.buffering.max.kbytes",
                config.queue_buffering_max_kbytes.to_string(),
            ) // 最大队列大小 (1GB)`
            .set(
                "queue.buffering.max.ms",
                config.queue_buffering_max_ms.to_string(),
            ) // 最大缓冲时间，100毫秒后强制发送
            .set("batch.num.messages", config.batch_num_messages.to_string()) // 每个批次最大消息数
            .set("linger.ms", config.linger_ms.to_string()) // 等待更多消息以形成更大批次
            .set("compression.codec", config.compression_codec.to_string()) // 启用压缩以减少网络开销
            // 添加重试机制
            .set("retries", "5")
            .set("retry.backoff.ms", "1000")
            .create()
            .expect("Failed to create KafkaMessagePublisher");
        Self {
            producer: Arc::new(producer),
            topic: topic_name,
        }
    }
}
impl Clone for KafkaMessagePublisher {
    fn clone(&self) -> Self {
        KafkaMessagePublisher {
            producer: Arc::clone(&self.producer),
            topic: self.topic.clone(),
        }
    }
}

#[async_trait]
impl MessagePublisher for KafkaMessagePublisher {
    async fn send_message(&self, msg: &DicomStoreMeta) -> Result<(), Box<dyn Error>> {
        info!(
            "KafkaMessagePublisher send_message to topic: {}",
            self.topic
        );
        let payload = serde_json::to_vec(msg)?;
        let key = String::from(msg.trace_id.as_str());
        let record: FutureRecord<String, Vec<u8>> =
            FutureRecord::to(&*self.topic).key(&key).payload(&payload);

        match self
            .producer
            .send(record, Timeout::After(Duration::from_secs(10))) // 增加超时时间到10秒
            .await
            .map_err(|(e, _)| e)
        {
            Ok(_) => {
                self.producer.flush(Duration::from_micros(500))?;
                debug!("Flushed KafkaMessagePublisher");
                Ok(())
            }
            Err(e) => {
                error!("Failed to send message to Kafka: {:?}", e);
                Err(Box::new(e))
            }
        }
    }

    async fn send_batch_messages(&self, messages: &[DicomStoreMeta]) -> Result<(), Box<dyn Error>> {
        info!(
            "KafkaMessagePublisher send_batch_messages: {}  to topic {}",
            messages.len(),
            self.topic
        );

        let mut wait_message = HashMap::new();

        for msg in messages {
            match serde_json::to_vec(msg) {
                Ok(payload) => {
                    let key = String::from(msg.trace_id.as_str());
                    wait_message.insert(key.clone(), payload.clone());
                }
                Err(e) => {
                    error!("Failed to serialize message: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        }

        let futures: Vec<_> = wait_message
            .iter()
            .map(|(key, payload)| {
                let record = FutureRecord::to(&*self.topic)
                    .key(&key[..])
                    .payload(&payload[..]);
                // 增加超时时间到10秒
                self.producer
                    .send(record, Timeout::After(Duration::from_secs(10)))
            })
            .collect();

        let results = join_all(futures).await;

        let mut success_count = 0;
        let mut error_count = 0;

        for result in results {
            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error!("Failed to send message: {:?}", e);
                    error_count += 1;
                }
            }
        }

        info!(
            "✅ 批量发送完成: 成功 {} 条, 失败 {} 条",
            success_count, error_count
        );

        // 如果有错误，返回错误信息
        if error_count > 0 {
            Err("Some messages failed to send".into())
        } else {
            Ok(())
        }
    }

    async fn send_state_messages(&self, messages: &[DicomStateMeta]) -> Result<(), Box<dyn Error>> {
        info!(
            "KafkaMessagePublisher send_state_messages: {} to topic {}",
            messages.len(),
            self.topic
        );

        let mut wait_message = HashMap::new();

        for msg in messages {
            match serde_json::to_vec(msg) {
                Ok(payload) => {
                    // 使用 tenant_id + patient_id + study_uid + series_uid 作为唯一键
                    // 使用哈希值作为消息键，避免键过长
                    let key_source = format!("{}_{}_{}_{}",
                                             msg.tenant_id.as_str(),
                                             msg.patient_id.as_str(),
                                             msg.study_uid.as_str(),
                                             msg.series_uid.as_str());
                    let key = format!("{:x}", md5::compute(key_source));
                    wait_message.insert(key, payload);
                }
                Err(e) => {
                    error!("Failed to serialize DicomStateMeta message: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        }

        let futures: Vec<_> = wait_message
            .iter()
            .map(|(key, payload)| {
                let record = FutureRecord::to(&*self.topic)
                    .key(&key[..])
                    .payload(&payload[..]);
                // 增加超时时间到10秒
                self.producer
                    .send(record, Timeout::After(Duration::from_secs(10)))
            })
            .collect();

        let results = join_all(futures).await;

        let mut success_count = 0;
        let mut error_count = 0;

        for result in results {
            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error!("Failed to send DicomStateMeta message: {:?}", e);
                    error_count += 1;
                }
            }
        }

        info!(
            "✅ 批量发送 DicomStateMeta 完成: 成功 {} 条, 失败 {} 条",
            success_count, error_count
        );

        // 如果有错误，返回错误信息
        if error_count > 0 {
            Err("Some DicomStateMeta messages failed to send".into())
        } else {
            Ok(())
        }
    }

    async fn send_image_messages(&self, messages: &[DicomImageMeta]) -> Result<(), Box<dyn Error>> {
        info!(
            "KafkaMessagePublisher send_image_messages: {} to topic {}",
            messages.len(),
            self.topic
        );

        let mut wait_message = HashMap::new();

        for msg in messages {
            match serde_json::to_vec(msg) {
                Ok(payload) => {
                    // 使用 tenant_id + patient_id + study_uid + series_uid + sop_uid 作为唯一键
                    let key_source = format!(
                        "{}_{}_{}_{}_{}",
                        msg.tenant_id.as_str(),
                        msg.patient_id.as_str(),
                        msg.study_uid.as_str(),
                        msg.series_uid.as_str(),
                        msg.sop_uid.as_str()
                    );
                    let key = format!("{:x}", md5::compute(key_source));
                    wait_message.insert(key, payload);
                }
                Err(e) => {
                    error!("Failed to serialize DicomImageMeta message: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        }

        let futures: Vec<_> = wait_message
            .iter()
            .map(|(key, payload)| {
                let record = FutureRecord::to(&*self.topic)
                    .key(&key[..])
                    .payload(&payload[..]);
                // 增加超时时间到10秒
                self.producer
                    .send(record, Timeout::After(Duration::from_secs(10)))
            })
            .collect();

        let results = join_all(futures).await;

        let mut success_count = 0;
        let mut error_count = 0;

        for result in results {
            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error!("Failed to send DicomImageMeta message: {:?}", e);
                    error_count += 1;
                }
            }
        }

        info!(
            "✅ 批量发送 DicomImageMeta 完成: 成功 {} 条, 失败 {} 条",
            success_count, error_count
        );

        // 如果有错误，返回错误信息
        if error_count > 0 {
            Err("Some DicomImageMeta messages failed to send".into())
        } else {
            Ok(())
        }
    }

    async fn send_webapi_messages(&self, messages: &[ApiLogEvent]) -> Result<(), Box<dyn Error>> {
        info!(
            "KafkaMessagePublisher send_api_log_messages: {} to topic {}",
            messages.len(),
            self.topic
        );

        let mut wait_message = HashMap::new();

        for msg in messages {
            match serde_json::to_vec(msg) {
                Ok(payload) => {
                    let key = msg.request_id.as_str();
                    wait_message.insert(key, payload);
                }
                Err(e) => {
                    error!("Failed to serialize DicomImageMeta message: {:?}", e);
                    return Err(Box::new(e));
                }
            }
        }

        let futures: Vec<_> = wait_message
            .iter()
            .map(|(key, payload)| {
                let record = FutureRecord::to(&*self.topic)
                    .key(&key[..])
                    .payload(&payload[..]);
                // 增加超时时间到10秒
                self.producer
                    .send(record, Timeout::After(Duration::from_secs(10)))
            })
            .collect();

        let results = join_all(futures).await;

        let mut success_count = 0;
        let mut error_count = 0;

        for result in results {
            match result {
                Ok(_) => success_count += 1,
                Err(e) => {
                    error!("Failed to send ApiLogEvent message: {:?}", e);
                    error_count += 1;
                }
            }
        }

        info!(
            "✅ 批量发送 ApiLogEvent 完成: 成功 {} 条, 失败 {} 条",
            success_count, error_count
        );

        // 如果有错误，返回错误信息
        if error_count > 0 {
            Err("Some ApiLogEvent messages failed to send".into())
        } else {
            Ok(())
        }
    }
    // fn clone_box(&self) -> Box<dyn MessagePublisher> {
    //     Box::new(self.clone())
    // }
}
