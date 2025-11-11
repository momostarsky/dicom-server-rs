use actix_web::HttpRequest;
use common::dicom_json_helper::walk_directory;
use common::dicom_utils::get_tag_values;
use common::storage_config::{dicom_series_dir, json_metadata_for_series};
use database::dicom_meta::DicomStateMeta;
use dicom_dictionary_std::tags;
use dicom_object::collector::CharacterSetOverride;
use dicom_object::OpenFileOptions;
use serde_json::{json, Map};
use std::collections::HashMap;
use std::io::Error;
use tokio::task;

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

pub async fn generate_series_json(series_info: &DicomStateMeta) -> Result<String, Error> {
    let json_file_path = match json_metadata_for_series(&series_info, true) {
        Ok(v) => v,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to get json_file_path for generate: {}", e),
            ));
        }
    };

    let dicom_dir = match dicom_series_dir(series_info, false) {
        Ok(vv) => vv,
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to retrieve dicom_dir",
            ));
        }
    };

    let files = match walk_directory(dicom_dir) {
        Ok(files) => files,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to walk directory: {}", e),
            ));
        }
    };
    let mut handles = vec![];

    for file_path in &files {
        // 读取 DICOM 文件内容
        let file_path_clone = file_path.clone(); // 克隆路径供异步任务使用
        let handle = task::spawn_blocking(move || {
            // 读取 DICOM 文件内容
            let sop_json = match OpenFileOptions::new()
                .charset_override(CharacterSetOverride::AnyVr)
                .read_until(tags::PIXEL_DATA)
                .open_file(&file_path_clone)
            {
                Ok(dicom_object) => {
                    let mut dicom_json = Map::new();
                    dicom_object.tags().into_iter().for_each(|tag| {
                        let value_str: Vec<String> = get_tag_values(tag, &dicom_object);
                        let vr = dicom_object.element(tag).expect("REASON").vr().to_string();
                        let tag_key = format!("{:04X}{:04X}", tag.group(), tag.element());
                        let element_json = json!({
                            "vr": vr,
                            "Value": value_str
                        });
                        dicom_json.insert(tag_key, element_json);
                    });
                    Ok(dicom_json)
                }
                Err(e) => Err(format!(
                    "Failed to read DICOM file {}: {}",
                    file_path_clone.display(),
                    e
                )),
            };
            sop_json
        });
        handles.push(handle);
    }

    // 等待所有任务完成
    let mut arr = vec![];
    for handle in handles {
        match handle.await {
            Ok(result) => match result {
                Ok(sop_json) => arr.push(sop_json),
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to walk directory: {}", e),
                    ));
                }
            },
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to walk directory: {}", e),
                ));
            }
        }
    }

    let json = match serde_json::to_string(&arr) {
        Ok(json) => {
            if let Err(e) = std::fs::write(&json_file_path, &json) {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Failed to write  JSON to file: {}", e),
                ));
            }
            json
        }
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to generate JSON: {}", e),
            ));
        }
    };
    Ok(json)
}
