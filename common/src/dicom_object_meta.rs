use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug,Serialize,Deserialize)]
#[non_exhaustive]
pub enum TransferStatus {
    NoNeedTransfer,
    Success,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomObjectMeta {
    #[serde(rename = "trace_id")]
    pub trace_id: String,
    #[serde(rename = "tenant_id")]
    pub tenant_id: String,
    #[serde(rename = "patient_id")]
    pub patient_id:String,
    #[serde(rename = "study_uid")]
    pub study_uid: String,
    #[serde(rename = "series_uid")]
    pub series_uid: String,
    #[serde(rename = "sop_uid")]
    pub sop_uid: String,
    #[serde(rename = "file_size")]
    pub file_size: u64,
    #[serde(rename = "file_path")]
    pub file_path: String,
    #[serde(rename = "transfer_syntax_uid")]
    pub transfer_syntax_uid: String,
    #[serde(rename = "number_of_frames")]
    pub number_of_frames: i32,
    #[serde(rename = "created_time")]
    pub created_time:  NaiveDateTime,
    #[serde(rename = "updated_time")]
    pub updated_time: NaiveDateTime,
    #[serde(rename = "series_uid_hash")]
    pub series_uid_hash: u64,
    #[serde(rename = "study_uid_hash")]
    pub study_uid_hash: u64,
    #[serde(rename = "accession_number")]
    pub accession_number: String,
    #[serde(rename = "target_ts")]
    pub target_ts: String,
    #[serde(rename = "study_date")]
    pub study_date: String,
    #[serde(rename = "transfer_status")]
    pub transfer_status: TransferStatus,
    #[serde(rename = "source_ip")]
    pub source_ip: String,
    #[serde(rename = "source_ae")]
    pub source_ae: String,
}
// 为 DicomObjectMeta 实现 Hash trait 以便可以在 HashSet 中使用
impl Hash for DicomObjectMeta {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tenant_id.hash(state);
        self.patient_id.hash(state);
        self.study_uid.hash(state);
        self.series_uid.hash(state);
        self.sop_uid.hash(state);
    }
}
impl PartialEq for DicomObjectMeta {
    fn eq(&self, other: &Self) -> bool {
        self.tenant_id == other.tenant_id
            && self.patient_id == other.patient_id
            && self.study_uid == other.study_uid
            && self.series_uid == other.series_uid
            && self.sop_uid == other.sop_uid
    }
}
impl Eq for DicomObjectMeta {}