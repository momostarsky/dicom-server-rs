mod storage_consumer;

use common::utils::setup_logging;
use crate::storage_consumer::start_process;

// 使用内存存储已处理的消息ID（生产环境建议使用数据库或Redis）

#[tokio::main]
async fn main() {
    setup_logging("kafak-storage-consumer");
    tracing::info!("Message processor started");
    start_process().await;
    tracing::info!("Message processor stopped");
}
