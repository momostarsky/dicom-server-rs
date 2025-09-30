use chrono::{DateTime, NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogStatus {
    Success,
    Failed,
    Warning,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogType {
    DicomIngest,
    Transcode,
    Storage,
    WadoAccess,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomIngestLog {
    pub log_time: DateTime<Utc>,                     // 日志时间
    pub log_type: LogType,                           // 日志类型：DicomIngest
    pub status: LogStatus,                           // success / failed / warning
    pub trace_id: Uuid,                              // 唯一请求标识
    pub source_ip: String,                           // 客户端/设备 IP
    pub processing_step: String,                     // 如：c_store_received
    pub sop_instance_uid: String,                    // DICOM 唯一标识
    pub study_instance_uid: String,                  // 检查 UID
    pub series_instance_uid: String,                 // 序列 UID
    pub patient_id: String,                          // 患者 ID
    pub original_filename: Option<String>,           // 原始文件名（可选）
    pub transfer_syntax_uid: String,                 // 原始传输语法 UID
    pub file_size_bytes: u64,                        // 文件大小（字节）
    pub source_ae_title: Option<String>,             // 发送端 AE Title（可选）
    pub received_from: Option<String>,               // 接收来源描述（可选）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscodeLog {
    pub log_time: DateTime<Utc>,
    pub log_type: LogType,
    pub status: LogStatus,
    pub trace_id: Uuid,
    pub sop_instance_uid: String,
    pub original_transfer_syntax: String,            // 原始传输语法，如 JPEG2000
    pub target_transfer_syntax: String,              // 目标传输语法，如 JPEG
    pub transcode_status: LogStatus,                 // 转码是否成功
    pub transcode_error: Option<String>,             // 转码失败原因（可选）
    // 可选：转码耗时相关字段
    pub transcode_start_time: DateTime<Utc>,
    pub transcode_end_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageLog {
    pub log_time: DateTime<Utc>,
    pub log_type: LogType,
    pub status: LogStatus,
    pub trace_id: Uuid,
    pub sop_instance_uid: String,
    pub storage_type: String,                        // 如：filesystem, object_storage, database
    pub storage_path: Option<String>,                // 存储路径或 URL（可选）
    pub metadata_stored: bool,                       // 元数据是否成功存储
    pub dicom_stored: bool,                          // DICOM 文件是否成功存储
    pub store_error: Option<String>,                 // 存储失败时的错误信息（可选）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WadoAccessLog {
    pub log_time: DateTime<Utc>,
    pub log_type: LogType,
    pub status: LogStatus,
    pub trace_id: Uuid,
    pub study_instance_uid: String,
    pub series_instance_uid: String,
    pub sop_instance_uid: String,
    pub access_time: DateTime<Utc>,                  // 访问发生时间
    pub user_id: Option<String>,                     // 用户标识（如登录用户，可选）
    pub client_ip: String,                           // 客户端 IP
    pub response_status: u16,                        // HTTP 响应状态码，如 200, 404
    pub response_time_ms: u32,                       // 响应时间，单位毫秒
    pub requested_format: Option<String>,            // 如 image/jpeg, application/dicom（可选）
}