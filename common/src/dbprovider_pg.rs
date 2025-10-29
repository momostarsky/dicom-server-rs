use std::option::Option;
use async_trait::async_trait; 
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::{PgRow, PgTypeInfo};
use sqlx::{Database, Encode, Error, FromRow, PgPool, Row};

use crate::database_entities::{SeriesEntity, StudyEntity};
use crate::database_provider::{DbError, DbProvider};
use crate::dicom_object_meta::DicomStateMeta;
use tracing::error;

use crate::string_ext::{
    BoundedString, DicomDateString, FixedLengthString, SopUidString, UidHashString, UuidString,
};
use sqlx::Postgres;

// 为 FixedLengthString 实现 PostgreSQL 的 Type trait
impl<const N: usize> sqlx::Type<Postgres> for FixedLengthString<N> {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::with_name("VARCHAR")
    }
}

impl<const N: usize> Encode<'_, Postgres> for FixedLengthString<N> {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}

impl<const N: usize> sqlx::Type<Postgres> for BoundedString<N> {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::with_name("VARCHAR")
    }
}

// 为 BoundedString 实现 PostgreSQL 的 Encode trait
impl<const N: usize> Encode<'_, Postgres> for BoundedString<N> {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}

impl sqlx::Type<Postgres> for UidHashString {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::with_name("CHAR")
    }
}

impl Encode<'_, Postgres> for UidHashString {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}

// 为 SopUidString 实现 PostgreSQL 的 Type trait
impl sqlx::Type<Postgres> for SopUidString {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::with_name("VARCHAR")
    }
}

impl Encode<'_, Postgres> for SopUidString {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}
// 为 UuidString 实现 PostgreSQL 的 Type trait
impl sqlx::Type<Postgres> for UuidString {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::with_name("VARCHAR")
    }
}
impl Encode<'_, Postgres> for UuidString {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}
// 为 DicomDateString 实现 PostgreSQL 的 Type trait
impl sqlx::Type<Postgres> for DicomDateString {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        PgTypeInfo::with_name("CHAR")
    }
}
impl Encode<'_, Postgres> for DicomDateString {
    fn encode_by_ref(
        &self,
        buf: &mut <Postgres as Database>::ArgumentBuffer<'_>,
    ) -> Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}

use sqlx::decode::Decode;
//
// // 为 BoundedString 实现 PostgreSQL 的 Decode trait
impl<'r, const N: usize> Decode<'r, Postgres> for BoundedString<N> {
    fn decode(value: <Postgres as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        // 如果失败，尝试转换为 String
        let string_val = <String as Decode<Postgres>>::decode(value)?;
        Ok(BoundedString::<N>::try_from(string_val).map_err(|e| Box::new(e) as BoxDynError)?)
    }
}
//
// // 为 FixedLengthString 实现 PostgreSQL 的 Decode trait
impl<'r, const N: usize> Decode<'r, Postgres> for FixedLengthString<N> {
    fn decode(value: <Postgres as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let str_val = <String as Decode<Postgres>>::decode(value)?;
        Ok(FixedLengthString::<N>::try_from(str_val).map_err(|e| Box::new(e) as BoxDynError)?)
    }
}
//
// // 为 SopUidString 实现 PostgreSQL 的 Decode trait
impl<'r> Decode<'r, Postgres> for SopUidString {
    fn decode(value: <Postgres as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let str_val = <String as Decode<Postgres>>::decode(value)?;
        Ok(SopUidString::try_from(str_val).map_err(|e| Box::new(e) as BoxDynError)?)
    }
}

//   为 UuidString 实现 PostgreSQL 的 Decode trait
impl<'r> Decode<'r, Postgres> for UuidString {
    fn decode(value: <Postgres as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let str_val = <String as Decode<Postgres>>::decode(value)?;
        Ok(UuidString::try_from(str_val).map_err(|e| Box::new(e) as BoxDynError)?)
    }
}
impl<'r> Decode<'r, Postgres> for UidHashString {
    fn decode(value: <Postgres as Database>::ValueRef<'r>) -> Result<Self, BoxDynError> {
        let str_val = <&str as Decode<Postgres>>::decode(value)?;
        Ok(UidHashString::make_from_db(str_val))
    }
}
impl FromRow<'_, PgRow> for SeriesEntity {
    fn from_row(row: &'_ PgRow) -> Result<Self, Error> {
        Ok(SeriesEntity {
            tenant_id: row.get("tenant_id"),
            series_instance_uid: row.get("series_uid"),
            study_instance_uid: row.get("study_uid"),
            patient_id: row.get("patient_id"),
            modality: row.get("modality"),
            series_number: row.get("series_number"),
            series_date: row.get("series_date"),
            series_time: row.get("series_time"),
            series_description: row.get("series_description"),
            body_part_examined: row.get("body_part_examined"),
            protocol_name: row.get("protocol_name"),
            created_time: row.get("created_time"),
            updated_time: row.get("updated_time"),
        })
    }
}

impl FromRow<'_, PgRow> for StudyEntity {
    fn from_row(row: &'_ PgRow) -> Result<Self, Error> {
        Ok(StudyEntity {
            tenant_id: row.get("tenant_id"),
            patient_id: row.get("patient_id"),
            patient_name: row.get("patient_name"),
            patient_age: row.get("patient_age"),
            patient_sex: row.get("patient_sex"),
            patient_size: row.get("patient_size"),
            patient_weight: row.get("patient_weight"),
            patient_birth_date: row.get("patient_birth_date"),
            study_instance_uid: row.get("study_uid"),
            study_uid_hash: UidHashString::from_string(row.get::<String, _>("study_uid_hash")),
            study_date: row.get("study_date"),
            study_time: row.get("study_time"),
            accession_number: row.get("accession_number"),
            study_id: row.get("study_id"),
            study_description: row.get("study_description"),
            patient_birth_time: row.get("patient_birth_time"),
            study_date_origin: row.get("study_date_origin"),
        })
    }
}

impl FromRow<'_, PgRow> for DicomStateMeta {
    fn from_row(row: &'_ PgRow) -> Result<Self, Error> {
        let s = row.get::<_,&str>("study_uid_hash");
        let ss = row.get::<_, &str>("series_uid_hash");
        let date_str = row.get::<_, &str>("study_date_origin");
        let study_uid_hash_v = UidHashString::make_from_db(s);
        let series_uid_hash_v = UidHashString::make_from_db(ss);
        let study_date_origin_v = DicomDateString::make_from_db(date_str);

        Ok(DicomStateMeta {
            tenant_id: row.get("tenant_id"),
            patient_id: row.get("patient_id"),
            study_uid: row.get("study_uid"),
            series_uid: row.get("series_uid"),
            study_uid_hash:  study_uid_hash_v,
            series_uid_hash: series_uid_hash_v,
            study_date_origin: study_date_origin_v,
            patient_name: row.get("patient_name"),
            patient_sex: row.get("patient_sex"),
            patient_birth_date: row.get("patient_birth_date"),
            patient_birth_time: row.get("patient_birth_time"),
            patient_age: row.get("patient_age"),
            patient_size: row.get("patient_size"),
            patient_weight: row.get("patient_weight"),
            study_date: row.get("study_date"),
            study_time: row.get("study_time"),
            accession_number: row.get("accession_number"),
            study_id: row.get("study_id"),
            study_description: row.get("study_description"),
            modality: row.get("modality"),
            series_number: row.get("series_number"),
            series_date: row.get("series_date"),
            series_time: row.get("series_time"),
            series_description: row.get("series_description"),
            body_part_examined: row.get("body_part_examined"),
            protocol_name: row.get("protocol_name"),
            series_related_instances: row.get("series_related_instances"),
            created_time: row.get("created_time"),
            updated_time: row.get("updated_time"),
        })
    }
}

pub struct PgDbProvider {
    pub pool: PgPool,
}

impl PgDbProvider {
    const GET_SERIES_INFO_QUERY: &'static str = r#"SELECT tenant_id,
                        patient_id,
                        study_uid,
                        series_uid,
                        study_uid_hash,
                        series_uid_hash,
                        study_date_origin,
                        accession_number,
                        modality,
                        series_number,
                        series_date,
                        series_time,
                        series_description,
                        body_part_examined,
                        protocol_name,
                        created_time,
                        updated_time
             FROM   dicom_state_meta  WHERE tenant_id = $1  and series_uid= $2"#;

    const GET_STUDY_INFO_QUERY: &'static str = r#"SELECT distinct   tenant_id,
                patient_id,
                study_uid,
                study_uid_hash,
                study_date,
                study_time,
                accession_number,
                study_id,
                study_description,
                patient_age,
                patient_size,
                patient_weight,
                patient_sex,
                patient_name,
                patient_birth_date,
                patient_birth_time,
                study_date_origin
            FROM dicom_state_meta   WHERE tenant_id = $1 AND study_uid = $2"#;

    const INSERT_OR_UPDATE_STATE_META_QUERY: &'static str = r#"
        INSERT INTO dicom_state_meta (
            tenant_id,
            patient_id,
            study_uid,
            series_uid,
            study_uid_hash,
            series_uid_hash,
            study_date_origin,
            accession_number,
            modality,
            series_number,
            series_date,
            series_time,
            series_description,
            body_part_examined,
            protocol_name,
            study_date,
            study_time,
            study_id,
            study_description,
            patient_age,
            patient_size,
            patient_weight,
            patient_sex,
            patient_name,
            patient_birth_date,
            patient_birth_time,
            created_time,
            updated_time
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28
        )
        ON CONFLICT (tenant_id, study_uid, series_uid)
        DO UPDATE SET
            patient_id = EXCLUDED.patient_id,
            study_uid_hash = EXCLUDED.study_uid_hash,
            series_uid_hash = EXCLUDED.series_uid_hash,
            study_date_origin = EXCLUDED.study_date_origin,
            accession_number = EXCLUDED.accession_number,
            modality = EXCLUDED.modality,
            series_number = EXCLUDED.series_number,
            series_date = EXCLUDED.series_date,
            series_time = EXCLUDED.series_time,
            series_description = EXCLUDED.series_description,
            body_part_examined = EXCLUDED.body_part_examined,
            protocol_name = EXCLUDED.protocol_name,
            study_date = EXCLUDED.study_date,
            study_time = EXCLUDED.study_time,
            study_id = EXCLUDED.study_id,
            study_description = EXCLUDED.study_description,
            patient_age = EXCLUDED.patient_age,
            patient_size = EXCLUDED.patient_size,
            patient_weight = EXCLUDED.patient_weight,
            patient_sex = EXCLUDED.patient_sex,
            patient_name = EXCLUDED.patient_name,
            patient_birth_date = EXCLUDED.patient_birth_date,
            patient_birth_time = EXCLUDED.patient_birth_time,
            updated_time = EXCLUDED.updated_time
    "#;

    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}
#[async_trait]
impl DbProvider for PgDbProvider {
    async fn get_study_info(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<Option<StudyEntity>, DbError> {
        let pool = self.pool.clone();
        match sqlx::query_as(Self::GET_STUDY_INFO_QUERY)
            .bind(tenant_id)
            .bind(study_uid)
            .fetch_optional(&pool)
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => {
                error!("Failed to get series info: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn get_series_info(
        &self,
        tenant_id: &str,
        series_uid: &str,
    ) -> Result<Option<SeriesEntity>, DbError> {
        let pool = self.pool.clone();
        match sqlx::query_as(Self::GET_SERIES_INFO_QUERY)
            .bind(tenant_id)
            .bind(series_uid)
            .fetch_optional(&pool)
            .await
        {
            Ok(result) => Ok(result),
            Err(e) => {
                error!("Failed to get series info: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn save_state_info(&self, state_meta: &DicomStateMeta) -> Result<(), DbError> {
        let pool = self.pool.clone();

        // 构建基础查询
        let query = sqlx::query::<Postgres>(Self::INSERT_OR_UPDATE_STATE_META_QUERY)
            .bind(state_meta.tenant_id.as_str())
            .bind(state_meta.patient_id.as_str())
            .bind(state_meta.study_uid.as_str())
            .bind(state_meta.series_uid.as_str())
            .bind(state_meta.study_uid_hash.as_str())
            .bind(state_meta.series_uid_hash.as_str())
            .bind(state_meta.study_date_origin.as_str())
            .bind(state_meta.accession_number.as_str());

        // 安全处理 Option 字段
        let query = match &state_meta.modality {
            Some(modality) => query.bind(modality.as_str()),
            None => query.bind(None::<&str>),
        };

        let query = query.bind(state_meta.series_number.unwrap_or(0));

        let query = query.bind(state_meta.series_date);

        let query = match &state_meta.series_time {
            Some(time) => query.bind(time),
            None => query.bind(None::<chrono::NaiveTime>),
        };

        let query = match &state_meta.series_description {
            Some(desc) => query.bind(desc.as_str()),
            None => query.bind(None::<&str>),
        };

        let query = match &state_meta.body_part_examined {
            Some(body_part) => query.bind(body_part.as_str()),
            None => query.bind(None::<&str>),
        };

        let query = match &state_meta.protocol_name {
            Some(protocol) => query.bind(protocol.as_str()),
            None => query.bind(None::<&str>),
        };

        let query = query.bind(state_meta.study_date);

        let query = match &state_meta.study_time {
            Some(time) => query.bind(time),
            None => query.bind(None::<chrono::NaiveTime>),
        };

        let query = match &state_meta.study_id {
            Some(id) => query.bind(id.as_str()),
            None => query.bind(None::<&str>),
        };

        let query = match &state_meta.study_description {
            Some(desc) => query.bind(desc.as_str()),
            None => query.bind(None::<&str>),
        };

        let query = match &state_meta.patient_age {
            Some(age) => query.bind(age.as_str()),
            None => query.bind(None::<&str>),
        };

        let query = query
            .bind(state_meta.patient_size)
            .bind(state_meta.patient_weight);

        let query = match &state_meta.patient_sex {
            Some(sex) => query.bind(sex.as_str()),
            None => query.bind(None::<&str>),
        };

        let query = match &state_meta.patient_name {
            Some(name) => query.bind(name.as_str()),
            None => query.bind(None::<&str>),
        };

        let query = query.bind(state_meta.patient_birth_date);

        let query = match &state_meta.patient_birth_time {
            Some(time) => query.bind(time),
            None => query.bind(None::<chrono::NaiveTime>),
        };

        match query
            .bind(state_meta.created_time)
            .bind(state_meta.updated_time)
            .execute(&pool)
            .await
        {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to save dicom state meta: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn save_state_list(&self, state_meta_list: &[DicomStateMeta]) -> Result<(), DbError> {
        if state_meta_list.is_empty() {
            return Ok(());
        }

        let pool = self.pool.clone();

        // 使用事务来确保所有数据要么全部保存成功，要么全部失败
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to begin transaction: {}", e);
                return Err(DbError::DatabaseError(e));
            }
        };

        for state_meta in state_meta_list {
            // 构建基础查询
            let query = sqlx::query::<Postgres>(Self::INSERT_OR_UPDATE_STATE_META_QUERY)
                .bind(state_meta.tenant_id.as_str())
                .bind(state_meta.patient_id.as_str())
                .bind(state_meta.study_uid.as_str())
                .bind(state_meta.series_uid.as_str())
                .bind(state_meta.study_uid_hash.as_str())
                .bind(state_meta.series_uid_hash.as_str())
                .bind(state_meta.study_date_origin.as_str())
                .bind(state_meta.accession_number.as_str());

            // 安全处理 Option 字段
            let query = match &state_meta.modality {
                Some(modality) => query.bind(modality.as_str()),
                None => query.bind(None::<&str>),
            };

            let query = query.bind(state_meta.series_number.unwrap_or(0));

            let query = query.bind(state_meta.series_date);

            let query = match &state_meta.series_time {
                Some(time) => query.bind(time),
                None => query.bind(None::<chrono::NaiveTime>),
            };

            let query = match &state_meta.series_description {
                Some(desc) => query.bind(desc.as_str()),
                None => query.bind(None::<&str>),
            };

            let query = match &state_meta.body_part_examined {
                Some(body_part) => query.bind(body_part.as_str()),
                None => query.bind(None::<&str>),
            };

            let query = match &state_meta.protocol_name {
                Some(protocol) => query.bind(protocol.as_str()),
                None => query.bind(None::<&str>),
            };

            let query = query.bind(state_meta.study_date);

            let query = match &state_meta.study_time {
                Some(time) => query.bind(time),
                None => query.bind(None::<chrono::NaiveTime>),
            };

            let query = match &state_meta.study_id {
                Some(id) => query.bind(id.as_str()),
                None => query.bind(None::<&str>),
            };

            let query = match &state_meta.study_description {
                Some(desc) => query.bind(desc.as_str()),
                None => query.bind(None::<&str>),
            };

            let query = match &state_meta.patient_age {
                Some(age) => query.bind(age.as_str()),
                None => query.bind(None::<&str>),
            };

            let query = query
                .bind(state_meta.patient_size)
                .bind(state_meta.patient_weight);

            let query = match &state_meta.patient_sex {
                Some(sex) => query.bind(sex.as_str()),
                None => query.bind(None::<&str>),
            };

            let query = match &state_meta.patient_name {
                Some(name) => query.bind(name.as_str()),
                None => query.bind(None::<&str>),
            };

            let query = query.bind(state_meta.patient_birth_date);

            let query = match &state_meta.patient_birth_time {
                Some(time) => query.bind(time),
                None => query.bind(None::<chrono::NaiveTime>),
            };

            match query
                .bind(state_meta.created_time)
                .bind(state_meta.updated_time)
                .execute(&mut *tx)
                .await
            {
                Ok(_) => continue,
                Err(e) => {
                    error!("Failed to save dicom state meta: {}", e);
                    // 回滚事务
                    if let Err(rollback_err) = tx.rollback().await {
                        error!("Failed to rollback transaction: {}", rollback_err);
                    }
                    return Err(DbError::DatabaseError(e));
                }
            }
        }

        // 提交事务
        match tx.commit().await {
            Ok(_) => Ok(()),
            Err(e) => {
                error!("Failed to commit transaction: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn get_state_metaes(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<Vec<DicomStateMeta>, DbError> {
        let pool = self.pool.clone();
        match sqlx::query_as(
            "SELECT * FROM dicom_state_meta WHERE tenant_id = $1 AND study_uid = $2",
        )
        .bind(tenant_id)
        .bind(study_uid)
        .fetch_all(&pool)
        .await
        {
            Ok(result) => {
                tracing::debug!(
                    "Retrieved {} state meta records for tenant_id: {}, study_uid: {}",
                    result.len(),
                    tenant_id,
                    study_uid
                );
                Ok(result)
            }
            Err(e) => {
                error!(
                    "Failed to get state meta info for tenant_id: {}, study_uid: {}: {}",
                    tenant_id, study_uid, e
                );
                Err(DbError::DatabaseError(e))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_ext::*;
    use chrono::{NaiveDate, NaiveTime};
    use sqlx::postgres::PgPoolOptions;
    #[tokio::test]
    async fn test_get_study_info() -> Result<(), Box<dyn std::error::Error>> {
        // 连接到 PostgreSQL 数据库
        let pool = match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(5))
            .connect("postgresql://root:jp%23123@192.168.1.14:5432/postgres")
            .await
        {
            Ok(pool) => pool,
            Err(err) => panic!("Error connecting to PostgreSQL: {}", err),
        };

        let db_provider = PgDbProvider::new(pool);

        let tenant_id = "1234567890";
        let study_uid = "1.2.156.112605.0.1685486876.2025061710152134339.2.1.1";

        // 添加超时包装
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            db_provider.get_study_info(tenant_id, study_uid),
        )
        .await??;

        // 验证结果
        assert!(result.is_some(), "Expected study info to be found");

        let study_info = result.unwrap();
        assert_eq!(study_info.tenant_id, tenant_id);
        assert_eq!(study_info.study_instance_uid, study_uid);

        Ok(())
    }

    #[tokio::test]
    async fn test_get_state_info() -> Result<(), Box<dyn std::error::Error>> {
        // 连接到 PostgreSQL 数据库
        let pool = match PgPoolOptions::new()
            .max_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(5)) // 添加超时
            .connect("postgresql://root:jp%23123@192.168.1.14:5432/postgres")
            .await
        {
            Ok(pool) => pool,
            Err(err) => panic!("Error connecting to PostgreSQL: {}", err),
        };

        let db_provider = PgDbProvider::new(pool);

        let tenant_id = "1234567890";
        let study_uid = "1.2.156.112605.0.1685486876.2025061710152134339.2.1.1";

        // 添加超时包装
        let result = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            db_provider.get_state_metaes(&tenant_id, &study_uid),
        )
        .await??;

        // 验证保存成功
        // 验证返回结果
        assert_eq!(result.len(), 14, "Expected 14 records for the study_uid");

        // 验证每条记录的 tenant_id 和 study_uid 是否正确
        for state_meta in result {
            assert_eq!(state_meta.tenant_id.as_str(), tenant_id);
            assert_eq!(state_meta.study_uid.as_str(), study_uid);
        }

        Ok(())
    }
    #[tokio::test]
    async fn test_save_state_info() -> Result<(), Box<dyn std::error::Error>> {
        // 连接到 PostgreSQL 数据库
        let pool = match PgPoolOptions::new()
            .max_connections(5)
            .connect("postgresql://root:jp%23123@192.168.1.14:5432/postgres")
            .await
        {
            Ok(pool) => pool,
            Err(err) => panic!("Error connecting to PostgreSQL: {}", err),
        };

        let db_provider = PgDbProvider::new(pool);

        // 创建测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_123".to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_456".to_string())?;
        let study_uid = SopUidString::try_from("1.2.3.4.5.6.7.8.9")?;
        let series_uid = SopUidString::try_from("9.8.7.6.5.4.3.2.1")?;
        let study_uid_hash = UidHashString::make_from_db("1.2.3.4.5.6.7.8.9");
        let series_uid_hash = UidHashString::make_from_db("9.8.7.6.5.4.3.2.1");
        let study_date_origin = DicomDateString::try_from("20231201".to_string())?;
        let accession_number = BoundedString::<16>::try_from("ACC123456".to_string())?;
        let modality = Some(BoundedString::<16>::try_from("CT".to_string())?);
        let series_number = Some(1);
        let series_date = Some(NaiveDate::from_ymd_opt(2023, 12, 1).unwrap());
        let series_time = Some(NaiveTime::parse_from_str("120000", "%H%M%S")?);
        let series_description = Some(BoundedString::<256>::try_from("Test Series".to_string())?);
        let body_part_examined = Some(BoundedString::<64>::try_from("CHEST".to_string())?);
        let protocol_name = Some(BoundedString::<64>::try_from("CHEST CT".to_string())?);
        let study_date = NaiveDate::from_ymd_opt(2023, 12, 1).unwrap();
        let study_time = Some(NaiveTime::parse_from_str("120000", "%H%M%S")?);
        let study_id = Some(BoundedString::<16>::try_from("STUDY123".to_string())?);
        let study_description = Some(BoundedString::<64>::try_from("Test Study".to_string())?);
        let patient_age = Some(BoundedString::<16>::try_from("045Y".to_string())?);
        let patient_sex = Some(BoundedString::<1>::try_from("M".to_string())?);
        let patient_name = Some(BoundedString::<64>::try_from("TEST^PATIENT".to_string())?);
        let patient_birth_date = Some(NaiveDate::from_ymd_opt(1978, 1, 1).unwrap());
        let patient_birth_time = Some(NaiveTime::parse_from_str("080000", "%H%M%S")?);
        let created_time = chrono::Local::now().naive_local();
        let updated_time = chrono::Local::now().naive_local();

        // 创建 DicomStateMeta 实例
        let state_meta = DicomStateMeta {
            tenant_id,
            patient_id,
            study_uid,
            series_uid,
            study_uid_hash,
            series_uid_hash,
            study_date_origin,
            patient_name,
            patient_sex,
            patient_birth_date,
            patient_birth_time,
            patient_age,
            patient_size: Some(175.5),
            patient_weight: Some(70.2),
            study_date,
            study_time,
            accession_number,
            study_id,
            study_description,

            modality,
            series_number,
            series_date,
            series_time,
            series_description,
            body_part_examined,
            protocol_name,
            series_related_instances: None,
            created_time,
            updated_time,
        };

        // 执行保存操作
        let result = db_provider.save_state_info(&state_meta).await;

        // 验证保存成功
        assert!(
            result.is_ok(),
            "Failed to save DicomStateMeta: {:?}",
            result.err()
        );

        // 验证数据是否正确保存到数据库
        let saved_series = db_provider
            .get_series_info("test_tenant_123", "9.8.7.6.5.4.3.2.1")
            .await?;

        assert!(saved_series.is_some());
        let saved_series = saved_series.unwrap();
        assert_eq!(saved_series.tenant_id, "test_tenant_123");
        assert_eq!(saved_series.series_instance_uid, "9.8.7.6.5.4.3.2.1");
        // 修改：将 Option<String> 转换为 String 进行比较
        assert_eq!(saved_series.modality, Some("CT".to_string()));

        // 验证研究信息是否正确保存
        let saved_study = db_provider
            .get_study_info("test_tenant_123", "1.2.3.4.5.6.7.8.9")
            .await?;

        assert!(saved_study.is_some());
        let saved_study = saved_study.unwrap();
        assert_eq!(saved_study.tenant_id, "test_tenant_123");
        assert_eq!(saved_study.study_instance_uid, "1.2.3.4.5.6.7.8.9");
        assert_eq!(saved_study.patient_age, Some("045Y".to_string()));

        Ok(())
    }
}
