use crate::dicom_dbprovider::{DbError, DbProvider};
use crate::dicom_meta::DicomStateMeta;
use async_trait::async_trait;
use mysql::prelude::*;
use mysql::*;
pub struct MySqlDbProvider {
    db_connection_string: String,
}

impl MySqlDbProvider {
    pub fn new(db_connection_string: String) -> Self {
        MySqlDbProvider {
            db_connection_string,
        }
    }
}
 #[async_trait]
impl DbProvider for MySqlDbProvider {
    async fn save_state_info(&self, state_meta: &DicomStateMeta) -> Result<(), DbError> {
        // 创建数据库连接
        let mut conn = mysql::Conn::new(self.db_connection_string.as_str())
            .map_err(|e| DbError::DatabaseError(format!("Failed to connect to MySQL: {}", e)))?;

        // 准备SQL语句
        let query = r#"
            INSERT INTO dicom_state_meta (
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
                patient_size,
                patient_weight,
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
                series_related_instances,
                created_time,
                updated_time
            ) VALUES (
                :tenant_id,
                :patient_id,
                :study_uid,
                :series_uid,
                :study_uid_hash,
                :series_uid_hash,
                :study_date_origin,
                :patient_name,
                :patient_sex,
                :patient_birth_date,
                :patient_birth_time,
                :patient_age,
                :patient_size,
                :patient_weight,
                :study_date,
                :study_time,
                :accession_number,
                :study_id,
                :study_description,
                :modality,
                :series_number,
                :series_date,
                :series_time,
                :series_description,
                :body_part_examined,
                :protocol_name,
                :series_related_instances,
                :created_time,
                :updated_time
            ) ON DUPLICATE KEY UPDATE
                patient_id = VALUES(patient_id),
                study_uid_hash = VALUES(study_uid_hash),
                series_uid_hash = VALUES(series_uid_hash),
                study_date_origin = VALUES(study_date_origin),
                patient_name = VALUES(patient_name),
                patient_sex = VALUES(patient_sex),
                patient_birth_date = VALUES(patient_birth_date),
                patient_birth_time = VALUES(patient_birth_time),
                patient_age = VALUES(patient_age),
                patient_size = VALUES(patient_size),
                patient_weight = VALUES(patient_weight),
                study_date = VALUES(study_date),
                study_time = VALUES(study_time),
                accession_number = VALUES(accession_number),
                study_id = VALUES(study_id),
                study_description = VALUES(study_description),
                modality = VALUES(modality),
                series_number = VALUES(series_number),
                series_date = VALUES(series_date),
                series_time = VALUES(series_time),
                series_description = VALUES(series_description),
                body_part_examined = VALUES(body_part_examined),
                protocol_name = VALUES(protocol_name),
                series_related_instances = VALUES(series_related_instances),
                updated_time = VALUES(updated_time)
        "#;

        // 执行参数化查询
        conn.exec_drop(
            query,
            params! {
                "tenant_id" => &state_meta.tenant_id,
                "patient_id" => &state_meta.patient_id,
                "study_uid" => &state_meta.study_uid,
                "series_uid" => &state_meta.series_uid,
                "study_uid_hash" => &state_meta.study_uid_hash,
                "series_uid_hash" => &state_meta.series_uid_hash,
                "study_date_origin" => &state_meta.study_date_origin,
                "patient_name" => &state_meta.patient_name,
                "patient_sex" => &state_meta.patient_sex,
                "patient_birth_date" => &state_meta.patient_birth_date,
                "patient_birth_time" => &state_meta.patient_birth_time,
                "patient_age" => &state_meta.patient_age,
                "patient_size" => &state_meta.patient_size,
                "patient_weight" => &state_meta.patient_weight,
                "study_date" => &state_meta.study_date,
                "study_time" => &state_meta.study_time,
                "accession_number" => &state_meta.accession_number,
                "study_id" => &state_meta.study_id,
                "study_description" => &state_meta.study_description,
                "modality" => &state_meta.modality,
                "series_number" => &state_meta.series_number,
                "series_date" => &state_meta.series_date,
                "series_time" => &state_meta.series_time,
                "series_description" => &state_meta.series_description,
                "body_part_examined" => &state_meta.body_part_examined,
                "protocol_name" => &state_meta.protocol_name,
                "series_related_instances" => &state_meta.series_related_instances,
                "created_time" => &state_meta.created_time,
                "updated_time" => &state_meta.updated_time,
            },
        )
        .map_err(|e| DbError::DatabaseError(format!("Failed to execute query: {}", e)))?;

        Ok(())
    }

    async fn save_state_list(&self, state_meta_list: &[DicomStateMeta]) -> Result<(), DbError> {
        if state_meta_list.is_empty() {
            return Ok(());
        }

        // 创建数据库连接
        let mut conn = mysql::Conn::new(self.db_connection_string.as_str())
            .map_err(|e| DbError::DatabaseError(format!("Failed to connect to MySQL: {}", e)))?;

        // 开始事务
        conn.query_drop("START TRANSACTION")
            .map_err(|e| DbError::DatabaseError(format!("Failed to start transaction: {}", e)))?;

        // 准备SQL语句
        let query = r#"
            INSERT INTO dicom_state_meta (
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
                patient_size,
                patient_weight,
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
                series_related_instances,
                created_time,
                updated_time
            ) VALUES (
                :tenant_id,
                :patient_id,
                :study_uid,
                :series_uid,
                :study_uid_hash,
                :series_uid_hash,
                :study_date_origin,
                :patient_name,
                :patient_sex,
                :patient_birth_date,
                :patient_birth_time,
                :patient_age,
                :patient_size,
                :patient_weight,
                :study_date,
                :study_time,
                :accession_number,
                :study_id,
                :study_description,
                :modality,
                :series_number,
                :series_date,
                :series_time,
                :series_description,
                :body_part_examined,
                :protocol_name,
                :series_related_instances,
                :created_time,
                :updated_time
            ) ON DUPLICATE KEY UPDATE
                patient_id = VALUES(patient_id),
                study_uid_hash = VALUES(study_uid_hash),
                series_uid_hash = VALUES(series_uid_hash),
                study_date_origin = VALUES(study_date_origin),
                patient_name = VALUES(patient_name),
                patient_sex = VALUES(patient_sex),
                patient_birth_date = VALUES(patient_birth_date),
                patient_birth_time = VALUES(patient_birth_time),
                patient_age = VALUES(patient_age),
                patient_size = VALUES(patient_size),
                patient_weight = VALUES(patient_weight),
                study_date = VALUES(study_date),
                study_time = VALUES(study_time),
                accession_number = VALUES(accession_number),
                study_id = VALUES(study_id),
                study_description = VALUES(study_description),
                modality = VALUES(modality),
                series_number = VALUES(series_number),
                series_date = VALUES(series_date),
                series_time = VALUES(series_time),
                series_description = VALUES(series_description),
                body_part_examined = VALUES(body_part_examined),
                protocol_name = VALUES(protocol_name),
                series_related_instances = VALUES(series_related_instances),
                updated_time = VALUES(updated_time)
        "#;

        // 批量执行插入操作
        for state_meta in state_meta_list {
            let result = conn.exec_drop(
                query,
                params! {
                    "tenant_id" => &state_meta.tenant_id,
                    "patient_id" => &state_meta.patient_id,
                    "study_uid" => &state_meta.study_uid,
                    "series_uid" => &state_meta.series_uid,
                    "study_uid_hash" => &state_meta.study_uid_hash,
                    "series_uid_hash" => &state_meta.series_uid_hash,
                    "study_date_origin" => &state_meta.study_date_origin,
                    "patient_name" => &state_meta.patient_name,
                    "patient_sex" => &state_meta.patient_sex,
                    "patient_birth_date" => &state_meta.patient_birth_date,
                    "patient_birth_time" => &state_meta.patient_birth_time,
                    "patient_age" => &state_meta.patient_age,
                    "patient_size" => &state_meta.patient_size,
                    "patient_weight" => &state_meta.patient_weight,
                    "study_date" => &state_meta.study_date,
                    "study_time" => &state_meta.study_time,
                    "accession_number" => &state_meta.accession_number,
                    "study_id" => &state_meta.study_id,
                    "study_description" => &state_meta.study_description,
                    "modality" => &state_meta.modality,
                    "series_number" => &state_meta.series_number,
                    "series_date" => &state_meta.series_date,
                    "series_time" => &state_meta.series_time,
                    "series_description" => &state_meta.series_description,
                    "body_part_examined" => &state_meta.body_part_examined,
                    "protocol_name" => &state_meta.protocol_name,
                    "series_related_instances" => &state_meta.series_related_instances,
                    "created_time" => &state_meta.created_time,
                    "updated_time" => &state_meta.updated_time,
                },
            );

            // 如果任何一个操作失败，回滚事务并返回错误
            if let Err(e) = result {
                conn.query_drop("ROLLBACK")
                    .map_err(|rollback_err| {
                        DbError::DatabaseError(format!(
                            "Failed to rollback transaction after error {}: {}",
                            e, rollback_err
                        ))
                    })?;

                return Err(DbError::DatabaseError(format!(
                    "Failed to execute query for state meta: {}",
                    e
                )));
            }
        }

        // 提交事务
        conn.query_drop("COMMIT")
            .map_err(|e| DbError::DatabaseError(format!("Failed to commit transaction: {}", e)))?;

        Ok(())
    }

    async fn get_state_metaes(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<Vec<DicomStateMeta>, DbError> {
        // 创建数据库连接
        let mut conn = mysql::Conn::new(self.db_connection_string.as_str())
            .map_err(|e| DbError::DatabaseError(format!("Failed to connect to MySQL: {}", e)))?;

        // 准备查询语句
        let query = r#"
            SELECT
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
                patient_size,
                patient_weight,
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
                series_related_instances,
                created_time,
                updated_time
            FROM dicom_state_meta
            WHERE tenant_id = :tenant_id AND study_uid = :study_uid
        "#;

        // 执行查询
        let result: Vec<DicomStateMeta> = conn
            .exec_map(
                query,
                params! {
                    "tenant_id" => tenant_id,
                    "study_uid" => study_uid,
                },
                |row: mysql::Row| {
                    // 映射查询结果到 DicomStateMeta 结构体
                    DicomStateMeta {
                        tenant_id: row.get("tenant_id").unwrap_or_default(),
                        patient_id: row.get("patient_id").unwrap_or_default(),
                        study_uid: row.get("study_uid").unwrap_or_default(),
                        series_uid: row.get("series_uid").unwrap_or_default(),
                        study_uid_hash: row.get("study_uid_hash").unwrap_or_default(),
                        series_uid_hash: row.get("series_uid_hash").unwrap_or_default(),
                        study_date_origin: row.get("study_date_origin").unwrap_or_default(),
                        patient_name: row.get("patient_name").unwrap_or_default(),
                        patient_sex: row.get("patient_sex").unwrap_or_default(),
                        patient_birth_date: row.get("patient_birth_date").unwrap_or_default(),
                        patient_birth_time: row.get("patient_birth_time").unwrap_or_default(),
                        patient_age: row.get("patient_age").unwrap_or_default(),
                        patient_size: row.get("patient_size").unwrap_or_default(),
                        patient_weight: row.get("patient_weight").unwrap_or_default(),
                        study_date: row.get("study_date").unwrap_or_default(),
                        study_time: row.get("study_time").unwrap_or_default(),
                        accession_number: row.get("accession_number").unwrap_or_default(),
                        study_id: row.get("study_id").unwrap_or_default(),
                        study_description: row.get("study_description").unwrap_or_default(),
                        modality: row.get("modality").unwrap_or_default(),
                        series_number: row.get("series_number").unwrap_or_default(),
                        series_date: row.get("series_date").unwrap_or_default(),
                        series_time: row.get("series_time").unwrap_or_default(),
                        series_description: row.get("series_description").unwrap_or_default(),
                        body_part_examined: row.get("body_part_examined").unwrap_or_default(),
                        protocol_name: row.get("protocol_name").unwrap_or_default(),
                        series_related_instances: row.get("series_related_instances").unwrap_or_default(),
                        created_time: row.get("created_time").unwrap_or_default(),
                        updated_time: row.get("updated_time").unwrap_or_default(),
                    }
                },
            )
            .map_err(|e| DbError::DatabaseError(format!("Failed to execute query: {}", e)))?;

        Ok(result)
    }
}

