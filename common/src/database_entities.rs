use dicom_core::chrono::{NaiveDateTime};
use crate::dicom_utils;
use dicom_dictionary_std::tags;
use dicom_object::InMemDicomObject;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// patient.rs
#[derive(Debug, Clone, Serialize, Deserialize,FromRow)]
pub struct PatientEntity {
    pub tenant_id: String,
    pub patient_id: String,
    pub patient_name: Option<String>,
    pub patient_birth_date: Option<String>,
    pub patient_sex: Option<String>,
    pub patient_birth_time: Option<String>,
    pub ethnic_group: Option<String>,
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
}

// study.rs
#[derive(Debug, Clone, Serialize, Deserialize,FromRow)]
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
    pub patient_age_at_study: Option<String>,
    pub performing_physician_name: Option<String>,
    pub procedure_code_sequence: Option<String>,
    pub received_instances: Option<i32>, // 新增字段
    pub space_size: Option<i64>,         // 新增字段
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
}

// series.rs
#[derive(Debug, Clone, Serialize, Deserialize,FromRow)]
pub struct SeriesEntity {
    pub tenant_id: String,
    pub series_instance_uid: String,
    pub study_instance_uid: String,
    pub modality: String,
    pub series_number: Option<i32>,
    pub series_date: Option<dicom_core::chrono::NaiveDate>,
    pub series_time: Option<dicom_core::chrono::NaiveTime>,
    pub series_description: Option<String>,
    pub body_part_examined: Option<String>,
    pub protocol_name: Option<String>,
    pub image_type: Option<String>,
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
#[derive(Debug, Clone, Serialize, Deserialize,FromRow)]
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
    pub acquisition_date_time: Option<String>,
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
    pub file_path: String,
    pub tenant_id: String,
    pub transfer_synatx_uid: String,
    pub number_of_frames: i32,
}
pub struct DbProviderBase {}
impl DbProviderBase {
    pub fn extract_patient_entity(tenant_id: &str, dicom_obj: &InMemDicomObject) -> PatientEntity {
        PatientEntity {
            tenant_id: tenant_id.to_string(),
            patient_id: dicom_utils::get_text_value(dicom_obj, tags::PATIENT_ID)
                .unwrap_or_default(),
            patient_name: dicom_utils::get_text_value(dicom_obj, tags::PATIENT_NAME),
            patient_birth_date: dicom_utils::get_date_value(dicom_obj, tags::PATIENT_BIRTH_DATE),
            patient_sex: dicom_utils::get_text_value(dicom_obj, tags::PATIENT_SEX),
            patient_birth_time: dicom_utils::get_time_value(dicom_obj, tags::PATIENT_BIRTH_TIME),
            ethnic_group: dicom_utils::get_text_value(dicom_obj, tags::ETHNIC_GROUP),
            created_time: None,
            updated_time: None,
        }
    }

    pub fn extract_study_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
        patient_id: &str,
    ) -> StudyEntity {
        StudyEntity {
            tenant_id: tenant_id.to_string(),
            study_instance_uid: dicom_utils::get_text_value(dicom_obj, tags::STUDY_INSTANCE_UID)
                .unwrap_or_default(),
            patient_id: patient_id.to_string(),
            patient_age: dicom_utils::get_text_value(dicom_obj, tags::PATIENT_AGE),
            patient_size: dicom_utils::get_decimal_value(dicom_obj, tags::PATIENT_SIZE),
            patient_weight: dicom_utils::get_decimal_value(dicom_obj, tags::PATIENT_WEIGHT),
            medical_alerts: dicom_utils::get_text_value(dicom_obj, tags::MEDICAL_ALERTS),
            allergies: dicom_utils::get_text_value(dicom_obj, tags::ALLERGIES),
            pregnancy_status: dicom_utils::get_int_value(dicom_obj, tags::PREGNANCY_STATUS),
            occupation: dicom_utils::get_text_value(dicom_obj, tags::OCCUPATION),
            additional_patient_history: dicom_utils::get_text_value(
                dicom_obj,
                tags::ADDITIONAL_PATIENT_HISTORY,
            ),
            patient_comments: dicom_utils::get_text_value(dicom_obj, tags::PATIENT_COMMENTS),
            study_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::STUDY_DATE),
            study_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::STUDY_TIME),
            accession_number: dicom_utils::get_text_value(dicom_obj, tags::ACCESSION_NUMBER),
            study_id: dicom_utils::get_text_value(dicom_obj, tags::STUDY_ID),
            study_description: dicom_utils::get_text_value(dicom_obj, tags::STUDY_DESCRIPTION),
            referring_physician_name: dicom_utils::get_text_value(
                dicom_obj,
                tags::REFERRING_PHYSICIAN_NAME,
            ),
            admission_id: dicom_utils::get_text_value(dicom_obj, tags::ADMISSION_ID),
            patient_age_at_study: dicom_utils::get_text_value(dicom_obj, tags::PATIENT_AGE),
            performing_physician_name: dicom_utils::get_text_value(
                dicom_obj,
                tags::PERFORMING_PHYSICIAN_NAME,
            ),

            received_instances: Some(0),
            space_size: Some(0),
            procedure_code_sequence: dicom_utils::get_text_value(
                dicom_obj,
                tags::PROCEDURE_CODE_SEQUENCE,
            ),
            created_time: None,
            updated_time: None,
        }
    }

    pub fn extract_series_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
        study_uid: &str,
    ) -> SeriesEntity {
        SeriesEntity {
            tenant_id: tenant_id.to_string(),
            series_instance_uid: dicom_utils::get_text_value(dicom_obj, tags::SERIES_INSTANCE_UID)
                .unwrap_or_default(),
            study_instance_uid: study_uid.to_string(),
            modality: dicom_utils::get_text_value(dicom_obj, tags::MODALITY).unwrap_or_default(),
            series_number: dicom_utils::get_int_value(dicom_obj, tags::SERIES_NUMBER),
            series_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::SERIES_DATE),
            series_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::SERIES_TIME),
            series_description: dicom_utils::get_text_value(dicom_obj, tags::SERIES_DESCRIPTION),
            body_part_examined: dicom_utils::get_text_value(dicom_obj, tags::BODY_PART_EXAMINED),
            protocol_name: dicom_utils::get_text_value(dicom_obj, tags::PROTOCOL_NAME),
            image_type: dicom_utils::get_text_value(dicom_obj, tags::IMAGE_TYPE),
            acquisition_number: dicom_utils::get_int_value(dicom_obj, tags::ACQUISITION_NUMBER),
            acquisition_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::ACQUISITION_TIME),
            acquisition_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::ACQUISITION_DATE),
            performing_physician_name: dicom_utils::get_text_value(
                dicom_obj,
                tags::PERFORMING_PHYSICIAN_NAME,
            ),
            operators_name: dicom_utils::get_text_value(dicom_obj, tags::OPERATORS_NAME),
            number_of_series_related_instances: dicom_utils::get_int_value(
                dicom_obj,
                tags::NUMBER_OF_SERIES_RELATED_INSTANCES,
            ),
            received_instances: Some(0),
            space_size: Some(0),
            created_time: None,
            updated_time: None,
        }
    }

    pub fn extract_image_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
        study_uid: &str,
        series_uid: &str,
        patient_id: &str,
    ) -> ImageEntity {
        let acquisition_date_time =
            dicom_utils::get_text_value(dicom_obj, tags::ACQUISITION_DATE_TIME);
        let acquisition_date_time_parsed =
            acquisition_date_time.and_then(|dt| if !dt.is_empty() { Some(dt) } else { None });
        let img_number_of_frames = dicom_utils::get_tag_value(tags::NUMBER_OF_FRAMES, dicom_obj, 1);
        ImageEntity {
            tenant_id: tenant_id.to_string(),
            sop_instance_uid: dicom_utils::get_text_value(dicom_obj, tags::SOP_INSTANCE_UID)
                .unwrap_or_default(),
            series_instance_uid: series_uid.to_string(),
            study_instance_uid: study_uid.to_string(),
            patient_id: patient_id.to_string(),
            instance_number: dicom_utils::get_int_value(dicom_obj, tags::INSTANCE_NUMBER),
            image_comments: dicom_utils::get_text_value(dicom_obj, tags::IMAGE_COMMENTS),
            content_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::CONTENT_DATE),
            content_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::CONTENT_TIME),
            acquisition_date_time: acquisition_date_time_parsed,
            image_type: dicom_utils::get_text_value(dicom_obj, tags::IMAGE_TYPE),
            image_orientation_patient: dicom_utils::get_text_value(
                dicom_obj,
                tags::IMAGE_ORIENTATION_PATIENT,
            ),
            image_position_patient: dicom_utils::get_text_value(
                dicom_obj,
                tags::IMAGE_POSITION_PATIENT,
            ),
            slice_thickness: dicom_utils::get_decimal_value(dicom_obj, tags::SLICE_THICKNESS),
            spacing_between_slices: dicom_utils::get_decimal_value(
                dicom_obj,
                tags::SPACING_BETWEEN_SLICES,
            ),
            slice_location: dicom_utils::get_decimal_value(dicom_obj, tags::SLICE_LOCATION),
            samples_per_pixel: dicom_utils::get_int_value(dicom_obj, tags::SAMPLES_PER_PIXEL),
            photometric_interpretation: dicom_utils::get_text_value(
                dicom_obj,
                tags::PHOTOMETRIC_INTERPRETATION,
            ),
            width: dicom_utils::get_int_value(dicom_obj, tags::ROWS),
            columns: dicom_utils::get_int_value(dicom_obj, tags::COLUMNS),
            bits_allocated: dicom_utils::get_int_value(dicom_obj, tags::BITS_ALLOCATED),
            bits_stored: dicom_utils::get_int_value(dicom_obj, tags::BITS_STORED),
            high_bit: dicom_utils::get_int_value(dicom_obj, tags::HIGH_BIT),
            pixel_representation: dicom_utils::get_int_value(dicom_obj, tags::PIXEL_REPRESENTATION),
            rescale_intercept: dicom_utils::get_decimal_value(dicom_obj, tags::RESCALE_INTERCEPT),
            rescale_slope: dicom_utils::get_decimal_value(dicom_obj, tags::RESCALE_SLOPE),
            rescale_type: dicom_utils::get_text_value(dicom_obj, tags::RESCALE_TYPE),
            number_of_frames:  img_number_of_frames,
            acquisition_device_processing_description: dicom_utils::get_text_value(
                dicom_obj,
                tags::ACQUISITION_DEVICE_PROCESSING_DESCRIPTION,
            ),
            acquisition_device_processing_code: dicom_utils::get_text_value(
                dicom_obj,
                tags::ACQUISITION_DEVICE_PROCESSING_CODE,
            ),
            device_serial_number: dicom_utils::get_text_value(
                dicom_obj,
                tags::DEVICE_SERIAL_NUMBER,
            ),
            software_versions: dicom_utils::get_text_value(dicom_obj, tags::SOFTWARE_VERSIONS),
            transfer_syntax_uid: dicom_utils::get_text_value(dicom_obj, tags::TRANSFER_SYNTAX_UID).unwrap_or_default(),
            pixel_data_location: None,
            thumbnail_location: None,
            sop_class_uid: dicom_utils::get_text_value(dicom_obj, tags::SOP_CLASS_UID)
                .unwrap_or_default(),
            image_status: Some("ACTIVE".to_string()),
            space_size: None,
            created_time: None,
            updated_time: None,
        }
    }
}
