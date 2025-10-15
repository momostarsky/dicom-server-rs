use common::utils::process_storage_messages;
use common::{database_factory, server_config};
use futures::StreamExt;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use slog;
use slog::{Logger, error, info};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Handle;
use common::dicom_object_meta::DicomStoreMeta;

// 全局logger静态变量，使用线程安全的方式
static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

// 设置全局logger
pub fn set_global_logger(logger: Logger) {
    let _ = GLOBAL_LOGGER.set(logger);
}

// 获取全局logger
fn get_logger() -> &'static Logger {
    GLOBAL_LOGGER.get().expect("Logger not initialized")
}

pub async fn start_process(logger: &Logger) {
    // 设置全局logger
    set_global_logger(logger.clone());

    // 使用全局logger
    let global_logger = get_logger();

    // 设置日志系统
    info!(global_logger, "start process");

    let config = server_config::load_config();
    let config = match config {
        Ok(config) => config,
        Err(e) => {
            error!(global_logger, "load config failed: {:?}", e);
            std::process::exit(-2);
        }
    };

    let db_provider = match database_factory::create_db_instance().await {
        Some(provider) => provider,
        None => {
            error!(
                global_logger,
                "Failed to create database provider: provider is None"
            );
            std::process::exit(-2);
        }
    };

    match db_provider.echo().await {
        Ok(msg) => info!(global_logger, "Database provider echo: {}", msg),
        Err(e) => {
            error!(global_logger, "Failed to echo from database provider: {}", e);
            std::process::exit(-2);
        }
    }

    let kafka_config = config.kafka;

    let queue_config = config.message_queue;

    // 配置消费者
    let consumer: StreamConsumer = ClientConfig::new()
        .set("group.id", queue_config.consumer_group_id.as_str())
        .set("bootstrap.servers", kafka_config.brokers.as_str())
        .set("enable.auto.commit", "false")
        .set("auto.offset.reset", "earliest")
        .set("session.timeout.ms", "6000")
        .set("enable.partition.eof", "false")
        .create()
        .expect("create consumer-storage-kafka failed");

    let topic = queue_config.topic_main.as_str();
    info!(global_logger, "Subscribing to topic: {}", topic);

    match consumer.subscribe(&[topic]) {
        Ok(_) => info!(global_logger, "Successfully subscribed to topic: {}", topic),
        Err(e) => {
            error!(global_logger, "Failed to subscribe to topic {}: {}", topic, e);
            std::process::exit(-1);
        }
    }

    // 创建一个线程安全的共享 Vec 和时间戳
    let shared_vec = Arc::new(Mutex::new(Vec::new()));
    let last_process_time = Arc::new(Mutex::new(Instant::now()));

    // 克隆 Arc 供不同任务使用
    let vec_for_reader = Arc::clone(&shared_vec);
    let vec_for_writer = Arc::clone(&shared_vec);
    let time_for_writer = Arc::clone(&last_process_time);

    // 获取当前的 Tokio 运行时句柄
    let handle = Handle::current();
    let handle_for_reader = handle.clone();
    let handle_for_writer = handle.clone();

    // 启动消息读取任务（在新线程中运行异步代码）
    let reader_thread = thread::spawn(move || {
        handle_for_reader.block_on(async {
            read_message(consumer, vec_for_reader, last_process_time).await;
        });
    });

    // 启动消息处理任务（在新线程中运行异步代码）
    let writer_thread = thread::spawn(move || {
        handle_for_writer.block_on(async {
            persist_message_loop(vec_for_writer, time_for_writer).await;
        });
    });

    // 等待两个线程完成
    let reader_result = reader_thread.join();
    let writer_result = writer_thread.join();

    match reader_result {
        Ok(_) => info!(global_logger, "Reader thread completed successfully"),
        Err(e) => error!(global_logger, "Reader thread panicked: {:?}", e),
    }

    match writer_result {
        Ok(_) => info!(global_logger, "Writer thread completed successfully"),
        Err(e) => error!(global_logger, "Writer thread panicked: {:?}", e),
    }

    // 主线程查看最终结果
    let final_vec = shared_vec.lock().unwrap();
    info!(global_logger, "Final Vec: {:?}", *final_vec);
}

async fn read_message(
    consumer: StreamConsumer,
    vec: Arc<Mutex<Vec<DicomStoreMeta>>>,
    last_process_time: Arc<Mutex<Instant>>,
) {
    let logger = get_logger();
    let mut message_stream = consumer.stream();
    info!(logger, "Starting to read messages...");

    while let Some(result) = message_stream.next().await {
        match result {
            Ok(message) => {
                match message.payload() {
                    Some(payload) => {
                        match serde_json::from_slice::<DicomStoreMeta>(payload) {
                            Ok(dicom_message) => {
                                // 将消息添加到共享向量中
                                {
                                    let mut vec = vec.lock().unwrap();
                                    vec.push(dicom_message);
                                    // 更新最后处理时间
                                    let mut time = last_process_time.lock().unwrap();
                                    *time = Instant::now();
                                }

                                // 处理成功后提交偏移量
                                if let Err(e) = consumer.commit_message(&message, CommitMode::Sync)
                                {
                                    error!(logger, "Failed to commit message: {}", e);
                                } else {
                                    info!(logger, "Successfully processed and committed message:{}", message.offset());
                                }
                            }
                            Err(e) => {
                                error!(logger, "Failed to deserialize message: {}", e);
                                // 反序列化失败也提交偏移量
                                if let Err(e) = consumer.commit_message(&message, CommitMode::Sync)
                                {
                                    error!(logger, "Failed to commit message: {}", e);
                                }
                            }
                        }
                    }
                    None => {
                        error!(logger, "Received message with no payload");
                        if let Err(e) = consumer.commit_message(&message, CommitMode::Sync) {
                            error!(logger, "Failed to commit message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!(logger, "Error receiving message: {}", e);
            }
        }
    }
}

static MAX_MESSAGES_PER_BATCH: usize = 50;
static MAX_TIME_BETWEEN_BATCHES: Duration = Duration::from_secs(5);
async fn persist_message_loop(
    vec: Arc<Mutex<Vec<DicomStoreMeta>>>,
    last_process_time: Arc<Mutex<Instant>>,
) {
    let logger = get_logger();
    info!(logger, "Starting message persistence loop...");

    loop {
        let should_process = {
            let vec = vec.lock().unwrap();
            let time = last_process_time.lock().unwrap();

            // 检查是否满足处理条件：
            // 1. 队列中有消息且数量>=100
            // 2. 队列中有消息且距离上次处理超过10秒
            let queue_size = vec.len();
            let time_since_last_process = Instant::now().duration_since(*time);

            (queue_size > 0 && queue_size >= MAX_MESSAGES_PER_BATCH)
                || (queue_size > 0 && time_since_last_process >= MAX_TIME_BETWEEN_BATCHES)
        };
        if !should_process {
            // 休眠一段时间，等待下一次处理
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }

        // 批量处理消息
        let messages_to_process = {
            let mut vec = vec.lock().unwrap();
            let mut messages = Vec::new();

            // 取出所有消息或最多100条消息进行处理
            let take_count = vec.len().min(MAX_MESSAGES_PER_BATCH);
            for _ in 0..take_count {
                if let Some(msg) = vec.pop() {
                    messages.push(msg);
                }
            }

            messages
        };

        if messages_to_process.is_empty() {
            // 休眠一段时间，等待下一次处理
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }

        // 创建数据库提供者
        let db_provider = match database_factory::create_db_instance().await {
            Some(provider) => provider,
            None => {
                error!(
                    logger,
                    "Failed to create database provider: provider is None"
                );
                // 休眠一段时间，等待下一次处理
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };
        match process_storage_messages(&messages_to_process, &db_provider).await {
            Ok(_) => {
                info!(logger, "Successfully processed batch of {} messages", messages_to_process.len());

            }
            Err(e) => {
                error!(logger, "Failed to process storage messages: {}", e);
                // 休眠一段时间，等待下一次处理
                tokio::time::sleep(Duration::from_secs(5)).await;
                continue;
            }
        }
        // 更新最后处理时间
        {
            let mut time = last_process_time.lock().unwrap();
            *time = Instant::now();
        }

        info!(
            logger,
            "Successfully processed batch of {} messages",
            messages_to_process.len()
        );
    }
}
