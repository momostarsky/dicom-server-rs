use crate::logevents::ApiLogEvent;
use async_trait::async_trait;
use database::dicom_meta::{DicomImageMeta, DicomStateMeta, DicomStoreMeta};
use std::error::Error;

#[async_trait]
pub trait MessagePublisher: Sync + Send {
    async fn send_message(&self, msg: &DicomStoreMeta) -> Result<(), Box<dyn Error>>;

    async fn send_batch_messages(
        &self,
        messages: &[DicomStoreMeta],
    ) -> Result<(), Box<dyn Error>>;

    async fn send_state_messages(
        &self,
        messages: &[DicomStateMeta],
    ) -> Result<(), Box<dyn Error>>;

    async fn send_image_messages(
        &self,
        messages: &[DicomImageMeta],
    ) -> Result<(), Box<dyn Error>>;



    async fn send_webapi_messages(
        &self,
        messages: &[ApiLogEvent],
    ) -> Result<(), Box<dyn Error>>;

    // ... 其他方法
    // fn clone_box(&self) -> Box<dyn MessagePublisher>;

}
