use chrono::{NaiveDateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::dicom_dbtype::{BoundedString, FixedLengthString};

/// DICOM访问日志实体，用于记录所有访问操作
/// 包含用户、操作、资源和时间等信息，以满足医疗系统合规要求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DicomAccessLog {
    /// 全局唯一追踪ID
    #[serde(rename = "log_id")]
    pub log_id: FixedLengthString<36>,

    /// 租户ID
    #[serde(rename = "tenant_id")]
    pub tenant_id: BoundedString<64>,

    /// 用户ID
    #[serde(rename = "user_id")]
    pub user_id: BoundedString<64>,

    /// 用户名
    #[serde(rename = "username")]
    pub username: BoundedString<128>,

    /// 操作类型 (READ, WRITE, DELETE, QUERY等)
    #[serde(rename = "operation_type")]
    pub operation_type: BoundedString<32>,

    /// 操作路径
    #[serde(rename = "operation_path")]
    pub operation_path: BoundedString<512>,

    /// 操作方法 (GET, POST, PUT, DELETE等)
    #[serde(rename = "operation_method")]
    pub operation_method: BoundedString<10>,

    /// 操作结果 (SUCCESS, FAILED)
    #[serde(rename = "operation_result")]
    pub operation_result: BoundedString<16>,

    /// 操作的资源类型 (STUDY, SERIES, INSTANCE等)
    #[serde(rename = "resource_type")]
    pub resource_type: BoundedString<32>,

    /// 资源ID (StudyUID, SeriesUID, SOPInstanceUID)
    #[serde(rename = "resource_id")]
    pub resource_id: BoundedString<64>,

    /// IP地址
    #[serde(rename = "ip_address")]
    pub ip_address: BoundedString<45>,

    /// 用户代理
    #[serde(rename = "user_agent")]
    pub user_agent: Option<BoundedString<512>>,

    /// 响应时间 (毫秒)
    #[serde(rename = "response_time")]
    pub response_time: i64,

    /// 操作描述
    #[serde(rename = "description")]
    pub description: Option<BoundedString<1024>>,

    /// 创建时间
    #[serde(rename = "created_time")]
    pub created_time: NaiveDateTime,
}

impl DicomAccessLog {
    /// 创建新的访问日志记录
    pub fn new(
        tenant_id: &str,
        user_id: &str,
        username: &str,
        operation_type: &str,
        operation_path: &str,
        operation_method: &str,
        operation_result: &str,
        resource_type: &str,
        resource_id: &str,
        ip_address: &str,
        user_agent: Option<&str>,
        response_time: i64,
        description: Option<&str>,
    ) -> Self {
        Self {
            log_id: FixedLengthString::<36>::make(uuid::Uuid::new_v4().to_string()),
            tenant_id: BoundedString::<64>::make_str(tenant_id),
            user_id: BoundedString::<64>::make_str(user_id),
            username: BoundedString::<128>::make_str(username),
            operation_type: BoundedString::<32>::make_str(operation_type),
            operation_path: BoundedString::<512>::make_str(operation_path),
            operation_method: BoundedString::<10>::make_str(operation_method),
            operation_result: BoundedString::<16>::make_str(operation_result),
            resource_type: BoundedString::<32>::make_str(resource_type),
            resource_id: BoundedString::<64>::make_str(resource_id),
            ip_address: BoundedString::<45>::make_str(ip_address),
            user_agent: user_agent.map(|ua| BoundedString::<512>::make_str(ua)),
            response_time,
            description: description.map(|desc| BoundedString::<1024>::make_str(desc)),
            created_time: Utc::now().naive_utc(),
        }
    }
}
