use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct ApiLogEvent {
    pub timestamp: NaiveDateTime,
    pub tenant_id:String,
    pub request_id: String,
    pub method: String,
    pub path: String,
    pub query_params: String,
    pub peer_addr: String,
    pub headers: String,
    pub user: String,
    pub user_id: String,
    pub status: u16,
    pub content_length: String,
    pub duration_ms: u64,
}
