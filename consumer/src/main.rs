mod message_dispatcher;
mod message_processor;

use crate::message_dispatcher::MessageDispatcher;
use crate::message_processor::start_process;

// 使用内存存储已处理的消息ID（生产环境建议使用数据库或Redis）

#[tokio::main]
async fn main() {
    let dispatcher = MessageDispatcher::new().await;
    dispatcher.start_dispatch().await;
}
