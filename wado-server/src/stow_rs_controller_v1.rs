use actix_web::{HttpRequest, HttpResponse, Result, http::header, post, web};
use futures_util::StreamExt as _;
// use dicom_object::open_file; // 如果需要解析 DICOM，取消注释
use crate::AppState;
use crate::constants::STOW_RS_TAG;
use slog::{error, info, warn};
use std::io::Write;
use actix_multipart::{
    Multipart,
    form::{
        MultipartForm,
        tempfile::{TempFile, TempFileConfig},
    },
};
const MULTIPART_CONTENT_TYPE: &str = "multipart/related"; // multipart/related


/// 假设的元数据结构（根据您的实际 STOW-RS 需求调整）
#[derive(serde::Deserialize, Debug)]
struct StowMetadata {
    patient_id: String,
    study_uid: String,
}

/// 处理 /studies 端点的 POST 请求
/// 接收 multipart/related 数据流并保存 DICOM 实例
#[utoipa::path(
    post,
    params(
        ("x-tenant" = String, Header, description = "Tenant ID from request header"),
        ("Accept" =  String, Header, example="application/json", description = "Accept Content Type: application/dicom+json or application/json"),
        ("Authorization" = Option<String>, Header,   description = "Optional JWT Access Token in Bearer format")
    ),
    responses(
        (status = 200, description = "Store DICOM Instance  successfully", content_type = "application/json"),
        (status = 404, description = "Study not found"),
        (status = 406, description = " Accept header must be multipart/related"),
        (status = 500, description = "Internal server error")
    ),
    tag =  STOW_RS_TAG,
    description = "Save DICOM Files",
)]
#[post("/studies")]
async fn store_instances(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    mut payload: Multipart,
) -> Result<HttpResponse> {
    let log = app_state.log.clone();
    info!(log, "Received POST request on /studies");

    // 1. 检查 Content-Type 是否为 multipart/related
    let content_type_header = req.headers().get(header::CONTENT_TYPE);
    info!(log, "Content-Type: {:?}", content_type_header);

    match content_type_header {
        Some(ct_header_value) => {
            if let Ok(ct_str) = ct_header_value.to_str() {
                // 简单检查是否包含 multipart/related
                // 更严格的检查需要解析整个 Content-Type header (e.g., boundary)
                if !ct_str.to_lowercase().starts_with(MULTIPART_CONTENT_TYPE) {
                    warn!(log, "Invalid Content-Type for STOW-RS: {}", ct_str);
                    return Ok(HttpResponse::UnsupportedMediaType()
                        .body("Content-Type must be multipart/related"));
                }
                info!(log, "Content-Type confirmed as multipart/related");
            } else {
                error!(log, "Failed to parse Content-Type header value");
                return Ok(HttpResponse::BadRequest().body("Malformed Content-Type header"));
            }
        }
        None => {
            warn!(log, "Missing Content-Type header");
            return Ok(HttpResponse::BadRequest().body("Missing Content-Type header"));
        }
    }

    // 2. 处理 multipart payload
    // Actix-web 提供了 Multipart extractor，但它更适合 form-data。
    // 对于 multipart/related，我们需要手动处理流。
    // 这里我们简化处理，直接读取 payload 并尝试写入文件。

    // --- 简化版处理逻辑 ---
    // 实际应用中，你需要根据 Content-Type 中的 boundary 来正确分割 parts.
    // 这个例子只是将整个流保存下来作为一个示例。

    let mut file_data = Vec::new();
    while let Some(chunk) = payload.next().await {
        match chunk {
            Ok(data) => {
                file_data.extend_from_slice(&data);
            }
            Err(e) => {
                error!(log,"Error reading payload chunk: {}", e);
                return Ok(HttpResponse::InternalServerError().body("Error reading data"));
            }
        }
    }

    // --- 模拟处理: 将接收到的数据保存到文件 ---
    // 注意：真实的 STOW-RS 需要解析 multipart/related，提取每个 DICOM part，
    // 验证 DICOM 格式，并存储到合适的 Study/Series 结构中。
    let filename = format!("received_stow_data_{}.bin", uuid::Uuid::new_v4()); // 简单命名
    match std::fs::File::create(&filename) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&file_data) {
                error!(log,"Failed to write received data to file {}: {}", filename, e);
                return Ok(HttpResponse::InternalServerError().body("Failed to save data"));
            }
            info!(log,"Saved raw multipart/related data to {}", filename);
        }
        Err(e) => {
            error!(log,"Failed to create file {}: {}", filename, e);
            return Ok(HttpResponse::InternalServerError().body("Failed to create storage file"));
        }
    }

    // 3. 构造简单的成功响应 (实际应包含 StoreInstanceResponse XML/JSON)
    // 参考 DICOM PS3.18 Annex CC.2.2 for response format
    // 这里仅返回 200 OK 作为示意
    info!(log,"STOW-RS request processed successfully (simplified)");
    Ok(HttpResponse::Ok()
        // .content_type("application/dicom+xml") // 或 application/json 根据实际情况
        .body("<NativeDicomModel><Message><Status>Success</Status></Message></NativeDicomModel>")) // 简化的 XML 响应
}

/// 处理 /studies/{study_instance_uid} 端点的 POST 请求
/// 接收 multipart/related 数据流并保存 DICOM 实例到指定 Study
#[utoipa::path(
    post,
    params(
        ("study_instance_uid" = String, Path, description = "Study Instance UID"),
        ("x-tenant" = String, Header, description = "Tenant ID from request header"),
        ("Accept" =  String, Header, example="application/json", description = "Accept Content Type: application/dicom+json or application/json"),
        ("Authorization" = Option<String>, Header,   description = "Optional JWT Access Token in Bearer format")
    ),
    responses(
        (status = 200, description = "Store DICOM Instance  successfully", content_type = "application/json"),
        (status = 404, description = "Study not found"),
        (status = 406, description = " Accept header must be multipart/related"),
        (status = 500, description = "Internal server error")
    ),
    tag =  STOW_RS_TAG,
    description = "Save DICOM Files",
)]
#[post("/studies/{study_instance_uid}")]
async fn store_instances_to_study(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    path: web::Path<String>, // 获取 study_instance_uid
    mut payload: web::Payload,
) -> Result<HttpResponse> {
    let log = app_state.log.clone();
    let study_instance_uid = path.into_inner(); // 提取路径参数
    info!(
        log,
        "Received POST request on /studies/{}", study_instance_uid
    );

    // 1. 检查 Content-Type 是否为 multipart/related (同上)
    let content_type_header = req.headers().get(header::CONTENT_TYPE);
    if let Some(ct_header_value) = content_type_header {
        if let Ok(ct_str) = ct_header_value.to_str() {
            if !ct_str.to_lowercase().starts_with("multipart/related") {
                warn!(log,"Invalid Content-Type for STOW-RS: {}", ct_str);
                return Ok(HttpResponse::UnsupportedMediaType()
                    .body("Content-Type must be multipart/related"));
            }
            info!(log,"Content-Type confirmed as multipart/related for specific study");
        } else {
            error!(log,"Failed to parse Content-Type header value");
            return Ok(HttpResponse::BadRequest().body("Malformed Content-Type header"));
        }
    } else {
        warn!(log,"Missing Content-Type header");
        return Ok(HttpResponse::BadRequest().body("Missing Content-Type header"));
    }

    // 2. 处理 multipart payload (同上 - 简化版)
    let mut file_data = Vec::new();
    while let Some(chunk) = payload.next().await {
        match chunk {
            Ok(data) => {
                file_data.extend_from_slice(&data);
            }
            Err(e) => {
                error!(log,"Error reading payload chunk: {}", e);
                return Ok(HttpResponse::InternalServerError().body("Error reading data"));
            }
        }
    }

    // --- 模拟处理: 将接收到的数据保存到文件 ---
    let filename = format!(
        "received_stow_data_for_study_{}_{}.bin",
        study_instance_uid,
        uuid::Uuid::new_v4()
    );
    match std::fs::File::create(&filename) {
        Ok(mut file) => {
            if let Err(e) = file.write_all(&file_data) {
                error!(log,"Failed to write received data to file {}: {}", filename, e);
                return Ok(HttpResponse::InternalServerError().body("Failed to save data"));
            }
            info!(log,
                "Saved raw multipart/related data for study {} to {}",
                study_instance_uid,
                filename
            );
        }
        Err(e) => {
            error!(log,"Failed to create file {}: {}", filename, e);
            return Ok(HttpResponse::InternalServerError().body("Failed to create storage file"));
        }
    }

    // 3. 构造简单的成功响应 (同上)
    info!(log,
        "STOW-RS request for study {} processed successfully (simplified)",
        study_instance_uid
    );
    Ok(HttpResponse::Ok()
        // .content_type("application/dicom+xml")
        .body("<NativeDicomModel><Message><Status>Success</Status></Message></NativeDicomModel>"))

    // TODO: 实际实现中，需要:
    // - 解析 multipart/related 流
    // - 提取每个 part
    // - 验证每个 part 是有效的 DICOM 文件
    // - 检查 DICOM 文件头中的 Study Instance UID 是否与路径参数一致
    // - 将实例存储到指定的 Study 下
}
