use crate::database_entities::DicomObjectMeta;
use async_trait::async_trait;

#[async_trait]
pub trait MessagePublisher:Sync + Send  {
    async fn send_message(
        &self,
        msg: &DicomObjectMeta,
    ) -> Result<(), Box<dyn std::error::Error>>;


    async fn send_batch_messages(
        &self,
        messages: &[DicomObjectMeta],
    ) -> Result<(), Box<dyn std::error::Error>>;

}