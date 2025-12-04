use actix_web::{HttpRequest, HttpResponse, Result, http::header, post, web};

use futures_util::StreamExt as _;
// use dicom_object::open_file; // 如果需要解析 DICOM，取消注释
use crate::AppState;
use crate::constants::STOW_RS_TAG;
use multer::{Multipart, bytes};
use slog::{error, info, warn};
use std::io::Write;

const MULTIPART_CONTENT_TYPE: &str = "multipart/related"; // multipart/related

/// 假设的元数据结构（根据您的实际 STOW-RS 需求调整）
#[derive(serde::Deserialize, Debug)]
struct StowMetadata {
    patient_id: String,
    study_uid: String,
}
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
// 移除之前的 unfold 代码，改用 multer 的流式处理
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, ReadBuf};

// 创建一个适配器将 actix-web Payload 转换为 AsyncRead
struct PayloadReader {
    payload: web::Payload,
    buffer: Option<bytes::Bytes>,
}

impl PayloadReader {
    fn new(payload: web::Payload) -> Self {
        Self {
            payload,
            buffer: None,
        }
    }
}

impl AsyncRead for PayloadReader {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        loop {
            if let Some(data) = self.buffer.as_mut() {
                let len = std::cmp::min(buf.remaining(), data.len());
                buf.put_slice(&data[..len]);
                if len < data.len() {
                    *data = data.split_off(len);
                } else {
                    self.buffer = None;
                }
                return Poll::Ready(Ok(()));
            }

            match futures_util::ready!(self.payload.poll_next_unpin(cx)) {
                Some(Ok(bytes)) => {
                    self.buffer = Some(bytes);
                }
                Some(Err(e)) => {
                    return Poll::Ready(Err(std::io::Error::new(std::io::ErrorKind::Other, e)));
                }
                None => return Poll::Ready(Ok(())),
            }
        }
    }
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
    mut payload: web::Payload,
) -> Result<HttpResponse> {
    let log = app_state.log.clone();
    info!(log, "Received POST request on /studies");

    // 1. 检查 Content-Type 是否为 multipart/related
    let content_type_header = req.headers().get(header::CONTENT_TYPE);
    info!(log, "Content-Type: {:?}", content_type_header);

    let boundary = match content_type_header {
        Some(ct_header_value) => {
            info!(log, "xxxContent-Type: {:?}", ct_header_value);
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
                        return Ok(HttpResponse::UnsupportedMediaType()
                            .body("Content-Type must be multipart/related"));
                    }
                }
            } else {
                error!(log, "Failed to parse Content-Type header value");
                return Ok(HttpResponse::BadRequest().body("Malformed Content-Type header"));
            }
        }
        None => {
            warn!(log, "Missing Content-Type header");
            return Ok(HttpResponse::BadRequest().body("Missing Content-Type header"));
        }
    };
    if boundary.is_none() {
        warn!(log, "Missing boundary in Content-Type header");
        return Ok(HttpResponse::BadRequest().body("Missing boundary in Content-Type header"));
    }

    // 替换现有的文件数据收集逻辑
    let boundary = boundary.unwrap(); // 已经检查过不为 None

    // 使用临时文件缓冲整个 payload
    let temp_file_path = format!("./temp_payload_{}.dat", uuid::Uuid::new_v4());
    {
        let temp_file_path_write = temp_file_path.clone();
        let mut file = web::block(move || std::fs::File::create(temp_file_path_write)).await??;
        while let Some(chunk) = payload.next().await {
            match chunk {
                Ok(data) => {
                    info!(log, "Received chunk of size: {}", data.len());
                    let prefix = format!("Received chunk of size: {:?}",String::from_utf8(data[0..30].to_vec()));
                    info!(log, "{}", prefix);

                    file = web::block(move || file.write_all(&data).map(|_| file)).await??;
                }
                Err(e) => {
                    error!(log, "Error reading payload chunk: {}", e);
                    // 清理临时文件
                    let _ = std::fs::remove_file(&temp_file_path);
                    return Ok(HttpResponse::InternalServerError().body("Error reading data"));
                }
            }
        }
    }


    // 从临时文件读取数据并处理 multipart
    let file_content = {
        let temp_file_path_read = temp_file_path.clone();
        web::block(move || std::fs::read(temp_file_path_read)).await??
    };
    // 清理临时文件
    let _ = std::fs::remove_file(&temp_file_path);

    // Create a `Multipart` instance from that async reader and the boundary.
    // 使用 multer 处理流
    let mut multipart = Multipart::with_reader(file_content.as_slice(), boundary.as_str());
    // 在清理后添加调试信息

    // Iterate over the fields, use `next_field()` to get the next field.
    // 在循环中处理错误
    // 将原来的 while let 循环替换为:
    loop {
        match multipart.next_field().await {
            Ok(Some(mut field)) => {
                // 处理字段内容
                // 使用 match 表达式更清晰地处理
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
                                    format!("./{}.dcm", filename)
                                }
                                "application/json" => {
                                    format!("./{}.json", filename)
                                }
                                _ => {
                                    format!("./{}.data", filename)
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

    info!(log, "STOW-RS request processed successfully (simplified)");
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
    match content_type_header {
        Some(ct_header_value) => {
            info!(log, "xxxContent-Type: {:?}", ct_header_value);
            if let Ok(ct_str) = ct_header_value.to_str() {
                match parse_multipart_related_content_type(ct_str) {
                    Some((_mime_type, subtype, boundary)) => {
                        info!(log, "Content-Type confirmed as multipart/related");
                        info!(log, "Content-Type subtype: {:?}", subtype);
                        info!(log, "Content-Type boundary:{:?}", boundary);
                    }
                    None => {
                        warn!(log, "Invalid Content-Type for STOW-RS: {}", ct_str);
                        return Ok(HttpResponse::UnsupportedMediaType()
                            .body("Content-Type must be multipart/related"));
                    }
                }
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

    // 2. 处理 multipart payload (同上 - 简化版)
    let mut file_data = Vec::new();
    while let Some(chunk) = payload.next().await {
        match chunk {
            Ok(data) => {
                file_data.extend_from_slice(&data);
            }
            Err(e) => {
                error!(log, "Error reading payload chunk: {}", e);
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
                error!(
                    log,
                    "Failed to write received data to file {}: {}", filename, e
                );
                return Ok(HttpResponse::InternalServerError().body("Failed to save data"));
            }
            info!(
                log,
                "Saved raw multipart/related data for study {} to {}", study_instance_uid, filename
            );
        }
        Err(e) => {
            error!(log, "Failed to create file {}: {}", filename, e);
            return Ok(HttpResponse::InternalServerError().body("Failed to create storage file"));
        }
    }

    // 3. 构造简单的成功响应 (同上)
    info!(
        log,
        "STOW-RS request for study {} processed successfully (simplified)", study_instance_uid
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
