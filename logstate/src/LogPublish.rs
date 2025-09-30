use std::error::Error;
use rdkafka::ClientConfig;
use rdkafka::producer::FutureProducer;
use crate::LogEntity::{DicomIngestLog, StorageLog, TranscodeLog, WadoAccessLog};

/// DICOM 文件接收、转码、存储、WADO 访问日志
pub const LOG_DICOM_INGEST: &str = "dicom_storescp";
/// 转码日志  DICOM文件转码记录.
pub const LOG_TRANSCODE: &str = "dicom_transcode";
/// TAG写入DB
pub const LOG_STORAGE: &str = "dicom_meta_persist";
/// WADO 访问日志
pub const LOG_WADO_ACCESS: &str = "wado_access";
//     pub client_ip: String,                           // 访问客户端 IP

pub trait LogPublisher {
    fn publish_dicom_storage(&self, topic:&str, log: &[DicomIngestLog]) -> Result<(), Box<dyn Error>>;
    fn publish_transcode(&self, topic:&str, log: &[TranscodeLog]) -> Result<(), Box<dyn Error>>;
    fn publish_wado_access(&self, topic:&str, log: &[WadoAccessLog]) -> Result<(), Box<dyn Error>>;
    fn publish_storage(&self, topic:&str, log: &[StorageLog]) -> Result<(), Box<dyn Error>>;
}
pub struct LogPulish {
    pub broker: String,
    // 消息超时时间(毫秒)
    pub message_timeout_ms: u32,
    // Producer内部队列最大消息数
    pub queue_buffering_max_messages: u32,
    // 最大队列大小(KB)
    pub queue_buffering_max_kbytes: u32,
    // 最大缓冲时间(毫秒)
    pub queue_buffering_max_ms: u32,
    // 每个批次最大消息数
    pub batch_num_messages: u32,
    // 等待更多消息以形成更大批次的时间(毫秒)
    pub linger_ms: u32,
    // 压缩编解码器
    pub compression_codec: String,
    // 重试次数
    pub retries: u32,
    // 重试退避时间(毫秒)
    pub retry_backoff_ms: u32,
}
impl LogPulish {
    pub fn new(broker: String) -> Self {
        Self {
            broker,
            message_timeout_ms: 30000,
            queue_buffering_max_messages: 100000,
            queue_buffering_max_kbytes: 1048576, // 1GB
            queue_buffering_max_ms: 100,
            batch_num_messages: 1000,
            linger_ms: 5,
            compression_codec: "none".to_string(),
            retries: 5,
            retry_backoff_ms: 1000,
        }
    }
    // 可以添加一个带配置的构造函数
    #[allow(dead_code)]
    pub fn from_config(
        broker: String,
        message_timeout_ms: u32,
        queue_buffering_max_messages: u32,
        queue_buffering_max_kbytes: u32,
        queue_buffering_max_ms: u32,
        batch_num_messages: u32,
        linger_ms: u32,
        compression_codec: String,
        retries: u32,
        retry_backoff_ms: u32,
    ) -> Self {
        Self {
            broker,
            message_timeout_ms,
            queue_buffering_max_messages,
            queue_buffering_max_kbytes,
            queue_buffering_max_ms,
            batch_num_messages,
            linger_ms,
            compression_codec,
            retries,
            retry_backoff_ms,
        }
    }

    pub fn create_producer(&self) -> Result<FutureProducer, Box<dyn Error>>  {
        let producer =  ClientConfig::new()
            .set("bootstrap.servers", &self.broker)
            .set("message.timeout.ms", &self.message_timeout_ms.to_string())
            .set("queue.buffering.max.messages", &self.queue_buffering_max_messages.to_string())
            .set("queue.buffering.max.kbytes", &self.queue_buffering_max_kbytes.to_string())
            .set("queue.buffering.max.ms", &self.queue_buffering_max_ms.to_string())
            .set("batch.num.messages", &self.batch_num_messages.to_string())
            .set("linger.ms", &self.linger_ms.to_string())
            .set("compression.codec", &self.compression_codec)
            .set("retries", &self.retries.to_string())
            .set("retry.backoff.ms", &self.retry_backoff_ms.to_string())
            .create()
            .map_err(|e| Box::new(e) as Box<dyn Error>)?;
        Ok(producer)
    }
}