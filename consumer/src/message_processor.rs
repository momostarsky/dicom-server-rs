use common::database_entities::DicomObjectMeta;
use common::utils::{get_unique_tenant_ids, group_dicom_messages};
use common::{database_factory, server_config};
use futures::StreamExt;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Handle;
use tracing::log::error;

pub async fn start_process() {
    // 设置日志系统
    tracing::info!("start process");

    let config = server_config::load_config();
    let config = match config {
        Ok(config) => config,
        Err(e) => {
            error!("{:?}", e);
            std::process::exit(-2);
        }
    };

    let db_provider = match database_factory::create_db_instance().await {
        Some(provider) => provider,
        None => {
            tracing::error!("Failed to create database provider: provider is None");
            std::process::exit(-2);
        }
    };

    match db_provider.echo().await {
        Ok(msg) => tracing::info!("Database provider echo: {}", msg),
        Err(e) => {
            tracing::error!("Failed to echo from database provider: {}", e);
            std::process::exit(-2);
        }
    }

    let kafka_config_opt = config.kafka;
    let kafka_config = match kafka_config_opt {
        None => {
            error!("kafka config is None");
            std::process::exit(-2);
        }
        Some(kafka_config) => kafka_config,
    };

    let queue_config_opt = config.message_queue;
    let queue_config = match queue_config_opt {
        None => {
            error!("message queue config is None");
            std::process::exit(-2);
        }
        Some(queue_config) => queue_config,
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

    let topic = queue_config.topic_main.as_str();
    tracing::info!("Subscribing to topic: {}", topic);

    match consumer.subscribe(&[topic]) {
        Ok(_) => tracing::info!("Successfully subscribed to topic: {}", topic),
        Err(e) => {
            tracing::error!("Failed to subscribe to topic {}: {}", topic, e);
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
        Ok(_) => tracing::info!("Reader thread completed successfully"),
        Err(e) => tracing::error!("Reader thread panicked: {:?}", e),
    }

    match writer_result {
        Ok(_) => tracing::info!("Writer thread completed successfully"),
        Err(e) => tracing::error!("Writer thread panicked: {:?}", e),
    }

    // 主线程查看最终结果
    let final_vec = shared_vec.lock().unwrap();
    tracing::info!("Final Vec: {:?}", *final_vec);
}

async fn read_message(
    consumer: StreamConsumer,
    vec: Arc<Mutex<Vec<DicomObjectMeta>>>,
    last_process_time: Arc<Mutex<Instant>>,
) {
    let mut message_stream = consumer.stream();
    tracing::info!("Starting to read messages...");

    while let Some(result) = message_stream.next().await {
        match result {
            Ok(message) => {
                match message.payload() {
                    Some(payload) => {
                        match serde_json::from_slice::<DicomObjectMeta>(payload) {
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
                                    tracing::error!("Failed to commit message: {}", e);
                                } else {
                                    tracing::debug!("Successfully processed and committed message");
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to deserialize message: {}", e);
                                // 反序列化失败也提交偏移量
                                if let Err(e) = consumer.commit_message(&message, CommitMode::Sync)
                                {
                                    tracing::error!("Failed to commit message: {}", e);
                                }
                            }
                        }
                    }
                    None => {
                        tracing::warn!("Received message with no payload");
                        if let Err(e) = consumer.commit_message(&message, CommitMode::Sync) {
                            tracing::error!("Failed to commit message: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Error receiving message: {}", e);
            }
        }
    }
}

static MAX_MESSAGES_PER_BATCH: usize = 50;
static MAX_TIME_BETWEEN_BATCHES: Duration = Duration::from_secs(5);
async fn persist_message_loop(
    vec: Arc<Mutex<Vec<DicomObjectMeta>>>,
    last_process_time: Arc<Mutex<Instant>>,
) {
    tracing::info!("Starting message persistence loop...");

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
        tracing::info!("Processing batch of {} messages", messages_to_process.len());

        // 获取所有不同的 tenant_id
        let unique_tenant_ids = get_unique_tenant_ids(&messages_to_process);
        tracing::info!("Processing data for tenant IDs: {:?}", unique_tenant_ids);

        // 创建数据库提供者
        let db_provider = match database_factory::create_db_instance().await {
            Some(provider) => provider,
            None => {
                tracing::error!("Failed to create database provider: provider is None");
                // 休眠一段时间，等待下一次处理
                tokio::time::sleep(Duration::from_secs(1)).await;
                continue;
            }
        };

        for tenant_id in unique_tenant_ids {
            let tenant_msg = messages_to_process
                .iter()
                .filter(|m| m.tenant_id == tenant_id)
                .cloned()
                .collect::<Vec<_>>();
            let (patients, studies, series, images) = match group_dicom_messages(&tenant_msg) {
                Ok((patients, studies, series, images)) => (patients, studies, series, images),
                Err(_) => {
                    db_provider.save_dicommeta_info(&tenant_msg).await.expect("解析DICOM信息出现错误,并写入数据库失败!");
                    continue;
                }
            };
            if images.len() != tenant_msg.len() {
                db_provider.save_dicommeta_info(&tenant_msg).await.expect("解析DICOM信息成功但是文件个数和消息个数不对,并写入数据库失败!");
                continue;
            }
            match db_provider
                .persist_to_database(tenant_id.as_str(), &patients, &studies, &series, &images)
                .await
            {
                Ok(()) => {
                    tracing::info!("Successfully persisted data for tenant {}", tenant_id);
                }
                Err(e) => {
                    tracing::error!("Failed to persist data for tenant {}: {}", tenant_id, e);
                    db_provider.save_dicommeta_info(&tenant_msg).await.expect("解析DICOM信息成功但是写入数据库失败!");
                    continue;
                }
            }
        }

        // 更新最后处理时间
        {
            let mut time = last_process_time.lock().unwrap();
            *time = Instant::now();
        }

        tracing::info!(
            "Successfully processed batch of {} messages",
            messages_to_process.len()
        );
    }
}
