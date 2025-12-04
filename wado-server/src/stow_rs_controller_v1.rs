use actix_web::{HttpRequest, HttpResponse, Result, http::header, post, web};

use futures_util::StreamExt as _;
// use dicom_object::open_file; // 如果需要解析 DICOM，取消注释
use crate::AppState;
use crate::constants::STOW_RS_TAG;
use multer::Multipart;
use slog::{error, info, warn};
use std::io::Write;

fn parse_multipart_related_content_type(
    content_type: &str,
) -> Option<(String, Option<String>, Option<String>)> {
    let parts: Vec<&str> = content_type.split(';').collect();
    let mime_type = parts[0].trim().to_lowercase();

    if mime_type != "multipart/related" {
        return None;
    }

    let mut boundary = None;
    let mut subtype = None;

    for part in &parts[1..] {
        let trimmed = part.trim();
        if trimmed.starts_with("boundary=") {
            boundary = Some(trimmed[9..].trim_matches('"').to_string());
        } else if trimmed.starts_with("type=") {
            subtype = Some(trimmed[5..].trim_matches('"').to_string());
        }
    }

    Some((mime_type, subtype, boundary))
}

/// 处理 /studies 端点的 POST 请求
/// 接收 multipart/related 数据流并保存 DICOM 实例
#[utoipa::path(
    post,
    params(
        ("x-tenant" = String, Header, description = "Tenant ID from request header"),
        ("Accept" =  String, Header, example="application/json", description = "Accept Content Type: application/dicom  or application/json"),
        ("Authorization" = Option<String>, Header,   description = "Optional JWT Access Token in Bearer format")
    ),
    responses(
        (status = 200, description = "Store DICOM Instance  successfully", content_type = "application/json"),
        (status = 404, description = "Study not found"),
        (status = 406, description = "Accept header must be multipart/related"),
        (status = 500, description = "Internal server error")
    ),
    tag =  STOW_RS_TAG,
    description = "Save DICOM Files",
)]
#[post("/studies")]
async fn store_instances(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    payload: web::Payload,
) -> Result<HttpResponse> {
    let log = app_state.log.clone();
    info!(log, "Received POST request on /studies");

    // 处理请求并保存实例
    process_and_store_instances(req, payload, log, None).await
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
    payload: web::Payload,
) -> Result<HttpResponse> {
    let log = app_state.log.clone();
    let study_instance_uid = path.into_inner(); // 提取路径参数
    info!(
        log,
        "Received POST request on /studies/{}", study_instance_uid
    );

    // 处理请求并保存实例到指定study
    process_and_store_instances(req, payload, log, Some(study_instance_uid)).await
}

/// 解析 Content-Type 中的 boundary 信息
async fn parse_boundary_info(
    req: &HttpRequest,
    log: &slog::Logger,
) -> Result<String, HttpResponse> {
    let content_type_header = req.headers().get(header::CONTENT_TYPE);
    info!(log, "Content-Type: {:?}", content_type_header);

    let boundary = match content_type_header {
        Some(ct_header_value) => {
            if let Ok(ct_str) = ct_header_value.to_str() {
                match parse_multipart_related_content_type(ct_str) {
                    Some((_mime_type, subtype, boundary)) => {
                        info!(log, "Content-Type maintype:{:?}", _mime_type);
                        info!(log, "Content-Type subtype: {:?}", subtype);
                        info!(log, "Content-Type boundary:{:?}", boundary);
                        boundary
                    }
                    None => {
                        warn!(log, "Invalid Content-Type for STOW-RS: {}", ct_str);
                        return Err(HttpResponse::UnsupportedMediaType()
                            .body("Content-Type must be multipart/related"));
                    }
                }
            } else {
                error!(log, "Failed to parse Content-Type header value");
                return Err(HttpResponse::BadRequest().body("Malformed Content-Type header"));
            }
        }
        None => {
            warn!(log, "Missing Content-Type header");
            return Err(HttpResponse::BadRequest().body("Missing Content-Type header"));
        }
    };

    if boundary.is_none() {
        warn!(log, "Missing boundary in Content-Type header");
        return Err(HttpResponse::BadRequest().body("Missing boundary in Content-Type header"));
    }

    Ok(boundary.unwrap())
}

/// 处理并存储 DICOM 实例的主逻辑
async fn process_and_store_instances(
    req: HttpRequest,
    mut payload: web::Payload,
    log: slog::Logger,
    study_instance_uid: Option<String>,
) -> Result<HttpResponse> {
    // 解析 boundary 信息
    let boundary = match parse_boundary_info(&req, &log).await {
        Ok(boundary) => boundary,
        Err(response) => return Ok(response),
    };

    let content_length = req.headers().get(header::CONTENT_LENGTH)
        .and_then(|hv| hv.to_str().ok())
        .and_then(|s| s.parse::<usize>().ok());

    // 根据 Content-Length 动态设置容量
    let initial_capacity = match content_length {
        Some(len) if len > 0 && len < 50 * 1024 * 1024 => {
            // 使用实际长度的 1.2 倍作为容量，但不超过 10MB
            std::cmp::min((len as f64 * 1.2) as usize, 10 * 1024 * 1024)
        }
        _ => 512 * 1024, // 默认 512KB
    };


    info!(log, "Initial capacity: {}", initial_capacity);
    // 收集 multipart 数据到内存缓冲区
    let mut buffer = Vec::with_capacity(initial_capacity); // 512KB
    while let Some(chunk) = payload.next().await {
        match chunk {
            Ok(data) => {
                buffer.extend_from_slice(&data);
            }
            Err(e) => {
                error!(log, "Error reading payload chunk: {}", e);
                return Ok(HttpResponse::InternalServerError().body("Error reading data"));
            }
        }
    }

    // 使用 multer 处理流
    let mut multipart = Multipart::with_reader(buffer.as_slice(), boundary.as_str());

    // 处理 multipart 字段
    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => {
                // 处理字段内容
                let field_name = match field.name() {
                    Some(name) => name.to_string(),
                    _ => "UN".to_string(),
                };
                let field_content_type = match field.content_type() {
                    Some(content_type) => content_type.to_string(),
                    _ => "UN".to_string(),
                };
                info!(
                    log,
                    "Processing field: name={:?}, content_type={:?}",
                    field_name,
                    field_content_type
                );

                if !(field_content_type == "application/json"
                    || field_content_type == "application/dicom")
                {
                    warn!(
                        log,
                        "Field content type is missing, defaulting to application/dicom"
                    );
                    continue;
                }

                let mut field_bytes_len = 0;
                loop {
                    match field.chunk().await {
                        Ok(Some(field_chunk)) => {
                            // 保存 DICOM 文件
                            let filename = uuid::Uuid::new_v4().to_string();
                            let filepath = match field_content_type.as_str() {
                                "application/dicom" => {
                                    if let Some(ref uid) = study_instance_uid {
                                        format!("./{}_{}.dcm", uid, filename)
                                    } else {
                                        format!("./{}.dcm", filename)
                                    }
                                }
                                "application/json" => {
                                    if let Some(ref uid) = study_instance_uid {
                                        format!("./{}_{}.json", uid, filename)
                                    } else {
                                        format!("./{}.json", filename)
                                    }
                                }
                                _ => {
                                    if let Some(ref uid) = study_instance_uid {
                                        format!("./{}_{}.data", uid, filename)
                                    } else {
                                        format!("./{}.data", filename)
                                    }
                                }
                            };

                            match std::fs::File::create(&filepath) {
                                Ok(mut file) => {
                                    if let Err(e) = file.write_all(&field_chunk) {
                                        error!(
                                            log,
                                            "Failed to write DICOM file {}: {}", filepath, e
                                        );
                                        return Ok(HttpResponse::InternalServerError()
                                            .body("Failed to save DICOM file"));
                                    }
                                    info!(log, "Saved DICOM file to: {}", filepath);
                                }
                                Err(e) => {
                                    error!(log, "Failed to create file {}: {}", filepath, e);
                                    return Ok(HttpResponse::InternalServerError()
                                        .body("Failed to create storage file"));
                                }
                            }

                            field_bytes_len += field_chunk.len();
                        }
                        Ok(None) => break, // 没有更多数据块了
                        Err(e) => {
                            error!(log, "Error reading field chunk: {}", e);
                            return Ok(HttpResponse::BadRequest()
                                .body(format!("Error reading field data: {}", e)));
                        }
                    }
                }

                info!(log, "Field Bytes Length: {:?}", field_bytes_len);
            }
            Ok(None) => break, // 没有更多字段了
            Err(e) => {
                error!(log, "Error parsing multipart field: {}", e);
                return Ok(
                    HttpResponse::BadRequest().body(format!("Error parsing multipart data: {}", e))
                );
            }
        }
    }

    // 构造响应
    if let Some(uid) = study_instance_uid {
        info!(
            log,
            "STOW-RS request for study {} processed successfully (simplified)", uid
        );
    } else {
        info!(log, "STOW-RS request processed successfully (simplified)");
    }

    Ok(HttpResponse::Ok()
        .body("<NativeDicomModel><Message><Status>Success</Status></Message></NativeDicomModel>"))
}
