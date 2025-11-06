use crate::dicom_dbprovider::{DbError, DbProvider};
use crate::dicom_meta::{DicomJsonMeta, DicomStateMeta};
use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};
pub struct PgDbProvider {
    db_connection_string: String,
}

impl PgDbProvider {
    pub fn new(database_url: String) -> Self {
        Self {
            db_connection_string: database_url,
        }
    }
    async fn make_client(&self) -> Result<Client, DbError> {
        let (client, connection) =
            tokio_postgres::connect(self.db_connection_string.as_str(), NoTls)
                .await
                .map_err(|e| DbError::DatabaseError(e.to_string()))?;
        // Spawn the connection processor
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("Database connection error: {}", e);
            }
        });
        Ok(client)
    }
}

#[async_trait]
impl DbProvider for PgDbProvider {
    async fn save_state_info(&self, state_meta: &DicomStateMeta) -> Result<(), DbError> {
        let client = self.make_client().await?;
        let statement = client
                .prepare(
                    "INSERT INTO dicom_state_meta (
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
                   ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29)
                   ON CONFLICT (tenant_id, study_uid, series_uid)
                   DO UPDATE SET
                       patient_id = EXCLUDED.patient_id,
                       study_uid_hash = EXCLUDED.study_uid_hash,
                       series_uid_hash = EXCLUDED.series_uid_hash,
                       study_date_origin = EXCLUDED.study_date_origin,
                       patient_name = EXCLUDED.patient_name,
                       patient_sex = EXCLUDED.patient_sex,
                       patient_birth_date = EXCLUDED.patient_birth_date,
                       patient_birth_time = EXCLUDED.patient_birth_time,
                       patient_age = EXCLUDED.patient_age,
                       patient_size = EXCLUDED.patient_size,
                       patient_weight = EXCLUDED.patient_weight,
                       study_date = EXCLUDED.study_date,
                       study_time = EXCLUDED.study_time,
                       accession_number = EXCLUDED.accession_number,
                       study_id = EXCLUDED.study_id,
                       study_description = EXCLUDED.study_description,
                       modality = EXCLUDED.modality,
                       series_number = EXCLUDED.series_number,
                       series_date = EXCLUDED.series_date,
                       series_time = EXCLUDED.series_time,
                       series_description = EXCLUDED.series_description,
                       body_part_examined = EXCLUDED.body_part_examined,
                       protocol_name = EXCLUDED.protocol_name,
                       series_related_instances = EXCLUDED.series_related_instances,
                       updated_time = EXCLUDED.updated_time"
                )
                .await
                .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        client
            .execute(
                &statement,
                &[
                    &state_meta.tenant_id,
                    &state_meta.patient_id,
                    &state_meta.study_uid,
                    &state_meta.series_uid,
                    &state_meta.study_uid_hash,
                    &state_meta.series_uid_hash,
                    &state_meta.study_date_origin,
                    &state_meta.patient_name,
                    &state_meta.patient_sex,
                    &state_meta.patient_birth_date,
                    &state_meta.patient_birth_time,
                    &state_meta.patient_age,
                    &state_meta.patient_size,
                    &state_meta.patient_weight,
                    &state_meta.study_date,
                    &state_meta.study_time,
                    &state_meta.accession_number,
                    &state_meta.study_id,
                    &state_meta.study_description,
                    &state_meta.modality,
                    &state_meta.series_number,
                    &state_meta.series_date,
                    &state_meta.series_time,
                    &state_meta.series_description,
                    &state_meta.body_part_examined,
                    &state_meta.protocol_name,
                    &state_meta.series_related_instances,
                    &state_meta.created_time,
                    &state_meta.updated_time,
                ],
            )
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn save_state_list(&self, state_meta_list: &[DicomStateMeta]) -> Result<(), DbError> {
        // 使用事务确保所有数据要么全部保存成功，要么全部失败
        let mut client = self.make_client().await?;
        let transaction = client.transaction().await.map_err(|e| {
            println!("Failed to start transaction: {}", e);
            DbError::DatabaseError(e.to_string())
        })?;
        println!(
            "Starting transaction to save state meta list of length {}",
            state_meta_list.len()
        );

        let statement = transaction
        .prepare(
            "INSERT INTO dicom_state_meta (
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
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29)
            ON CONFLICT (tenant_id, study_uid, series_uid)
            DO UPDATE SET
                patient_id = EXCLUDED.patient_id,
                study_uid_hash = EXCLUDED.study_uid_hash,
                series_uid_hash = EXCLUDED.series_uid_hash,
                study_date_origin = EXCLUDED.study_date_origin,
                patient_name = EXCLUDED.patient_name,
                patient_sex = EXCLUDED.patient_sex,
                patient_birth_date = EXCLUDED.patient_birth_date,
                patient_birth_time = EXCLUDED.patient_birth_time,
                patient_age = EXCLUDED.patient_age,
                patient_size = EXCLUDED.patient_size,
                patient_weight = EXCLUDED.patient_weight,
                study_date = EXCLUDED.study_date,
                study_time = EXCLUDED.study_time,
                accession_number = EXCLUDED.accession_number,
                study_id = EXCLUDED.study_id,
                study_description = EXCLUDED.study_description,
                modality = EXCLUDED.modality,
                series_number = EXCLUDED.series_number,
                series_date = EXCLUDED.series_date,
                series_time = EXCLUDED.series_time,
                series_description = EXCLUDED.series_description,
                body_part_examined = EXCLUDED.body_part_examined,
                protocol_name = EXCLUDED.protocol_name,
                series_related_instances = EXCLUDED.series_related_instances,
                updated_time = EXCLUDED.updated_time"
        )
        .await
        .map_err(|e| {
            println!("Error transaction.prepare: {:?}", e);
            DbError::DatabaseError(e.to_string())
        })?;

        // 遍历所有 DicomStateMeta 对象并执行插入操作
        for state_meta in state_meta_list {
            transaction
                .execute(
                    &statement,
                    &[
                        &state_meta.tenant_id,
                        &state_meta.patient_id,
                        &state_meta.study_uid,
                        &state_meta.series_uid,
                        &state_meta.study_uid_hash,
                        &state_meta.series_uid_hash,
                        &state_meta.study_date_origin,
                        &state_meta.patient_name,
                        &state_meta.patient_sex,
                        &state_meta.patient_birth_date,
                        &state_meta.patient_birth_time,
                        &state_meta.patient_age,
                        &state_meta.patient_size,
                        &state_meta.patient_weight,
                        &state_meta.study_date,
                        &state_meta.study_time,
                        &state_meta.accession_number,
                        &state_meta.study_id,
                        &state_meta.study_description,
                        &state_meta.modality,
                        &state_meta.series_number,
                        &state_meta.series_date,
                        &state_meta.series_time,
                        &state_meta.series_description,
                        &state_meta.body_part_examined,
                        &state_meta.protocol_name,
                        &state_meta.series_related_instances,
                        &state_meta.created_time,
                        &state_meta.updated_time,
                    ],
                )
                .await
                .map_err(|e| {
                    println!("Error transaction.execute: {:?}", e);
                    DbError::DatabaseError(e.to_string())
                })?;
        }

        // 提交事务
        transaction.commit().await.map_err(|e| {
            println!("Error transaction.commit: {:?}", e);
            DbError::DatabaseError(e.to_string())
        })?;

        Ok(())
    }

    async fn save_json_list(&self, json_meta_list: &[DicomJsonMeta]) -> Result<(), DbError> {
        if json_meta_list.is_empty() {
            return Ok(());
        }

        let mut client = self.make_client().await?;
        let transaction = client.transaction().await.map_err(|e| {
            println!("Failed to start transaction: {}", e);
            DbError::DatabaseError(e.to_string())
        })?;

        println!(
            "Starting transaction to save json meta list of length {}",
            json_meta_list.len()
        );

        let statement = transaction
            .prepare(
                "INSERT INTO dicom_json_meta (
                tenant_id,
                study_uid,
                series_uid,
                study_uid_hash,
                series_uid_hash,
                study_date_origin,
                flag_time,
                created_time,json_status,retry_times

            ) VALUES ($1, $2, $3, $4, $5, $6, $7,$8,$9,$10)
            ON CONFLICT (tenant_id, study_uid, series_uid)
            DO UPDATE SET
                study_uid_hash = EXCLUDED.study_uid_hash,
                series_uid_hash = EXCLUDED.series_uid_hash,
                study_date_origin = EXCLUDED.study_date_origin,
                flag_time = EXCLUDED.flag_time,
                created_time = EXCLUDED.created_time,
                json_status = EXCLUDED.json_status,
                retry_times = EXCLUDED.retry_times
                ",
            )
            .await
            .map_err(|e| {
                println!("Error transaction.prepare: {:?}", e);
                DbError::DatabaseError(e.to_string())
            })?;

        // 遍历所有 DicomJsonMeta 对象并执行插入操作
        for json_meta in json_meta_list {
            transaction
                .execute(
                    &statement,
                    &[
                        &json_meta.tenant_id,
                        &json_meta.study_uid,
                        &json_meta.series_uid,
                        &json_meta.study_uid_hash,
                        &json_meta.series_uid_hash,
                        &json_meta.study_date_origin,
                        &json_meta.flag_time,
                        &json_meta.created_time,
                        &json_meta.json_status,
                        &json_meta.retry_times,
                    ],
                )
                .await
                .map_err(|e| {
                    println!("Error transaction.execute: {:?}", e);
                    DbError::DatabaseError(e.to_string())
                })?;
        }

        // 提交事务
        transaction.commit().await.map_err(|e| {
            println!("Error transaction.commit: {:?}", e);
            DbError::DatabaseError(e.to_string())
        })?;

        Ok(())
    }

    async fn get_state_metaes(
        &self,
        tenant_id: &str,
        study_uid: &str,
    ) -> Result<Vec<DicomStateMeta>, DbError> {
        let client = self.make_client().await?;
        let statement = client
            .prepare(
                "SELECT
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
            WHERE tenant_id = $1 AND study_uid = $2",
            )
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let rows = client
            .query(&statement, &[&tenant_id, &study_uid])
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let state_meta = DicomStateMeta {
                tenant_id: row.get(0),
                patient_id: row.get(1),
                study_uid: row.get(2),
                series_uid: row.get(3),
                study_uid_hash: row.get(4),
                series_uid_hash: row.get(5),
                study_date_origin: row.get(6),
                patient_name: row.get(7),
                patient_sex: row.get(8),
                patient_birth_date: row.get(9),
                patient_birth_time: row.get(10),
                patient_age: row.get(11),
                patient_size: row.get(12),
                patient_weight: row.get(13),
                study_date: row.get(14),
                study_time: row.get(15),
                accession_number: row.get(16),
                study_id: row.get(17),
                study_description: row.get(18),
                modality: row.get(19),
                series_number: row.get(20),
                series_date: row.get(21),
                series_time: row.get(22),
                series_description: row.get(23),
                body_part_examined: row.get(24),
                protocol_name: row.get(25),
                series_related_instances: row.get(26),
                created_time: row.get(27),
                updated_time: row.get(28),
            };
            result.push(state_meta);
        }

        Ok(result)
    }

    async fn get_json_metaes(
        &self,
        end_time: chrono::NaiveDateTime,
    ) -> Result<Vec<DicomStateMeta>, DbError> {
        let client = self.make_client().await?;
        let statement = client
            .prepare(
                " Select tenant_id,
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
                      WHERE djm.tenant_id IS NULL AND dsm.updated_time <  $1
                      UNION ALL
                      SELECT dsm.*
                      FROM dicom_state_meta dsm
                               INNER JOIN dicom_json_meta djm
                                          ON dsm.tenant_id = djm.tenant_id
                                              AND dsm.study_uid = djm.study_uid
                                              AND dsm.series_uid = djm.series_uid
                      WHERE dsm.updated_time != djm.flag_time AND dsm.updated_time <  $1
                      ) AS t
                order by t.updated_time asc limit 10;",
            )
            .await
            .map_err(|e| {
                println!("{:?}", e);
                DbError::DatabaseError(e.to_string())
            })?;

        let rows = client
            .query(&statement, &[&end_time])
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let state_meta = DicomStateMeta {
                tenant_id: row.get(0),
                patient_id: row.get(1),
                study_uid: row.get(2),
                series_uid: row.get(3),
                study_uid_hash: row.get(4),
                series_uid_hash: row.get(5),
                study_date_origin: row.get(6),
                patient_name: row.get(7),
                patient_sex: row.get(8),
                patient_birth_date: row.get(9),
                patient_birth_time: row.get(10),
                patient_age: row.get(11),
                patient_size: row.get(12),
                patient_weight: row.get(13),
                study_date: row.get(14),
                study_time: row.get(15),
                accession_number: row.get(16),
                study_id: row.get(17),
                study_description: row.get(18),
                modality: row.get(19),
                series_number: row.get(20),
                series_date: row.get(21),
                series_time: row.get(22),
                series_description: row.get(23),
                body_part_examined: row.get(24),
                protocol_name: row.get(25),
                series_related_instances: row.get(26),
                created_time: row.get(27),
                updated_time: row.get(28),
            };
            result.push(state_meta);
        }

        Ok(result)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dicom_dbprovider::current_time;
    use crate::dicom_dbtype::*;
    use chrono::{NaiveDate, NaiveTime};
    use std::ops::Sub;

    #[tokio::test]
    async fn test_save_state_info() -> Result<(), Box<dyn std::error::Error>> {
        let db_provider =
            PgDbProvider::new("postgresql://root:jp%23123@192.168.1.14:5432/postgres".to_string());

        // 创建测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_123".to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_456".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9".to_string())?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1".to_string())?;
        let study_uid_hash = BoundedString::<20>::from_str("1.2.3.4.5.6.7.8.9").unwrap();
        let series_uid_hash = BoundedString::<20>::from_str("9.8.7.6.5.4.3.2.1").unwrap();
        let study_date_origin = DicomDateString::from_db("20231201");
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
        let created_time = current_time();
        let updated_time = current_time();

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

        Ok(())
    }

    #[tokio::test]
    async fn test_get_state_metaes() -> Result<(), Box<dyn std::error::Error>> {
        let db_provider =
            PgDbProvider::new("postgresql://root:jp%23123@192.168.1.14:5432/postgres".to_string());

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
    async fn test_get_json_metaes() -> Result<(), Box<dyn std::error::Error>> {
        let db_provider =
            PgDbProvider::new("postgresql://root:jp%23123@192.168.1.14:5432/postgres".to_string());

        let cd = current_time();
        let cd = cd.sub(chrono::Duration::minutes(3));
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
        let db_provider =
            PgDbProvider::new("postgresql://root:jp%23123@192.168.1.14:5432/postgres".to_string());

        // 创建测试数据列表
        let mut state_meta_list = Vec::new();

        // 创建第一个测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_list_123".to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_list_456".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9.list".to_string())?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1.list".to_string())?;
        let study_uid_hash = BoundedString::<20>::from_str("0AA07C2AA455BEB01D5A").unwrap();
        let series_uid_hash = BoundedString::<20>::from_str("0AB07C2AA455BEB01D5A").unwrap();
        let study_date_origin = DicomDateString::from_db("20231202");
        let accession_number = BoundedString::<16>::try_from("ACC123457".to_string())?;
        let modality = Some(BoundedString::<16>::try_from("MRI".to_string())?);
        let series_number = Some(2);
        let series_date = Some(NaiveDate::from_ymd_opt(2023, 12, 2).unwrap());
        let series_time = Some(NaiveTime::parse_from_str("130000", "%H%M%S")?);
        let series_description = Some(BoundedString::<256>::try_from(
            "Test Series List".to_string(),
        )?);
        let body_part_examined = Some(BoundedString::<64>::try_from("HEAD".to_string())?);
        let protocol_name = Some(BoundedString::<64>::try_from("HEAD MRI".to_string())?);
        let study_date = NaiveDate::from_ymd_opt(2023, 12, 2).unwrap();
        let study_time = Some(NaiveTime::parse_from_str("130000", "%H%M%S")?);
        let study_id = Some(BoundedString::<16>::try_from("STUDY124".to_string())?);
        let study_description = Some(BoundedString::<64>::try_from(
            "Test Study List".to_string(),
        )?);
        let patient_age = Some(BoundedString::<16>::try_from("046Y".to_string())?);
        let patient_sex = Some(BoundedString::<1>::try_from("F".to_string())?);
        let patient_name = Some(BoundedString::<64>::try_from("TEST^PATIENT2".to_string())?);
        let patient_birth_date = Some(NaiveDate::from_ymd_opt(1977, 1, 1).unwrap());
        let patient_birth_time = Some(NaiveTime::parse_from_str("090000", "%H%M%S")?);
        let created_time = current_time();
        let updated_time = current_time();

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

        state_meta_list.push(state_meta);

        // 执行批量保存操作
        let result = db_provider.save_state_list(&state_meta_list).await;

        // 验证保存成功并打印详细错误信息
        match &result {
            Ok(_) => {
                println!("Successfully saved DicomStateMeta list");
            }
            Err(e) => {
                eprintln!("Failed to save DicomStateMeta list with error: {:?}", e);
                eprintln!("Error details: {}", e);
            }
        }

        assert!(
            result.is_ok(),
            "Failed to save DicomStateMeta list: {:?}",
            result.err()
        );

        Ok(())
    }
    #[tokio::test]
    async fn test_save_json_list() -> Result<(), Box<dyn std::error::Error>> {
        let db_provider =
            PgDbProvider::new("postgresql://root:jp%23123@192.168.1.14:5432/postgres".to_string());

        // 创建测试数据列表
        let mut json_meta_list = Vec::new();

        // 创建测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_json_123".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9.json".to_string())?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1.json".to_string())?;
        let study_uid_hash = BoundedString::<20>::from_str("0AC07C2AA455BEB01D5A").unwrap();
        let series_uid_hash = BoundedString::<20>::from_str("0AD07C2AA455BEB01D5A").unwrap();
        let study_date_origin = DicomDateString::from_db("20231203");
        let flag_time = current_time();
        let created_time = current_time();
        let json_status = 0;
        let retry_times = 0;

        let json_meta = DicomJsonMeta {
            tenant_id: tenant_id.clone(),
            study_uid: study_uid.clone(),
            series_uid: series_uid.clone(),
            study_uid_hash,
            series_uid_hash,
            study_date_origin,
            flag_time,
            created_time,
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
}
