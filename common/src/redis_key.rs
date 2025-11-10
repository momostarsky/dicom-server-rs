use crate::server_config::RedisConfig;
use database::dicom_meta::DicomStateMeta;
use redis::Commands;

#[derive(Debug, Clone)]
pub struct RedisHelper {
    config: RedisConfig,
}

impl RedisHelper {
    pub fn new(config: RedisConfig) -> Self {
        RedisHelper { config }
    }

    fn make_client(&self) -> Result<redis::Connection, redis::RedisError> {
        // redis::Client::open(self.config.url.as_str())
        //     .expect("Invalid Redis connection URL")
        //     .get_connection()
        let client =
            redis::Client::open(self.config.url.as_str()).expect("Invalid Redis connection URL");

        let mut connection = client.get_connection()?;

        // 如果配置中有密码，则进行认证
        if let Some(password) = &self.config.password {
            redis::cmd("AUTH")
                .arg(password)
                .query::<()>(&mut connection)?;
        }

        Ok(connection)
    }
    /// 生成的KEY用于将StudyUID的元数据缓存在Redis中
    pub fn key_for_study_metadata(&self, tenant_id: &str, study_uid: &str) -> String {
        format!("wado:{}:study:{}:metadata", tenant_id, study_uid)
    }
    /// 生成的KEY用于将StudyUID 对应的实体是否在数据库中,防止因为Redis中不存在,重复查询数据库
    pub fn key_for_study_enity(&self, tenant_id: &str, study_uid: &str) -> String {
        format!("db:{}:study:{}:metadata", tenant_id, study_uid)
    }

    /// 1小时 , 3600秒
    pub const ONE_HOUR: u64 = 3600;

    /// 10分钟 , 600秒
    pub const TEN_MINULE: u64 = 600;
    /// 1分钟 , 60秒
    pub const ONE_MINULE: u64 = 60;
    pub fn set_study_metadata(
        &self,
        tenant_id: &str,
        study_uid: &str,
        metas: &[DicomStateMeta],
        expire_seconds: u64,
    ) {
        let key = self.key_for_study_metadata(tenant_id, study_uid);
        let client = self.make_client();
        if !client.is_ok() {
            return;
        }
        match serde_json::to_string(&metas) {
            Ok(serialized_metas) => {
                let mut cl = client.unwrap();
                let _: Result<String, _> = cl.set_ex(key, serialized_metas, expire_seconds);
            }
            Err(_) => {}
        }
    }

    pub fn set_jwks_url_content(&self, txt: String, expire_seconds: u64) {
        let client = self.make_client();
        if !client.is_ok() {
            return;
        }
        let key = "jwksurl:8e646686-9d36-480b-95ea-1718b24c1c98".to_string();
        let mut cl = client.unwrap();
        let _: Result<String, _> = cl.set_ex(key, txt, expire_seconds);
    }

    pub fn get_jwks_url_content(&self) -> Result<String, redis::RedisError> {
        let mut client = match self.make_client() {
            Ok(client) => client,
            Err(e) => return Err(e),
        };
        let key = "jwksurl:8e646686-9d36-480b-95ea-1718b24c1c98".to_string();
        client.get::<String, String>(key)
    }

    pub fn del_study_metadata(&self, tenant_id: &str, study_uid: &str) {
        let key = self.key_for_study_metadata(tenant_id, study_uid);
        let client = self.make_client();
        if !client.is_ok() {
            return;
        }
        let mut cl = client.unwrap();
        let _: Result<String, _> = cl.del(key);
    }

    pub fn get_study_metadata(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<Vec<DicomStateMeta>, redis::RedisError> {
        let key = self.key_for_study_metadata(tenant_id, study_uid);
        let mut client = match self.make_client() {
            Ok(client) => client,
            Err(e) => return Err(e),
        };
        match client.get::<String, String>(key) {
            Ok(cached_data) => match serde_json::from_str::<Vec<DicomStateMeta>>(&cached_data) {
                Ok(metas) => Ok(metas),
                Err(e) => Err(redis::RedisError::from((
                    redis::ErrorKind::TypeError,
                    "Failed to deserialize DicomStateMeta",
                    e.to_string(),
                ))),
            },
            Err(e) => Err(e),
        }
    }

    pub fn set_study_entity_not_exists(
        &self,
        tenant_id: &str,
        study_uid: &str,
        expire_seconds: u64,
    ) {
        let key = self.key_for_study_enity(tenant_id, study_uid);
        let client = self.make_client();
        if !client.is_ok() {
            return;
        }
        let mut cl = client.unwrap();
        let _: Result<String, _> = cl.set_ex(key, "1", expire_seconds);
    }
    pub fn del_study_entity_not_exists(&self, tenant_id: &str, study_uid: &str) {
        let key = self.key_for_study_enity(tenant_id, study_uid);
        let client = self.make_client();
        if !client.is_ok() {
            return;
        }
        let mut cl = client.unwrap();
        let _: Result<String, _> = cl.del(key);
    }

    pub fn get_study_entity_not_exists(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<bool, redis::RedisError> {
        let key = self.key_for_study_enity(tenant_id, study_uid);
        let mut client = match self.make_client() {
            Ok(client) => client,
            Err(e) => return Err(e),
        };
        match client.get::<String, String>(key) {
            Ok(cached_data) => Ok(cached_data == "1"),
            Err(e) => Err(e),
        }
    }
}
