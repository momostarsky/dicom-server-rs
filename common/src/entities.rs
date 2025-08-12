use dicom_core::Tag;
use dicom_dictionary_std::tags;
use dicom_object::InMemDicomObject;

// patient.rs
#[derive(Debug, Clone)]
pub struct PatientEntity {
    pub tenant_id: String,
    pub patient_id: String,
    pub patient_name: Option<String>,
    pub patient_birth_date: Option<String>,
    pub patient_sex: Option<String>,
    pub patient_birth_time: Option<String>,
    pub ethnic_group: Option<String>,
    pub created_time: Option<String>,
    pub updated_time: Option<String>,
}

// study.rs
#[derive(Debug, Clone)]
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
    pub study_date: Option<String>,
    pub study_time: Option<String>,
    pub accession_number: Option<String>,
    pub study_id: Option<String>,
    pub study_description: Option<String>,
    pub referring_physician_name: Option<String>,
    pub admission_id: Option<String>,
    pub patient_age_at_study: Option<String>,
    pub performing_physician_name: Option<String>,
    pub procedure_code_sequence: Option<String>,
    pub created_time: Option<String>,
    pub updated_time: Option<String>,
}

// series.rs
#[derive(Debug, Clone)]
pub struct SeriesEntity {
    pub tenant_id: String,
    pub series_instance_uid: String,
    pub study_instance_uid: String,
    pub modality: String,
    pub series_number: Option<i32>,
    pub series_date: Option<String>,
    pub series_time: Option<String>,
    pub series_description: Option<String>,
    pub body_part_examined: Option<String>,
    pub protocol_name: Option<String>,
    pub image_type: Option<String>,
    pub acquisition_number: Option<i32>,
    pub acquisition_time: Option<String>,
    pub acquisition_date: Option<String>,
    pub performing_physician_name: Option<String>,
    pub operators_name: Option<String>,
    pub number_of_series_related_instances: Option<i32>,
    pub created_time: Option<String>,
    pub updated_time: Option<String>,
}

// image.rs
#[derive(Debug, Clone)]
pub struct ImageEntity {
    pub tenant_id: String,
    pub sop_instance_uid: String,
    pub series_instance_uid: String,
    pub study_instance_uid: String,
    pub patient_id: String,
    pub instance_number: Option<i32>,
    pub image_comments: Option<String>,
    pub content_date: Option<String>,
    pub content_time: Option<String>,
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
    pub acquisition_device_processing_description: Option<String>,
    pub acquisition_device_processing_code: Option<String>,
    pub device_serial_number: Option<String>,
    pub software_versions: Option<String>,
    pub transfer_syntax_uid: Option<String>,
    pub pixel_data_location: Option<String>,
    pub thumbnail_location: Option<String>,
    pub sop_class_uid: String,
    pub image_status: Option<String>,
    pub created_time: Option<String>,
    pub updated_time: Option<String>,
}
pub struct DbProviderBase {}
impl DbProviderBase {
    fn get_text_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<String> {
        dicom_obj
            .element(tag)
            .ok()
            .and_then(|e| e.to_str().ok())
            .map(|s| s.trim_end_matches('\0').to_string())
    }

    fn get_date_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<String> {
        Self::get_text_value(dicom_obj, tag).and_then(|s| {
            // 尝试解析DICOM日期格式 (YYYYMMDD)
            if s.len() == 8 && s.chars().all(|c| c.is_ascii_digit()) {
                Some(format!("{}-{}-{}", &s[0..4], &s[4..6], &s[6..8]))
            } else {
                None
            }
        })
    }

    fn get_time_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<String> {
        Self::get_text_value(dicom_obj, tag).and_then(|s| {
            // 简单处理时间格式，实际可能需要更复杂的解析
            if !s.is_empty() { Some(s) } else { None }
        })
    }

    fn get_int_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<i32> {
        dicom_obj.element(tag).ok().and_then(|e| e.to_int().ok())
    }

    fn get_decimal_value(dicom_obj: &InMemDicomObject, tag: Tag) -> Option<f64> {
        dicom_obj
            .element(tag)
            .ok()
            .and_then(|e| e.to_float64().ok())
    }

    pub(crate) fn extract_patient_entity(tenant_id: &str, dicom_obj: &InMemDicomObject) -> PatientEntity {
        PatientEntity {
            tenant_id: tenant_id.to_string(),
            patient_id: Self::get_text_value(dicom_obj, tags::PATIENT_ID).unwrap_or_default(),
            patient_name: Self::get_text_value(dicom_obj, tags::PATIENT_NAME),
            patient_birth_date: Self::get_date_value(dicom_obj, tags::PATIENT_BIRTH_DATE),
            patient_sex: Self::get_text_value(dicom_obj, tags::PATIENT_SEX),
            patient_birth_time: Self::get_time_value(dicom_obj, tags::PATIENT_BIRTH_TIME),
            ethnic_group: Self::get_text_value(dicom_obj, tags::ETHNIC_GROUP),
            created_time: None,
            updated_time: None,
        }
    }

    pub(crate) fn extract_study_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
        patient_id: &str,
    ) -> StudyEntity {
        StudyEntity {
            tenant_id: tenant_id.to_string(),
            study_instance_uid: Self::get_text_value(dicom_obj, tags::STUDY_INSTANCE_UID)
                .unwrap_or_default(),
            patient_id: patient_id.to_string(),
            patient_age: Self::get_text_value(dicom_obj, tags::PATIENT_AGE),
            patient_size: Self::get_decimal_value(dicom_obj, tags::PATIENT_SIZE),
            patient_weight: Self::get_decimal_value(dicom_obj, tags::PATIENT_WEIGHT),
            medical_alerts: Self::get_text_value(dicom_obj, tags::MEDICAL_ALERTS),
            allergies: Self::get_text_value(dicom_obj, tags::ALLERGIES),
            pregnancy_status: Self::get_int_value(dicom_obj, tags::PREGNANCY_STATUS),
            occupation: Self::get_text_value(dicom_obj, tags::OCCUPATION),
            additional_patient_history: Self::get_text_value(
                dicom_obj,
                tags::ADDITIONAL_PATIENT_HISTORY,
            ),
            patient_comments: Self::get_text_value(dicom_obj, tags::PATIENT_COMMENTS),
            study_date: Self::get_date_value(dicom_obj, tags::STUDY_DATE),
            study_time: Self::get_time_value(dicom_obj, tags::STUDY_TIME),
            accession_number: Self::get_text_value(dicom_obj, tags::ACCESSION_NUMBER),
            study_id: Self::get_text_value(dicom_obj, tags::STUDY_ID),
            study_description: Self::get_text_value(dicom_obj, tags::STUDY_DESCRIPTION),
            referring_physician_name: Self::get_text_value(
                dicom_obj,
                tags::REFERRING_PHYSICIAN_NAME,
            ),
            admission_id: Self::get_text_value(dicom_obj, tags::ADMISSION_ID),
            patient_age_at_study: Self::get_text_value(dicom_obj, tags::PATIENT_AGE),
            performing_physician_name: Self::get_text_value(
                dicom_obj,
                tags::PERFORMING_PHYSICIAN_NAME,
            ),
            procedure_code_sequence: Self::get_text_value(dicom_obj, tags::PROCEDURE_CODE_SEQUENCE),
            created_time: None,
            updated_time: None,
        }
    }

    pub(crate) fn extract_series_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
        study_uid: &str,
    ) -> SeriesEntity {
        SeriesEntity {
            tenant_id: tenant_id.to_string(),
            series_instance_uid: Self::get_text_value(dicom_obj, tags::SERIES_INSTANCE_UID)
                .unwrap_or_default(),
            study_instance_uid: study_uid.to_string(),
            modality: Self::get_text_value(dicom_obj, tags::MODALITY).unwrap_or_default(),
            series_number: Self::get_int_value(dicom_obj, tags::SERIES_NUMBER),
            series_date: Self::get_date_value(dicom_obj, tags::SERIES_DATE),
            series_time: Self::get_time_value(dicom_obj, tags::SERIES_TIME),
            series_description: Self::get_text_value(dicom_obj, tags::SERIES_DESCRIPTION),
            body_part_examined: Self::get_text_value(dicom_obj, tags::BODY_PART_EXAMINED),
            protocol_name: Self::get_text_value(dicom_obj, tags::PROTOCOL_NAME),
            image_type: Self::get_text_value(dicom_obj, tags::IMAGE_TYPE),
            acquisition_number: Self::get_int_value(dicom_obj, tags::ACQUISITION_NUMBER),
            acquisition_time: Self::get_time_value(dicom_obj, tags::ACQUISITION_TIME),
            acquisition_date: Self::get_date_value(dicom_obj, tags::ACQUISITION_DATE),
            performing_physician_name: Self::get_text_value(
                dicom_obj,
                tags::PERFORMING_PHYSICIAN_NAME,
            ),
            operators_name: Self::get_text_value(dicom_obj, tags::OPERATORS_NAME),
            number_of_series_related_instances: Self::get_int_value(
                dicom_obj,
                tags::NUMBER_OF_SERIES_RELATED_INSTANCES,
            ),
            created_time: None,
            updated_time: None,
        }
    }

    pub(crate) fn extract_image_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
        series_uid: &str,
        study_uid: &str,
        patient_id: &str,
    ) -> ImageEntity {
        let acquisition_date_time = Self::get_text_value(dicom_obj, tags::ACQUISITION_DATE_TIME);
        let acquisition_date_time_parsed =
            acquisition_date_time.and_then(|dt| if !dt.is_empty() { Some(dt) } else { None });

        ImageEntity {
            tenant_id: tenant_id.to_string(),
            sop_instance_uid: Self::get_text_value(dicom_obj, tags::SOP_INSTANCE_UID)
                .unwrap_or_default(),
            series_instance_uid: series_uid.to_string(),
            study_instance_uid: study_uid.to_string(),
            patient_id: patient_id.to_string(),
            instance_number: Self::get_int_value(dicom_obj, tags::INSTANCE_NUMBER),
            image_comments: Self::get_text_value(dicom_obj, tags::IMAGE_COMMENTS),
            content_date: Self::get_date_value(dicom_obj, tags::CONTENT_DATE),
            content_time: Self::get_time_value(dicom_obj, tags::CONTENT_TIME),
            acquisition_date_time: acquisition_date_time_parsed,
            image_type: Self::get_text_value(dicom_obj, tags::IMAGE_TYPE),
            image_orientation_patient: Self::get_text_value(
                dicom_obj,
                tags::IMAGE_ORIENTATION_PATIENT,
            ),
            image_position_patient: Self::get_text_value(dicom_obj, tags::IMAGE_POSITION_PATIENT),
            slice_thickness: Self::get_decimal_value(dicom_obj, tags::SLICE_THICKNESS),
            spacing_between_slices: Self::get_decimal_value(
                dicom_obj,
                tags::SPACING_BETWEEN_SLICES,
            ),
            slice_location: Self::get_decimal_value(dicom_obj, tags::SLICE_LOCATION),
            samples_per_pixel: Self::get_int_value(dicom_obj, tags::SAMPLES_PER_PIXEL),
            photometric_interpretation: Self::get_text_value(
                dicom_obj,
                tags::PHOTOMETRIC_INTERPRETATION,
            ),
            width: Self::get_int_value(dicom_obj, tags::ROWS),
            columns: Self::get_int_value(dicom_obj, tags::COLUMNS),
            bits_allocated: Self::get_int_value(dicom_obj, tags::BITS_ALLOCATED),
            bits_stored: Self::get_int_value(dicom_obj, tags::BITS_STORED),
            high_bit: Self::get_int_value(dicom_obj, tags::HIGH_BIT),
            pixel_representation: Self::get_int_value(dicom_obj, tags::PIXEL_REPRESENTATION),
            rescale_intercept: Self::get_decimal_value(dicom_obj, tags::RESCALE_INTERCEPT),
            rescale_slope: Self::get_decimal_value(dicom_obj, tags::RESCALE_SLOPE),
            rescale_type: Self::get_text_value(dicom_obj, tags::RESCALE_TYPE),
            acquisition_device_processing_description: Self::get_text_value(
                dicom_obj,
                tags::ACQUISITION_DEVICE_PROCESSING_DESCRIPTION,
            ),
            acquisition_device_processing_code: Self::get_text_value(
                dicom_obj,
                tags::ACQUISITION_DEVICE_PROCESSING_CODE,
            ),
            device_serial_number: Self::get_text_value(dicom_obj, tags::DEVICE_SERIAL_NUMBER),
            software_versions: Self::get_text_value(dicom_obj, tags::SOFTWARE_VERSIONS),
            transfer_syntax_uid: Self::get_text_value(dicom_obj, tags::TRANSFER_SYNTAX_UID),
            pixel_data_location: None,
            thumbnail_location: None,
            sop_class_uid: Self::get_text_value(dicom_obj, tags::SOP_CLASS_UID).unwrap_or_default(),
            image_status: Some("ACTIVE".to_string()),
            created_time: None,
            updated_time: None,
        }
    }
}
