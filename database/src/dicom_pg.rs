use crate::dicom_dbprovider::{DbError, DbProvider};
use crate::dicom_meta::{DicomImageMeta, DicomJsonMeta, DicomStateMeta, DicomStoreMeta};
use async_trait::async_trait;
use tokio_postgres::{Client, NoTls};
#[derive(Debug, Clone)]
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
    async fn save_store_list(&self, store_meta_list: &[DicomStoreMeta]) -> Result<(), DbError> {
        if store_meta_list.is_empty() {
            return Ok(());
        }

        let mut client = self.make_client().await?;
        let transaction = client.transaction().await.map_err(|e| {
            println!("Failed to start transaction: {}", e);
            DbError::DatabaseError(e.to_string())
        })?;

        println!(
            "Starting transaction to save store meta list of length {}",
            store_meta_list.len()
        );

        let statement = transaction
            .prepare(
                r#"
            INSERT INTO dicom_object_meta (
                trace_id,
                worker_node_id,
                tenant_id,
                patient_id,
                study_uid,
                series_uid,
                sop_uid,
                file_size,
                file_path,
                transfer_syntax_uid,
                number_of_frames,
                series_uid_hash,
                study_uid_hash,
                accession_number,
                target_ts,
                study_date,
                transfer_status,
                source_ip,
                source_ae,
                created_time
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
                $11, $12, $13, $14, $15, $16, $17, $18, $19, $20
            )
            ON CONFLICT (trace_id)
            DO UPDATE SET
                worker_node_id = EXCLUDED.worker_node_id,
                tenant_id = EXCLUDED.tenant_id,
                patient_id = EXCLUDED.patient_id,
                study_uid = EXCLUDED.study_uid,
                series_uid = EXCLUDED.series_uid,
                sop_uid = EXCLUDED.sop_uid,
                file_size = EXCLUDED.file_size,
                file_path = EXCLUDED.file_path,
                transfer_syntax_uid = EXCLUDED.transfer_syntax_uid,
                number_of_frames = EXCLUDED.number_of_frames,
                series_uid_hash = EXCLUDED.series_uid_hash,
                study_uid_hash = EXCLUDED.study_uid_hash,
                accession_number = EXCLUDED.accession_number,
                target_ts = EXCLUDED.target_ts,
                study_date = EXCLUDED.study_date,
                transfer_status = EXCLUDED.transfer_status,
                source_ip = EXCLUDED.source_ip,
                source_ae = EXCLUDED.source_ae,
                created_time = EXCLUDED.created_time
            "#,
            )
            .await
            .map_err(|e| {
                println!("Error preparing statement: {:?}", e);
                DbError::DatabaseError(e.to_string())
            })?;

        for store_meta in store_meta_list {
            transaction
                .execute(
                    &statement,
                    &[
                        &store_meta.trace_id,
                        &store_meta.worker_node_id,
                        &store_meta.tenant_id,
                        &store_meta.patient_id,
                        &store_meta.study_uid,
                        &store_meta.series_uid,
                        &store_meta.sop_uid,
                        &store_meta.file_size,
                        &store_meta.file_path,
                        &store_meta.transfer_syntax_uid,
                        &store_meta.number_of_frames,
                        &store_meta.series_uid_hash,
                        &store_meta.study_uid_hash,
                        &store_meta.accession_number,
                        &store_meta.target_ts,
                        &store_meta.study_date,
                        &store_meta.transfer_status.to_string(),
                        &store_meta.source_ip,
                        &store_meta.source_ae,
                        &store_meta.created_time,
                    ],
                )
                .await
                .map_err(|e| {
                    println!("Error executing statement: {:?}", e);
                    DbError::DatabaseError(e.to_string())
                })?;
        }

        transaction.commit().await.map_err(|e| {
            println!("Error committing transaction: {:?}", e);
            DbError::DatabaseError(e.to_string())
        })?;

        Ok(())
    }

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
    // File: `database/src/dicom_pg.rs`
    async fn save_image_list(&self, image_meta_list: &[DicomImageMeta]) -> Result<(), DbError> {
        if image_meta_list.is_empty() {
            return Ok(());
        }

        let mut client = self.make_client().await?;
        let transaction = client.transaction().await.map_err(|e| {
            println!("Failed to start transaction: {}", e);
            DbError::DatabaseError(e.to_string())
        })?;

        println!(
            "Starting transaction to save image meta list of length {}",
            image_meta_list.len()
        );

        let sql_statement = r#"
    INSERT INTO dicom_image_meta (
        tenant_id,
        patient_id,
        study_uid,
        series_uid,
        sop_uid,
        study_uid_hash,
        series_uid_hash,
        content_date,
        content_time,
        instance_number,
        image_type,
        image_orientation_patient,
        image_position_patient,
        slice_thickness,
        spacing_between_slices,
        slice_location,
        samples_per_pixel,
        photometric_interpretation,
        width,
        columns,
        bits_allocated,
        bits_stored,
        high_bit,
        pixel_representation,
        rescale_intercept,
        rescale_slope,
        rescale_type,
        window_center,
        window_width,
        transfer_syntax_uid,
        pixel_data_location,
        thumbnail_location,
        sop_class_uid,
        image_status,
        space_size,
        created_time,
        updated_time
    ) VALUES (
        $1, $2, $3, $4, $5, $6, $7, $8, $9, $10,
        $11, $12, $13, $14, $15, $16, $17, $18, $19, $20,
        $21, $22, $23, $24, $25, $26, $27, $28, $29, $30,
        $31, $32, $33, $34, $35, $36, $37
    )
    ON CONFLICT (tenant_id, study_uid, series_uid, sop_uid)
    DO UPDATE SET
        patient_id = EXCLUDED.patient_id,
        study_uid_hash = EXCLUDED.study_uid_hash,
        series_uid_hash = EXCLUDED.series_uid_hash,
        content_date = EXCLUDED.content_date,
        content_time = EXCLUDED.content_time,
        instance_number = EXCLUDED.instance_number,
        image_type = EXCLUDED.image_type,
        image_orientation_patient = EXCLUDED.image_orientation_patient,
        image_position_patient = EXCLUDED.image_position_patient,
        slice_thickness = EXCLUDED.slice_thickness,
        spacing_between_slices = EXCLUDED.spacing_between_slices,
        slice_location = EXCLUDED.slice_location,
        samples_per_pixel = EXCLUDED.samples_per_pixel,
        photometric_interpretation = EXCLUDED.photometric_interpretation,
        width = EXCLUDED.width,
        columns = EXCLUDED.columns,
        bits_allocated = EXCLUDED.bits_allocated,
        bits_stored = EXCLUDED.bits_stored,
        high_bit = EXCLUDED.high_bit,
        pixel_representation = EXCLUDED.pixel_representation,
        rescale_intercept = EXCLUDED.rescale_intercept,
        rescale_slope = EXCLUDED.rescale_slope,
        rescale_type = EXCLUDED.rescale_type,
        window_center = EXCLUDED.window_center,
        window_width = EXCLUDED.window_width,
        transfer_syntax_uid = EXCLUDED.transfer_syntax_uid,
        pixel_data_location = EXCLUDED.pixel_data_location,
        thumbnail_location = EXCLUDED.thumbnail_location,
        sop_class_uid = EXCLUDED.sop_class_uid,
        image_status = EXCLUDED.image_status,
        space_size = EXCLUDED.space_size,
        updated_time = EXCLUDED.updated_time
    "#;

        let statement = transaction.prepare(sql_statement).await.map_err(|e| {
            println!("Error preparing statement: {:?}", e);
            DbError::DatabaseError(e.to_string())
        })?;

        for image_meta in image_meta_list {
            transaction
                .execute(
                    &statement,
                    &[
                        &image_meta.tenant_id,
                        &image_meta.patient_id,
                        &image_meta.study_uid,
                        &image_meta.series_uid,
                        &image_meta.sop_uid,
                        &image_meta.study_uid_hash,
                        &image_meta.series_uid_hash,
                        &image_meta.content_date,
                        &image_meta.content_time,
                        &image_meta.instance_number,
                        &image_meta.image_type,
                        &image_meta.image_orientation_patient,
                        &image_meta.image_position_patient,
                        &image_meta.slice_thickness,
                        &image_meta.spacing_between_slices,
                        &image_meta.slice_location,
                        &image_meta.samples_per_pixel,
                        &image_meta.photometric_interpretation,
                        &image_meta.width,
                        &image_meta.columns,
                        &image_meta.bits_allocated,
                        &image_meta.bits_stored,
                        &image_meta.high_bit,
                        &image_meta.pixel_representation,
                        &image_meta.rescale_intercept,
                        &image_meta.rescale_slope,
                        &image_meta.rescale_type,
                        &image_meta.window_center,
                        &image_meta.window_width,
                        &image_meta.transfer_syntax_uid,
                        &image_meta.pixel_data_location,
                        &image_meta.thumbnail_location,
                        &image_meta.sop_class_uid,
                        &image_meta.image_status,
                        &image_meta.space_size,
                        &image_meta.created_time,
                        &image_meta.updated_time,
                    ],
                )
                .await
                .map_err(|e| {
                    println!("Error executing statement: {:?}", e);
                    DbError::DatabaseError(e.to_string())
                })?;
        }

        transaction.commit().await.map_err(|e| {
            println!("Error committing transaction: {:?}", e);
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

        // 第一个语句：插入或更新 dicom_json_meta
        let insert_statement = transaction
            .prepare(
                "INSERT INTO dicom_json_meta (
                tenant_id,
                study_uid,
                series_uid,
                study_uid_hash,
                series_uid_hash,
                study_date_origin,
                flag_time,
                created_time,
                json_status,
                retry_times
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            ON CONFLICT (tenant_id, study_uid, series_uid)
            DO UPDATE SET
                study_uid_hash = EXCLUDED.study_uid_hash,
                series_uid_hash = EXCLUDED.series_uid_hash,
                study_date_origin = EXCLUDED.study_date_origin,
                flag_time = EXCLUDED.flag_time,
                created_time = EXCLUDED.created_time,
                json_status = EXCLUDED.json_status,
                retry_times = EXCLUDED.retry_times",
            )
            .await
            .map_err(|e| {
                println!("Error preparing insert statement: {:?}", e);
                DbError::DatabaseError(e.to_string())
            })?;

        // 第二个语句：更新 dicom_state_meta 中的 series_related_instances
        let update_statement = transaction
            .prepare(
                "WITH image_counts AS (
                SELECT tenant_id, study_uid, series_uid, COUNT(*) AS count
                FROM dicom_image_meta
                WHERE tenant_id = $1 AND study_uid = $2 AND series_uid = $3
                GROUP BY tenant_id, study_uid, series_uid
            )
            UPDATE dicom_state_meta dsm
            SET series_related_instances = c.count
            FROM image_counts c
            WHERE dsm.tenant_id = c.tenant_id
              AND dsm.study_uid = c.study_uid
              AND dsm.series_uid = c.series_uid",
            )
            .await
            .map_err(|e| {
                println!("Error preparing update statement: {:?}", e);
                DbError::DatabaseError(e.to_string())
            })?;

        // 遍历所有 DicomJsonMeta 对象并执行插入操作
        for json_meta in json_meta_list {
            // 插入或更新 dicom_json_meta
            transaction
                .execute(
                    &insert_statement,
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
                    println!("Error inserting into dicom_json_meta: {:?}", e);
                    DbError::DatabaseError(e.to_string())
                })?;

            // 更新 dicom_state_meta
            transaction
                .execute(
                    &update_statement,
                    &[
                        &json_meta.tenant_id,
                        &json_meta.study_uid,
                        &json_meta.series_uid,
                    ],
                )
                .await
                .map_err(|e| {
                    println!("Error updating dicom_state_meta: {:?}", e);
                    DbError::DatabaseError(e.to_string())
                })?;
        }

        // 提交事务
        transaction.commit().await.map_err(|e| {
            println!("Error committing transaction: {:?}", e);
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

    async fn get_json_meta(
        &self,
        tenant_id: &str,
        study_uid: &str,
        series_uid: &str,
    ) -> Result<DicomJsonMeta, DbError> {
        let client = self.make_client().await?;
        let statement = client
            .prepare(
                "SELECT
                tenant_id,
                study_uid,
                series_uid,
                study_uid_hash,
                series_uid_hash,
                study_date_origin,
                flag_time,
                created_time,
                json_status,
                retry_times
            FROM dicom_json_meta
            WHERE series_uid = $1  and tenant_id  = $2 and study_uid = $3 ",
            )
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        let rows = client
            .query(&statement, &[&series_uid, &tenant_id, &study_uid])
            .await
            .map_err(|e| DbError::DatabaseError(e.to_string()))?;

        if rows.is_empty() {
            return Err(DbError::RecordNotExists(format!(
                "DicomJsonMeta with series_uid {} not found",
                series_uid
            )));
        }

        let row = &rows[0];
        let json_meta = DicomJsonMeta {
            tenant_id: row.get(0),
            study_uid: row.get(1),
            series_uid: row.get(2),
            study_uid_hash: row.get(3),
            series_uid_hash: row.get(4),
            study_date_origin: row.get(5),
            flag_time: row.get(6),
            created_time: row.get(7),
            json_status: row.get(8),
            retry_times: row.get(9),
        };

        Ok(json_meta)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dicom_dbprovider::current_time;
    use crate::dicom_dbtype::*;
    use crate::dicom_meta::TransferStatus;
    use chrono::{NaiveDate, NaiveTime};
    use ctor::ctor;
    use dotenv::dotenv;
    use std::env;
    use std::ops::Sub;

    #[cfg(test)]
    #[ctor]
    fn init_tests() {
        // 这个函数会在所有测试运行之前执行一次
        dotenv().ok();
        println!("Initializing tests...");
        // 可以在这里进行全局的测试设置
    }

    #[cfg(test)]
    struct TestCleanup {
        tenant_ids: Vec<String>,
        #[allow(dead_code)]
        db_provider: PgDbProvider,
    }
    #[cfg(test)]
    impl TestCleanup {
        fn new(db_provider: PgDbProvider) -> Self {
            Self {
                tenant_ids: Vec::new(),
                db_provider,
            }
        }

        fn add_tenant(&mut self, tenant_id: String) {
            self.tenant_ids.push(tenant_id);
        }
    }
    #[cfg(test)]
    impl Drop for TestCleanup {
        fn drop(&mut self) {
            // 在这里执行清理操作
            println!("Cleaning up test data...");
            // 在实际实现中，这里应该执行数据库清理操作
            // 例如删除测试创建的记录
            // tokio::runtime::Runtime::new().unwrap().block_on(async {
            for tenant_id in &self.tenant_ids {
                // 执行清理SQL
                println!(
                    "exec sql: DELETE FROM dicom_state_meta WHERE tenant_id = {}",
                    tenant_id
                );
            }

            // });
        }
    }
    #[tokio::test]
    async fn test_save_state_info() -> Result<(), Box<dyn std::error::Error>> {
        let sql_cnn = env::var("DICOM_PGSQL");
        if sql_cnn.is_err() {
            println!("DICOM_PGSQL environment variable not set");
            println!("eg:postgresql://root:jp%23123@192.168.1.14:5432/postgres");
            return Ok(());
        }

        let t_id = "test_tenant_123";
        let db_provider = PgDbProvider::new(sql_cnn?);
        // 构造 TestCleanup 实例
        let mut cleanup = TestCleanup::new(db_provider.clone());
        // 注册需要清理的 tenant_id
        cleanup.add_tenant(t_id.to_string());
        // 创建测试数据
        let tenant_id = BoundedString::<64>::try_from(t_id.to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_456".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9".to_string())?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1".to_string())?;
        let study_uid_hash = BoundedString::<20>::make_str("1.2.3.4.5.6.7.8.9");
        let series_uid_hash = BoundedString::<20>::make_str("9.8.7.6.5.4.3.2.1");
        let study_date_origin = DicomDateString::from_str("20231201")?;
        let accession_number = BoundedString::<16>::make_str("ACC123456");
        let modality = Some(BoundedString::<16>::make_str("CT"));
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
            accession_number: Some(accession_number),
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

        Ok(()) //   cleanup 会在函数结束时自动调用 Drop
    }

    #[tokio::test]
    async fn test_get_state_metaes() -> Result<(), Box<dyn std::error::Error>> {
        let sql_cnn = env::var("DICOM_PGSQL");
        if sql_cnn.is_err() {
            println!("DICOM_PGSQL environment variable not set");
            println!("eg:postgresql://root:jp%23123@192.168.1.14:5432/postgres");
            return Ok(());
        }

        let db_provider = PgDbProvider::new(sql_cnn?);

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

        let state_meta_list = result?;

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
        let sql_cnn = env::var("DICOM_PGSQL");
        if sql_cnn.is_err() {
            println!("DICOM_PGSQL environment variable not set");
            println!("eg:postgresql://root:jp%23123@192.168.1.14:5432/postgres");
            return Ok(());
        }

        let db_provider = PgDbProvider::new(sql_cnn?);

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

        let state_meta_list = result?;

        // 验证返回结果不为空
        assert!(!state_meta_list.is_empty(), "Expected non-empty result");

        if state_meta_list.is_empty() {
            return Ok(());
        }
        // 验证每条记录的 tenant_id 和 study_uid 是否正确
        for state_meta in state_meta_list {
            let json = serde_json::to_string_pretty(&state_meta)?;
            println!("DicomStateMeta JSON: {}", json);
        }

        Ok(())
    }
    #[tokio::test]
    async fn test_save_state_list() -> Result<(), Box<dyn std::error::Error>> {
        let sql_cnn = env::var("DICOM_PGSQL");
        if sql_cnn.is_err() {
            println!("DICOM_PGSQL environment variable not set");
            println!("eg:postgresql://root:jp%23123@192.168.1.14:5432/postgres");
            return Ok(());
        }

        let db_provider = PgDbProvider::new(sql_cnn?);
        // 创建测试数据列表
        let mut state_meta_list = Vec::new();

        // 创建第一个测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_list_123".to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_list_456".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9.list".to_string())?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1.list".to_string())?;
        let study_uid_hash = BoundedString::<20>::from_str("0AA07C2AA455BEB01D5A")?;
        let series_uid_hash = BoundedString::<20>::from_str("0AB07C2AA455BEB01D5A")?;
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
            accession_number: Some(accession_number),
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
        let sql_cnn = env::var("DICOM_PGSQL");
        if sql_cnn.is_err() {
            println!("DICOM_PGSQL environment variable not set");
            println!("eg:postgresql://root:jp%23123@192.168.1.14:5432/postgres");
            return Ok(());
        }

        let db_provider = PgDbProvider::new(sql_cnn?);

        // 创建测试数据列表
        let mut json_meta_list = Vec::new();

        // 创建测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_json_123".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9.json".to_string())?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1.json".to_string())?;
        let study_uid_hash = BoundedString::<20>::from_str("0AC07C2AA455BEB01D5A")?;
        let series_uid_hash = BoundedString::<20>::from_str("0AD07C2AA455BEB01D5A")?;
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
    #[tokio::test]
    async fn test_save_store_info() -> Result<(), Box<dyn std::error::Error>> {
        let sql_cnn = env::var("DICOM_PGSQL");
        if sql_cnn.is_err() {
            println!("DICOM_PGSQL environment variable not set");
            println!("eg:postgresql://root:jp%23123@192.168.1.14:5432/postgres");
            return Ok(());
        }

        let db_provider = PgDbProvider::new(sql_cnn?);

        // 创建测试数据
        let trace_id = FixedLengthString::<36>::from_str("123e4567-e89b-12d3-a456-426614174000")?;
        let worker_node_id = BoundedString::<64>::try_from("TEST_WORKER_NODE".to_string())?;
        let tenant_id = BoundedString::<64>::try_from("test_tenant_store_123".to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_store_456".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9.store".to_string())?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1.store".to_string())?;
        let sop_uid =
            BoundedString::<64>::try_from("1.3.6.1.4.1.5962.1.1.0.0.0.1234567891".to_string())?;
        let file_size = 1024i64;
        let file_path = BoundedString::<512>::try_from("/data/test/file.dcm".to_string())?;
        let transfer_syntax_uid = BoundedString::<64>::try_from("1.2.840.10008.1.2.1".to_string())?;
        let number_of_frames = 1;
        let series_uid_hash = BoundedString::<20>::from_str("0BA07C2AA455BEB01D5A")?;
        let study_uid_hash = BoundedString::<20>::from_str("0BB07C2AA455BEB01D5A")?;
        let accession_number = BoundedString::<16>::try_from("ACC123458".to_string())?;
        let target_ts = BoundedString::<64>::try_from("1.2.840.10008.1.2.1".to_string())?;
        let study_date = NaiveDate::from_ymd_opt(2023, 12, 5).unwrap();
        let transfer_status = TransferStatus::Success;
        let source_ip = BoundedString::<24>::try_from("192.168.1.100".to_string())?;
        let source_ae = BoundedString::<64>::try_from("TEST_AE".to_string())?;
        let created_time = current_time();

        let store_meta = DicomStoreMeta {
            trace_id,
            worker_node_id,
            tenant_id,
            patient_id,
            study_uid,
            series_uid,
            sop_uid,
            file_size,
            file_path,
            transfer_syntax_uid,
            number_of_frames,
            series_uid_hash,
            study_uid_hash,
            accession_number: Some(accession_number),
            target_ts,
            study_date,
            transfer_status,
            source_ip,
            source_ae,
            created_time,
        };

        // 创建存储元数据列表
        let store_meta_list = vec![store_meta];

        // 执行保存操作
        let result = db_provider.save_store_list(&store_meta_list).await;

        // 验证保存成功
        assert!(
            result.is_ok(),
            "Failed to save DicomStoreMeta list: {:?}",
            result.err()
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_save_image_list() -> Result<(), Box<dyn std::error::Error>> {
        let sql_cnn = env::var("DICOM_PGSQL");
        if sql_cnn.is_err() {
            println!("DICOM_PGSQL environment variable not set");
            println!("eg:postgresql://root:jp%23123@192.168.1.14:5432/postgres");
            return Ok(());
        }

        let db_provider = PgDbProvider::new(sql_cnn?);

        // 创建测试数据列表
        let mut image_meta_list = Vec::new();

        // 创建测试数据
        let tenant_id = BoundedString::<64>::try_from("test_tenant_image_123".to_string())?;
        let patient_id = BoundedString::<64>::try_from("test_patient_image_456".to_string())?;
        let study_uid = BoundedString::<64>::try_from("1.2.3.4.5.6.7.8.9.image".to_string())?;
        let series_uid = BoundedString::<64>::try_from("9.8.7.6.5.4.3.2.1.image".to_string())?;
        let sop_uid =
            BoundedString::<64>::try_from("1.3.6.1.4.1.5962.1.1.0.0.0.1234567890".to_string())?;
        let study_uid_hash = BoundedString::<20>::from_str("0AE07C2AA455BEB01D5A")?;
        let series_uid_hash = BoundedString::<20>::from_str("0AF07C2AA455BEB01D5A")?;

        let content_date = Some(NaiveDate::parse_from_str("20231204", "%Y%m%d")?);
        let content_time = Some(NaiveTime::parse_from_str("120000", "%H%M%S")?);
        let instance_number = Some(1);
        let image_type = Some(BoundedString::<128>::try_from(
            "ORIGINAL\\PRIMARY\\AXIAL".to_string(),
        )?);
        let image_orientation_patient = Some(BoundedString::<128>::try_from(
            "1.0\\0.0\\0.0\\0.0\\1.0\\0.0".to_string(),
        )?);
        let image_position_patient = Some(BoundedString::<64>::try_from(
            "-125.0\\-125.0\\0.0".to_string(),
        )?);
        let slice_thickness = Some(5.0);
        let spacing_between_slices = Some(5.0);
        let slice_location = Some(0.0);
        let samples_per_pixel = Some(1);
        let photometric_interpretation =
            Some(BoundedString::<32>::try_from("MONOCHROME2".to_string())?);
        let width = Some(512);
        let columns = Some(512);
        let bits_allocated = Some(16);
        let bits_stored = Some(12);
        let high_bit = Some(11);
        let pixel_representation = Some(0);
        let rescale_intercept = Some(0.0);
        let rescale_slope = Some(1.0);
        let rescale_type = Some(BoundedString::<64>::make_str("US"));
        let window_center = Some(BoundedString::<64>::make_str("50"));
        let window_width = Some(BoundedString::<64>::make_str("400"));
        let transfer_syntax_uid = BoundedString::<64>::make_str("1.2.840.10008.1.2.1");
        let pixel_data_location = Some(BoundedString::<512>::make_str("/data/pixel/1234567890"));
        let thumbnail_location = Some(BoundedString::<512>::make_str("/data/thumb/1234567890"));
        let sop_class_uid = BoundedString::<64>::make_str("1.2.840.10008.5.1.4.1.1.2");
        let image_status = Some(BoundedString::<32>::make_str("AVAILABLE"));
        let space_size = Some(2049i64);
        let created_time = current_time();
        let updated_time = current_time();

        let image_meta = DicomImageMeta {
            tenant_id,
            patient_id,
            study_uid,
            series_uid,
            sop_uid,
            study_uid_hash,
            series_uid_hash,
            content_date,
            content_time,
            instance_number,
            image_type,
            image_orientation_patient,
            image_position_patient,
            slice_thickness,
            spacing_between_slices,
            slice_location,
            samples_per_pixel,
            photometric_interpretation,
            width,
            columns,
            bits_allocated,
            bits_stored,
            high_bit,
            pixel_representation,
            rescale_intercept,
            rescale_slope,
            rescale_type,
            window_center,
            window_width,
            transfer_syntax_uid,
            pixel_data_location,
            thumbnail_location,
            sop_class_uid,
            image_status,
            space_size,

            created_time,
            updated_time,
        };

        image_meta_list.push(image_meta);

        // 执行批量保存操作
        let result = db_provider.save_image_list(&image_meta_list).await;

        // 验证保存成功
        assert!(
            result.is_ok(),
            "Failed to save DicomImageMeta list: {:?}",
            result.err()
        );

        Ok(())
    }
}
