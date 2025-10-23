use dicom_core::chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use crate::string_ext::UidHashValue;

// study.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StudyEntity {
    pub tenant_id: String,
    pub study_instance_uid: String,
    pub study_uid_hash: UidHashValue,
    pub patient_id: String,
    pub patient_name: Option<String>,
    pub patient_age: Option<String>,
    pub patient_sex: Option<String>,
    pub patient_size: Option<f64>,
    pub patient_weight: Option<f64>,
    pub patient_birth_date: Option<chrono::NaiveDate>,
    pub patient_birth_time: Option<chrono::NaiveTime>,
    pub study_date: chrono::NaiveDate,
    pub study_time: Option<chrono::NaiveTime>,
    pub accession_number: Option<String>,
    pub study_id: Option<String>,
    pub study_description: Option<String>,
    pub study_date_origin: String,

}

// series.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeriesEntity {
    pub tenant_id: String,
    pub series_instance_uid: String,
    pub study_instance_uid: String,
    pub patient_id: String,
    pub modality: Option<String>,
    pub series_number: Option<i32>,
    pub series_date: Option<chrono::NaiveDate>,
    pub series_time:  Option<chrono::NaiveTime>,
    pub series_description: Option<String>,
    pub body_part_examined: Option<String>,
    pub protocol_name: Option<String>,
    pub created_time: Option<NaiveDateTime>,
    pub updated_time: Option<NaiveDateTime>,
}
