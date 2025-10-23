use crate::database_entities::{SeriesEntity, StudyEntity};
use crate::database_provider::{DbError, DbProvider};
use async_trait::async_trait;


use sqlx::mysql::MySqlRow;
use sqlx::{Database, Encode, FromRow, MySql, MySqlPool, Row};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use tracing::{error, info};
use crate::dicom_object_meta::DicomStateMeta;
use crate::string_ext::UidHashString;

impl sqlx::Type<MySql> for UidHashString {
    fn type_info() -> <MySql as Database>::TypeInfo {
        <i64 as sqlx::Type<MySql>>::type_info()
    }
}

impl Encode<'_, MySql> for  UidHashString {
    fn encode_by_ref(&self, buf: &mut <MySql as Database>::ArgumentBuffer<'_>) -> Result<IsNull, BoxDynError> {
        <&str as Encode<MySql>>::encode(&self.0.as_str(), buf)
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
            series_time:  row.get("series_time"),
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
            study_time: row.get("study_time"),
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
            study_uid_hash: UidHashString::from_string(row.get::<String,_>("study_uid_hash")),
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


    // 添加插入或更新的查询语句
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
            ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        )
        ON DUPLICATE KEY UPDATE
            patient_id = VALUES(patient_id),
            study_uid_hash = VALUES(study_uid_hash),
            series_uid_hash = VALUES(series_uid_hash),
            study_date_origin = VALUES(study_date_origin),
            accession_number = VALUES(accession_number),
            modality = VALUES(modality),
            series_number = VALUES(series_number),
            series_date = VALUES(series_date),
            series_time = VALUES(series_time),
            series_description = VALUES(series_description),
            body_part_examined = VALUES(body_part_examined),
            protocol_name = VALUES(protocol_name),
            study_date = VALUES(study_date),
            study_time = VALUES(study_time),
            study_id = VALUES(study_id),
            study_description = VALUES(study_description),
            patient_age = VALUES(patient_age),
            patient_size = VALUES(patient_size),
            patient_weight = VALUES(patient_weight),
            patient_sex = VALUES(patient_sex),
            patient_name = VALUES(patient_name),
            patient_birth_date = VALUES(patient_birth_date),
            patient_birth_time = VALUES(patient_birth_time),
            updated_time = VALUES(updated_time)
    "#;

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
        let pool = self.pool.clone();
        match sqlx::query::<MySql>(Self::INSERT_OR_UPDATE_STATE_META_QUERY)
            .bind(state_meta.tenant_id.as_str())
            .bind(state_meta.patient_id.as_str())
            .bind(state_meta.study_uid.as_str())
            .bind(state_meta.series_uid.as_str())
            .bind(state_meta.study_uid_hash.as_str())
            .bind(state_meta.series_uid_hash.as_str())
            .bind(state_meta.study_date_origin.as_str())
            .bind(state_meta.accession_number.as_str())
            .bind(state_meta.modality.as_ref().unwrap().as_str())
            .bind(state_meta.series_number.unwrap())
            .bind(state_meta.series_date)
            .bind(state_meta.series_time.as_ref().unwrap().as_naive_time())
            .bind(state_meta.series_description.as_ref().unwrap().as_str())
            .bind(state_meta.body_part_examined.as_ref().unwrap().as_str())
            .bind(state_meta.protocol_name.as_ref().unwrap().as_str())
            .bind(state_meta.study_date)
            .bind(state_meta.study_time.as_ref().unwrap().as_naive_time())
            .bind(state_meta.study_id.as_ref().unwrap().as_str())
            .bind(state_meta.study_description.as_ref().unwrap().as_str())
            .bind(state_meta.patient_age.as_ref().unwrap().as_str())
            .bind(state_meta.patient_size)
            .bind(state_meta.patient_weight)
            .bind(state_meta.patient_sex.as_ref().unwrap().as_str())
            .bind(state_meta.patient_name.as_ref().unwrap().as_str())
            .bind(state_meta.patient_birth_date)
            .bind(state_meta.patient_birth_time.as_ref().unwrap().as_naive_time())
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
            match sqlx::query::<MySql>(Self::INSERT_OR_UPDATE_STATE_META_QUERY)
                .bind(state_meta.tenant_id.as_str())
                .bind(state_meta.patient_id.as_str())
                .bind(state_meta.study_uid.as_str())
                .bind(state_meta.series_uid.as_str())
                .bind(state_meta.study_uid_hash.as_str())
                .bind(state_meta.series_uid_hash.as_str())
                .bind(state_meta.study_date_origin.as_str())
                .bind(state_meta.accession_number.as_str())
                .bind(state_meta.modality.as_ref().unwrap().as_str())
                .bind(state_meta.series_number.unwrap())
                .bind(state_meta.series_date)
                .bind(state_meta.series_time.as_ref().unwrap().as_naive_time())
                .bind(state_meta.series_description.as_ref().unwrap().as_str())
                .bind(state_meta.body_part_examined.as_ref().unwrap().as_str())
                .bind(state_meta.protocol_name.as_ref().unwrap().as_str())
                .bind(state_meta.study_date)
                .bind(state_meta.study_time.as_ref().unwrap().as_naive_time())
                .bind(state_meta.study_id.as_ref().unwrap().as_str())
                .bind(state_meta.study_description.as_ref().unwrap().as_str())
                .bind(state_meta.patient_age.as_ref().unwrap().as_str())
                .bind(state_meta.patient_size)
                .bind(state_meta.patient_weight)
                .bind(state_meta.patient_sex.as_ref().unwrap().as_str())
                .bind(state_meta.patient_name.as_ref().unwrap().as_str())
                .bind(state_meta.patient_birth_date)
                .bind(state_meta.patient_birth_time.as_ref().unwrap().as_naive_time())
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
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_ext::*;
    use chrono::NaiveDate;
    use sqlx::mysql::MySqlPoolOptions;


    #[tokio::test]
    async fn test_save_state_info_mysql() -> Result<(), Box<dyn std::error::Error>> {
        // 连接到 MySQL 数据库 (请根据实际环境修改连接信息)
        let pool = match MySqlPoolOptions::new()
            .max_connections(5)
            .connect("mysql://dicomstore:hzjp%23123@192.168.1.14:3306/dicomdb")
            .await {
            Ok(pool) => pool,
            Err(err) => {
                eprintln!("Warning: Error connecting to MySQL: {}", err);
                // 如果无法连接到数据库，跳过测试而不是panic
                return Ok(());
            },
        };

        let db_provider = MySqlProvider::new(pool);

        // 创建测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_123".to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_456".to_string())?;
        let study_uid = SopUidString::try_from("1.2.3.4.5.6.7.8.9")?;
        let series_uid = SopUidString::try_from("9.8.7.6.5.4.3.2.1")?;
        let study_uid_hash = UidHashString::make_from("1.2.3.4.5.6.7.8.9");
        let series_uid_hash = UidHashString::make_from("9.8.7.6.5.4.3.2.1");
        let study_date_origin = DicomDateString::try_from("20231201".to_string())?;
        let accession_number = BoundedString::<16>::try_from("ACC123456".to_string())?;
        let modality = Some(BoundedString::<16>::try_from("CT".to_string())?);
        let series_number = Some(1);
        let series_date = Some(NaiveDate::from_ymd_opt(2023, 12, 1).unwrap());
        let series_time = Some(ExtDicomTime::try_from("120000".to_string())?);
        let series_description = Some(BoundedString::<256>::try_from("Test Series".to_string())?);
        let body_part_examined = Some(BoundedString::<64>::try_from("CHEST".to_string())?);
        let protocol_name = Some(BoundedString::<64>::try_from("CHEST CT".to_string())?);
        let study_date = NaiveDate::from_ymd_opt(2023, 12, 1).unwrap();
        let study_time = Some(ExtDicomTime::try_from("120000".to_string())?);
        let study_id = Some(BoundedString::<16>::try_from("STUDY123".to_string())?);
        let study_description = Some(BoundedString::<64>::try_from("Test Study".to_string())?);
        let patient_age = Some(BoundedString::<16>::try_from("045Y".to_string())?);
        let patient_sex = Some(BoundedString::<1>::try_from("M".to_string())?);
        let patient_name = Some(BoundedString::<64>::try_from("TEST^PATIENT".to_string())?);
        let patient_birth_date = Some(NaiveDate::from_ymd_opt(1978, 1, 1).unwrap());
        let patient_birth_time = Some(ExtDicomTime::try_from("080000".to_string())?);
        // 修改时间字段创建方式，确保与数据库TIMESTAMP类型兼容
        let now = chrono::Utc::now().naive_utc();
        let created_time = Some(now);
        let updated_time = Some(now);

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
            medical_alerts: None,
            allergies: None,
            pregnancy_status: None,
            occupation: None,
            study_date,
            study_time,
            accession_number,
            study_id,
            study_description,
            referring_physician_name: None,
            admission_id: None,
            performing_physician_name: None,
            modality,
            series_number,
            series_date,
            series_time,
            series_description,
            body_part_examined,
            protocol_name,
            operators_name: None,
            manufacturer: None,
            institution_name: None,
            device_serial_number: None,
            software_versions: None,
            series_related_instances: None,
            created_time,
            updated_time,
        };

        // 执行保存操作
        let result = db_provider.save_state_info(&state_meta).await;

        // 验证保存成功
        assert!(result.is_ok(), "Failed to save DicomStateMeta: {:?}", result.err());

        // 验证数据是否正确保存到数据库
        let saved_series = db_provider
            .get_series_info("test_tenant_123", "9.8.7.6.5.4.3.2.1")
            .await?;

        assert!(saved_series.is_some());
        let saved_series = saved_series.unwrap();
        assert_eq!(saved_series.tenant_id, "test_tenant_123");
        assert_eq!(saved_series.series_instance_uid, "9.8.7.6.5.4.3.2.1");
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

