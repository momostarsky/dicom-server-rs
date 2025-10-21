use crate::database_entities::{SeriesEntity, StudyEntity};
use async_trait::async_trait;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DbError {
    #[error("Database operation failed: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Record already exists")]
    AlreadyExists,

    #[error("Entity extraction failed: {0}")]
    ExtractionFailed(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
}

#[async_trait]
pub trait DbProvider: Send + Sync {
    async fn get_study_info(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<Option<StudyEntity>, DbError>;
    async fn get_series_info(
        &self,
        tenant_id: &str,
        series_uid: &str,
    ) -> Result<Option<SeriesEntity>, DbError>;
}
