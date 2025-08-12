use crate::db_provider::DbProvider;
use crate::entities::DbProviderBase;
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
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    PatientName = VALUES(PatientName),
                    PatientBirthDate = VALUES(PatientBirthDate),
                    PatientSex = VALUES(PatientSex),
                    PatientBirthTime = VALUES(PatientBirthTime),
                    EthnicGroup = VALUES(EthnicGroup)",
        )
        .bind(&patient_entity.tenant_id)
        .bind(&patient_entity.patient_id)
        .bind(&patient_entity.patient_name)
        .bind(&patient_entity.patient_birth_date)
        .bind(&patient_entity.patient_sex)
        .bind(&patient_entity.patient_birth_time)
        .bind(&patient_entity.ethnic_group)
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
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    StudyDate = VALUES(StudyDate),
                    StudyTime = VALUES(StudyTime),
                    AccessionNumber = VALUES(AccessionNumber),
                    StudyID = VALUES(StudyID),
                    StudyDescription = VALUES(StudyDescription),
                    ReferringPhysicianName = VALUES(ReferringPhysicianName),
                    PatientAge = VALUES(PatientAge),
                    PatientSize = VALUES(PatientSize),
                    PatientWeight = VALUES(PatientWeight),
                    MedicalAlerts = VALUES(MedicalAlerts),
                    Allergies = VALUES(Allergies),
                    PregnancyStatus = VALUES(PregnancyStatus),
                    Occupation = VALUES(Occupation),
                    AdditionalPatientHistory = VALUES(AdditionalPatientHistory),
                    PatientComments = VALUES(PatientComments),
                    AdmissionID = VALUES(AdmissionID),
                    PatientAgeAtStudy = VALUES(PatientAgeAtStudy),
                    PerformingPhysicianName = VALUES(PerformingPhysicianName),
                    ProcedureCodeSequence = VALUES(ProcedureCodeSequence)",
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
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    Modality = VALUES(Modality),
                    SeriesNumber = VALUES(SeriesNumber),
                    SeriesDate = VALUES(SeriesDate),
                    SeriesTime = VALUES(SeriesTime),
                    SeriesDescription = VALUES(SeriesDescription),
                    BodyPartExamined = VALUES(BodyPartExamined),
                    ProtocolName = VALUES(ProtocolName),
                    ImageType = VALUES(ImageType),
                    AcquisitionNumber = VALUES(AcquisitionNumber),
                    AcquisitionTime = VALUES(AcquisitionTime),
                    AcquisitionDate = VALUES(AcquisitionDate),
                    PerformingPhysicianName = VALUES(PerformingPhysicianName),
                    OperatorsName = VALUES(OperatorsName),
                    NumberOfSeriesRelatedInstances = VALUES(NumberOfSeriesRelatedInstances)",
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
    use dicom_core::{DataElement, PrimitiveValue, VR};
    use dicom_dictionary_std::tags;
    use dicom_object::InMemDicomObject;
    use sqlx::MySqlPool;
    use std::path::Path;

    // 测试数据库连接配置 - 使用测试数据库
    const TEST_DATABASE_URL: &str = "mysql://dicomstore:hzjp%23123@192.168.1.14:3306/dicomdb";

    // 设置测试数据库
    async fn setup_test_database() -> MySqlPool {
        let pool = MySqlPool::connect(TEST_DATABASE_URL).await.unwrap();
        pool
    }

    // 创建测试用的 DICOM 对象
    fn create_test_dicom_object() -> InMemDicomObject {
        let mut obj = InMemDicomObject::new_empty();

        // 添加基本的 DICOM 元素用于测试
        obj.put(DataElement::new(
            tags::PATIENT_ID,
            VR::LO,
            PrimitiveValue::from("TEST_PATIENT_001"),
        ));

        obj.put(DataElement::new(
            tags::PATIENT_NAME,
            VR::PN,
            PrimitiveValue::from("Doe^John"),
        ));

        obj.put(DataElement::new(
            tags::PATIENT_BIRTH_DATE,
            VR::DA,
            PrimitiveValue::from("19800101"),
        ));

        obj.put(DataElement::new(
            tags::PATIENT_SEX,
            VR::CS,
            PrimitiveValue::from("M"),
        ));

        obj.put(DataElement::new(
            tags::STUDY_INSTANCE_UID,
            VR::UI,
            PrimitiveValue::from("1.2.3.4.5.6.7.8.9.10"),
        ));

        obj.put(DataElement::new(
            tags::STUDY_DATE,
            VR::DA,
            PrimitiveValue::from("20230101"),
        ));

        obj.put(DataElement::new(
            tags::STUDY_TIME,
            VR::TM,
            PrimitiveValue::from("120000"),
        ));

        obj.put(DataElement::new(
            tags::STUDY_DESCRIPTION,
            VR::LO,
            PrimitiveValue::from("CT Chest"),
        ));

        obj.put(DataElement::new(
            tags::SERIES_INSTANCE_UID,
            VR::UI,
            PrimitiveValue::from("1.2.3.4.5.6.7.8.9.11"),
        ));

        obj.put(DataElement::new(
            tags::MODALITY,
            VR::CS,
            PrimitiveValue::from("CT"),
        ));

        obj.put(DataElement::new(
            tags::SERIES_NUMBER,
            VR::IS,
            PrimitiveValue::from(1),
        ));

        obj.put(DataElement::new(
            tags::SOP_INSTANCE_UID,
            VR::UI,
            PrimitiveValue::from("1.2.3.4.5.6.7.8.9.12"),
        ));

        obj.put(DataElement::new(
            tags::SOP_CLASS_UID,
            VR::UI,
            PrimitiveValue::from("1.2.840.10008.5.1.4.1.1.2"), // CT Image Storage
        ));

        obj.put(DataElement::new(
            tags::INSTANCE_NUMBER,
            VR::IS,
            PrimitiveValue::from(1),
        ));

        obj.put(DataElement::new(
            tags::ROWS,
            VR::US,
            PrimitiveValue::from(512),
        ));

        obj.put(DataElement::new(
            tags::COLUMNS,
            VR::US,
            PrimitiveValue::from(512),
        ));

        obj
    }

    #[tokio::test]
    async fn test_save_dicom_info_success() {
        let pool = setup_test_database().await;
        let provider = MySqlProvider { pool: pool.clone() };

        let dir_path = "./testdata";
        let dicom_files = file_utils::get_dicom_files_in_dir(dir_path).await;
        // 需要处理 Result 类型
        let dicom_files = match dicom_files {
            Ok(files) => files,
            Err(e) => {
                eprintln!("Failed to get DICOM files: {}", e);
                return;
            }
        };
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
                    provider.save_dicom_info("1234567890", &dcmobj).await;
                }
                Err(e) => {
                    eprintln!("Failed to open DICOM file: {}", e);
                }
            }
        }
    }
}
