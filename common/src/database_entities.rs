use std::path::PathBuf;
use dicom_core::chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

// patient.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatientEntity {
    pub tenant_id: String,
    pub patient_id: String,
    pub patient_name: Option<String>,
    pub patient_sex: Option<String>,
    pub patient_birth_date: Option<dicom_core::chrono::NaiveDate>,
    pub patient_birth_time: Option<dicom_core::chrono::NaiveTime>,
    pub ethnic_group: Option<String>,
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
}

// study.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyEntity {
    pub tenant_id: String,
    pub study_instance_uid: String,
    pub patient_id: String,
    pub patient_age: Option<String>,
    pub patient_size: Option<f64>,
    pub patient_weight: Option<f64>,
    pub medical_alerts: Option<String>,
    pub allergies: Option<String>,
    pub pregnancy_status: Option<i32>,
    pub occupation: Option<String>,
    pub additional_patient_history: Option<String>,
    pub patient_comments: Option<String>,
    pub study_date: Option<dicom_core::chrono::NaiveDate>,
    pub study_time: Option<dicom_core::chrono::NaiveTime>,
    pub accession_number: Option<String>,
    pub study_id: Option<String>,
    pub study_description: Option<String>,
    pub referring_physician_name: Option<String>,
    pub admission_id: Option<String>,
    pub performing_physician_name: Option<String>,
    pub procedure_code_sequence: Option<String>,
    pub received_instances: Option<i32>, // 新增字段
    pub space_size: Option<i64>,         // 新增字段
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
}

// series.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesEntity {
    pub tenant_id: String,
    pub series_instance_uid: String,
    pub study_instance_uid: String,
    pub patient_id: String,
    pub modality: String,
    pub series_number: Option<i32>,
    pub series_date: Option<dicom_core::chrono::NaiveDate>,
    pub series_time: Option<dicom_core::chrono::NaiveTime>,
    pub series_description: Option<String>,
    pub body_part_examined: Option<String>,
    pub protocol_name: Option<String>, 
    pub acquisition_number: Option<i32>,
    pub acquisition_time: Option<dicom_core::chrono::NaiveTime>,
    pub acquisition_date: Option<dicom_core::chrono::NaiveDate>,
    pub performing_physician_name: Option<String>,
    pub operators_name: Option<String>,
    pub number_of_series_related_instances: Option<i32>,
    pub received_instances: Option<u32>, // 新增字段
    pub space_size: Option<u64>,         // 新增字段
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
}

// image.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEntity {
    pub tenant_id: String,
    pub sop_instance_uid: String,
    pub series_instance_uid: String,
    pub study_instance_uid: String,
    pub patient_id: String,
    pub instance_number: Option<i32>,
    pub image_comments: Option<String>,
    pub content_date: Option<dicom_core::chrono::NaiveDate>,
    pub content_time: Option<dicom_core::chrono::NaiveTime>,
    pub acquisition_date: Option<dicom_core::chrono::NaiveDate>,
    pub acquisition_time: Option<dicom_core::chrono::NaiveTime>,
    pub acquisition_date_time: Option<dicom_core::chrono::NaiveDateTime>,
    pub image_type: Option<String>,
    pub image_orientation_patient: Option<String>,
    pub image_position_patient: Option<String>,
    pub slice_thickness: Option<f64>,
    pub spacing_between_slices: Option<f64>,
    pub slice_location: Option<f64>,
    pub samples_per_pixel: Option<i32>,
    pub photometric_interpretation: Option<String>,
    pub width: Option<i32>,
    pub columns: Option<i32>,
    pub bits_allocated: Option<i32>,
    pub bits_stored: Option<i32>,
    pub high_bit: Option<i32>,
    pub pixel_representation: Option<i32>,
    pub rescale_intercept: Option<f64>,
    pub rescale_slope: Option<f64>,
    pub rescale_type: Option<String>,
    pub number_of_frames: i32,
    pub acquisition_device_processing_description: Option<String>,
    pub acquisition_device_processing_code: Option<String>,
    pub device_serial_number: Option<String>,
    pub software_versions: Option<String>,
    pub transfer_syntax_uid: String,
    pub pixel_data_location: Option<String>,
    pub thumbnail_location: Option<String>,
    pub sop_class_uid: String,
    pub image_status: Option<String>,
    pub space_size: Option<u64>, // 新增字段
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomObjectMeta {
    pub patient_info: PatientEntity,
    pub study_info: StudyEntity,
    pub series_info: SeriesEntity,
    pub image_info: ImageEntity,
    pub file_size: u64,
    pub file_path: PathBuf,
    pub tenant_id: String,
    pub transfer_synatx_uid: String,
    pub number_of_frames: i32,
}
