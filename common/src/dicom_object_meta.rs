use crate::string_ext::{BoundedString, DicomDateString, SopUidString, UuidString};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::hash::{Hash, Hasher};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TransferStatus {
    NoNeedTransfer,
    Success,
    Failed,
}
/// DicomStoreMeta 用于DICOM-CStoreSCP服务记录收图日志.
/// 包含了所有必要的元数据字段.每一个DicomStoreMeta实例标识接收一个DICOM文件.并成功写入磁盘.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomStoreMeta {
    #[serde(rename = "trace_id")]
    pub trace_id: UuidString,
    #[serde(rename = "worker_node_id")]
    pub worker_node_id: BoundedString<64>,
    #[serde(rename = "tenant_id")]
    pub tenant_id: BoundedString<64>,
    #[serde(rename = "patient_id")]
    pub patient_id: BoundedString<64>,
    #[serde(rename = "study_uid")]
    pub study_uid: SopUidString,
    #[serde(rename = "series_uid")]
    pub series_uid: SopUidString,
    #[serde(rename = "sop_uid")]
    pub sop_uid: SopUidString,
    #[serde(rename = "file_size")]
    pub file_size: u64,
    #[serde(rename = "file_path")]
    pub file_path: BoundedString<512>,
    #[serde(rename = "transfer_syntax_uid")]
    pub transfer_syntax_uid: SopUidString,
    #[serde(rename = "number_of_frames")]
    pub number_of_frames: i32,
    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,
    #[serde(rename = "series_uid_hash")]
    pub series_uid_hash: u32,
    #[serde(rename = "study_uid_hash")]
    pub study_uid_hash: u64,
    #[serde(rename = "accession_number")]
    pub accession_number: BoundedString<64>,
    #[serde(rename = "target_ts")]
    pub target_ts: SopUidString,
    #[serde(rename = "study_date")]
    pub study_date: DicomDateString,
    #[serde(rename = "transfer_status")]
    pub transfer_status: TransferStatus,
    #[serde(rename = "source_ip")]
    pub source_ip: BoundedString<32>,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub enum PerisitStatus {
    /// 默认状态
    Unknown,
    /// 解析失败
    ParseTagFailed,
    /// 写入数据库失败
    WriteToDatabaseFailed,
    /// 写入数据库成功
    WriteToDatabaseSuccess,
}

/// DicomParseMeta 用于记录DICOM文件元数据入库前的解析日志.
/// 包含了所有必要的元数据字段.每一个DicomParseMeta实例标识解析一个DICOM文件.并成功提取元数据.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomPerisitMeta {
    #[serde(rename = "trace_id")]
    pub trace_id: UuidString,
    #[serde(rename = "worker_node_id")]
    pub worker_node_id: BoundedString<64>,
    #[serde(rename = "tenant_id")]
    pub tenant_id: BoundedString<64>,
    #[serde(rename = "patient_id")]
    pub patient_id: BoundedString<64>,
    #[serde(rename = "study_uid")]
    pub study_uid: SopUidString,
    #[serde(rename = "series_uid")]
    pub series_uid: SopUidString,
    #[serde(rename = "sop_uid")]
    pub sop_uid: SopUidString,
    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,
    #[serde(rename = "persist_state")]
    pub persist_state: PerisitStatus,
    #[serde(rename = "persist_time")]
    pub persist_time: NaiveDateTime,
    #[serde(rename = "persist_message")]
    pub persist_message: BoundedString<512>,
}
