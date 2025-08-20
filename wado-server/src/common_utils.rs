use actix_web::HttpRequest;
use std::collections::HashMap;
use common::server_config::LocalStorageConfig;

// 解析查询字符串，支持重复键
pub fn parse_query_string_case_insensitive(query: &str) -> HashMap<String, Vec<String>> {
    let mut params: HashMap<String, Vec<String>> = HashMap::new();

    for pair in query.split('&') {
        // 跳过空的键值对
        if pair.is_empty() {
            continue;
        }

        if let Some((key, value)) = pair.split_once('=') {
            let key = urlencoding::decode(key)
                .unwrap_or_else(|_| key.into())
                .into_owned();
            let value = urlencoding::decode(value)
                .unwrap_or_else(|_| value.into())
                .into_owned();
            // 去除参数值的首尾空格以提高兼容性
            let trimmed_value = value.trim().to_string();
            params
                .entry(key.to_lowercase())
                .or_insert_with(Vec::new)
                .push(trimmed_value);
        }
    }

    params
}

// pub fn genrate_study_meta(tenant_id: &str, study_instance_uid: &str, storage_config:&LocalStorageConfig) -> String {
//      
// 
// }

// 不区分大小写的参数获取
pub fn get_param_case_insensitive<'a>(
    params: &'a HashMap<String, Vec<String>>,
    key: &str,
) -> Option<&'a Vec<String>> {
    params.get(&key.to_lowercase())
}

static X_TENANT_HEADER: &str = "x-tenant";
static DEFAULT_TENANT_HEADER: &str = "1234567890";
pub(crate) fn get_tenant_from_handler(req: &HttpRequest) -> String {
    // 获取 X-Tenant 请求头
    let tenant_header = req.headers().get(X_TENANT_HEADER);

    match tenant_header {
        Some(value) => {
            // 将 HeaderValue 转换为字符串
            let tenant = value.to_str().unwrap_or_else(|_| DEFAULT_TENANT_HEADER);
            tenant.to_string() // 错误处理或默认值
        }
        None => {
            DEFAULT_TENANT_HEADER.to_string() // 默认值或错误处理
        }
    }
}
