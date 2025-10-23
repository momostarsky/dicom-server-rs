use crate::database_entities::{SeriesEntity, StudyEntity};
use crate::database_provider::{DbError, DbProvider};
use crate::dicom_utils::{parse_dicom_time_from_str};
use async_trait::async_trait;


use sqlx::mysql::MySqlRow;
use sqlx::{Database, FromRow, MySql, MySqlPool, Row};
use tracing::{error, info};
use crate::dicom_object_meta::DicomStateMeta;
use crate::string_ext::UidHashValue;

impl sqlx::Type<MySql> for UidHashValue {
    fn type_info() -> <MySql as Database>::TypeInfo {
        <i64 as sqlx::Type<MySql>>::type_info()
    }
}


impl FromRow<'_, MySqlRow> for SeriesEntity {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(SeriesEntity {
            tenant_id: row.get("tenant_id"),
            series_instance_uid: row.get("series_uid"),
            study_instance_uid: row.get("study_uid"),
            patient_id: row.get("patient_id"),
            modality: row.get("modality"),
            series_number: row.get("series_number"),
            series_date: row.get("series_date"),
            series_time: parse_dicom_time_from_str(row.get("series_time")),
            series_description: row.get("series_description"),
            body_part_examined: row.get("body_part_examined"),
            protocol_name: row.get("protocol_name"),
            created_time: row.get("created_time"),
            updated_time: row.get("updated_time"),
        })
    }
}

impl FromRow<'_, MySqlRow> for StudyEntity {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        // 先获取为字符串，再解析为 u64

        Ok(StudyEntity {
            tenant_id: row.get("tenant_id"),
            study_instance_uid: row.get("study_uid"),
            patient_id: row.get("patient_id"),
            study_date: row.get("study_date"),
            study_time: parse_dicom_time_from_str(row.get("study_time")),
            accession_number: row.get("accession_number"),
            study_id: row.get("study_id"),
            study_description: row.get("study_description"),
            patient_age: row.get("patient_age"),
            patient_size: row.get("patient_size"),
            patient_weight: row.get("patient_weight"),
            patient_sex: row.get("patient_sex"),
            patient_name: row.get("patient_name"),
            patient_birth_date: row.get("patient_birth_date"),
            patient_birth_time: row.get("patient_birth_time"),
            study_uid_hash: UidHashValue::from(row.get::<i64,_>("study_uid_hash")),
            study_date_origin: row.get("study_date_origin"),
        })
    }
}

pub struct MySqlProvider {
    pub pool: MySqlPool,
}

impl MySqlProvider {
    // 更新查询语句以包含新增字段
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
             FROM   dicom_state_meta  WHERE tenant_id = ?  and series_uid= ?"#;

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
            FROM dicom_state_meta   WHERE tenant_id = ? AND study_uid = ?"#;

    pub fn new(pool: MySqlPool) -> Self {
        info!("MySqlProvider created with pool: {:?}", pool);

        Self { pool }
    }
}

#[async_trait]
impl DbProvider for MySqlProvider {
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

    // 查询语句常量

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
        todo!()
    }
}
