use common::entities::DicomObjectMeta;
use common::{DicomMessage, server_config};
use futures::StreamExt;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::log::error;

// 使用内存存储已处理的消息ID（生产环境建议使用数据库或Redis）
type ProcessedMessages = Arc<Mutex<HashSet<String>>>;

#[tokio::main]
async fn main() {
    let config = server_config::load_config();
    let config = match config {
        Ok(config) => config,
        Err(e) => {
            error!("{:?}", e);
            std::process::exit(-2);
        }
    };

    let kafka_config_opt = config.kafka;
    let kafka_config = match kafka_config_opt {
        None => {
            error!("kafka config is None");
            std::process::exit(-2);
        }
        Some(kafka_config) => kafka_config,
    };

    let processed_messages: ProcessedMessages = Arc::new(Mutex::new(HashSet::new()));

    // 这三行配置组合起来的作用是：
    // 确保能消费到所有历史消息：通过 auto.offset.reset=earliest 保证即使新消费者也能读取历史数据
    // 合理的心跳检测：通过 session.timeout.ms=6000 在及时检测故障和避免误判之间取得平衡
    // 持续消费模式：通过 enable.partition.eof=false 确保消费者在没有新消息时继续等待而不是停止
    // 这种配置适用于需要处理所有历史消息且持续监听新消息的场景。
    //
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", kafka_config.consumer_group_id.as_str())
        .set("bootstrap.servers", kafka_config.brokers.as_str())
        .set("enable.auto.commit", "false") // 手动提交确保精确一次处理
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "6000")
        .set("enable.partition.eof", "false")
        .create()
        .expect("create consumer failed");

    let topic = kafka_config.topic.as_str();
    println!("Subscribing to topic: {}", topic);

    match consumer.subscribe(&[topic]) {
        Ok(_) => println!("Successfully subscribed to topic: {}", topic),
        Err(e) => {
            eprintln!("Failed to subscribe to topic {}: {}", topic, e);
            std::process::exit(-1);
        }
    }

    let mut message_stream = consumer.stream();

    while let Some(result) = message_stream.next().await {
        match result {
            Ok(message) => {
                match message.payload() {
                    Some(payload) => {
                        match serde_json::from_slice::<DicomObjectMeta>(payload) {
                            Ok(dicom_message) => {
                                // 处理消息（带幂等性检查）
                                if let Err(e) = process_dicom_message_with_idempotency(
                                    dicom_message,
                                    processed_messages.clone(),
                                )
                                .await
                                {
                                    eprintln!("Failed to process message: {}", e);
                                    // 处理失败时不提交偏移量，消息会重新消费
                                    continue;
                                }

                                // 处理成功后提交偏移量
                                if let Err(e) = consumer.commit_message(&message, CommitMode::Sync)
                                {
                                    eprintln!("Failed to commit message: {}", e);
                                } else {
                                    println!("Successfully processed and committed message");
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to deserialize message: {}", e);
                                // 反序列化失败也提交偏移量
                                if let Err(e) = consumer.commit_message(&message, CommitMode::Sync)
                                {
                                    eprintln!("Failed to commit message: {}", e);
                                }
                            }
                        }
                    }
                    None => {
                        println!("Received message with no payload");
                        if let Err(e) = consumer.commit_message(&message, CommitMode::Sync) {
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

async fn process_dicom_message_with_idempotency(
    message: DicomObjectMeta,
    processed_messages: ProcessedMessages,
) -> Result<(), Box<dyn std::error::Error>> {
    let message_id = format!(
        "{}_{}",
        message.patient_info.tenant_id, message.image_info.sop_instance_uid
    );

    // 检查是否已经处理过
    {
        let processed = processed_messages.lock().await;
        if processed.contains(&message_id) {
            println!("Message {} already processed, skipping", message_id);
            return Ok(());
        }
    }

    // 处理消息
    println!("Processing DICOM message: {:?}", message);

    // 模拟处理时间
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 标记为已处理
    {
        let mut processed = processed_messages.lock().await;
        processed.insert(message_id);
    }

    Ok(())
}
