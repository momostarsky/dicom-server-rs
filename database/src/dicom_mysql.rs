use crate::dicom_dbprovider::{DbError, DbProvider};
use crate::dicom_meta::{DicomJsonMeta, DicomStateMeta};
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
                conn.query_drop("ROLLBACK").map_err(|rollback_err| {
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

    async fn save_json_list(
        &self,
        json_meta_list: &[DicomJsonMeta],
    ) -> std::result::Result<(), DbError> {
        if json_meta_list.is_empty() {
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
            INSERT INTO dicom_json_meta (
                tenant_id,
                study_uid,
                series_uid,
                study_uid_hash,
                series_uid_hash,
                study_date_origin,
                created_time,
                flag_time,
                json_status,
                retry_times
            ) VALUES (
                :tenant_id,
                :study_uid,
                :series_uid,
                :study_uid_hash,
                :series_uid_hash,
                :study_date_origin,
                :created_time,
                :flag_time,
                :json_status,
                :retry_times
            ) ON DUPLICATE KEY UPDATE
                study_uid_hash = VALUES(study_uid_hash),
                series_uid_hash = VALUES(series_uid_hash),
                study_date_origin = VALUES(study_date_origin),
                created_time = VALUES(created_time),
                flag_time = VALUES(flag_time),
                json_status = VALUES(json_status),
                retry_times = VALUES(retry_times)
        "#;

        // 批量执行插入操作
        for json_meta in json_meta_list {
            let result = conn.exec_drop(
                query,
                params! {
                    "tenant_id" => &json_meta.tenant_id,
                    "study_uid" => &json_meta.study_uid,
                    "series_uid" => &json_meta.series_uid,
                    "study_uid_hash" => &json_meta.study_uid_hash,
                    "series_uid_hash" => &json_meta.series_uid_hash,
                    "study_date_origin" => &json_meta.study_date_origin,
                    "created_time" => &json_meta.created_time,
                    "flag_time" => &json_meta.flag_time,
                    "json_status" => &json_meta.json_status,
                    "retry_times" => &json_meta.retry_times,
                },
            );

            // 如果任何一个操作失败，回滚事务并返回错误
            if let Err(e) = result {
                conn.query_drop("ROLLBACK").map_err(|rollback_err| {
                    DbError::DatabaseError(format!(
                        "Failed to rollback transaction after error {}: {}",
                        e, rollback_err
                    ))
                })?;

                return Err(DbError::DatabaseError(format!(
                    "Failed to execute query for json meta: {}",
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
                        series_related_instances: row
                            .get("series_related_instances")
                            .unwrap_or_default(),
                        created_time: row.get("created_time").unwrap_or_default(),
                        updated_time: row.get("updated_time").unwrap_or_default(),
                    }
                },
            )
            .map_err(|e| DbError::DatabaseError(format!("Failed to execute query: {}", e)))?;

        Ok(result)
    }

    async fn get_json_metaes(
        &self,
        end_time: chrono::NaiveDateTime,
    ) -> std::result::Result<Vec<DicomStateMeta>, DbError> {
        // 创建数据库连接
        let mut conn = mysql::Conn::new(self.db_connection_string.as_str())
            .map_err(|e| DbError::DatabaseError(format!("Failed to connect to MySQL: {}", e)))?;

        // 准备查询语句
        let query = r#"
            Select tenant_id,
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
            From (SELECT dsm.*
                  FROM dicom_state_meta dsm
                           LEFT JOIN dicom_json_meta djm
                                     ON dsm.tenant_id = djm.tenant_id
                                         AND dsm.study_uid = djm.study_uid
                                         AND dsm.series_uid = djm.series_uid
                  WHERE djm.tenant_id IS NULL  AND　dsm.updated_time < ?
                  UNION ALL
                  SELECT dsm.*
                  FROM dicom_state_meta dsm
                           INNER JOIN dicom_json_meta djm
                                      ON dsm.tenant_id = djm.tenant_id
                                          AND dsm.study_uid = djm.study_uid
                                          AND dsm.series_uid = djm.series_uid
                  WHERE dsm.updated_time != djm.flag_time
                   AND  dsm.updated_time < ?
                  ) AS t
                  order by t.updated_time asc limit 10;
        "#;

        // 执行查询
        let result: Vec<DicomStateMeta> = conn
            .exec_map(query, params! { end_time,end_time }, |row: mysql::Row| {
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
                    series_related_instances: row
                        .get("series_related_instances")
                        .unwrap_or_default(),
                    created_time: row.get("created_time").unwrap_or_default(),
                    updated_time: row.get("updated_time").unwrap_or_default(),
                }
            })
            .map_err(|e| DbError::DatabaseError(format!("Failed to execute query: {}", e)))?;

        Ok(result)
    }

    async fn get_json_meta(
        &self,
        tenant_id: &str,
        study_uid: &str,
        series_uid: &str,
    ) -> std::result::Result<DicomJsonMeta, DbError> {
        // 创建数据库连接
        let mut conn = mysql::Conn::new(self.db_connection_string.as_str())
            .map_err(|e| DbError::DatabaseError(format!("Failed to connect to MySQL: {}", e)))?;

        // 准备查询语句
        let query = r#"
        SELECT
            tenant_id,
            study_uid,
            series_uid,
            study_uid_hash,
            series_uid_hash,
            study_date_origin,
            created_time,
            flag_time,
            json_status,
            retry_times
        FROM dicom_json_meta
        WHERE series_uid = :series_uid and tenant_id = :tenant_id and study_uid = :study_uid
    "#;

        // 执行查询并手动映射结果
        let result: Option<DicomJsonMeta> = conn
            .exec_first(
                query,
                params! {
                    "series_uid" => series_uid,
                    "tenant_id" => tenant_id,
                    "study_uid" => study_uid,
                },
            )
            .map_err(|e| DbError::DatabaseError(format!("Failed to execute query: {}", e)))?
            .map(|row: mysql::Row| DicomJsonMeta {
                tenant_id: row.get("tenant_id").unwrap_or_default(),
                study_uid: row.get("study_uid").unwrap_or_default(),
                series_uid: row.get("series_uid").unwrap_or_default(),
                study_uid_hash: row.get("study_uid_hash").unwrap_or_default(),
                series_uid_hash: row.get("series_uid_hash").unwrap_or_default(),
                study_date_origin: row.get("study_date_origin").unwrap_or_default(),
                created_time: row.get("created_time").unwrap_or_default(),
                flag_time: row.get("flag_time").unwrap_or_default(),
                json_status: row.get("json_status").unwrap_or_default(),
                retry_times: row.get("retry_times").unwrap_or_default(),
            });

        // 检查是否找到记录
        match result {
            Some(json_meta) => Ok(json_meta),
            None => Err(DbError::DatabaseError(format!(
                "DicomJsonMeta with series_uid {} not found",
                series_uid
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dicom_dbprovider::current_time;
    use crate::dicom_dbtype::*;
    use chrono::{NaiveDate, NaiveTime};
    use std::env;
    use std::ops::Sub;
    use ctor::ctor;
    use dotenv::dotenv;

    #[cfg(test)]
    #[ctor]
    fn init_tests() {
        // 这个函数会在所有测试运行之前执行一次
        dotenv().ok();
        println!("Initializing tests...");
        // 可以在这里进行全局的测试设置
    }

    #[tokio::test]
    async fn test_save_state_info() -> Result<(), Box<dyn std::error::Error>> {
        let mysql_cnn = env::var("DICOM_MySQL");
        if mysql_cnn.is_err() {
            println!("DICOM_MySQL environment variable not set");
            println!("eg:mysql://dicomstore:hzjp%23123@192.168.1.14:3306/dicomdb");
            return Ok(());
        }

        let db_provider = MySqlDbProvider::new(mysql_cnn.unwrap());

        // 创建测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_mysql_123".to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_mysql_456".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9.mysql")?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1.mysql")?;
        let study_uid_hash = BoundedString::<20>::from_str("1.2.3.4.5.6.7.8.9.my").unwrap();
        let series_uid_hash = BoundedString::<20>::from_str("9.8.7.6.5.4.3.2.1.my").unwrap();
        let study_date_origin = DicomDateString::from_db("20231201");
        let accession_number = BoundedString::<16>::try_from("ACC123456MYSQL".to_string())?;
        let modality = Some(BoundedString::<16>::try_from("MR".to_string())?);
        let series_number = Some(1);
        let series_date = Some(NaiveDate::from_ymd_opt(2023, 12, 1).unwrap());
        let series_time = Some(NaiveTime::parse_from_str("120000", "%H%M%S")?);
        let series_description = Some(BoundedString::<256>::try_from(
            "Test Series MySQL".to_string(),
        )?);
        let body_part_examined = Some(BoundedString::<64>::try_from("HEAD".to_string())?);
        let protocol_name = Some(BoundedString::<64>::try_from("HEAD MRI".to_string())?);
        let study_date = NaiveDate::from_ymd_opt(2023, 12, 1).unwrap();
        let study_time = Some(NaiveTime::parse_from_str("120000", "%H%M%S")?);
        let study_id = Some(BoundedString::<16>::try_from("STUDY123MYSQL".to_string())?);
        let study_description = Some(BoundedString::<64>::try_from(
            "Test Study MySQL".to_string(),
        )?);
        let patient_age = Some(BoundedString::<16>::try_from("045Y".to_string())?);
        let patient_sex = Some(BoundedString::<1>::try_from("F".to_string())?);
        let patient_name = Some(BoundedString::<64>::try_from(
            "TEST^PATIENT_MYSQL".to_string(),
        )?);
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
            patient_size: Some(165.5),
            patient_weight: Some(60.2),
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

        Ok(())
    }

    #[tokio::test]
    async fn test_get_state_metaes() -> Result<(), Box<dyn std::error::Error>> {
        let mysql_cnn = env::var("DICOM_MySQL");
        if mysql_cnn.is_err() {
            println!("DICOM_MySQL environment variable not set");
            return Ok(());
        }

        let db_provider = MySqlDbProvider::new(mysql_cnn.unwrap());

        let tenant_id = "1234567890";
        let study_uid = "1.2.156.112605.0.1685486876.2025061710152134339.2.1.1";

        // 执行查询操作
        let result = db_provider.get_state_metaes(tenant_id, study_uid).await;

        // 验证查询成功
        assert!(
            result.is_ok(),
            "Failed to get DicomStateMeta list: {:?}",
            result.err()
        );

        let state_meta_list = result.unwrap();

        // 验证返回结果不为空
        assert!(!state_meta_list.is_empty(), "Expected non-empty result");

        // 验证每条记录的 tenant_id 和 study_uid 是否正确
        for state_meta in state_meta_list {
            assert_eq!(state_meta.tenant_id.as_str(), tenant_id);
            assert_eq!(state_meta.study_uid.as_str(), study_uid);
            let json = serde_json::to_string_pretty(&state_meta)?;
            println!("DicomStateMeta JSON: {}", json);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_save_json_list() -> Result<(), Box<dyn std::error::Error>> {
        let mysql_cnn = env::var("DICOM_MySQL");
        if mysql_cnn.is_err() {
            println!("DICOM_MySQL environment variable not set");
            return Ok(());
        }

        let db_provider = MySqlDbProvider::new(mysql_cnn.unwrap());
        // 创建测试数据列表
        let mut json_meta_list = Vec::new();

        // 创建测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_json_list_123".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9.json.list".to_string())?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1.json.list".to_string())?;
        let study_uid_hash = BoundedString::<20>::from_str("1.2.3.4.5.6.7.8.9").unwrap();
        let series_uid_hash = BoundedString::<20>::from_str("9.8.7.6.5.4.3.2.1").unwrap();
        let study_date_origin = DicomDateString::from_db("20231203");
        let created_time = current_time();
        let flag_time = current_time();
        let json_status = 0;
        let retry_times = 0;

        let json_meta = DicomJsonMeta {
            tenant_id,
            study_uid,
            series_uid,
            study_uid_hash,
            series_uid_hash,
            study_date_origin,
            created_time,
            flag_time,
            json_status,
            retry_times,
        };

        json_meta_list.push(json_meta);

        // 执行批量保存操作
        let result = db_provider.save_json_list(&json_meta_list).await;

        // 验证保存成功
        assert!(
            result.is_ok(),
            "Failed to save DicomJsonMeta list: {:?}",
            result.err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_get_json_metaes() -> Result<(), Box<dyn std::error::Error>> {
        let mysql_cnn = env::var("DICOM_MySQL");
        if mysql_cnn.is_err() {
            println!("DICOM_MySQL environment variable not set");
            return Ok(());
        }

        let db_provider = MySqlDbProvider::new(mysql_cnn.unwrap());

        let cd = current_time();
        let cd = cd.sub(chrono::Duration::minutes(5));
        // 执行查询操作
        let result = db_provider.get_json_metaes(cd).await;

        // 验证查询成功
        assert!(
            result.is_ok(),
            "Failed to get DicomStateMeta list: {:?}",
            result.err()
        );

        let state_meta_list = result.unwrap();

        // 验证返回结果不为空
        assert!(!state_meta_list.is_empty(), "Expected non-empty result");

        // 验证每条记录的 tenant_id 和 study_uid 是否正确
        for state_meta in state_meta_list {
            let json = serde_json::to_string_pretty(&state_meta)?;
            println!("DicomStateMeta JSON: {}", json);
        }

        Ok(())
    }
    #[tokio::test]
    async fn test_save_state_list() -> Result<(), Box<dyn std::error::Error>> {
        let mysql_cnn = env::var("DICOM_MySQL");
        if mysql_cnn.is_err() {
            println!("DICOM_MySQL environment variable not set");
            return Ok(());
        }

        let db_provider = MySqlDbProvider::new(mysql_cnn.unwrap());

        // 创建测试数据列表
        let mut state_meta_list = Vec::new();

        // 创建第一个测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_mysql_list_123".to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_mysql_list_456".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9.mysql.list")?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1.mysql.list")?;
        let study_uid_hash = BoundedString::<20>::from_str("1.2.3.4.5.6.7.8.9.my").unwrap();
        let series_uid_hash = BoundedString::<20>::from_str("9.8.7.6.5.4.3.2.1.my").unwrap();
        let study_date_origin = DicomDateString::from_db("20231202");
        let accession_number = BoundedString::<16>::try_from("ACC123457MYSQL".to_string())?;
        let modality = Some(BoundedString::<16>::try_from("CT".to_string())?);
        let series_number = Some(2);
        let series_date = Some(NaiveDate::from_ymd_opt(2023, 12, 2).unwrap());
        let series_time = Some(NaiveTime::parse_from_str("130000", "%H%M%S")?);
        let series_description = Some(BoundedString::<256>::try_from(
            "Test Series List MySQL".to_string(),
        )?);
        let body_part_examined = Some(BoundedString::<64>::try_from("CHEST".to_string())?);
        let protocol_name = Some(BoundedString::<64>::try_from("CHEST CT".to_string())?);
        let study_date = NaiveDate::from_ymd_opt(2023, 12, 2).unwrap();
        let study_time = Some(NaiveTime::parse_from_str("130000", "%H%M%S")?);
        let study_id = Some(BoundedString::<16>::try_from("STUDY124MYSQL".to_string())?);
        let study_description = Some(BoundedString::<64>::try_from(
            "Test Study List MySQL".to_string(),
        )?);
        let patient_age = Some(BoundedString::<16>::try_from("046Y".to_string())?);
        let patient_sex = Some(BoundedString::<1>::try_from("M".to_string())?);
        let patient_name = Some(BoundedString::<64>::try_from(
            "TEST^PATIENT2_MYSQL".to_string(),
        )?);
        let patient_birth_date = Some(NaiveDate::from_ymd_opt(1977, 1, 1).unwrap());
        let patient_birth_time = Some(NaiveTime::parse_from_str("090000", "%H%M%S")?);
        let created_time = chrono::Local::now().naive_local();
        let updated_time = chrono::Local::now().naive_local();

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

        state_meta_list.push(state_meta);

        // 执行批量保存操作
        let result = db_provider.save_state_list(&state_meta_list).await;

        // 验证保存成功
        assert!(
            result.is_ok(),
            "Failed to save DicomStateMeta list: {:?}",
            result.err()
        );

        Ok(())
    }
}
