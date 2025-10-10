use crate::database_entities::{ImageEntity, PatientEntity, SeriesEntity, StudyEntity};
use crate::dicom_utils;
use crate::dicom_utils::get_tag_value;
use dicom_dictionary_std::tags;
use dicom_object::InMemDicomObject;
use std::fmt;

#[derive(Debug)]
pub enum ExtractionError {
    MissingPatientId,
    EmptyPatientId,
    MissingStudyUid,
    EmptyStudyUid,
    MissingSeriesUid,
    EmptySeriesUid,
    MissingSopUid,
    EmptySopUid,
}

impl fmt::Display for ExtractionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExtractionError::MissingPatientId => write!(f, "Missing patient ID in DICOM object"),
            ExtractionError::EmptyPatientId => write!(f, "Patient ID is empty in DICOM object"),
            ExtractionError::MissingStudyUid => write!(f, "Missing study UID in DICOM object"),
            ExtractionError::EmptyStudyUid => write!(f, "Study UID is empty in DICOM object"),
            ExtractionError::MissingSeriesUid => write!(f, "Missing series UID in DICOM object"),
            ExtractionError::EmptySeriesUid => write!(f, "Series UID is empty in DICOM object"),
            ExtractionError::MissingSopUid => write!(f, "Missing SOP UID in DICOM object"),
            ExtractionError::EmptySopUid => write!(f, "SOP UID is empty in DICOM object"),
        }
    }
}

impl std::error::Error for ExtractionError {}

pub struct DbProviderBase {}
impl DbProviderBase {
    pub fn extract_patient_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
    ) -> Result<PatientEntity, ExtractionError> {
        let patient_id = get_tag_value(tags::PATIENT_ID, dicom_obj, "".to_string());

        if patient_id.is_empty() {
            return Err(ExtractionError::EmptyPatientId);
        }

        Ok(PatientEntity {
            tenant_id: tenant_id.to_string(),
            patient_id: patient_id.to_string(),
            patient_name: dicom_utils::get_text_value(dicom_obj, tags::PATIENT_NAME),
            patient_sex: dicom_utils::get_text_value(dicom_obj, tags::PATIENT_SEX),
            patient_birth_date: dicom_utils::get_date_value_dicom(
                dicom_obj,
                tags::PATIENT_BIRTH_DATE,
            ),
            patient_birth_time: dicom_utils::get_time_value_dicom(
                dicom_obj,
                tags::PATIENT_BIRTH_TIME,
            ),
            ethnic_group: dicom_utils::get_text_value(dicom_obj, tags::ETHNIC_GROUP),
            created_time: None,
            updated_time: None,
        })
    }

    pub fn extract_study_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
        patient_id: &str,
    ) -> Result<StudyEntity, ExtractionError> {
        let study_uid = get_tag_value(tags::STUDY_INSTANCE_UID, dicom_obj, "".to_string());
        if study_uid.is_empty() {
            return Err(ExtractionError::EmptyStudyUid);
        }
        Ok(StudyEntity {
            tenant_id: tenant_id.to_string(),
            study_instance_uid: study_uid.to_string(),
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
            study_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::STUDY_DATE).unwrap(),
            study_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::STUDY_TIME),
            accession_number: dicom_utils::get_text_value(dicom_obj, tags::ACCESSION_NUMBER),
            study_id: dicom_utils::get_text_value(dicom_obj, tags::STUDY_ID),
            study_description: dicom_utils::get_text_value(dicom_obj, tags::STUDY_DESCRIPTION),
            referring_physician_name: dicom_utils::get_text_value(
                dicom_obj,
                tags::REFERRING_PHYSICIAN_NAME,
            ),
            admission_id: dicom_utils::get_text_value(dicom_obj, tags::ADMISSION_ID),

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
        })
    }

    pub fn extract_series_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
        study_uid: &str,
    ) -> Result<SeriesEntity, ExtractionError> {
        let patid = get_tag_value(tags::PATIENT_ID, dicom_obj, "".to_string());
        let series_uid = get_tag_value(tags::SERIES_INSTANCE_UID, dicom_obj, "".to_string());

        if patid.is_empty() {
            return Err(ExtractionError::EmptyPatientId);
        }

        if study_uid.is_empty() {
            return Err(ExtractionError::EmptyStudyUid);
        }

        if series_uid.is_empty() {
            return Err(ExtractionError::EmptySeriesUid);
        }

        Ok(SeriesEntity {
            tenant_id: tenant_id.to_string(),
            series_instance_uid: series_uid.to_string(),
            study_instance_uid: study_uid.to_string(),
            patient_id: patid.to_string(),
            modality: dicom_utils::get_text_value(dicom_obj, tags::MODALITY).unwrap_or_default(),
            series_number: dicom_utils::get_int_value(dicom_obj, tags::SERIES_NUMBER),
            series_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::SERIES_DATE),
            series_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::SERIES_TIME),
            series_description: dicom_utils::get_text_value(dicom_obj, tags::SERIES_DESCRIPTION),
            body_part_examined: dicom_utils::get_text_value(dicom_obj, tags::BODY_PART_EXAMINED),
            protocol_name: dicom_utils::get_text_value(dicom_obj, tags::PROTOCOL_NAME),
            acquisition_number: dicom_utils::get_int_value(dicom_obj, tags::ACQUISITION_NUMBER),
            acquisition_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::ACQUISITION_TIME),
            acquisition_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::ACQUISITION_DATE),
            acquisition_date_time: dicom_utils::get_datetime_value_dicom(
                dicom_obj,
                tags::ACQUISITION_DATE_TIME,
            ),
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
        })
    }

    pub fn extract_image_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
        patient_id: &str,
        study_uid: &str,
        series_uid: &str,
    ) -> Result<ImageEntity, ExtractionError> {
        let sop_uid = get_tag_value(tags::SOP_INSTANCE_UID, dicom_obj, "".to_string());

        if sop_uid.is_empty() {
            return Err(ExtractionError::EmptySopUid);
        }

        let img_number_of_frames = get_tag_value(tags::NUMBER_OF_FRAMES, dicom_obj, 1);
        Ok(ImageEntity {
            tenant_id: tenant_id.to_string(),
            sop_instance_uid: sop_uid.to_string(),
            series_instance_uid: series_uid.to_string(),
            study_instance_uid: study_uid.to_string(),
            patient_id: patient_id.to_string(),
            instance_number: dicom_utils::get_int_value(dicom_obj, tags::INSTANCE_NUMBER),
            image_comments: dicom_utils::get_text_value(dicom_obj, tags::IMAGE_COMMENTS),
            content_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::CONTENT_DATE),
            content_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::CONTENT_TIME),
            acquisition_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::ACQUISITION_DATE),
            acquisition_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::ACQUISITION_TIME),
            acquisition_date_time: dicom_utils::get_datetime_value_dicom(
                dicom_obj,
                tags::ACQUISITION_DATE_TIME,
            ),
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
            window_center: dicom_utils::get_text_value(dicom_obj, tags::WINDOW_CENTER),
            window_width: dicom_utils::get_text_value(dicom_obj, tags::WINDOW_WIDTH),
            number_of_frames: img_number_of_frames,
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
            transfer_syntax_uid: dicom_utils::get_text_value(dicom_obj, tags::TRANSFER_SYNTAX_UID)
                .unwrap_or_default(),
            pixel_data_location: None,
            thumbnail_location: None,
            sop_class_uid: dicom_utils::get_text_value(dicom_obj, tags::SOP_CLASS_UID)
                .unwrap_or_default(),
            image_status: Some("ACTIVE".to_string()),
            space_size: None,
            created_time: None,
            updated_time: None,
        })
    }

        pub fn extract_entity(
        tenant_id: &str,
        dicom_obj: &InMemDicomObject,
    ) -> Result<(PatientEntity, StudyEntity, SeriesEntity, ImageEntity), ExtractionError> {
        let patient_id = get_tag_value(tags::PATIENT_ID, dicom_obj, "".to_string());

        if patient_id.is_empty() {
            return Err(ExtractionError::EmptyPatientId);
        }
        let study_uid = get_tag_value(tags::STUDY_INSTANCE_UID, dicom_obj, "".to_string());
        if study_uid.is_empty() {
            return Err(ExtractionError::EmptyStudyUid);
        }
        let series_uid = get_tag_value(tags::SERIES_INSTANCE_UID, dicom_obj, "".to_string());
        if series_uid.is_empty() {
            return Err(ExtractionError::EmptySeriesUid);
        }
        let sop_uid = get_tag_value(tags::SOP_INSTANCE_UID, dicom_obj, "".to_string());

        if sop_uid.is_empty() {
            return Err(ExtractionError::EmptySopUid);
        }
        let img_number_of_frames = get_tag_value(tags::NUMBER_OF_FRAMES, dicom_obj, 1);

        let patient_entity = PatientEntity {
            tenant_id: tenant_id.to_string(),
            patient_id: patient_id.to_string(),
            patient_name: dicom_utils::get_text_value(dicom_obj, tags::PATIENT_NAME),
            patient_sex: dicom_utils::get_text_value(dicom_obj, tags::PATIENT_SEX),
            patient_birth_date: dicom_utils::get_date_value_dicom(
                dicom_obj,
                tags::PATIENT_BIRTH_DATE,
            ),
            patient_birth_time: dicom_utils::get_time_value_dicom(
                dicom_obj,
                tags::PATIENT_BIRTH_TIME,
            ),
            ethnic_group: dicom_utils::get_text_value(dicom_obj, tags::ETHNIC_GROUP),
            created_time: None,
            updated_time: None,
        };

        let study_entity = StudyEntity {
            tenant_id: tenant_id.to_string(),
            study_instance_uid: study_uid.to_string(),
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
            study_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::STUDY_DATE).unwrap(),
            study_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::STUDY_TIME),
            accession_number: dicom_utils::get_text_value(dicom_obj, tags::ACCESSION_NUMBER),
            study_id: dicom_utils::get_text_value(dicom_obj, tags::STUDY_ID),
            study_description: dicom_utils::get_text_value(dicom_obj, tags::STUDY_DESCRIPTION),
            referring_physician_name: dicom_utils::get_text_value(
                dicom_obj,
                tags::REFERRING_PHYSICIAN_NAME,
            ),
            admission_id: dicom_utils::get_text_value(dicom_obj, tags::ADMISSION_ID),
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
        };

        let series_entity = SeriesEntity {
            tenant_id: tenant_id.to_string(),
            series_instance_uid: series_uid.to_string(),
            study_instance_uid: study_uid.to_string(),
            patient_id: patient_id.to_string(),
            modality: dicom_utils::get_text_value(dicom_obj, tags::MODALITY).unwrap_or_default(),
            series_number: dicom_utils::get_int_value(dicom_obj, tags::SERIES_NUMBER),
            series_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::SERIES_DATE),
            series_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::SERIES_TIME),
            series_description: dicom_utils::get_text_value(dicom_obj, tags::SERIES_DESCRIPTION),
            body_part_examined: dicom_utils::get_text_value(dicom_obj, tags::BODY_PART_EXAMINED),
            protocol_name: dicom_utils::get_text_value(dicom_obj, tags::PROTOCOL_NAME),
            acquisition_number: dicom_utils::get_int_value(dicom_obj, tags::ACQUISITION_NUMBER),
            acquisition_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::ACQUISITION_TIME),
            acquisition_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::ACQUISITION_DATE),
            acquisition_date_time: dicom_utils::get_datetime_value_dicom(
                dicom_obj,
                tags::ACQUISITION_DATE_TIME,
            ),
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
        };

        let image_entity = ImageEntity {
            tenant_id: tenant_id.to_string(),
            sop_instance_uid: sop_uid.to_string(),
            series_instance_uid: series_uid.to_string(),
            study_instance_uid: study_uid.to_string(),
            patient_id: patient_id.to_string(),
            instance_number: dicom_utils::get_int_value(dicom_obj, tags::INSTANCE_NUMBER),
            image_comments: dicom_utils::get_text_value(dicom_obj, tags::IMAGE_COMMENTS),
            content_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::CONTENT_DATE),
            content_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::CONTENT_TIME),
            acquisition_date: dicom_utils::get_date_value_dicom(dicom_obj, tags::ACQUISITION_DATE),
            acquisition_time: dicom_utils::get_time_value_dicom(dicom_obj, tags::ACQUISITION_TIME),
            acquisition_date_time: dicom_utils::get_datetime_value_dicom(
                dicom_obj,
                tags::ACQUISITION_DATE_TIME,
            ),
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
            window_center: dicom_utils::get_text_value(dicom_obj, tags::WINDOW_CENTER),
            window_width: dicom_utils::get_text_value(dicom_obj, tags::WINDOW_WIDTH),
            number_of_frames: img_number_of_frames,
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
            transfer_syntax_uid: dicom_utils::get_text_value(dicom_obj, tags::TRANSFER_SYNTAX_UID)
                .unwrap_or_default(),
            pixel_data_location: None,
            thumbnail_location: None,
            sop_class_uid: dicom_utils::get_text_value(dicom_obj, tags::SOP_CLASS_UID)
                .unwrap_or_default(),
            image_status: Some("ACTIVE".to_string()),
            space_size: None,
            created_time: None,
            updated_time: None,
        };
        println!("TimeVlaues:{:?}", image_entity.acquisition_date_time);    

        Ok((patient_entity, study_entity, series_entity, image_entity))
    }
}
