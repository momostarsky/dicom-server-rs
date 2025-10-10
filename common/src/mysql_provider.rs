use crate::database_entities::{
    DicomObjectMeta, ImageEntity, PatientEntity, SeriesEntity, StudyEntity,
};
use crate::database_provider::{DbError, DbProvider};
use crate::database_provider_base::DbProviderBase;
use crate::dicom_utils::{parse_dicom_date_from_sql, parse_dicom_time_from_sql};
use async_trait::async_trait;
use dicom_object::DefaultDicomObject;
use sqlx::mysql::MySqlRow;
use sqlx::{FromRow, MySql, MySqlPool, Row, Transaction};
use tracing::{error, info};

#[async_trait]
trait DbEntity {
    async fn save_impl(
        provider: &MySqlProvider,
        tenant_id: &str,
        entities: &[Self],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error>
    where
        Self: Sized;
}

// 为各个实体实现该 trait
#[async_trait]
impl DbEntity for PatientEntity {
    async fn save_impl(
        provider: &MySqlProvider,
        tenant_id: &str,
        entities: &[Self],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error>
    where
        Self: Sized,
    {
        provider
            .save_patient_info_impl(tenant_id, entities, tx)
            .await
    }
}

#[async_trait]
impl DbEntity for StudyEntity {
    async fn save_impl(
        provider: &MySqlProvider,
        tenant_id: &str,
        entities: &[Self],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error>
    where
        Self: Sized,
    {
        provider.save_study_info_impl(tenant_id, entities, tx).await
    }
}

#[async_trait]
impl DbEntity for SeriesEntity {
    async fn save_impl(
        provider: &MySqlProvider,
        tenant_id: &str,
        entities: &[Self],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error>
    where
        Self: Sized,
    {
        provider
            .save_series_info_impl(tenant_id, entities, tx)
            .await
    }
}

#[async_trait]
impl DbEntity for ImageEntity {
    async fn save_impl(
        provider: &MySqlProvider,
        tenant_id: &str,
        entities: &[Self],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error>
    where
        Self: Sized,
    {
        provider
            .save_instance_info_impl(tenant_id, entities, tx)
            .await
    }
}

impl FromRow<'_, MySqlRow> for SeriesEntity {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(SeriesEntity {
            tenant_id: row.get("tenant_id"),
            series_instance_uid: row.get("SeriesInstanceUID"),
            study_instance_uid: row.get("StudyInstanceUID"),
            patient_id: row.get("PatientID"),
            modality: row.get("Modality"),
            series_number: row.get("SeriesNumber"),
            series_date: parse_dicom_date_from_sql(row.get("series_date")),
            series_time: parse_dicom_time_from_sql(row.get("series_time")),
            series_description: row.get("SeriesDescription"),
            body_part_examined: row.get("BodyPartExamined"),
            protocol_name: row.get("ProtocolName"),
            acquisition_number: row.get("AcquisitionNumber"),
            acquisition_time: parse_dicom_time_from_sql(row.get("acquisition_time")),
            acquisition_date: parse_dicom_date_from_sql(row.get("acquisition_date")),
            acquisition_date_time: row.get("AcquisitionDateTime"),
            performing_physician_name: row.get("PerformingPhysicianName"),
            operators_name: row.get("OperatorsName"),
            number_of_series_related_instances: row.get("NumberOfSeriesRelatedInstances"),
            received_instances: row.get("ReceivedInstances"),
            space_size: row.get("SpaceSize"),
            created_time: row.get("CreatedTime"),
            updated_time: row.get("UpdatedTime"),
        })
    }
}

impl FromRow<'_, MySqlRow> for StudyEntity {
    fn from_row(row: &MySqlRow) -> Result<Self, sqlx::Error> {
        Ok(StudyEntity {
            tenant_id: row.get("tenant_id"),
            study_instance_uid: row.get("StudyInstanceUID"),
            patient_id: row.get("PatientID"),
            study_date: parse_dicom_date_from_sql(row.get("StudyDate")).unwrap(),
            study_time: parse_dicom_time_from_sql(row.get("StudyTime")),
            accession_number: row.get("AccessionNumber"),
            study_id: row.get("StudyID"),
            study_description: row.get("StudyDescription"),
            referring_physician_name: row.get("ReferringPhysicianName"),
            patient_age: row.get("PatientAge"),
            patient_size: row.get("PatientSize"),
            patient_weight: row.get("PatientWeight"),
            medical_alerts: row.get("MedicalAlerts"),
            allergies: row.get("Allergies"),
            pregnancy_status: row.get("PregnancyStatus"),
            occupation: row.get("Occupation"),
            additional_patient_history: row.get("AdditionalPatientHistory"),
            patient_comments: row.get("PatientComments"),
            admission_id: row.get("AdmissionID"),
            performing_physician_name: row.get("PerformingPhysicianName"),
            procedure_code_sequence: row.get("ProcedureCodeSequence"),
            received_instances: row.get("ReceivedInstances"),
            space_size: row.get("SpaceSize"),
            created_time: row.get("CreatedTime"),
            updated_time: row.get("UpdatedTime"),
        })
    }
}

pub struct MySqlProvider {
    pub pool: MySqlPool,
}

impl MySqlProvider {
    const GET_SERIES_INFO_QUERY: &'static str = r#"SELECT tenant_id, SeriesInstanceUID, StudyInstanceUID, PatientID, Modality, SeriesNumber,
                    COALESCE(SeriesDate, '') as series_date,
                    COALESCE(SeriesTime, '') as series_time,
                    SeriesDescription, BodyPartExamined, ProtocolName,
                    AcquisitionNumber,
                    COALESCE(AcquisitionTime, '') as acquisition_time,
                    COALESCE(AcquisitionDate, '') as acquisition_date,
                    AcquisitionDateTime,
                    PerformingPhysicianName,
                    OperatorsName, NumberOfSeriesRelatedInstances, ReceivedInstances, SpaceSize,
                    CreatedTime, UpdatedTime
             FROM SeriesEntity
             WHERE tenant_id = ? AND SeriesInstanceUID = ?"#;

    const GET_STUDY_INFO_QUERY: &'static str = r#"SELECT tenant_id, StudyInstanceUID, PatientID,
                  COALESCE(StudyDate, '') as StudyDate,
                  COALESCE(StudyTime, '') as StudyTime,
                  AccessionNumber, StudyID, StudyDescription, ReferringPhysicianName,
                  PatientAge, PatientSize, PatientWeight, MedicalAlerts, Allergies,
                  PregnancyStatus, Occupation, AdditionalPatientHistory, PatientComments,
                  AdmissionID, PerformingPhysicianName, ProcedureCodeSequence,
                  ReceivedInstances,
                  SpaceSize,CreatedTime, UpdatedTime
           FROM StudyEntity
           WHERE tenant_id = ? AND StudyInstanceUID = ?"#;
    pub fn new(pool: MySqlPool) -> Self {
        info!("MySqlProvider created with pool: {:?}", pool);

        Self { pool }
    }

    pub(crate) async fn save_patient_info_impl(
        &self,
        tenant_id: &str,
        patient_lists: &[PatientEntity],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error> {
        if patient_lists.is_empty() {
            return Ok(());
        }

        // 移除了 batch 分组，直接处理所有数据
        // 构建批量插入语句，确保字段顺序与表结构完全一致
        let mut query_builder = "INSERT INTO PatientEntity (tenant_id, PatientID, PatientName, PatientBirthDate, PatientSex, \
        PatientBirthTime, EthnicGroup) VALUES ".to_string();
        let placeholders: Vec<String> = (0..patient_lists.len())
            .map(|_| "(?, ?, ?, ?, ?, ?, ?)".to_string())
            .collect();
        query_builder.push_str(&placeholders.join(", "));
        query_builder.push_str(" ON DUPLICATE KEY UPDATE PatientName = VALUES(PatientName), PatientBirthDate = VALUES(PatientBirthDate), \
        PatientSex = VALUES(PatientSex), PatientBirthTime = VALUES(PatientBirthTime), EthnicGroup = VALUES(EthnicGroup)");

        let mut query = sqlx::query(&query_builder);
        for patient in patient_lists {
            query = query
                .bind(tenant_id)
                .bind(&patient.patient_id)
                .bind(&patient.patient_name)
                .bind(&patient.patient_birth_date)
                .bind(&patient.patient_sex)
                .bind(&patient.patient_birth_time)
                .bind(&patient.ethnic_group);
        }
        query.execute(&mut **tx).await?;

        Ok(())
    }

    pub(crate) async fn save_study_info_impl(
        &self,
        tenant_id: &str,
        study_lists: &[StudyEntity],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error> {
        if study_lists.is_empty() {
            return Ok(());
        }

        // 移除了 batch 分组，直接处理所有数据
        // 构建批量插入语句，确保字段顺序与表结构完全一致
        let mut query_builder = "INSERT INTO StudyEntity (tenant_id, StudyInstanceUID, PatientID, StudyDate, StudyTime, \
        AccessionNumber, StudyID, StudyDescription, \
    ReferringPhysicianName, PatientAge, PatientSize, PatientWeight, MedicalAlerts, \
    Allergies, PregnancyStatus, Occupation, AdditionalPatientHistory, PatientComments, \
    AdmissionID, PerformingPhysicianName, ProcedureCodeSequence, ReceivedInstances, SpaceSize) VALUES ".to_string();
        let placeholders: Vec<String> = (0..study_lists.len())
            .map(|_| {
                "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string()
            })
            .collect();
        query_builder.push_str(&placeholders.join(", "));
        query_builder.push_str(" ON DUPLICATE KEY UPDATE StudyDate = VALUES(StudyDate), StudyTime = VALUES(StudyTime), \
        AccessionNumber = VALUES(AccessionNumber), StudyID = VALUES(StudyID),\
    StudyDescription = VALUES(StudyDescription), ReferringPhysicianName = VALUES(ReferringPhysicianName), PatientAge = VALUES(PatientAge),\
    PatientSize = VALUES(PatientSize), PatientWeight = VALUES(PatientWeight), MedicalAlerts = VALUES(MedicalAlerts), Allergies = VALUES(Allergies),\
     PregnancyStatus = VALUES(PregnancyStatus), Occupation = VALUES(Occupation), AdditionalPatientHistory = VALUES(AdditionalPatientHistory), \
     PatientComments = VALUES(PatientComments), AdmissionID = VALUES(AdmissionID), \
     PerformingPhysicianName = VALUES(PerformingPhysicianName), ProcedureCodeSequence = VALUES(ProcedureCodeSequence), \
     ReceivedInstances = VALUES(ReceivedInstances), SpaceSize = VALUES(SpaceSize)");

        let mut query = sqlx::query(&query_builder);
        for study in study_lists {
            query = query
                .bind(tenant_id)
                .bind(&study.study_instance_uid)
                .bind(&study.patient_id)
                .bind(&study.study_date)
                .bind(&study.study_time)
                .bind(&study.accession_number)
                .bind(&study.study_id)
                .bind(&study.study_description)
                .bind(&study.referring_physician_name)
                .bind(&study.patient_age)
                .bind(&study.patient_size)
                .bind(&study.patient_weight)
                .bind(&study.medical_alerts)
                .bind(&study.allergies)
                .bind(&study.pregnancy_status)
                .bind(&study.occupation)
                .bind(&study.additional_patient_history)
                .bind(&study.patient_comments)
                .bind(&study.admission_id)
                .bind(&study.performing_physician_name)
                .bind(&study.procedure_code_sequence)
                .bind(&study.received_instances)
                .bind(&study.space_size);
        }

        query.execute(&mut **tx).await?;

        Ok(())
    }

    pub(crate) async fn save_series_info_impl(
        &self,
        tenant_id: &str,
        series_lists: &[SeriesEntity],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error> {
        if series_lists.is_empty() {
            return Ok(());
        }

        // 移除了 batch 分组，直接处理所有数据
        // 构建批量插入语句，确保字段顺序与表结构完全一致
        let mut query_builder = "INSERT INTO SeriesEntity (tenant_id, SeriesInstanceUID, StudyInstanceUID, PatientID, Modality, SeriesNumber, SeriesDate, SeriesTime, SeriesDescription, BodyPartExamined, ProtocolName, AcquisitionNumber, AcquisitionTime, AcquisitionDate, AcquisitionDateTime, PerformingPhysicianName, OperatorsName, NumberOfSeriesRelatedInstances, ReceivedInstances, SpaceSize) VALUES ".to_string();
        let placeholders: Vec<String> = (0..series_lists.len())
            .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string())
            .collect();
        query_builder.push_str(&placeholders.join(", "));
        query_builder.push_str(" ON DUPLICATE KEY UPDATE Modality = VALUES(Modality), SeriesNumber = VALUES(SeriesNumber), SeriesDate = VALUES(SeriesDate), SeriesTime = VALUES(SeriesTime), SeriesDescription = VALUES(SeriesDescription), BodyPartExamined = VALUES(BodyPartExamined), ProtocolName = VALUES(ProtocolName), AcquisitionNumber = VALUES(AcquisitionNumber), AcquisitionTime = VALUES(AcquisitionTime), AcquisitionDate = VALUES(AcquisitionDate), AcquisitionDateTime = VALUES(AcquisitionDateTime), PerformingPhysicianName = VALUES(PerformingPhysicianName), OperatorsName = VALUES(OperatorsName), NumberOfSeriesRelatedInstances = VALUES(NumberOfSeriesRelatedInstances), ReceivedInstances = VALUES(ReceivedInstances), SpaceSize = VALUES(SpaceSize)");
        let mut query = sqlx::query(&query_builder);
        for series in series_lists {
            query = query
                .bind(tenant_id)
                .bind(&series.series_instance_uid)
                .bind(&series.study_instance_uid)
                .bind(&series.patient_id)
                .bind(&series.modality)
                .bind(&series.series_number)
                .bind(&series.series_date)
                .bind(&series.series_time)
                .bind(&series.series_description)
                .bind(&series.body_part_examined)
                .bind(&series.protocol_name)
                // 移除了 image_type 字段的绑定
                .bind(&series.acquisition_number)
                .bind(&series.acquisition_time)
                .bind(&series.acquisition_date)
                .bind(&series.acquisition_date_time)
                .bind(&series.performing_physician_name)
                .bind(&series.operators_name)
                .bind(&series.number_of_series_related_instances)
                .bind(&series.received_instances)
                .bind(&series.space_size);
        }

        query.execute(&mut **tx).await?;

        Ok(())
    }

    pub(crate) async fn save_instance_info_impl(
        &self,
        tenant_id: &str,
        dicom_obj: &[ImageEntity],
        tx: &mut Transaction<'_, MySql>,
    ) -> Result<(), sqlx::Error> {
        if dicom_obj.is_empty() {
            return Ok(());
        }

        // 移除了 batch 分组，直接处理所有数据
        // 构建批量插入语句，确保字段顺序与表结构完全一致
        let mut query_builder = "INSERT INTO ImageEntity (tenant_id, SOPInstanceUID, SeriesInstanceUID, StudyInstanceUID, PatientID, InstanceNumber, ImageComments, ContentDate, ContentTime, \
AcquisitionDate, AcquisitionTime, AcquisitionDateTime, \
ImageType, ImageOrientationPatient, ImagePositionPatient, SliceThickness, SpacingBetweenSlices, SliceLocation, SamplesPerPixel, PhotometricInterpretation, Width, Columns, BitsAllocated, BitsStored, HighBit, PixelRepresentation, RescaleIntercept, RescaleSlope, RescaleType, WindowCenter, WindowWidth, AcquisitionDeviceProcessingDescription, AcquisitionDeviceProcessingCode, DeviceSerialNumber, SoftwareVersions, TransferSyntaxUID, SOPClassUID, NumberOfFrames, SpaceSize, PixelDataLocation, ThumbnailLocation, ImageStatus) VALUES ".to_string();
        let placeholders: Vec<String> = (0..dicom_obj.len())
            .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string())
            .collect();
        query_builder.push_str(&placeholders.join(", "));
        query_builder.push_str(" ON DUPLICATE KEY UPDATE InstanceNumber = VALUES(InstanceNumber), ImageComments = VALUES(ImageComments), ContentDate = VALUES(ContentDate), \
ContentTime = VALUES(ContentTime), \
AcquisitionDate = VALUES(AcquisitionDate), \
AcquisitionTime = VALUES(AcquisitionTime), \
AcquisitionDateTime = VALUES(AcquisitionDateTime), \
ImageType = VALUES(ImageType), ImageOrientationPatient = VALUES(ImageOrientationPatient), ImagePositionPatient = VALUES(ImagePositionPatient), SliceThickness = VALUES(SliceThickness), SpacingBetweenSlices = VALUES(SpacingBetweenSlices), SliceLocation = VALUES(SliceLocation), SamplesPerPixel = VALUES(SamplesPerPixel), PhotometricInterpretation = VALUES(PhotometricInterpretation), Width = VALUES(Width), Columns = VALUES(Columns), BitsAllocated = VALUES(BitsAllocated), BitsStored = VALUES(BitsStored), HighBit = VALUES(HighBit), PixelRepresentation = VALUES(PixelRepresentation), RescaleIntercept = VALUES(RescaleIntercept), RescaleSlope = VALUES(RescaleSlope), RescaleType = VALUES(RescaleType), WindowCenter = VALUES(WindowCenter), WindowWidth = VALUES(WindowWidth), AcquisitionDeviceProcessingDescription = VALUES(AcquisitionDeviceProcessingDescription), AcquisitionDeviceProcessingCode = VALUES(AcquisitionDeviceProcessingCode), DeviceSerialNumber = VALUES(DeviceSerialNumber), SoftwareVersions = VALUES(SoftwareVersions), TransferSyntaxUID = VALUES(TransferSyntaxUID), NumberOfFrames = VALUES(NumberOfFrames), SpaceSize = VALUES(SpaceSize), PixelDataLocation = VALUES(PixelDataLocation), ThumbnailLocation = VALUES(ThumbnailLocation), ImageStatus = VALUES(ImageStatus)");

        let mut query = sqlx::query(&query_builder);

        for image in dicom_obj {
            query = query
                .bind(tenant_id)
                .bind(&image.sop_instance_uid)
                .bind(&image.series_instance_uid)
                .bind(&image.study_instance_uid)
                .bind(&image.patient_id)
                .bind(&image.instance_number)
                .bind(&image.image_comments)
                .bind(&image.content_date)
                .bind(&image.content_time)
                .bind(&image.acquisition_date)
                .bind(&image.acquisition_time)
                .bind(&image.acquisition_date_time)
                .bind(&image.image_type)
                .bind(&image.image_orientation_patient)
                .bind(&image.image_position_patient)
                .bind(&image.slice_thickness)
                .bind(&image.spacing_between_slices)
                .bind(&image.slice_location)
                .bind(&image.samples_per_pixel)
                .bind(&image.photometric_interpretation)
                .bind(&image.width)
                .bind(&image.columns)
                .bind(&image.bits_allocated)
                .bind(&image.bits_stored)
                .bind(&image.high_bit)
                .bind(&image.pixel_representation)
                .bind(&image.rescale_intercept)
                .bind(&image.rescale_slope)
                .bind(&image.rescale_type)
                .bind(&image.window_center)
                .bind(&image.window_width)
                .bind(&image.acquisition_device_processing_description)
                .bind(&image.acquisition_device_processing_code)
                .bind(&image.device_serial_number)
                .bind(&image.software_versions)
                .bind(&image.transfer_syntax_uid)
                .bind(&image.sop_class_uid)
                .bind(&image.number_of_frames)
                .bind(&image.space_size)
                .bind(&image.pixel_data_location)
                .bind(&image.thumbnail_location)
                .bind(&image.image_status);
        }

        query.execute(&mut **tx).await?;

        Ok(())
    }

    async fn save_entities<T>(&self, tenant_id: &str, entities: &[T]) -> Result<(), DbError>
    where
        T: DbEntity + Send + Sync,
    {
        if entities.is_empty() {
            return Ok(());
        }

        let pool = self.pool.clone();
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to start transaction: {}", e);
                return Err(DbError::TransactionFailed(format!(
                    "Failed to start transaction: {}",
                    e
                )));
            }
        };

        for chunk in entities.chunks(BATCH_SIZE) {
            match T::save_impl(self, tenant_id, chunk, &mut tx).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to save entities: {}", e);
                    if let Err(rollback_err) = tx.rollback().await {
                        error!("Failed to rollback transaction: {}", rollback_err);
                    }
                    return Err(DbError::DatabaseError(e));
                }
            }
        }

        match tx.commit().await {
            Ok(_) => {
                info!(
                    "Successfully saved entities: {}, {}",
                    tenant_id,
                    entities.len()
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to commit transaction: {}", e);
                Err(DbError::TransactionFailed(format!(
                    "Failed to commit transaction: {}",
                    e
                )))
            }
        }
    }

    async fn fetch_one_with_convert<T, A, F>(
        &self,
        command_text: &str,
        args: &[A],
        factory: F,
    ) -> Result<T, DbError>
    where
        T: Send + Unpin,
        A: for<'r> sqlx::Encode<'r, MySql> + sqlx::Type<MySql> + Send + Sync,
        F: Fn(&MySqlRow) -> Result<T, sqlx::Error> + Send + Sync,
    {
        let pool = self.pool.clone();
        let mut query = sqlx::query(command_text);

        for arg in args {
            query = query.bind(arg);
        }
        match query.fetch_one(&pool).await {
            Ok(row) => match factory(&row) {
                Ok(entity) => Ok(entity),
                Err(e) => {
                    error!("Failed to deserialize entity: {}", e);
                    Err(DbError::DatabaseError(e))
                }
            },
            Err(e) => {
                error!("Failed to execute query: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn fetch_one<T, A>(&self, command_text: &str, args: &[A]) -> Result<T, DbError>
    where
        T: for<'r> FromRow<'r, MySqlRow> + Send + Unpin,
        A: for<'r> sqlx::Encode<'r, MySql> + sqlx::Type<MySql> + Send + Sync,
    {
        let pool = self.pool.clone();
        let mut query = sqlx::query(command_text);

        for arg in args {
            query = query.bind(arg);
        }

        match query.fetch_one(&pool).await {
            Ok(row) => match T::from_row(&row) {
                Ok(entity) => Ok(entity),
                Err(e) => {
                    error!("Failed to deserialize entity: {}", e);
                    Err(DbError::DatabaseError(e))
                }
            },
            Err(e) => {
                error!("Failed to execute query: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }
    async fn fetch_all<T, A>(&self, command_text: &str, args: &[A]) -> Result<Vec<T>, DbError>
    where
        T: for<'r> FromRow<'r, MySqlRow> + Send + Unpin,
        A: for<'q> sqlx::Encode<'q, MySql> + sqlx::Type<MySql> + Send + Sync,
    {
        let pool = self.pool.clone();
        let mut query = sqlx::query(command_text);

        // 正确绑定参数
        for arg in args {
            query = query.bind(arg);
        }

        match query.fetch_all(&pool).await {
            Ok(rows) => {
                let mut entities = Vec::with_capacity(rows.len());
                for row in rows {
                    match T::from_row(&row) {
                        Ok(entity) => entities.push(entity),
                        Err(e) => {
                            error!("Failed to deserialize entity: {}", e);
                            return Err(DbError::DatabaseError(e));
                        }
                    }
                }
                Ok(entities)
            }
            Err(e) => {
                error!("Failed to execute query: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    /// 返回 (研究实体列表, (系列数量, 图像数量))
    async fn fetch_study_statistics(
        &self,
        study_uid: &str,
    ) -> Result<(Vec<StudyEntity>, (i32, i32)), DbError> {
        let mut tx = self.pool.begin().await.map_err(DbError::DatabaseError)?;
        // 查询Study基本信息
        let study_query = "SELECT tenant_id, StudyInstanceUID, PatientID,
                          COALESCE(StudyDate, '') as StudyDate,
                          COALESCE(StudyTime, '') as StudyTime,
                          AccessionNumber, StudyID, StudyDescription, ReferringPhysicianName,
                          PatientAge, PatientSize, PatientWeight, MedicalAlerts, Allergies,
                          PregnancyStatus, Occupation, AdditionalPatientHistory, PatientComments,
                          AdmissionID, PerformingPhysicianName, ProcedureCodeSequence,
                          ReceivedInstances,
                          SpaceSize,CreatedTime, UpdatedTime
                   FROM StudyEntity
                   WHERE StudyInstanceUID = ?";
        // 查询Study基本信息
        let study_query = sqlx::query_as::<_, StudyEntity>(study_query).bind(study_uid);
        let study_result = study_query
            .fetch_all(&mut *tx)
            .await
            .map_err(DbError::DatabaseError)?;

        // 查询Series统计信息
        let series_stats_query = sqlx::query(
            "SELECT COUNT(*) as series_count FROM SeriesEntity WHERE StudyInstanceUID = ?",
        )
        .bind(study_uid);
        let series_stats = series_stats_query
            .fetch_one(&mut *tx)
            .await
            .map_err(DbError::DatabaseError)?;

        // 查询Image统计信息
        let image_stats_query = sqlx::query(
            "SELECT COUNT(*) as image_count   FROM ImageEntity WHERE StudyInstanceUID = ?",
        )
        .bind(study_uid);
        let image_stats = image_stats_query
            .fetch_one(&mut *tx)
            .await
            .map_err(DbError::DatabaseError)?;

        tx.commit().await.map_err(DbError::DatabaseError)?;

        Ok((
            study_result,
            (
                series_stats.get("series_count"),
                image_stats.get("image_count"),
            ),
        ))
    }
}

static BATCH_SIZE: usize = 10;
#[async_trait]
impl DbProvider for MySqlProvider {
    async fn echo(&self) -> Result<String, DbError> {
        let pool = self.pool.clone();
        let result = sqlx::query("SELECT 'Hello, world!'").fetch_one(&pool).await;
        match result {
            Ok(row) => {
                let message: String = row.get(0);
                Ok(message)
            }
            Err(e) => {
                error!("Failed to execute query: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn save_dicommeta_info(&self, dicom_obj: &[DicomObjectMeta]) -> Result<(), DbError> {
        if dicom_obj.is_empty() {
            return Ok(());
        }

        let mut tx = self.pool.begin().await.map_err(DbError::DatabaseError)?;

        // 构建批量插入语句
        let mut query_builder = "INSERT INTO dicom_object_meta (
            tenant_id,
            patient_id,
            study_uid,
            series_uid,
            sop_uid,
            file_size,
            file_path,
            transfer_syntax_uid,
            number_of_frames
        ) VALUES "
            .to_string();

        let placeholders: Vec<String> = (0..dicom_obj.len())
            .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string())
            .collect();
        query_builder.push_str(&placeholders.join(", "));
        query_builder.push_str(
            " ON DUPLICATE KEY UPDATE
            patient_id = VALUES(patient_id),
            study_uid = VALUES(study_uid),
            series_uid = VALUES(series_uid),
            file_size = VALUES(file_size),
            file_path = VALUES(file_path),
            transfer_syntax_uid = VALUES(transfer_syntax_uid),
            number_of_frames = VALUES(number_of_frames),
            updated_at = CURRENT_TIMESTAMP",
        );

        let mut query = sqlx::query(&query_builder);
        for obj in dicom_obj {
            query = query
                .bind(&obj.tenant_id)
                .bind(&obj.patient_id)
                .bind(&obj.study_uid)
                .bind(&obj.series_uid)
                .bind(&obj.sop_uid)
                .bind(&obj.file_size)
                .bind(&obj.file_path)
                .bind(&obj.transfer_synatx_uid)
                .bind(obj.number_of_frames);
        }

        query
            .execute(&mut *tx)
            .await
            .map_err(DbError::DatabaseError)?;
        tx.commit().await.map_err(DbError::DatabaseError)?;

        Ok(())
    }

    async fn save_dicom_info(
        &self,
        tenant_id: &str,
        dicom_obj: &DefaultDicomObject,
    ) -> Result<(), DbError> {
        let tenant_id = tenant_id.to_string();
        // 使用 DbProviderBase 提取实体信息
        let (patient, study, series, image) =
            match DbProviderBase::extract_entity(&tenant_id, dicom_obj) {
                Ok(entities) => entities,
                Err(e) => {
                    error!("Failed to extract DICOM entities: {}", e);
                    return Err(DbError::ExtractionFailed(e.to_string()));
                }
            };

        let pool = self.pool.clone();

        // 开始事务
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to start transaction: {}", e);
                return Err(DbError::TransactionFailed(
                    "Failed to start transaction".to_string(),
                ));
            }
        };
        let sop_uid = image.sop_instance_uid.clone();
        // 保存患者信息
        if let Err(e) = self
            .save_patient_info_impl(&tenant_id, &[patient], &mut tx)
            .await
        {
            error!("Failed to save patient info: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Err(DbError::DatabaseError(e));
        }

        // 保存检查信息
        if let Err(e) = self
            .save_study_info_impl(&tenant_id, &[study], &mut tx)
            .await
        {
            error!("Failed to save study info: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Err(DbError::DatabaseError(e));
        }

        // 保存序列信息
        if let Err(e) = self
            .save_series_info_impl(&tenant_id, &[series], &mut tx)
            .await
        {
            error!("Failed to save series info: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Err(DbError::DatabaseError(e));
        }

        // 保存实例信息
        if let Err(e) = self
            .save_instance_info_impl(&tenant_id, &[image], &mut tx)
            .await
        {
            error!("Failed to save instance info: {}", e);
            if let Err(rollback_err) = tx.rollback().await {
                error!("Failed to rollback transaction: {}", rollback_err);
            }
            return Err(DbError::DatabaseError(e));
        }

        // 提交事务
        match tx.commit().await {
            Ok(_) => {
                info!(
                    "Successfully saved DICOM info for SOP Instance UID: {}",
                    sop_uid
                );
                Ok(())
            }
            Err(e) => {
                error!("Failed to commit transaction: {}", e);
                Err(DbError::TransactionFailed(format!(
                    "Failed to commit transaction: {}",
                    e
                )))
            }
        }
    }

    async fn save_patient_info(
        &self,
        tenant_id: &str,
        patient_lists: &[PatientEntity],
    ) -> Result<(), DbError> {
        self.save_entities(tenant_id, patient_lists).await
    }

    async fn save_study_info(
        &self,
        tenant_id: &str,
        study_lists: &[StudyEntity],
    ) -> Result<(), DbError> {
        self.save_entities(tenant_id, study_lists).await
    }

    async fn save_series_info(
        &self,
        tenant_id: &str,
        series_lists: &[SeriesEntity],
    ) -> Result<(), DbError> {
        self.save_entities(tenant_id, series_lists).await
    }

    async fn save_instance_info(
        &self,
        tenant_id: &str,
        image_lists: &[ImageEntity],
    ) -> Result<(), DbError> {
        self.save_entities(tenant_id, image_lists).await
    }

    async fn delete_study_info(&self, tenant_id: &str, study_uid: &str) -> Result<bool, DbError> {
        let tenant_id = tenant_id.to_string();
        let study_uid = study_uid.to_string();
        let pool = self.pool.clone();

        match sqlx::query("DELETE FROM StudyEntity WHERE tenant_id = ? AND StudyInstanceUID = ?")
            .bind(&tenant_id)
            .bind(&study_uid)
            .execute(&pool)
            .await
        {
            Ok(result) => Ok(result.rows_affected() > 0),
            Err(e) => {
                error!("Failed to delete study info: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn delete_series_info(
        &self,
        tenant_id: &str,
        study_uid: &str,
        series_uid: &str,
    ) -> Result<bool, DbError> {
        let tenant_id = tenant_id.to_string();
        let study_uid = study_uid.to_string();
        let series_uid = series_uid.to_string();
        let pool = self.pool.clone();

        match sqlx::query(
            "DELETE FROM SeriesEntity WHERE tenant_id = ? AND StudyInstanceUID = ? AND SeriesInstanceUID = ?"
        )
            .bind(&tenant_id)
            .bind(&study_uid)
            .bind(&series_uid)
            .execute(&pool)
            .await {
            Ok(result) => Ok(result.rows_affected() > 0),
            Err(e) => {
                error!("Failed to delete series info: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn delete_instance_info(
        &self,
        tenant_id: &str,
        study_uid: &str,
        series_uid: &str,
        instance_uid: &str,
    ) -> Result<bool, DbError> {
        let tenant_id = tenant_id.to_string();
        let study_uid = study_uid.to_string();
        let series_uid = series_uid.to_string();
        let instance_uid = instance_uid.to_string();
        let pool = self.pool.clone();

        match sqlx::query(
            "DELETE FROM ImageEntity WHERE tenant_id = ? AND StudyInstanceUID = ? AND SeriesInstanceUID = ? AND SOPInstanceUID = ?"
        )
            .bind(&tenant_id)
            .bind(&study_uid)
            .bind(&series_uid)
            .bind(&instance_uid)
            .execute(&pool)
            .await {
            Ok(result) => Ok(result.rows_affected() > 0),
            Err(e) => {
                error!("Failed to delete instance info: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn patient_exists(&self, tenant_id: &str, patient_id: &str) -> Result<bool, DbError> {
        let pool = self.pool.clone();

        match sqlx::query(
            "SELECT COUNT(*) FROM PatientEntity WHERE tenant_id = ? AND PatientID = ?",
        )
        .bind(tenant_id)
        .bind(patient_id)
        .fetch_one(&pool)
        .await
        {
            Ok(row) => {
                let count: i64 = row.get(0);
                Ok(count > 0)
            }
            Err(e) => {
                error!("Failed to check if patient exists: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn patient_study_exists(
        &self,
        tenant_id: &str,
        patient_id: &str,
        study_uid: &str,
    ) -> Result<bool, DbError> {
        let pool = self.pool.clone();

        match sqlx::query(
            "SELECT COUNT(*) FROM StudyEntity WHERE tenant_id = ? AND PatientID = ? AND StudyInstanceUID = ?"
        )
            .bind(tenant_id)
            .bind(patient_id)
            .bind(study_uid)
            .fetch_one(&pool)
            .await
        {
            Ok(row) => {
                let count: i64 = row.get(0);
                Ok(count > 0)
            }
            Err(e) => {
                error!("Failed to check if patient exists: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn patient_series_exists(
        &self,
        tenant_id: &str,
        patient_id: &str,
        study_uid: &str,
        series_uid: &str,
    ) -> Result<bool, DbError> {
        let pool = self.pool.clone();

        match sqlx::query(
            "SELECT COUNT(*) FROM SeriesEntity
         WHERE SeriesEntity.tenant_id = ? AND SeriesEntity.PatientID = ? AND SeriesEntity.StudyInstanceUID = ? AND SeriesEntity.SeriesInstanceUID = ?"
        )
            .bind(tenant_id)
            .bind(patient_id) // 使用 SeriesEntity 中的 PatientID 字段
            .bind(study_uid)
            .bind(series_uid)
            .fetch_one(&pool)
            .await
        {
            Ok(row) => {
                let count: i64 = row.get(0);
                Ok(count > 0)
            }
            Err(e) => {
                error!("Failed to check if patient series exists: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn patient_instance_exists(
        &self,
        tenant_id: &str,
        patient_id: &str,
        study_uid: &str,
        series_uid: &str,
        instance_uid: &str,
    ) -> Result<bool, DbError> {
        let pool = self.pool.clone();

        match sqlx::query(
            "SELECT COUNT(*) FROM ImageEntity
             INNER JOIN StudyEntity ON ImageEntity.StudyInstanceUID = StudyEntity.StudyInstanceUID
             WHERE ImageEntity.tenant_id = ? AND StudyEntity.PatientID = ?
               AND ImageEntity.StudyInstanceUID = ? AND ImageEntity.SeriesInstanceUID = ?
               AND ImageEntity.SOPInstanceUID = ?"
        )
            .bind(tenant_id)
            .bind(patient_id)
            .bind(study_uid)
            .bind(series_uid)
            .bind(instance_uid)
            .fetch_one(&pool)
            .await
        {
            Ok(row) => {
                let count: i64 = row.get(0);
                Ok(count > 0)
            }
            Err(e) => {
                error!("Failed to check if patient instance exists: {}", e);
                Err(DbError::DatabaseError(e))
            }
        }
    }

    async fn persist_to_database(
        &self,
        tenant_id: &str,
        patient_list: &[PatientEntity],
        study_list: &[StudyEntity],
        series_list: &[SeriesEntity],
        images_list: &[ImageEntity],
    ) -> Result<(), DbError> {
        // 批量插入到数据库，并处理结果
        if !patient_list.is_empty() {
            info!("开始保存 {} 条患者数据", patient_list.len());
            self.save_patient_info(tenant_id, patient_list).await?;
            info!("成功保存 {} 条患者数据", patient_list.len());
        }

        if !study_list.is_empty() {
            info!("开始保存 {} 条检查数据", study_list.len());
            self.save_study_info(tenant_id, study_list).await?;
            info!("成功保存 {} 条检查数据", study_list.len());
        }

        if !series_list.is_empty() {
            info!("开始保存 {} 条序列数据", series_list.len());
            self.save_series_info(tenant_id, series_list).await?;
            info!("成功保存 {} 条序列数据", series_list.len());
        }

        if !images_list.is_empty() {
            info!("开始保存 {} 条图像数据", images_list.len());
            self.save_instance_info(tenant_id, images_list).await?;
            info!("成功保存 {} 条图像数据", images_list.len());
        }

        Ok(())
    }

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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{server_config, utils};
    use dicom_dictionary_std::tags;
    use sqlx::MySqlPool;
    use std::collections::HashMap;
    use std::path::Path;
    use dicom_object::file::CharacterSetOverride;
    // use dicom_object::collector::CharacterSetOverride;
    // 测试数据库连接配置 - 使用测试数据库

    // 设置测试数据库
    async fn setup_test_database() -> MySqlProvider {
        // 对数据库 URL 进行解码处理
        let config = server_config::load_config();
        let config = match config {
            Ok(config) => config,
            Err(e) => {
                println!("Failed to load config: {:?}", e);
                std::process::exit(-2);
            }
        };
        let mysql_url = match server_config::generate_database_connection(&config) {
            Ok(url) => url,
            Err(e) => {
                tracing::log::error!("{:?}", e);
                std::process::exit(-2);
            }
        };
        // 使用 url crate 正确解析包含特殊字符的URL

        info!("mysql_url: {}", mysql_url);
        let pool = MySqlPool::connect(mysql_url.as_str())
            .await
            .expect("Failed to connect to MySQL");
        MySqlProvider { pool }
    }

    #[derive(FromRow, Debug)]
    struct StateInfo2 {
        #[sqlx(rename = "SeriesInstanceUID")]
        ssid: String,
        #[sqlx(rename = "Images")]
        sub_images: i32,
    }

    #[tokio::test]
    async fn test_fetch_one() {
        let provider = setup_test_database().await;
        let user_id = "1.2.156.112605.0.1685486876.2025061710152134339.2.1.1";
        let result = provider.fetch_one::<StateInfo2, _>(
            "select SeriesInstanceUID, count(*) as Images from ImageEntity where StudyInstanceUID = ? group by SeriesInstanceUID order by Images desc limit 1;",
            &[&user_id]
        ).await;

        let state_info = result.unwrap();
        assert_eq!(state_info.sub_images, 367, "ImageCount == 367");
    }
    //fetch_all
    #[tokio::test]
    async fn test_fetch_all() {
        let provider = setup_test_database().await;
        let user_id = "1.2.156.112605.0.1685486876.2025061710152134339.2.1.1";
        let result = provider.fetch_all::<StateInfo2, _>(
            "select SeriesInstanceUID, count(*) as Images from ImageEntity where StudyInstanceUID = ? group by SeriesInstanceUID order by Images desc",
            &[&user_id]
        ).await;

        let state_infos = result.unwrap();
        assert!(!state_infos.is_empty(), "Should have at least one result");

        // 检查第一个结果（图像数最多的序列）
        if let Some(first) = state_infos.first() {
            assert_eq!(first.sub_images, 367, "First series should have 367 images");
        }
    }
    struct StateInfo {
        ssid: String,
        sub_images: i32,
    }
    #[tokio::test]
    async fn test_fetch_one_with_convert() {
        let provider = setup_test_database().await;
        // 或者使用自定义工厂函数
        let user_id = "1.2.156.112605.0.1685486876.2025061710152134339.2.1.1";
        let result = provider.fetch_one_with_convert(
            "select SeriesInstanceUID, count(*) as Images from ImageEntity where  StudyInstanceUID = ? group by SeriesInstanceUID order by Images desc  limit 1 ;",
            &[&user_id],
            |row| {
                Ok(StateInfo {
                    ssid: row.get("SeriesInstanceUID"),
                    sub_images: row.get("Images"),
                })
            }
        ).await;

        let rk = result.unwrap();
        assert_eq!(rk.sub_images, 367, "ImageCount == 367");
    }
    #[tokio::test]
    async fn test_fetch_study_statistics() {
        let provider = setup_test_database().await;
        let study_uid = "1.2.156.112605.0.1685486876.2025061710152134339.2.1.1";

        let (studies, (series, images)) = provider.fetch_study_statistics(study_uid).await.unwrap();

        // 验证Study信息存在
        assert_eq!(studies.is_empty(), false);
        assert!(series > 0);
        assert!(images > 0);
    }
    #[tokio::test]
    async fn test_fetch_one_with_convert2() {
        let provider = setup_test_database().await;
        // 或者使用自定义工厂函数
        let user_id = "1.2.156.112605.0.1685486876.2025061710152134339.2.1.1";
        let result = provider.fetch_one_with_convert(
            "select SeriesInstanceUID, count(*) as Images from ImageEntity where  StudyInstanceUID = ? group by SeriesInstanceUID order by Images desc limit 1;",
            &[&user_id],
            |row| {
                Ok(
                    Ok::<(String, i32), DbError>((row.get("SeriesInstanceUID"), row.get("Images")))
                )
            }
        ).await;

        let (_, image_count): (String, i32) = result.unwrap().expect("REASON");
        assert_eq!(image_count, 367, "ImageCount == 367");
    }

    // 创建测试用的 DICOM 对象
    // fn create_test_dicom_object() -> InMemDicomObject {
    //     let mut obj = InMemDicomObject::new_empty();
    //
    //     // 添加基本的 DICOM 元素用于测试
    //     obj.put(DataElement::new(
    //         tags::PATIENT_ID,
    //         VR::LO,
    //         PrimitiveValue::from("TEST_PATIENT_001"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::PATIENT_NAME,
    //         VR::PN,
    //         PrimitiveValue::from("Doe^John"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::PATIENT_BIRTH_DATE,
    //         VR::DA,
    //         PrimitiveValue::from("19800101"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::PATIENT_SEX,
    //         VR::CS,
    //         PrimitiveValue::from("M"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::STUDY_INSTANCE_UID,
    //         VR::UI,
    //         PrimitiveValue::from("1.2.3.4.5.6.7.8.9.10"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::STUDY_DATE,
    //         VR::DA,
    //         PrimitiveValue::from("20230101"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::STUDY_TIME,
    //         VR::TM,
    //         PrimitiveValue::from("120000"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::STUDY_DESCRIPTION,
    //         VR::LO,
    //         PrimitiveValue::from("CT Chest"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::SERIES_INSTANCE_UID,
    //         VR::UI,
    //         PrimitiveValue::from("1.2.3.4.5.6.7.8.9.11"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::MODALITY,
    //         VR::CS,
    //         PrimitiveValue::from("CT"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::SERIES_NUMBER,
    //         VR::IS,
    //         PrimitiveValue::from(1),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::SOP_INSTANCE_UID,
    //         VR::UI,
    //         PrimitiveValue::from("1.2.3.4.5.6.7.8.9.12"),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::SOP_CLASS_UID,
    //         VR::UI,
    //         PrimitiveValue::from("1.2.840.10008.5.1.4.1.1.2"), // CT Image Storage
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::INSTANCE_NUMBER,
    //         VR::IS,
    //         PrimitiveValue::from(1),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::ROWS,
    //         VR::US,
    //         PrimitiveValue::from(512),
    //     ));
    //
    //     obj.put(DataElement::new(
    //         tags::COLUMNS,
    //         VR::US,
    //         PrimitiveValue::from(512),
    //     ));
    //
    //     obj
    // }

    #[tokio::test]
    async fn test_save_dicom_info_success() {
        let provider = setup_test_database().await;

        let dir_path = "/home/dhz/jpdata/CDSS/89269";
        let dicom_files = utils::get_dicom_files_in_dir(dir_path).await;
        // 需要处理 Result 类型
        let dicom_files = match dicom_files {
            Ok(files) => files,
            Err(e) => {
                eprintln!("Failed to get DICOM files: {}", e);
                return;
            }
        };
        let tenant_id = "1234567890";
        let mut patient_list = HashMap::new();
        let mut study_list = HashMap::new();
        let mut series_list = HashMap::new();
        let mut images_list = HashMap::new();

        for dicom_file_path in dicom_files {
            let path = Path::new(&dicom_file_path);

            // 检查路径是否存在
            if !path.exists() {
                eprintln!("File does not exist: {:?}", path);
                continue;
            }
            let dicom_obj: Result<_, Box<dyn std::error::Error>> =
                dicom_object::OpenFileOptions::new()
                    .charset_override(CharacterSetOverride::AnyVr)
                    .read_until(tags::PIXEL_DATA)
                    .open_file(path)
                    .map_err(Box::from);
            match dicom_obj {
                Ok(dcmobj) => {
                    let patient_entity =
                        DbProviderBase::extract_patient_entity(tenant_id, &dcmobj).unwrap();
                    // 修复：正确检查 patient_name 是否存在
                    let patient_id = patient_entity.patient_id.clone();
                    if !patient_list.contains_key(&patient_entity.patient_id) {
                        patient_list.insert(patient_id.clone(), patient_entity);
                    }
                    let study_entity = DbProviderBase::extract_study_entity(
                        tenant_id,
                        &dcmobj,
                        &patient_id, // 使用 clone 后的值，避免 move
                    )
                    .unwrap();
                    let study_uid = study_entity.study_instance_uid.clone();
                    if !study_list.contains_key(study_uid.as_str()) {
                        study_list.insert(study_uid.clone(), study_entity);
                    }
                    let series_entity = DbProviderBase::extract_series_entity(
                        tenant_id,
                        &dcmobj,
                        study_uid.as_str(), // 使用 clone 后的值，避免 move
                    )
                    .unwrap();
                    let series_id = series_entity.series_instance_uid.clone();
                    if !series_list.contains_key(series_id.as_str()) {
                        series_list.insert(series_id.clone(), series_entity);
                    }
                    let image_entity = DbProviderBase::extract_image_entity(
                        tenant_id,
                        &dcmobj,
                        &patient_id,        // 使用 clone 后的值，避免 move
                        study_uid.as_str(), // 使用 clone 后的值，避免 move
                        series_id.as_str(), // 使用 clone 后的值，避免 move
                    )
                    .unwrap();
                    let image_id = image_entity.sop_instance_uid.clone();
                    if !images_list.contains_key(image_id.as_str()) {
                        images_list.insert(image_id.clone(), image_entity);
                    }
                }
                Err(e) => {
                    eprintln!("Failed to open DICOM file: {}", e);
                }
            }
        }

        println!("开始保存数据:{}..", images_list.len());

        // 批量插入到数据库，并处理结果
        if !patient_list.is_empty() {
            match provider
                .save_patient_info(
                    tenant_id,
                    &patient_list.values().cloned().collect::<Vec<_>>(),
                )
                .await
            {
                Ok(()) => println!("成功保存 {} 条患者数据", patient_list.len()),
                Err(e) => println!("保存患者数据失败: {:?}", e),
            }
        }

        if !study_list.is_empty() {
            match provider
                .save_study_info(tenant_id, &study_list.values().cloned().collect::<Vec<_>>())
                .await
            {
                Ok(()) => println!("成功保存 {} 条检查数据", study_list.len()),
                Err(e) => println!("保存检查数据失败: {:?}", e),
            }
        }

        if !series_list.is_empty() {
            match provider
                .save_series_info(
                    tenant_id,
                    &series_list.values().cloned().collect::<Vec<_>>(),
                )
                .await
            {
                Ok(()) => println!("成功保存 {} 条序列数据", series_list.len()),
                Err(e) => println!("保存序列数据失败: {:?}", e),
            }
        }

        if !images_list.is_empty() {
            match provider
                .save_instance_info(
                    tenant_id,
                    &images_list.values().cloned().collect::<Vec<_>>(),
                )
                .await
            {
                Ok(()) => println!("成功保存 {} 条图像数据", images_list.len()),
                Err(e) => {
                    println!("保存图像数据失败: {:?}", e);
                    // 添加更详细的调试信息
                    println!("图像数据详情:");
                    for (i, (id, image)) in images_list.iter().take(3).enumerate() {
                        println!(
                            "  Image {}: ID={}, Series={}, Study={}",
                            i, id, image.series_instance_uid, image.study_instance_uid
                        );
                    }
                    if images_list.len() > 3 {
                        println!("  ... and {} more items", images_list.len() - 3);
                    }
                }
            }
        }
    }
}
