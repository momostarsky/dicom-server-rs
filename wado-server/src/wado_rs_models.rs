use chrono::{NaiveDate, NaiveTime};
use database::dicom_dbtype::BoundedString;
use database::dicom_meta::DicomStateMeta;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SubSeriesMeta {
    #[serde(rename = "tenant_id")]
    pub tenant_id: String,
    #[serde(rename = "patient_id")]
    pub patient_id: String,
    #[serde(rename = "study_uid")]
    pub study_uid: String,
    #[serde(rename = "series_uid")]
    pub series_uid: String,
    #[serde(rename = "patient_name")]
    pub patient_name: Option<String>,
    #[serde(rename = "patient_sex")]
    pub patient_sex: Option<String>,
    #[serde(rename = "patient_birth_date")]
    pub patient_birth_date: Option<NaiveDate>,
    #[serde(rename = "patient_birth_time")]
    pub patient_birth_time: Option<NaiveTime>,
    #[serde(rename = "patient_age")]
    pub patient_age: Option<String>,
    #[serde(rename = "patient_size")]
    pub patient_size: Option<f64>,
    #[serde(rename = "patient_weight")]
    pub patient_weight: Option<f64>,

    #[serde(rename = "study_date")]
    pub study_date: NaiveDate,
    #[serde(rename = "study_time")]
    pub study_time: Option<NaiveTime>,
    #[serde(rename = "accession_number")]
    pub accession_number: String,
    #[serde(rename = "study_id")]
    pub study_id: Option<String>,
    #[serde(rename = "study_description")]
    pub study_description: Option<String>,

    #[serde(rename = "modality")]
    pub modality: Option<String>,
    #[serde(rename = "series_number")]
    pub series_number: Option<i32>,
    #[serde(rename = "series_date")]
    pub series_date: Option<NaiveDate>,
    #[serde(rename = "series_time")]
    pub series_time: Option<NaiveTime>,
    #[serde(rename = "series_description")]
    pub series_description: Option<String>,
    #[serde(rename = "series_related_instances")]
    pub series_related_instances: Option<i32>,
    #[serde(rename = "body_part_examined")]
    pub body_part_examined: Option<String>,
    #[serde(rename = "protocol_name")]
    pub protocol_name: Option<String>,
}

impl SubSeriesMeta {
    pub fn new(dicom_state_meta: &DicomStateMeta) -> Self {
        Self {
            tenant_id: dicom_state_meta.tenant_id.to_string(),
            patient_id: dicom_state_meta.patient_id.to_string(),
            study_uid: dicom_state_meta.study_uid.to_string(),
            series_uid: dicom_state_meta.series_uid.to_string(),
            patient_name: dicom_state_meta
                .patient_name
                .clone()
                .unwrap_or(BoundedString::<64>::default())
                .to_string()
                .into(),
            patient_sex: dicom_state_meta.patient_sex.clone().map(|s| s.to_string()),
            patient_birth_date: dicom_state_meta.patient_birth_date,
            patient_birth_time: dicom_state_meta.patient_birth_time,
            patient_age: dicom_state_meta.patient_age.clone().map(|s| s.to_string()),
            patient_size: dicom_state_meta.patient_size,
            patient_weight: dicom_state_meta.patient_weight,
            study_date: dicom_state_meta.study_date,
            study_time: dicom_state_meta.study_time,
            accession_number: dicom_state_meta.accession_number.to_string(),
            study_id: dicom_state_meta.study_id.clone().map(|s| s.to_string()),
            study_description: dicom_state_meta
                .study_description
                .clone()
                .map(|s| s.to_string()),
            series_related_instances: dicom_state_meta.series_related_instances,
            modality: dicom_state_meta.modality.clone().map(|s| s.to_string()),
            series_number: dicom_state_meta.series_number,
            series_date: dicom_state_meta.series_date,
            series_time: dicom_state_meta.series_time,
            series_description: dicom_state_meta
                .series_description
                .clone()
                .map(|s| s.to_string()),
            body_part_examined: dicom_state_meta
                .body_part_examined
                .clone()
                .map(|s| s.to_string()),
            protocol_name: dicom_state_meta
                .protocol_name
                .clone()
                .map(|s| s.to_string()),
        }
    }
}
