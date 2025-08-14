use common::db_provider::DbProvider;
use common::entities::{DicomObjectMeta, ImageEntity, PatientEntity, SeriesEntity, StudyEntity};
use common::mysql_provider::MySqlProvider;
use common::server_config;
use futures::StreamExt;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use sqlx::MySqlPool;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Handle;
use tracing::log::error;

pub struct MessageDispatcher {
    consumer: StreamConsumer,
}

impl MessageDispatcher {
    pub async fn new() -> Self {
        let config = server_config::load_config();
        let config = match config {
            Ok(config) => config,
            Err(e) => {
                error!("{:?}", e);
                std::process::exit(-2);
            }
        };

        let mysql_url = match server_config::generate_database_connection(&config) {
            Ok(url) => url,
            Err(e) => {
                tracing::log::error!("{:?}", e);
                std::process::exit(-2);
            }
        };
        // 使用 url crate 正确解析包含特殊字符的URL
        tracing::info!("mysql_url: {}", mysql_url);
        let pool = MySqlPool::connect(mysql_url.as_str())
            .await
            .expect("Failed to connect to MySQL");
        let _db_provider = MySqlProvider { pool };

        let kafka_config_opt = config.kafka;
        let kafka_config = match kafka_config_opt {
            None => {
                tracing::log::error!("kafka config is None");
                std::process::exit(-2);
            }
            Some(kafka_config) => kafka_config,
        };

        // 配置消费者
        let consumer: StreamConsumer = ClientConfig::new()
            .set("group.id", kafka_config.consumer_group_id.as_str())
            .set("bootstrap.servers", kafka_config.brokers.as_str())
            .set("enable.auto.commit", "false")
            .set("auto.offset.reset", "earliest")
            .set("session.timeout.ms", "6000")
            .set("enable.partition.eof", "false")
            .create()
            .expect("create consumer failed");

        let topic = kafka_config.topic.as_str();
        tracing::info!("Subscribing to topic: {}", topic);

        match consumer.subscribe(&[topic]) {
            Ok(_) => println!("Successfully subscribed to topic: {}", topic),
            Err(e) => {
                eprintln!("Failed to subscribe to topic {}: {}", topic, e);
                std::process::exit(-1);
            }
        }

        Self { consumer }
    }

    pub async fn start_dispatch(&self) {
        let mut message_stream = self.consumer.stream();
        println!("Starting to read messages...");

        while let Some(result) = message_stream.next().await {
            match result {
                Ok(message) => {
                    match message.payload() {
                        Some(payload) => {
                            match serde_json::from_slice::<DicomObjectMeta>(payload) {
                                Ok(dicom_message) => {
                                    // 将消息添加到共享向量中
                                    println!("{:?}", dicom_message);
                                    // 处理成功后提交偏移量
                                    if let Err(e) =
                                        self.consumer.commit_message(&message, CommitMode::Sync)
                                    {
                                        eprintln!("Failed to commit message: {}", e);
                                    } else {
                                        println!("Successfully processed and committed message");
                                    }
                                }
                                Err(e) => {
                                    eprintln!("Failed to deserialize message: {}", e);
                                    // 反序列化失败也提交偏移量
                                    if let Err(e) =
                                        self.consumer.commit_message(&message, CommitMode::Sync)
                                    {
                                        eprintln!("Failed to commit message: {}", e);
                                    }
                                }
                            }
                        }
                        None => {
                            println!("Received message with no payload");
                            if let Err(e) = self.consumer.commit_message(&message, CommitMode::Sync)
                            {
                                eprintln!("Failed to commit message: {}", e);
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Error receiving message: {}", e);
                }
            }
        }
    }
}
