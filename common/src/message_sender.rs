
use async_trait::async_trait;
use crate::dicom_object_meta::DicomStoreMeta;

#[async_trait]
pub trait MessagePublisher:Sync + Send  {
    async fn send_message(
        &self,
        msg: &DicomStoreMeta,
    ) -> Result<(), Box<dyn std::error::Error>>;


    async fn send_batch_messages(
        &self,
        messages: &[DicomStoreMeta],
    ) -> Result<(), Box<dyn std::error::Error>>;

}