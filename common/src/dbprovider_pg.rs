use async_trait::async_trait;
use sqlx::{Database, Encode, Error, FromRow, PgPool, Row};
use sqlx::encode::IsNull;
use sqlx::error::BoxDynError;
use sqlx::postgres::PgRow;

use tracing::error;
use crate::database_entities::{SeriesEntity, StudyEntity};
use crate::database_provider::{DbError, DbProvider};
use crate::dicom_object_meta::DicomStateMeta;


use sqlx::Postgres;
use crate::string_ext::{BoundedString, DicomDateString, ExtDicomTime, FixedLengthString, SopUidString, UidHashValue, UuidString};


impl sqlx::Type<Postgres> for UidHashValue {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        <i64 as sqlx::Type<Postgres>>::type_info()
    }
}
impl Encode<'_, Postgres> for  UidHashValue {
    fn encode_by_ref(&self, buf: &mut <Postgres as Database>::ArgumentBuffer<'_>) -> Result<IsNull, BoxDynError> {
        <i64 as Encode<Postgres>>::encode(self.0 as i64, buf)
    }
}




// 为 BoundedString 实现 PostgreSQL 的 Encode trait
impl<const N: usize> Encode<'_, Postgres> for BoundedString<N> {
    fn encode_by_ref(&self, buf: &mut <Postgres as Database>::ArgumentBuffer<'_>) -> std::result::Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.value.as_str(), buf)
    }
}


impl<const N: usize> Encode<'_, Postgres> for FixedLengthString<N> {
    fn encode_by_ref(&self, buf: &mut <Postgres as Database>::ArgumentBuffer<'_>) -> std::result::Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.value.as_str(), buf)
    }
}


impl Encode<'_, Postgres> for SopUidString {
    fn encode_by_ref(&self, buf: &mut <Postgres as Database>::ArgumentBuffer<'_>) -> std::result::Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}

impl Encode<'_, Postgres> for UuidString {
    fn encode_by_ref(&self, buf: &mut <Postgres as Database>::ArgumentBuffer<'_>) -> std::result::Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}
impl Encode<'_, Postgres> for DicomDateString {
    fn encode_by_ref(&self, buf: &mut <Postgres as Database>::ArgumentBuffer<'_>) -> std::result::Result<IsNull, BoxDynError> {
        <&str as Encode<Postgres>>::encode(self.as_str(), buf)
    }
}
impl Encode<'_, Postgres> for ExtDicomTime {
    fn encode_by_ref(&self, buf: &mut <Postgres as Database>::ArgumentBuffer<'_>) -> std::result::Result<IsNull, BoxDynError> {
        match &self.value {
            Some(time) => {
                let time_str = time.format("%H%M%S%.f").to_string();
                <&str as Encode<Postgres>>::encode(time_str.as_str(), buf)
            }
            None => <&str as Encode<Postgres>>::encode("", buf)
        }
    }
}



// 为 FixedLengthString 实现 PostgreSQL 的 Type trait
impl<const N: usize> sqlx::Type<Postgres> for FixedLengthString<N> {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        <&str as sqlx::Type<Postgres>>::type_info()
    }
}


impl<const N: usize> sqlx::Type<Postgres> for  BoundedString<N>  {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        <&str as sqlx::Type<Postgres>>::type_info()
    }
}


// 为 SopUidString 实现 PostgreSQL 的 Type trait
impl sqlx::Type<Postgres> for SopUidString {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        <&str as sqlx::Type<Postgres>>::type_info()
    }
}

// 为 UuidString 实现 PostgreSQL 的 Type trait
impl sqlx::Type<Postgres> for UuidString {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        <&str as sqlx::Type<Postgres>>::type_info()
    }
}

// 为 DicomDateString 实现 PostgreSQL 的 Type trait
impl sqlx::Type<Postgres> for DicomDateString {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        <&str as sqlx::Type<Postgres>>::type_info()
    }
}

// 为 ExtDicomTime 实现 PostgreSQL 的 Type trait
impl sqlx::Type<Postgres> for ExtDicomTime {
    fn type_info() -> <Postgres as Database>::TypeInfo {
        <&str as sqlx::Type<Postgres>>::type_info()
    }
}

impl FromRow<'_, PgRow> for SeriesEntity {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, Error> {
       Ok(SeriesEntity{
           tenant_id: row.get("tenant_id"),
           series_instance_uid: row.get("series_uid"),
           study_instance_uid: row.get("study_uid"),
           patient_id: row.get("patient_id"),
           modality:row.get("modality"),
           series_number: row.get("series_number"),
           series_date:  row.get("series_date"),
           series_time:  row.get("series_time"),
           series_description:  row.get("series_description"),
           body_part_examined:  row.get("body_part_examined"),
           protocol_name:  row.get("protocol_name"),
           created_time:  row.get("created_time"),
           updated_time:  row.get("updated_time"),
       })
    }
}

impl FromRow<'_, PgRow> for StudyEntity {
    fn from_row(row: &'_ PgRow) -> std::result::Result<Self, Error> {
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
            study_uid_hash: UidHashValue::from(row.get::<i64,_>("study_uid_hash")),
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

pub struct  PgDbProvider {
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
    async fn get_study_info(&self, tenant_id: &str, study_uid: &str) -> std::result::Result<Option<StudyEntity>, DbError> {
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

    async fn get_series_info(&self, tenant_id: &str, series_uid: &str) -> std::result::Result<Option<SeriesEntity>, DbError> {
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


    async fn save_state_info(&self, state_meta: &DicomStateMeta) -> std::result::Result<(), DbError> {
        let pool = self.pool.clone();
        match sqlx::query::<Postgres>(Self::INSERT_OR_UPDATE_STATE_META_QUERY)
                        .bind(state_meta.tenant_id.as_str())
                        .bind(state_meta.patient_id.as_str())
                        .bind(state_meta.study_uid.as_str())
                        .bind(state_meta.series_uid.as_str())
                        .bind(state_meta.study_uid_hash)
                        .bind(state_meta.series_uid_hash)
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

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string_ext::*;
    use chrono::NaiveDate;
    use sqlx::postgres::PgPoolOptions;

    #[tokio::test]
    async fn test_save_state_info() -> Result<(), Box<dyn std::error::Error>> {
        // 连接到 PostgreSQL 数据库
        let pool =match  PgPoolOptions::new()
            .max_connections(5)
            .connect("postgresql://root:jp%23123@192.168.1.14:5432/postgres")
            .await{
            Ok(pool) => pool,
            Err(err) => panic!("Error connecting to PostgreSQL: {}", err),
        };

        let db_provider = PgDbProvider::new(pool);

        // 创建测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_123".to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_456".to_string())?;
        let study_uid = SopUidString::try_from("1.2.3.4.5.6.7.8.9")?;
        let series_uid = SopUidString::try_from("9.8.7.6.5.4.3.2.1")?;
        let study_uid_hash = UidHashValue::from(12345i64);
        let series_uid_hash = UidHashValue::from(67890i64);
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
        let created_time = Some(chrono::Local::now().naive_local());
        let updated_time = Some(chrono::Local::now().naive_local());

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
