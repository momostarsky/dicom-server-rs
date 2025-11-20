use crate::dicom_dbtype::{BoundedString, DicomDateString, FixedLengthString};
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TransferStatus {
    NoNeedTransfer,
    Success,
    Failed,
}

impl Display for TransferStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransferStatus::NoNeedTransfer => write!(f, "NoNeedTransfer"),
            TransferStatus::Success => write!(f, "Success"),
            TransferStatus::Failed => write!(f, "Failed"),
        }
    }
}

/// DicomStoreMeta 用于DICOM-CStoreSCP服务记录收图日志.
/// 包含了所有必要的元数据字段.每一个DicomStoreMeta实例标识接收一个DICOM文件.并成功写入磁盘.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomStoreMeta {
    #[serde(rename = "trace_id")]
    pub trace_id: FixedLengthString<36>,
    #[serde(rename = "worker_node_id")]
    pub worker_node_id: BoundedString<64>,
    #[serde(rename = "tenant_id")]
    pub tenant_id: BoundedString<64>,
    #[serde(rename = "patient_id")]
    pub patient_id: BoundedString<64>,
    #[serde(rename = "study_uid")]
    pub study_uid: BoundedString<64>,
    #[serde(rename = "series_uid")]
    pub series_uid: BoundedString<64>,
    #[serde(rename = "sop_uid")]
    pub sop_uid: BoundedString<64>,
    #[serde(rename = "file_size")]
    pub file_size: i64,
    #[serde(rename = "file_path")]
    pub file_path: BoundedString<512>,
    #[serde(rename = "transfer_syntax_uid")]
    pub transfer_syntax_uid: BoundedString<64>,
    #[serde(rename = "number_of_frames")]
    pub number_of_frames: i32,
    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,
    #[serde(rename = "series_uid_hash")]
    pub series_uid_hash: BoundedString<20>,
    #[serde(rename = "study_uid_hash")]
    pub study_uid_hash: BoundedString<20>,
    #[serde(rename = "accession_number")]
    pub accession_number: BoundedString<64>,
    #[serde(rename = "target_ts")]
    pub target_ts: BoundedString<64>,
    #[serde(rename = "study_date")]
    pub study_date: NaiveDate,
    #[serde(rename = "transfer_status")]
    pub transfer_status: TransferStatus,
    #[serde(rename = "source_ip")]
    pub source_ip: BoundedString<24>,
    #[serde(rename = "source_ae")]
    pub source_ae: BoundedString<64>,
}
// 为 DicomObjectMeta 实现 Hash trait 以便可以在 HashSet 中使用
impl Hash for DicomStoreMeta {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tenant_id.hash(state);
        self.patient_id.hash(state);
        self.study_uid.hash(state);
        self.series_uid.hash(state);
        self.sop_uid.hash(state);
    }
}
impl PartialEq for DicomStoreMeta {
    fn eq(&self, other: &Self) -> bool {
        self.tenant_id == other.tenant_id
            && self.patient_id == other.patient_id
            && self.study_uid == other.study_uid
            && self.series_uid == other.series_uid
            && self.sop_uid == other.sop_uid
    }
}
impl Eq for DicomStoreMeta {}

/// DicomJsonMeta 用于记录DICOM文件生成JSON格式的元数据给WADO-RS使用
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomJsonMeta {
    #[serde(rename = "tenant_id")]
    pub tenant_id: BoundedString<64>,
    #[serde(rename = "study_uid")]
    pub study_uid: BoundedString<64>,
    #[serde(rename = "series_uid")]
    pub series_uid: BoundedString<64>,
    #[serde(rename = "study_uid_hash")]
    pub study_uid_hash: BoundedString<20>,
    #[serde(rename = "series_uid_hash")]
    pub series_uid_hash: BoundedString<20>,
    #[serde(rename = "study_date_origin")]
    pub study_date_origin: DicomDateString,
    #[serde(rename = "flag_time")]
    pub flag_time: NaiveDateTime,
    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,
    #[serde(rename = "json_status")]
    pub json_status: i32,
    #[serde(rename = "retry_times")]
    pub retry_times: i32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomStateMeta {
    #[serde(rename = "tenant_id")]
    pub tenant_id: BoundedString<64>,
    #[serde(rename = "patient_id")]
    pub patient_id: BoundedString<64>,
    #[serde(rename = "study_uid")]
    pub study_uid: BoundedString<64>,
    #[serde(rename = "series_uid")]
    pub series_uid: BoundedString<64>,
    #[serde(rename = "study_uid_hash")]
    pub study_uid_hash: BoundedString<20>,
    #[serde(rename = "series_uid_hash")]
    pub series_uid_hash: BoundedString<20>,
    #[serde(rename = "study_date_origin")]
    pub study_date_origin: DicomDateString,

    #[serde(rename = "patient_name")]
    pub patient_name: Option<BoundedString<64>>,
    #[serde(rename = "patient_sex")]
    pub patient_sex: Option<BoundedString<1>>,
    #[serde(rename = "patient_birth_date")]
    pub patient_birth_date: Option<NaiveDate>,
    #[serde(rename = "patient_birth_time")]
    pub patient_birth_time: Option<NaiveTime>,
    #[serde(rename = "patient_age")]
    pub patient_age: Option<BoundedString<16>>,
    #[serde(rename = "patient_size")]
    pub patient_size: Option<f64>,
    #[serde(rename = "patient_weight")]
    pub patient_weight: Option<f64>,

    #[serde(rename = "study_date")]
    pub study_date: NaiveDate,
    #[serde(rename = "study_time")]
    pub study_time: Option<NaiveTime>,
    #[serde(rename = "accession_number")]
    pub accession_number: BoundedString<16>,
    #[serde(rename = "study_id")]
    pub study_id: Option<BoundedString<16>>,
    #[serde(rename = "study_description")]
    pub study_description: Option<BoundedString<64>>,

    #[serde(rename = "modality")]
    pub modality: Option<BoundedString<16>>,
    #[serde(rename = "series_number")]
    pub series_number: Option<i32>,
    #[serde(rename = "series_date")]
    pub series_date: Option<NaiveDate>,
    #[serde(rename = "series_time")]
    pub series_time: Option<NaiveTime>,
    #[serde(rename = "series_description")]
    pub series_description: Option<BoundedString<256>>,
    #[serde(rename = "body_part_examined")]
    pub body_part_examined: Option<BoundedString<64>>,
    #[serde(rename = "protocol_name")]
    pub protocol_name: Option<BoundedString<64>>,
    #[serde(rename = "series_related_instances")]
    pub series_related_instances: Option<i32>,
    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,
    #[serde(rename = "updated_time")]
    pub updated_time: NaiveDateTime,
}

impl DicomStateMeta {
    pub fn unique_key(&self) -> (String, String, String, String) {
        (
            self.tenant_id.as_str().to_string(),
            self.patient_id.as_str().to_string(),
            self.study_uid.as_str().to_string(),
            self.series_uid.as_str().to_string(),
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomImageMeta {
    #[serde(rename = "tenant_id")]
    pub tenant_id: BoundedString<64>,

    #[serde(rename = "patient_id")]
    pub patient_id: BoundedString<64>,

    #[serde(rename = "study_uid")]
    pub study_uid: BoundedString<64>,

    #[serde(rename = "series_uid")]
    pub series_uid: BoundedString<64>,

    #[serde(rename = "sop_uid")]
    pub sop_uid: BoundedString<64>,

    #[serde(rename = "study_uid_hash")]
    pub study_uid_hash: BoundedString<20>,

    #[serde(rename = "series_uid_hash")]
    pub series_uid_hash: BoundedString<20>,

    #[serde(rename = "instance_number")]
    pub instance_number: Option<i32>,

    #[serde(rename = "content_date")]
    pub content_date: Option<NaiveDate>,

    #[serde(rename = "content_time")]
    pub content_time: Option<NaiveTime>,

    #[serde(rename = "image_type")]
    pub image_type: Option<BoundedString<128>>,

    #[serde(rename = "image_orientation_patient")]
    pub image_orientation_patient: Option<BoundedString<128>>,

    #[serde(rename = "image_position_patient")]
    pub image_position_patient: Option<BoundedString<64>>,

    #[serde(rename = "slice_thickness")]
    pub slice_thickness: Option<f64>,

    #[serde(rename = "spacing_between_slices")]
    pub spacing_between_slices: Option<f64>,

    #[serde(rename = "slice_location")]
    pub slice_location: Option<f64>,

    #[serde(rename = "samples_per_pixel")]
    pub samples_per_pixel: Option<i32>,

    #[serde(rename = "photometric_interpretation")]
    pub photometric_interpretation: Option<BoundedString<32>>,

    #[serde(rename = "width")]
    pub width: Option<i32>,

    #[serde(rename = "columns")]
    pub columns: Option<i32>,

    #[serde(rename = "bits_allocated")]
    pub bits_allocated: Option<i32>,

    #[serde(rename = "bits_stored")]
    pub bits_stored: Option<i32>,

    #[serde(rename = "high_bit")]
    pub high_bit: Option<i32>,

    #[serde(rename = "pixel_representation")]
    pub pixel_representation: Option<i32>,

    #[serde(rename = "rescale_intercept")]
    pub rescale_intercept: Option<f64>,

    #[serde(rename = "rescale_slope")]
    pub rescale_slope: Option<f64>,

    #[serde(rename = "rescale_type")]
    pub rescale_type: Option<BoundedString<64>>,

    #[serde(rename = "window_center")]
    pub window_center: Option<BoundedString<64>>,

    #[serde(rename = "window_width")]
    pub window_width: Option<BoundedString<64>>,

    #[serde(rename = "transfer_syntax_uid")]
    pub transfer_syntax_uid: BoundedString<64>,

    #[serde(rename = "pixel_data_location")]
    pub pixel_data_location: Option<BoundedString<512>>,

    #[serde(rename = "thumbnail_location")]
    pub thumbnail_location: Option<BoundedString<512>>,

    #[serde(rename = "sop_class_uid")]
    pub sop_class_uid: BoundedString<64>,

    #[serde(rename = "image_status")]
    pub image_status: Option<BoundedString<32>>,

    #[serde(rename = "space_size")]
    pub space_size: Option<i64>,

    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,

    #[serde(rename = "updated_time")]
    pub updated_time: NaiveDateTime,
}
