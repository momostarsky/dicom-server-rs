use common::message_sender_kafka::KafkaMessagePublisher;
use common::utils::{get_logger, group_dicom_state};
use common::{database_factory, server_config};
use database::dicom_meta::{DicomImageMeta, DicomStateMeta, DicomStoreMeta};
use futures::StreamExt;
use rdkafka::consumer::{CommitMode, Consumer, StreamConsumer};
use rdkafka::{ClientConfig, Message};
use slog;
use slog::{error, info, o};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tokio::runtime::Handle;

pub async fn start_process() {
    // 设置全局logger

    let rlogger = get_logger();
    let global_logger = rlogger.new(o!("wado-consume"=>"start_process"));

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
        .expect("create wado-consumer failed");

    let topic = queue_config.topic_main.as_str();
    info!(global_logger, "Subscribing to topic: {}", topic);

    match consumer.subscribe(&[topic]) {
        Ok(_) => info!(global_logger, "Successfully subscribed to topic: {}", topic),
        Err(e) => {
            error!(
                global_logger,
                "Failed to subscribe to topic {}: {}", topic, e
            );
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
    // 在退出前调用

    // 等待一小段时间让清理完成
    tokio::time::sleep(Duration::from_millis(100)).await;

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
    info!(logger, "XXStarting to read messages ...");

    while let Some(result) = message_stream.next().await {
        match result {
            Ok(message) => {
                info!(logger, "Received message: {:?}", message);
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
                                    info!(
                                        logger,
                                        "Successfully processed and committed message:{}",
                                        message.offset()
                                    );
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
        let (state_metas, image_entities) = match group_dicom_state(&messages_to_process).await {
            Ok((state_metas, image_entities)) => (state_metas, image_entities),
            Err(e) => {
                error!(logger, "Failed to group dicom state: {}", e);
                continue;
            }
        };
        //TODO 遍历输出状态
        for state_meta in &state_metas {
            info!(logger, "DicomStateMeta: {:?}", state_meta);
        }
        //TODO 遍历输出图像
        for image_entity in &image_entities {
            info!(logger, "ImageEntity: {:?}", image_entity);
        }
        let app_config = server_config::load_config().unwrap();
        let queue_config = app_config.message_queue;

        let topic_state = &queue_config.topic_dicom_state.as_str();
        let topic_image = &queue_config.topic_dicom_image.as_str();

        let state_producer = KafkaMessagePublisher::new(topic_state.parse().unwrap());
        let image_producer = KafkaMessagePublisher::new(topic_image.parse().unwrap());

        // 发布状态消息和图像消息
        if let Err(e) = publish_dicom_meta(
            &state_metas,
            &image_entities,
            &state_producer,
            &image_producer,
        )
        .await
        {
            error!(logger, "Failed to publish dicom meta: {}", e);
            continue;
        }
        let db = match database_factory::create_db_instance(&app_config.main_database).await {
            Ok(db_provider) => db_provider,
            Err(e) => {
                error!(logger, "Failed to create database: {}", e);
                continue;
            }
        };
        // 插入状态消息
        if let Err(e) = db.save_state_list(&state_metas).await {
            error!(logger, "Failed to save_state_list: {}", e);
            continue;
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

async fn publish_dicom_meta(
    state_metaes: &Vec<DicomStateMeta>,
    image_metaes: &Vec<DicomImageMeta>,
    state_producer: &KafkaMessagePublisher,
    image_producer: &KafkaMessagePublisher,
) -> Result<(), Box<dyn std::error::Error>> {
    let root_logger = get_logger();
    let logger = root_logger.new(o!("wado-consume"=>"publish_dicom_meta"));
    if state_metaes.is_empty() || image_metaes.is_empty() {
        info!(
            logger,
            "Empty dicom state meta list and image meta list, skip"
        );
        return Ok(());
    }

    let state_topic_name = state_producer.topic();
    let image_topic_name = image_producer.topic();

    // 并行发布状态消息和图像消息
    let (state_result, image_result) = tokio::join!(
        common::utils::publish_state_messages(state_producer, &state_metaes),
        common::utils::publish_image_messages(image_producer, &image_metaes)
    );
    // 处理状态消息发布结果
    match state_result {
        Ok(_) => {
            info!(
                logger,
                "Successfully published {} supported messages to Kafka: {}",
                state_metaes.len(),
                state_topic_name
            );
        }
        Err(e) => {
            error!(
                logger,
                "Failed to publish messages to Kafka: {}, topic: {}", e, state_topic_name
            );
        }
    }

    // 处理图像消息发布结果
    match image_result {
        Ok(_) => {
            info!(
                logger,
                "Successfully published {} messages to Kafka: {}",
                image_metaes.len(),
                image_topic_name
            );
        }
        Err(e) => {
            error!(
                logger,
                "Failed to publish image messages to Kafka: {}, topic: {}", e, image_topic_name
            );
        }
    }
    //
    // match common::utils::publish_state_messages(state_producer, &state_metaes).await {
    //     Ok(_) => {
    //         info!(
    //             logger,
    //             "Successfully published {} supported messages to Kafka: {}",
    //             state_metaes.len(),
    //             state_topic_name
    //         );
    //     }
    //     Err(e) => {
    //         error!(
    //             logger,
    //             "Failed to publish messages to Kafka: {}, topic: {}", e, state_topic_name
    //         );
    //     }
    // }
    //
    //
    // match common::utils::publish_image_messages(image_producer, &image_metaes).await {
    //     Ok(_) => {
    //         info!(
    //             logger,
    //             "Successfully published {} messages to Kafka: {}",
    //             image_metaes.len(),
    //             image_topic_name
    //         );
    //     }
    //     Err(e) => {
    //         error!(
    //             logger,
    //             "Failed to publish image messages to Kafka: {}, topic: {}", e, image_topic_name
    //         );
    //     }
    // }

    Ok(())
}
