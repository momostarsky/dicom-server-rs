use crate::dicom_meta::{DicomJsonMeta, DicomStateMeta};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database operation failed: {0}")]
    DatabaseError(String),

    #[error("Record already exists")]
    AlreadyExists,

    #[error("Entity extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}
pub fn current_time() -> chrono::NaiveDateTime {
    chrono::Local::now().naive_local()
}
#[async_trait]
pub trait DbProvider: Send + Sync {
    async fn save_state_info(&self, state_meta: &DicomStateMeta) -> Result<(), DbError>;

    async fn save_state_list(&self, state_meta: &[DicomStateMeta]) -> Result<(), DbError>;

    async fn save_json_list(&self, state_meta: &[DicomJsonMeta]) -> Result<(), DbError>;

    async fn get_state_metaes(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<Vec<DicomStateMeta>, DbError>;


    /*
     * 获取需要生成JSON格式的Metadata的序列信息.
     * end_time: 截止时间.
     */
    async fn get_json_metaes(&self, end_time: chrono::NaiveDateTime) -> Result<Vec<DicomStateMeta>, DbError>;
}
