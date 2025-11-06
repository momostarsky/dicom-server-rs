use async_trait::async_trait;
use database::dicom_meta::{DicomImageMeta, DicomStateMeta, DicomStoreMeta};

#[async_trait]
pub trait MessagePublisher: Sync + Send {
    async fn send_message(&self, msg: &DicomStoreMeta) -> Result<(), Box<dyn std::error::Error>>;

    async fn send_batch_messages(
        &self,
        messages: &[DicomStoreMeta],
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn send_state_messages(
        &self,
        messages: &[DicomStateMeta],
    ) -> Result<(), Box<dyn std::error::Error>>;

    async fn send_image_messages(
        &self,
        messages: &[DicomImageMeta],
    ) -> Result<(), Box<dyn std::error::Error>>;
}
