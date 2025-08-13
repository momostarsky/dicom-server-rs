use crate::db_provider::DbProvider;
use crate::entities::{DbProviderBase, ImageEntity, PatientEntity, SeriesEntity, StudyEntity};
use async_trait::async_trait;
use dicom_object::DefaultDicomObject;
use sqlx::{MySqlPool, Row};
use tracing::{error, info};

pub struct MySqlProvider {
    pool: MySqlPool,
}

#[async_trait]
impl DbProvider for MySqlProvider {
    async fn save_dicom_info(
        &self,
        tenant_id: &str,
        dicom_obj: &DefaultDicomObject,
    ) -> Option<bool> {
        let tenant_id = tenant_id.to_string();
        let pool = self.pool.clone();

        // 使用 DbProviderBase 提取实体信息
        let patient_entity = DbProviderBase::extract_patient_entity(&tenant_id, dicom_obj);
        let study_entity =
            DbProviderBase::extract_study_entity(&tenant_id, dicom_obj, &patient_entity.patient_id);
        let series_entity = DbProviderBase::extract_series_entity(
            &tenant_id,
            dicom_obj,
            &study_entity.study_instance_uid,
        );
        let image_entity = DbProviderBase::extract_image_entity(
            &tenant_id,
            dicom_obj,
            &series_entity.series_instance_uid,
            &study_entity.study_instance_uid,
            &patient_entity.patient_id,
        );

        // 开始事务
        let mut tx = match pool.begin().await {
            Ok(tx) => tx,
            Err(e) => {
                error!("Failed to start transaction: {}", e);
                return None;
            }
        };

        // 1. 保存或更新患者信息
        let patient_result = sqlx::query(
            r"INSERT INTO PatientEntity (
            tenant_id, PatientID, PatientName, PatientBirthDate,
            PatientSex, PatientBirthTime, EthnicGroup
        ) SELECT ?, ?, ?, ?, ?, ?, ?
        WHERE NOT EXISTS (
            SELECT 1 FROM PatientEntity
            WHERE tenant_id = ? AND PatientID = ?
        )",
        )
        .bind(&patient_entity.tenant_id)
        .bind(&patient_entity.patient_id)
        .bind(&patient_entity.patient_name)
        .bind(&patient_entity.patient_birth_date)
        .bind(&patient_entity.patient_sex)
        .bind(&patient_entity.patient_birth_time)
        .bind(&patient_entity.ethnic_group)
        .bind(&patient_entity.tenant_id) // 重复绑定用于WHERE子句
        .bind(&patient_entity.patient_id) // 重复绑定用于WHERE子句
        .execute(&mut *tx)
        .await;

        if let Err(e) = patient_result {
            error!("Failed to save patient info: {}", e);
            return None;
        }

        // 2. 保存或更新检查信息
        let study_result = sqlx::query(
            r"INSERT INTO StudyEntity (
            tenant_id, StudyInstanceUID, PatientID, StudyDate, StudyTime,
            AccessionNumber, StudyID, StudyDescription, ReferringPhysicianName,
            PatientAge, PatientSize, PatientWeight, MedicalAlerts, Allergies,
            PregnancyStatus, Occupation, AdditionalPatientHistory, PatientComments,
            AdmissionID, PatientAgeAtStudy,
            PerformingPhysicianName, ProcedureCodeSequence
        ) SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        WHERE NOT EXISTS (
            SELECT 1 FROM StudyEntity 
            WHERE tenant_id = ? AND StudyInstanceUID = ?
        )",
        )
        .bind(&study_entity.tenant_id)
        .bind(&study_entity.study_instance_uid)
        .bind(&study_entity.patient_id)
        .bind(&study_entity.study_date)
        .bind(&study_entity.study_time)
        .bind(&study_entity.accession_number)
        .bind(&study_entity.study_id)
        .bind(&study_entity.study_description)
        .bind(&study_entity.referring_physician_name)
        .bind(&study_entity.patient_age)
        .bind(&study_entity.patient_size)
        .bind(&study_entity.patient_weight)
        .bind(&study_entity.medical_alerts)
        .bind(&study_entity.allergies)
        .bind(&study_entity.pregnancy_status)
        .bind(&study_entity.occupation)
        .bind(&study_entity.additional_patient_history)
        .bind(&study_entity.patient_comments)
        .bind(&study_entity.admission_id)
        .bind(&study_entity.patient_age_at_study)
        .bind(&study_entity.performing_physician_name)
        .bind(&study_entity.procedure_code_sequence)
        .bind(&study_entity.tenant_id) // 用于WHERE子句
        .bind(&study_entity.study_instance_uid) // 用于WHERE子句
        .execute(&mut *tx)
        .await;

        if let Err(e) = study_result {
            error!("Failed to save study info: {}", e);
            return None;
        }

        // 3. 保存或更新序列信息
        let series_result = sqlx::query(
            r"INSERT INTO SeriesEntity (
            tenant_id, SeriesInstanceUID, StudyInstanceUID, Modality,
            SeriesNumber, SeriesDate, SeriesTime, SeriesDescription,
            BodyPartExamined, ProtocolName, ImageType, AcquisitionNumber,
            AcquisitionTime, AcquisitionDate, PerformingPhysicianName,
            OperatorsName, NumberOfSeriesRelatedInstances
        ) SELECT ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?
        WHERE NOT EXISTS (
            SELECT 1 FROM SeriesEntity 
            WHERE tenant_id = ? AND SeriesInstanceUID = ?
        )",
        )
        .bind(&series_entity.tenant_id)
        .bind(&series_entity.series_instance_uid)
        .bind(&series_entity.study_instance_uid)
        .bind(&series_entity.modality)
        .bind(&series_entity.series_number)
        .bind(&series_entity.series_date)
        .bind(&series_entity.series_time)
        .bind(&series_entity.series_description)
        .bind(&series_entity.body_part_examined)
        .bind(&series_entity.protocol_name)
        .bind(&series_entity.image_type)
        .bind(&series_entity.acquisition_number)
        .bind(&series_entity.acquisition_time)
        .bind(&series_entity.acquisition_date)
        .bind(&series_entity.performing_physician_name)
        .bind(&series_entity.operators_name)
        .bind(&series_entity.number_of_series_related_instances)
        .bind(&series_entity.tenant_id) // 用于WHERE子句
        .bind(&series_entity.series_instance_uid) // 用于WHERE子句
        .execute(&mut *tx)
        .await;
        if let Err(e) = series_result {
            error!("Failed to save series info: {}", e);
            return None;
        }
        // 4. 保存图像信息
        let image_result = sqlx::query(
            r"INSERT INTO ImageEntity (
                    tenant_id, SOPInstanceUID, SeriesInstanceUID, StudyInstanceUID,
                    PatientID, InstanceNumber, ImageComments, ContentDate, ContentTime,
                    AcquisitionDateTime, ImageType, ImageOrientationPatient,
                    ImagePositionPatient, SliceThickness, SpacingBetweenSlices,
                    SliceLocation, SamplesPerPixel, PhotometricInterpretation,
                    Width, Columns, BitsAllocated, BitsStored, HighBit,
                    PixelRepresentation, RescaleIntercept, RescaleSlope,
                    RescaleType, AcquisitionDeviceProcessingDescription,
                    AcquisitionDeviceProcessingCode, DeviceSerialNumber,
                    SoftwareVersions, TransferSyntaxUID, SOPClassUID
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    InstanceNumber = VALUES(InstanceNumber),
                    ImageComments = VALUES(ImageComments),
                    ContentDate = VALUES(ContentDate),
                    ContentTime = VALUES(ContentTime),
                    AcquisitionDateTime = VALUES(AcquisitionDateTime),
                    ImageType = VALUES(ImageType),
                    ImageOrientationPatient = VALUES(ImageOrientationPatient),
                    ImagePositionPatient = VALUES(ImagePositionPatient),
                    SliceThickness = VALUES(SliceThickness),
                    SpacingBetweenSlices = VALUES(SpacingBetweenSlices),
                    SliceLocation = VALUES(SliceLocation),
                    SamplesPerPixel = VALUES(SamplesPerPixel),
                    PhotometricInterpretation = VALUES(PhotometricInterpretation),
                    Width = VALUES(Width),
                    Columns = VALUES(Columns),
                    BitsAllocated = VALUES(BitsAllocated),
                    BitsStored = VALUES(BitsStored),
                    HighBit = VALUES(HighBit),
                    PixelRepresentation = VALUES(PixelRepresentation),
                    RescaleIntercept = VALUES(RescaleIntercept),
                    RescaleSlope = VALUES(RescaleSlope),
                    RescaleType = VALUES(RescaleType),
                    AcquisitionDeviceProcessingDescription = VALUES(AcquisitionDeviceProcessingDescription),
                    AcquisitionDeviceProcessingCode = VALUES(AcquisitionDeviceProcessingCode),
                    DeviceSerialNumber = VALUES(DeviceSerialNumber),
                    SoftwareVersions = VALUES(SoftwareVersions),
                    TransferSyntaxUID = VALUES(TransferSyntaxUID)"
        )
            .bind(&image_entity.tenant_id)
            .bind(&image_entity.sop_instance_uid)
            .bind(&image_entity.series_instance_uid)
            .bind(&image_entity.study_instance_uid)
            .bind(&image_entity.patient_id)
            .bind(&image_entity.instance_number)
            .bind(&image_entity.image_comments)
            .bind(&image_entity.content_date)
            .bind(&image_entity.content_time)
            .bind(&image_entity.acquisition_date_time)
            .bind(&image_entity.image_type)
            .bind(&image_entity.image_orientation_patient)
            .bind(&image_entity.image_position_patient)
            .bind(&image_entity.slice_thickness)
            .bind(&image_entity.spacing_between_slices)
            .bind(&image_entity.slice_location)
            .bind(&image_entity.samples_per_pixel)
            .bind(&image_entity.photometric_interpretation)
            .bind(&image_entity.width)
            .bind(&image_entity.columns)
            .bind(&image_entity.bits_allocated)
            .bind(&image_entity.bits_stored)
            .bind(&image_entity.high_bit)
            .bind(&image_entity.pixel_representation)
            .bind(&image_entity.rescale_intercept)
            .bind(&image_entity.rescale_slope)
            .bind(&image_entity.rescale_type)
            .bind(&image_entity.acquisition_device_processing_description)
            .bind(&image_entity.acquisition_device_processing_code)
            .bind(&image_entity.device_serial_number)
            .bind(&image_entity.software_versions)
            .bind(&image_entity.transfer_syntax_uid)
            .bind(&image_entity.sop_class_uid)
            .execute(&mut *tx)
            .await;

        if let Err(e) = image_result {
            error!("Failed to save image info: {}", e);
            return None;
        }

        // 提交事务
        match tx.commit().await {
            Ok(_) => {
                info!(
                    "Successfully saved DICOM info for SOP Instance UID: {}",
                    image_entity.sop_instance_uid
                );
                Some(true)
            }
            Err(e) => {
                error!("Failed to commit transaction: {}", e);
                None
            }
        }
    }

    async fn save_patient_info(
        &self,
        tenant_id: &str,
        patient_lists: &[PatientEntity],
    ) -> Option<bool> {
        if patient_lists.is_empty() {
            return Some(true);
        }

        let pool = self.pool.clone();

        // 分批处理，避免SQL参数限制
        const BATCH_SIZE: usize = 100;
        for chunk in patient_lists.chunks(BATCH_SIZE) {
            // 构建批量插入语句，确保字段顺序与表结构完全一致
            let mut query_builder = "INSERT INTO PatientEntity (tenant_id, PatientID, PatientName, PatientBirthDate, PatientSex, PatientBirthTime, EthnicGroup) VALUES ".to_string();
            let placeholders: Vec<String> = (0..chunk.len())
                .map(|_| "(?, ?, ?, ?, ?, ?, ?)".to_string())
                .collect();
            query_builder.push_str(&placeholders.join(", "));
            query_builder.push_str(" ON DUPLICATE KEY UPDATE PatientName = VALUES(PatientName), PatientBirthDate = VALUES(PatientBirthDate), PatientSex = VALUES(PatientSex), PatientBirthTime = VALUES(PatientBirthTime), EthnicGroup = VALUES(EthnicGroup)");

            let mut query = sqlx::query(&query_builder);
            for patient in chunk {
                query = query
                    .bind(tenant_id)
                    .bind(&patient.patient_id)
                    .bind(&patient.patient_name)
                    .bind(&patient.patient_birth_date)
                    .bind(&patient.patient_sex)
                    .bind(&patient.patient_birth_time)
                    .bind(&patient.ethnic_group);
            }

            match query.execute(&pool).await {
                Ok(_) => continue,
                Err(e) => {
                    error!("Failed to save patient info: {}", e);
                    return None;
                }
            }
        }

        Some(true)
    }


    async fn save_study_info(&self, tenant_id: &str, study_lists: &[StudyEntity]) -> Option<bool> {
        if study_lists.is_empty() {
            return Some(true);
        }

        let pool = self.pool.clone();

        // 分批处理，避免SQL参数限制
        const BATCH_SIZE: usize = 100;
        for chunk in study_lists.chunks(BATCH_SIZE) {
            // 构建批量插入语句，确保字段顺序与表结构完全一致
            let mut query_builder = "INSERT INTO StudyEntity (tenant_id, StudyInstanceUID, PatientID, StudyDate, StudyTime, AccessionNumber, StudyID, StudyDescription, ReferringPhysicianName, PatientAge, PatientSize, PatientWeight, MedicalAlerts, Allergies, PregnancyStatus, Occupation, AdditionalPatientHistory, PatientComments, AdmissionID, PatientAgeAtStudy, PerformingPhysicianName, ProcedureCodeSequence) VALUES ".to_string();
            let placeholders: Vec<String> = (0..chunk.len())
                .map(|_| {
                    "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string()
                })
                .collect();
            query_builder.push_str(&placeholders.join(", "));
            query_builder.push_str(" ON DUPLICATE KEY UPDATE StudyDate = VALUES(StudyDate), StudyTime = VALUES(StudyTime), AccessionNumber = VALUES(AccessionNumber), StudyID = VALUES(StudyID), StudyDescription = VALUES(StudyDescription), ReferringPhysicianName = VALUES(ReferringPhysicianName), PatientAge = VALUES(PatientAge), PatientSize = VALUES(PatientSize), PatientWeight = VALUES(PatientWeight), MedicalAlerts = VALUES(MedicalAlerts), Allergies = VALUES(Allergies), PregnancyStatus = VALUES(PregnancyStatus), Occupation = VALUES(Occupation), AdditionalPatientHistory = VALUES(AdditionalPatientHistory), PatientComments = VALUES(PatientComments), AdmissionID = VALUES(AdmissionID), PatientAgeAtStudy = VALUES(PatientAgeAtStudy), PerformingPhysicianName = VALUES(PerformingPhysicianName), ProcedureCodeSequence = VALUES(ProcedureCodeSequence)");

            let mut query = sqlx::query(&query_builder);
            for study in chunk {
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
                    .bind(&study.patient_age_at_study)
                    .bind(&study.performing_physician_name)
                    .bind(&study.procedure_code_sequence);
            }

            match query.execute(&pool).await {
                Ok(_) => continue,
                Err(e) => {
                    error!("Failed to save study info: {}", e);
                    return None;
                }
            }
        }

        Some(true)
    }
    async fn save_series_info(
        &self,
        tenant_id: &str,
        series_lists: &[SeriesEntity],
    ) -> Option<bool> {
        if series_lists.is_empty() {
            return Some(true);
        }

        let pool = self.pool.clone();

        // 分批处理，避免SQL参数限制
        const BATCH_SIZE: usize = 100;
        for chunk in series_lists.chunks(BATCH_SIZE) {
            // 构建批量插入语句，确保字段顺序与表结构完全一致
            let mut query_builder = "INSERT INTO SeriesEntity (tenant_id, SeriesInstanceUID, StudyInstanceUID, Modality, SeriesNumber, SeriesDate, SeriesTime, SeriesDescription, BodyPartExamined, ProtocolName, ImageType, AcquisitionNumber, AcquisitionTime, AcquisitionDate, PerformingPhysicianName, OperatorsName, NumberOfSeriesRelatedInstances) VALUES ".to_string();
            let placeholders: Vec<String> = (0..chunk.len())
                .map(|_| "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string())
                .collect();
            query_builder.push_str(&placeholders.join(", "));
            query_builder.push_str(" ON DUPLICATE KEY UPDATE Modality = VALUES(Modality), SeriesNumber = VALUES(SeriesNumber), SeriesDate = VALUES(SeriesDate), SeriesTime = VALUES(SeriesTime), SeriesDescription = VALUES(SeriesDescription), BodyPartExamined = VALUES(BodyPartExamined), ProtocolName = VALUES(ProtocolName), ImageType = VALUES(ImageType), AcquisitionNumber = VALUES(AcquisitionNumber), AcquisitionTime = VALUES(AcquisitionTime), AcquisitionDate = VALUES(AcquisitionDate), PerformingPhysicianName = VALUES(PerformingPhysicianName), OperatorsName = VALUES(OperatorsName), NumberOfSeriesRelatedInstances = VALUES(NumberOfSeriesRelatedInstances)");

            let mut query = sqlx::query(&query_builder);
            for series in chunk {
                query = query
                    .bind(tenant_id)
                    .bind(&series.series_instance_uid)
                    .bind(&series.study_instance_uid)
                    .bind(&series.modality)
                    .bind(&series.series_number)
                    .bind(&series.series_date)
                    .bind(&series.series_time)
                    .bind(&series.series_description)
                    .bind(&series.body_part_examined)
                    .bind(&series.protocol_name)
                    .bind(&series.image_type)
                    .bind(&series.acquisition_number)
                    .bind(&series.acquisition_time)
                    .bind(&series.acquisition_date)
                    .bind(&series.performing_physician_name)
                    .bind(&series.operators_name)
                    .bind(&series.number_of_series_related_instances);
            }

            match query.execute(&pool).await {
                Ok(_) => continue,
                Err(e) => {
                    error!("Failed to save series info: {}", e);
                    return None;
                }
            }
        }

        Some(true)
    }
    async fn save_instance_info(&self, tenant_id: &str, dicom_obj: &[ImageEntity]) -> Option<bool> {
        if dicom_obj.is_empty() {
            return Some(true);
        }

        let pool = self.pool.clone();

        // 分批处理，避免SQL参数限制
        const BATCH_SIZE: usize = 100;
        for (batch_index, chunk) in dicom_obj.chunks(BATCH_SIZE).enumerate() {
            // 构建批量插入语句，确保字段顺序与表结构完全一致
            let mut query_builder = "INSERT INTO ImageEntity (tenant_id, SOPInstanceUID, SeriesInstanceUID, StudyInstanceUID, PatientID, InstanceNumber, ImageComments, ContentDate, ContentTime, AcquisitionDateTime, ImageType, ImageOrientationPatient, ImagePositionPatient, SliceThickness, SpacingBetweenSlices, SliceLocation, SamplesPerPixel, PhotometricInterpretation, Width, Columns, BitsAllocated, BitsStored, HighBit, PixelRepresentation, RescaleIntercept, RescaleSlope, RescaleType, AcquisitionDeviceProcessingDescription, AcquisitionDeviceProcessingCode, DeviceSerialNumber, SoftwareVersions, TransferSyntaxUID, SOPClassUID) VALUES ".to_string();
            let placeholders: Vec<String> = (0..chunk.len()).map(|_| "(?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)".to_string()).collect();
            query_builder.push_str(&placeholders.join(", "));
            query_builder.push_str(" ON DUPLICATE KEY UPDATE InstanceNumber = VALUES(InstanceNumber), ImageComments = VALUES(ImageComments), ContentDate = VALUES(ContentDate), ContentTime = VALUES(ContentTime), AcquisitionDateTime = VALUES(AcquisitionDateTime), ImageType = VALUES(ImageType), ImageOrientationPatient = VALUES(ImageOrientationPatient), ImagePositionPatient = VALUES(ImagePositionPatient), SliceThickness = VALUES(SliceThickness), SpacingBetweenSlices = VALUES(SpacingBetweenSlices), SliceLocation = VALUES(SliceLocation), SamplesPerPixel = VALUES(SamplesPerPixel), PhotometricInterpretation = VALUES(PhotometricInterpretation), Width = VALUES(Width), Columns = VALUES(Columns), BitsAllocated = VALUES(BitsAllocated), BitsStored = VALUES(BitsStored), HighBit = VALUES(HighBit), PixelRepresentation = VALUES(PixelRepresentation), RescaleIntercept = VALUES(RescaleIntercept), RescaleSlope = VALUES(RescaleSlope), RescaleType = VALUES(RescaleType), AcquisitionDeviceProcessingDescription = VALUES(AcquisitionDeviceProcessingDescription), AcquisitionDeviceProcessingCode = VALUES(AcquisitionDeviceProcessingCode), DeviceSerialNumber = VALUES(DeviceSerialNumber), SoftwareVersions = VALUES(SoftwareVersions), TransferSyntaxUID = VALUES(TransferSyntaxUID)");

            let mut query = sqlx::query(&query_builder);

            for image in chunk {
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
                    .bind(&image.acquisition_device_processing_description)
                    .bind(&image.acquisition_device_processing_code)
                    .bind(&image.device_serial_number)
                    .bind(&image.software_versions)
                    .bind(&image.transfer_syntax_uid)
                    .bind(&image.sop_class_uid);
            }

            match query.execute(&pool).await {
                Ok(_) => {
                    continue;
                }
                Err(e) => {
                    error!(
                        "Failed to save instance info in batch {}: {}",
                        batch_index + 1,
                        e
                    );
                    return None;
                }
            }
        }

        Some(true)
    }

    async fn delete_study_info(&self, tenant_id: &str, study_uid: &str) -> Option<bool> {
        let tenant_id = tenant_id.to_string();
        let study_uid = study_uid.to_string();
        let pool = self.pool.clone();

        match sqlx::query("DELETE FROM StudyEntity WHERE tenant_id = ? AND StudyInstanceUID = ?")
            .bind(&tenant_id)
            .bind(&study_uid)
            .execute(&pool)
            .await
        {
            Ok(_) => Some(true),
            Err(e) => {
                error!("Failed to delete study info: {}", e);
                None
            }
        }
    }

    async fn delete_series_info(
        &self,
        tenant_id: &str,
        study_uid: &str,
        series_uid: &str,
    ) -> Option<bool> {
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
            Ok(_) => Some(true),
            Err(e) => {
                error!("Failed to delete series info: {}", e);
                None
            }
        }
    }

    async fn delete_instance_info(
        &self,
        tenant_id: &str,
        study_uid: &str,
        series_uid: &str,
        instance_uid: &str,
    ) -> Option<bool> {
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
            Ok(_) => Some(true),
            Err(e) => {
                error!("Failed to delete instance info: {}", e);
                None
            }
        }
    }

    async fn patient_exists(&self, tenant_id: &str, patient_id: &str) -> Option<bool> {
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
                Some(count > 0)
            }
            Err(e) => {
                error!("Failed to check if patient exists: {}", e);
                None
            }
        }
    }

    async fn patient_study_exists(
        &self,
        tenant_id: &str,
        patient_id: &str,
        study_uid: &str,
    ) -> Option<bool> {
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
                Some(count > 0)
            }
            Err(e) => {
                error!("Failed to check if patient study exists: {}", e);
                None
            }
        }
    }
    async fn patient_series_exists(
        &self,
        tenant_id: &str,
        patient_id: &str,
        study_uid: &str,
        series_uid: &str,
    ) -> Option<bool> {
        let pool = self.pool.clone();

        match sqlx::query(
            "SELECT COUNT(*) FROM SeriesEntity
         INNER JOIN StudyEntity ON SeriesEntity.StudyInstanceUID = StudyEntity.StudyInstanceUID
         WHERE SeriesEntity.tenant_id = ? AND StudyEntity.PatientID = ? AND SeriesEntity.StudyInstanceUID = ? AND SeriesEntity.SeriesInstanceUID = ?"
        )
            .bind(tenant_id)
            .bind(patient_id)
            .bind(study_uid)
            .bind(series_uid)
            .fetch_one(&pool)
            .await
        {
            Ok(row) => {
                let count: i64 = row.get(0);
                Some(count > 0)
            }
            Err(e) => {
                error!("Failed to check if patient series exists: {}", e);
                None
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
    ) -> Option<bool> {
        let pool = self.pool.clone();

        match sqlx::query(
            "SELECT COUNT(*) FROM ImageEntity
         INNER JOIN StudyEntity ON ImageEntity.StudyInstanceUID = StudyEntity.StudyInstanceUID
         WHERE ImageEntity.tenant_id = ? AND StudyEntity.PatientID = ? AND ImageEntity.StudyInstanceUID = ? AND ImageEntity.SeriesInstanceUID = ? AND ImageEntity.SOPInstanceUID = ?"
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
                Some(count > 0)
            }
            Err(e) => {
                error!("Failed to check if patient instance exists: {}", e);
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::file_utils;
    use dicom_dictionary_std::tags;
    use sqlx::MySqlPool;
    use std::collections::HashMap;
    use std::path::Path;

    // 测试数据库连接配置 - 使用测试数据库
    const TEST_DATABASE_URL: &str = "mysql://dicomstore:hzjp%23123@192.168.1.14:3306/dicomdb";

    // 设置测试数据库
    async fn setup_test_database() -> MySqlPool {
        let pool = MySqlPool::connect(TEST_DATABASE_URL).await.unwrap();
        pool
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
        let pool = setup_test_database().await;
        let provider = MySqlProvider { pool: pool.clone() };

        let dir_path = "/home/dhz/jpdata/CDSS/89269";
        let dicom_files = file_utils::get_dicom_files_in_dir(dir_path).await;
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
                    .read_until(tags::PIXEL_DATA)
                    .open_file(path)
                    .map_err(Box::from);
            match dicom_obj {
                Ok(dcmobj) => {
                    let patient_entity =
                        crate::entities::DbProviderBase::extract_patient_entity(tenant_id, &dcmobj);
                    // 修复：正确检查 patient_name 是否存在
                    let patient_id = patient_entity.patient_id.clone();
                    if !patient_list.contains_key(&patient_entity.patient_id) {
                        patient_list.insert(patient_id.clone(), patient_entity);
                    }
                    let study_entity = crate::entities::DbProviderBase::extract_study_entity(
                        tenant_id,
                        &dcmobj,
                        &patient_id, // 使用 clone 后的值，避免 move
                    );
                    let study_uid = study_entity.study_instance_uid.clone();
                    if !study_list.contains_key(study_uid.as_str()) {
                        study_list.insert(study_uid.clone(), study_entity);
                    }
                    let series_entity = crate::entities::DbProviderBase::extract_series_entity(
                        tenant_id,
                        &dcmobj,
                        study_uid.as_str(), // 使用 clone 后的值，避免 move
                    );
                    let series_id = series_entity.series_instance_uid.clone();
                    if !series_list.contains_key(series_id.as_str()) {
                        series_list.insert(series_id.clone(), series_entity);
                    }
                    let image_entity = crate::entities::DbProviderBase::extract_image_entity(
                        tenant_id,
                        &dcmobj,
                        series_id.as_str(), // 使用 clone 后的值，避免 move
                        study_uid.as_str(), // 使用 clone 后的值，避免 move
                        &patient_id,        // 使用 clone 后的值，避免 move
                    );
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
                Some(true) => println!("成功保存 {} 条患者数据", patient_list.len()),
                Some(false) => println!("患者数据已存在"),
                None => println!("保存患者数据失败"),
            }
        }

        if !study_list.is_empty() {
            match provider
                .save_study_info(tenant_id, &study_list.values().cloned().collect::<Vec<_>>())
                .await
            {
                Some(true) => println!("成功保存 {} 条检查数据", study_list.len()),
                Some(false) => println!("检查数据已存在"),
                None => println!("保存检查数据失败"),
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
                Some(true) => println!("成功保存 {} 条序列数据", series_list.len()),
                Some(false) => println!("序列数据已存在"),
                None => println!("保存序列数据失败"),
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
                Some(true) => println!("成功保存 {} 条图像数据", images_list.len()),
                Some(false) => println!("图像数据已存在"),
                None => {
                    println!("保存图像数据失败");
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
