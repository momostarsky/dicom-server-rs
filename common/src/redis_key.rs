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
        let dead_cfg = DeadConfig::from_url(config.url.clone());
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
            .set_ex::<&str , &str, ()>(key.as_str(), txt.as_str(), expire_seconds)
            .await
        {
            Ok(_) => {}
            Err(_e) => {

                return Err(redis::RedisError::from((
                    redis::ErrorKind::IoError,
                    "redisHelper Failed to set_key_value_expire ",
                    key,
                )));
            }
        }
        Ok(())
    }

    async fn get_value(&self, key: String) -> Result<String, redis::RedisError> {
        let mut conn = self.get_connection().await?;
        match conn.get(&key).await {
            Ok(jwks_url) => Ok(jwks_url),
            Err(_e) => Err(redis::RedisError::from((
                redis::ErrorKind::IoError,
                "redisHelper Failed to get_value from key",
                key,
            ))),
        }
    }

    async fn del_key(&self, key: String) -> Result<(), redis::RedisError> {
        let mut conn = self.get_connection().await?;
        match conn.del(&key).await {
            Ok(()) => {}
            Err(_e) => {
                return Err(redis::RedisError::from((
                    redis::ErrorKind::IoError,
                    "redisHelper Failed to delete Redis key",
                    key ,
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

    /// 缓存study元数据 ,when metas is empty, do nothing
    pub async fn set_study_metadata(
        &self,
        tenant_id: &str,
        study_uid: &str,
        metas: &[DicomStateMeta],
        expire_seconds: u64,
    ) -> Result<(), redis::RedisError> {
        if metas.is_empty() {
            return Ok(());
        }
        let key = self.key_for_study_metadata(tenant_id, study_uid);
        let serialized_metas = serde_json::to_string(metas).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "redisHelper Failed to serialize DicomStateMeta",
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

    /// Get study metadata
    /// returns:
    ///    If not found, return empty vector
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
                        "redisHelper Failed to deserialize DicomStateMeta",
                        e.to_string(),
                    ))
                })
            }
            Err(_e) => {
                // 无论是什么错误，只要获取不到值就返回空向量

                Ok(vec![])
            }
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

    pub async fn del_study_entity_not_exists(&self, tenant_id: &str, study_uid: &str) {
        let key = self.key_for_study_enity(tenant_id, study_uid);
        match self.del_key(key).await {
            Ok(()) => {}
            Err(_) => {}
        }
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
#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_config::RedisConfig;
    use crate::utils::get_current_time;
    use chrono::{NaiveDate, NaiveTime};
    use database::dicom_dbtype::{BoundedString, DicomDateString};
    use database::dicom_meta::DicomStateMeta;

    fn get_test_config() -> RedisConfig {
        RedisConfig {
            url: "redis://192.168.1.14:6379/".to_string(),
            password: None,
            is_lts: None,
        }
    }

    #[tokio::test]
    async fn test_redis_helper_creation() {
        let config = get_test_config();
        let redis_helper = RedisHelper::new(config);
        assert!(redis_helper.pool.get().await.is_ok());
    }

    #[tokio::test]
    async fn test_study_metadata_operations() {
        let config = get_test_config();
        let redis_helper = RedisHelper::new(config);

        let tenant_id = "test_tenant";
        let study_uid = "test_study_uid";
        let expire_seconds = 10u64;

        // Create test metadata
        let test_metas = vec![DicomStateMeta {
            // Initialize with appropriate test values
            tenant_id: BoundedString::<64>::try_from("1234567890").unwrap(),
            patient_id: BoundedString::<64>::from_str("Patient_001").unwrap(),
            study_uid: BoundedString::<64>::from_str("1.333.902009.002").unwrap(),
            series_uid: BoundedString::<64>::from_str("1.23899.290").unwrap(),
            study_uid_hash: BoundedString::<20>::from_str("8832893289").unwrap(),
            series_uid_hash: BoundedString::<20>::from_str("832898923832").unwrap(),
            study_date_origin: DicomDateString::try_from("20200301").unwrap(),
            patient_name: Option::from(BoundedString::<64>::try_from("John^Doe").unwrap()),
            patient_sex: Option::from(BoundedString::<1>::try_from("F").unwrap()),
            patient_birth_date: Option::from(
                NaiveDate::parse_from_str("2020-03-01", "%Y-%m-%d").unwrap(),
            ),
            patient_birth_time: Option::from(
                NaiveTime::parse_from_str("120332", "%H%M%S").unwrap(),
            ),
            patient_age: Option::from(BoundedString::<16>::try_from("016Y").unwrap()),
            patient_size: Option::from(64.0),
            patient_weight: Option::from(37.0),
            study_date: NaiveDate::parse_from_str("2020-03-01", "%Y-%m-%d").unwrap(),
            study_time: Option::from(NaiveTime::parse_from_str("120346", "%H%M%S").unwrap()),
            accession_number: Some(BoundedString::<16>::make_str("8328989")),
            study_id: Option::from(BoundedString::<16>::from_str("8328989").unwrap()),
            study_description: Option::from(BoundedString::<64>::from_str("8328989").unwrap()),
            modality: Some(BoundedString::<16>::from_str("CT").unwrap()),
            series_number: Some(1),
            series_date: Some(NaiveDate::parse_from_str("2020-03-01", "%Y-%m-%d").unwrap()),
            series_time: Option::from(NaiveTime::parse_from_str("120346", "%H%M%S").unwrap()),
            series_description: Some(BoundedString::<256>::from_str("series_description").unwrap()),
            body_part_examined: Some(BoundedString::<64>::from_str("肺部").unwrap()),
            protocol_name: Some(BoundedString::<64>::from_str("PACS").unwrap()),
            series_related_instances: Some(1),
            created_time: get_current_time(),
            updated_time: get_current_time(),
        }];

        // Set study metadata
        let set_result = redis_helper
            .set_study_metadata(tenant_id, study_uid, &test_metas, expire_seconds)
            .await;
        assert!(set_result.is_ok());

        // Get study metadata
        let retrieved_metas = redis_helper.get_study_metadata(tenant_id, study_uid).await;
        assert!(retrieved_metas.is_ok());
        // Note: Actual comparison would depend on DicomStateMeta implementation

        // Delete study metadata
        let del_result = redis_helper.del_study_metadata(tenant_id, study_uid).await;
        assert!(del_result.is_ok());

        // Verify deletion
        let after_del = redis_helper.get_study_metadata(tenant_id, study_uid).await;
        assert!(after_del.is_ok());
        assert!(after_del.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_study_entity_not_exists_flag() {
        let config = get_test_config();
        let redis_helper = RedisHelper::new(config);

        let tenant_id = "test_tenant";
        let study_uid = "test_study_uid";
        let expire_seconds = 10u64;

        // Initially should not exist
        let initial_check = redis_helper
            .get_study_entity_not_exists(tenant_id, study_uid)
            .await;
        assert!(initial_check.is_ok());
        assert_eq!(initial_check.unwrap(), false);

        // Set entity not exists flag
        let set_result = redis_helper
            .set_study_entity_not_exists(tenant_id, study_uid, expire_seconds)
            .await;
        assert!(set_result.is_ok());

        // Check that it now exists
        let after_set = redis_helper
            .get_study_entity_not_exists(tenant_id, study_uid)
            .await;
        assert!(after_set.is_ok());
        assert_eq!(after_set.unwrap(), true);

        // Delete the flag
        redis_helper
            .del_study_entity_not_exists(tenant_id, study_uid)
            .await;

        // Should not exist again
        let after_delete = redis_helper
            .get_study_entity_not_exists(tenant_id, study_uid)
            .await;
        assert!(after_delete.is_ok());
        assert_eq!(after_delete.unwrap(), false);
    }

    #[tokio::test]
    async fn test_series_metadata_generation_flag() {
        let config = get_test_config();
        let redis_helper = RedisHelper::new(config);

        let tenant_id = "test_tenant";
        let series_uid = "test_series_uid";

        // Initially should not be generating
        let initial_check = redis_helper
            .get_series_metadata_gererate(tenant_id, series_uid)
            .await;
        assert!(initial_check.is_ok());
        assert_eq!(initial_check.unwrap(), false);

        // Set generation flag
        let set_result = redis_helper
            .set_series_metadata_gererate(tenant_id, series_uid)
            .await;
        assert!(set_result.is_ok());

        // Check that it's now generating
        let after_set = redis_helper
            .get_series_metadata_gererate(tenant_id, series_uid)
            .await;
        assert!(after_set.is_ok());
        assert_eq!(after_set.unwrap(), true);

        // Delete the flag
        let del_result = redis_helper
            .del_series_metadata_gererate(tenant_id, series_uid)
            .await;
        assert!(del_result.is_ok());

        // Should not be generating again
        let after_delete = redis_helper
            .get_series_metadata_gererate(tenant_id, series_uid)
            .await;
        assert!(after_delete.is_ok());
        assert_eq!(after_delete.unwrap(), false);
    }

    #[tokio::test]
    async fn test_jwks_url_content_operations() {
        let config = get_test_config();
        let redis_helper = RedisHelper::new(config);

        let test_content = "test_jwks_content_12345".to_string();
        let expire_seconds = 10u64;

        // Test setting JWKS URL content
        let set_result = redis_helper
            .set_jwks_url_content(test_content.clone(), expire_seconds)
            .await;
        assert!(set_result.is_ok(), "Failed to set JWKS URL content");

        // Test getting JWKS URL content
        let get_result = redis_helper.get_jwks_url_content().await;
        assert!(get_result.is_ok(), "Failed to get JWKS URL content");
        assert_eq!(
            get_result.unwrap(),
            test_content,
            "Retrieved content doesn't match expected"
        );
    }

    #[tokio::test]
    async fn test_jwks_url_content_expiration() {
        let config = get_test_config();
        let redis_helper = RedisHelper::new(config);

        let test_content = "expiring_jwks_content".to_string();
        let expire_seconds = 2u64; // Short expiration for testing

        // Set content with short expiration
        let set_result = redis_helper
            .set_jwks_url_content(test_content, expire_seconds)
            .await;
        assert!(
            set_result.is_ok(),
            "Failed to set expiring JWKS URL content"
        );

        // Content should be available immediately
        let get_result = redis_helper.get_jwks_url_content().await;
        assert!(
            get_result.is_ok(),
            "Content should be available before expiration"
        );

        // Wait for expiration
        tokio::time::sleep(tokio::time::Duration::from_secs(expire_seconds + 1)).await;

        // After expiration, getting content should fail
        let get_result_after_expire = redis_helper.get_jwks_url_content().await;
        assert!(
            get_result_after_expire.is_err(),
            "Content should not be available after expiration"
        );
    }
}
