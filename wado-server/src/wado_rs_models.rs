use chrono::{NaiveDate, NaiveTime};
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
        let accession_number_value = match &dicom_state_meta.accession_number {
            None => "".to_string(),
            Some(s) => s.to_string(),
        };

        let pat_name = match &dicom_state_meta.patient_name {
            None => None,
            Some(s) => Some(s.to_string()),
        };

        let sex_value = match &dicom_state_meta.patient_sex {
            None => None,
            Some(s) => Some(s.to_string()),
        };

        let age_value = match &dicom_state_meta.patient_age {
            None => None,
            Some(s) => Some(s.to_string()),
        };

        let study_id_value = match &dicom_state_meta.study_id {
            None => None,
            Some(s) => Some(s.to_string()),
        };

        let study_desc_value = match &dicom_state_meta.study_description {
            None => None,
            Some(s) => Some(s.to_string()),
        };

        let modality_value = match &dicom_state_meta.modality {
            None => None,
            Some(s) => Some(s.to_string()),
        };

        let series_desc_value = match &dicom_state_meta.series_description {
            None => None,
            Some(s) => Some(s.to_string()),
        };

        let body_part_examined_value = match &dicom_state_meta.body_part_examined {
            None => None,
            Some(s) => Some(s.to_string()),
        };

        let protocol_name_value = match &dicom_state_meta.protocol_name {
            None => None,
            Some(s) => Some(s.to_string()),
        };

        Self {
            tenant_id: dicom_state_meta.tenant_id.to_string(),
            patient_id: dicom_state_meta.patient_id.to_string(),
            study_uid: dicom_state_meta.study_uid.to_string(),
            series_uid: dicom_state_meta.series_uid.to_string(),
            patient_name: pat_name,
            patient_sex: sex_value,
            patient_birth_date: dicom_state_meta.patient_birth_date,
            patient_birth_time: dicom_state_meta.patient_birth_time,
            patient_age: age_value,
            patient_size: dicom_state_meta.patient_size,
            patient_weight: dicom_state_meta.patient_weight,
            study_date: dicom_state_meta.study_date,
            study_time: dicom_state_meta.study_time,
            accession_number: accession_number_value,
            study_id: study_id_value,
            study_description: study_desc_value,
            series_related_instances: dicom_state_meta.series_related_instances,
            modality: modality_value,
            series_number: dicom_state_meta.series_number,
            series_date: dicom_state_meta.series_date,
            series_time: dicom_state_meta.series_time,
            series_description: series_desc_value,
            body_part_examined: body_part_examined_value,
            protocol_name: protocol_name_value,
        }
    }
}
