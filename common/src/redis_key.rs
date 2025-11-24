use crate::server_config::RedisConfig;
use database::dicom_meta::DicomStateMeta;
use deadpool_redis::redis::AsyncCommands;
use deadpool_redis::{Config as DeadConfig, Pool};

use std::string::ToString;

#[derive(Debug, Clone)]
pub struct RedisHelper {
    pool: Pool,
}

impl RedisHelper {
    pub fn new(config: RedisConfig) -> Self {
        let mut dead_cfg = DeadConfig::default();
        dead_cfg.url = Some(config.url.clone());
        let pool = dead_cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .expect("Pool creation failed");
        RedisHelper { pool }
    }

    async fn get_connection(&self) -> Result<deadpool_redis::Connection, redis::RedisError> {
        match self.pool.get().await {
            Ok(conn) => Ok(conn),
            Err(e) => Err(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Failed to get Redis connection",
                e.to_string(),
            ))),
        }
    }

    async fn set_key_value_expire(
        &self,
        key: String,
        txt: String,
        expire_seconds: u64,
    ) -> Result<(), redis::RedisError> {
        let mut conn = self.get_connection().await?;
        match conn
            .set_ex::<String, String, ()>(key, txt, expire_seconds)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                return Err(redis::RedisError::from((
                    redis::ErrorKind::IoError,
                    "Failed to set_key_value_expire",
                    e.to_string(),
                )));
            }
        }
        Ok(())
    }

    async fn get_value(&self, key: String) -> Result<String, redis::RedisError> {
        let mut conn = self.get_connection().await?;
        match conn.get(key).await {
            Ok(jwks_url) => Ok(jwks_url),
            Err(e) => Err(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "Failed to get_value",
                e.to_string(),
            ))),
        }
    }

    async fn del_key(&self, key: String) -> Result<(), redis::RedisError> {
        let mut conn = self.get_connection().await?;
        match conn.del(key).await {
            Ok(()) => {}
            Err(e) => {
                return Err(redis::RedisError::from((
                    redis::ErrorKind::IoError,
                    "Failed to delete Redis key:{}",
                    e.to_string(),
                )));
            }
        }
        Ok(())
    }

    const JWKS_URL_KEY: &'static str = "jwksurl:8e646686-9d36-480b-95ea-1718b24c1c98";

    pub async fn set_jwks_url_content(
        &self,
        txt: String,
        expire_seconds: u64,
    ) -> Result<(), redis::RedisError> {
        self.set_key_value_expire(Self::JWKS_URL_KEY.to_string(), txt, expire_seconds)
            .await
    }

    pub async fn get_jwks_url_content(&self) -> Result<String, redis::RedisError> {
        self.get_value(Self::JWKS_URL_KEY.to_string()).await
    }

    pub fn key_for_study_metadata(&self, tenant_id: &str, study_uid: &str) -> String {
        format!("wado:{}:study:{}:metadata", tenant_id, study_uid)
    }

    pub fn key_for_study_enity(&self, tenant_id: &str, study_uid: &str) -> String {
        format!("db:{}:study:{}:metadata", tenant_id, study_uid)
    }

    pub const ONE_HOUR: u64 = 3600;
    pub const TEN_MINULE: u64 = 600;
    pub const ONE_MINULE: u64 = 60;

    pub async fn set_study_metadata(
        &self,
        tenant_id: &str,
        study_uid: &str,
        metas: &[DicomStateMeta],
        expire_seconds: u64,
    ) -> Result<(), redis::RedisError> {
        let key = self.key_for_study_metadata(tenant_id, study_uid);
        let serialized_metas = serde_json::to_string(metas).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "Failed to serialize DicomStateMeta",
                e.to_string(),
            ))
        })?;
        self.set_key_value_expire(key, serialized_metas, expire_seconds)
            .await
    }

    pub async fn del_study_metadata(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<(), redis::RedisError> {
        let key = self.key_for_study_metadata(tenant_id, study_uid);
        self.del_key(key).await
    }

    pub async fn get_study_metadata(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<Vec<DicomStateMeta>, redis::RedisError> {
        let key = self.key_for_study_metadata(tenant_id, study_uid);

        match self.get_value(key).await {
            Ok(cached_data) => {
                serde_json::from_str::<Vec<DicomStateMeta>>(&cached_data).map_err(|e| {
                    redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "Failed to deserialize DicomStateMeta",
                        e.to_string(),
                    ))
                })
            }
            Err(_e) => Ok(vec![]),
        }
    }

    pub async fn set_study_entity_not_exists(
        &self,
        tenant_id: &str,
        study_uid: &str,
        expire_seconds: u64,
    ) -> Result<(), redis::RedisError> {
        let key = self.key_for_study_enity(tenant_id, study_uid);
        self.set_key_value_expire(key, "1".to_string(), expire_seconds)
            .await
    }

    pub async fn del_study_entity_not_exists(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<(), redis::RedisError> {
        let key = self.key_for_study_enity(tenant_id, study_uid);
        self.del_key(key).await
    }

    pub async fn get_study_entity_not_exists(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<bool, redis::RedisError> {
        let key = self.key_for_study_enity(tenant_id, study_uid);

        match self.get_value(key).await {
            Ok(cached_data) => Ok(cached_data == "1"),
            Err(_e) => Ok(false),
        }
    }

    pub async fn set_series_metadata_gererate(
        &self,
        tenant_id: &str,
        series_uid: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("wado:{}:series:{}:json_generating", tenant_id, series_uid);

        self.set_key_value_expire(key, "1".to_string(), Self::TEN_MINULE)
            .await
    }

    pub async fn del_series_metadata_gererate(
        &self,
        tenant_id: &str,
        series_uid: &str,
    ) -> Result<(), redis::RedisError> {
        let key = format!("wado:{}:series:{}:json_generating", tenant_id, series_uid);
        self.del_key(key).await
    }

    pub async fn get_series_metadata_gererate(
        &self,
        tenant_id: &str,
        series_uid: &str,
    ) -> Result<bool, redis::RedisError> {
        let key = format!("wado:{}:series:{}:json_generating", tenant_id, series_uid);

        match self.get_value(key).await {
            Ok(cached_data) => Ok(cached_data == "1"),
            Err(_) => Ok(false),
        }
    }
}
