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
        ("Content-Type" =  String, Header, example="multipart/related; boundary=6c17d7b275f94d93f0b2a8c3d9xj; type=application/dicom+json", description = "Accept Content Type: application/dicom  or application/json"),
        ("Content-Length" =  u32, Header, example="5120000", description = "Content Length of the request body"),
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
        ("Content-Type" =  String, Header, example="multipart/related; boundary=6c17d7b275f94d93f0b2a8c3d9xj; type=application/dicom+json", description = "Accept Content Type: application/dicom  or application/json"),
        ("Content-Length" = u32, Header, example="5120000", description = "Content Length of the request body"),
        ("Accept" =  String, Header, example="application/json", description = "Accept Content Type: application/dicom+json or application/json"),
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

    // 修改为必须存在的 Content-Length
    let content_length = match req.headers().get(header::CONTENT_LENGTH) {
        Some(header_value) => match header_value.to_str() {
            Ok(value_str) => match value_str.parse::<usize>() {
                Ok(length) => length,
                Err(_) => {
                    error!(
                        log,
                        "Failed to parse Content-Length header value: {}", value_str
                    );
                    return Ok(
                        HttpResponse::BadRequest().body("Invalid Content-Length header value")
                    );
                }
            },
            Err(_) => {
                error!(log, "Failed to convert Content-Length header to string");
                return Ok(HttpResponse::BadRequest().body("Malformed Content-Length header"));
            }
        },
        None => {
            error!(log, "Missing required Content-Length header");
            return Ok(HttpResponse::BadRequest().body("Content-Length header is required"));
        }
    };

    // 根据 Content-Length 动态设置容量
    let initial_capacity = if content_length > 0 && content_length < 50 * 1024 * 1024 {
        // 使用实际长度的 1.2 倍作为容量，但不超过 10MB
        std::cmp::min((content_length as f64 * 1.2) as usize, 10 * 1024 * 1024)
    } else {
        512 * 1024 // 默认 512KB
    };

    // 策略: 小于10MB使用内存缓冲区，大于等于10MB使用内存映射文件
    const MEMORY_MAPPING_THRESHOLD: usize = 10 * 1024 * 1024; // 10MB阈值
    let use_memory_mapping = content_length >= MEMORY_MAPPING_THRESHOLD;

    info!(
        log,
        "Content-Length: {}, Initial capacity: {}, Using memory mapping: {}",
        content_length,
        initial_capacity,
        use_memory_mapping
    );

    if use_memory_mapping {
        // 处理大文件: 使用内存映射文件
        let temp_file_path = format!("./temp_multipart_{}.dat", uuid::Uuid::new_v4());

        let file = match std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&temp_file_path)
        {
            Ok(f) => f,
            Err(e) => {
                error!(
                    log,
                    "Failed to create temporary file for memory mapping: {}", e
                );
                return Ok(
                    HttpResponse::InternalServerError().body("Failed to create temporary file")
                );
            }
        };

        // 设置文件大小
        if let Err(e) = file.set_len(content_length as u64) {
            error!(log, "Failed to set temporary file size: {}", e);
            let _ = std::fs::remove_file(&temp_file_path); // 尝试清理
            return Ok(HttpResponse::InternalServerError().body("Failed to set file size"));
        }

        // 创建内存映射并在作用域内处理
        {
            let mmap = match unsafe { memmap2::MmapMut::map_mut(&file) } {
                Ok(mut mmap) => {
                    // 读取数据到内存映射区域
                    let mut position = 0;
                    while let Some(chunk) = payload.next().await {
                        match chunk {
                            Ok(data) => {
                                if position + data.len() <= mmap.len() {
                                    mmap[position..position + data.len()].copy_from_slice(&data);
                                    position += data.len();
                                } else {
                                    error!(log, "Payload larger than Content-Length");
                                    let _ = std::fs::remove_file(&temp_file_path);
                                    return Ok(HttpResponse::BadRequest()
                                        .body("Payload larger than Content-Length"));
                                }
                            }
                            Err(e) => {
                                error!(log, "Error reading payload chunk: {}", e);
                                let _ = std::fs::remove_file(&temp_file_path);
                                return Ok(
                                    HttpResponse::InternalServerError().body("Error reading data")
                                );
                            }
                        }
                    }

                    // 注意：这里不调用 flush()，因为我们只需要读取数据，不需要持久化到磁盘
                    mmap.make_read_only()
                        .unwrap_or_else(|_| unsafe { memmap2::Mmap::map(&file).unwrap() })
                }
                Err(e) => {
                    error!(log, "Failed to create memory mapping: {}", e);
                    let _ = std::fs::remove_file(&temp_file_path);
                    return Ok(
                        HttpResponse::InternalServerError().body("Failed to create memory mapping")
                    );
                }
            };

            // 在 mmap 作用域内处理 multipart 字段
            let mut multipart = Multipart::with_reader(&mmap[..], boundary.as_str());
            if let Err(err) = process_multipart_fields(&mut multipart, &log, &study_instance_uid).await {
                // mmap 会在作用域结束时自动释放
                if let Err(e) = std::fs::remove_file(&temp_file_path) {
                    warn!(
                        log,
                        "Failed to remove temporary file {}: {}", temp_file_path, e
                    );
                }
                return Ok(err);
            }
            // multipart 会在作用域结束时自动释放，然后 mmap 才会被释放
        } // mmap 的作用域结束，确保在这里释放

        // 处理完成后清理临时文件
        if let Err(e) = std::fs::remove_file(&temp_file_path) {
            warn!(
                log,
                "Failed to remove temporary file {}: {}", temp_file_path, e
            );
        }
    } else {
        // 处理小文件: 直接使用内存缓冲区
        let mut buffer = Vec::with_capacity(initial_capacity);
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

        let mut multipart = Multipart::with_reader(buffer.as_slice(), boundary.as_str());
        if let Err(err) = process_multipart_fields(&mut multipart, &log, &study_instance_uid).await {
            return Ok(err);
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



// 抽离处理 multipart 字段的逻辑
// 抽离处理 multipart 字段的逻辑
async fn process_multipart_fields(
    multipart: &mut Multipart<'_>,
    log: &slog::Logger,
    study_instance_uid: &Option<String>,
) -> Result<(), HttpResponse> {
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
                    error!(
                        log,
                        "Unsupported field content type: {}, only application/json and application/dicom are supported",
                        field_content_type
                    );
                    return Err(HttpResponse::BadRequest()
                        .body(format!("Unsupported content type: {}", field_content_type)));
                }

                let mut field_bytes_len = 0;
                loop {
                    match field.chunk().await {
                        Ok(Some(field_chunk)) => {
                            // 保存 DICOM 文件
                            let filename = uuid::Uuid::new_v4().to_string();
                            let filepath = if field_content_type == "application/dicom" {
                                if let Some(uid) = study_instance_uid {
                                    format!("./{}_{}.dcm", uid, filename)
                                } else {
                                    format!("./{}.dcm", filename)
                                }
                            } else {
                                // application/json 类型
                                if let Some(uid) = study_instance_uid {
                                    format!("./{}_{}.json", uid, filename)
                                } else {
                                    format!("./{}.json", filename)
                                }
                            };
                            let end_position = field_chunk.len();
                            // 在主逻辑中使用
                            let start_position = match validate_and_find_start_position(
                                field_content_type.as_str(),
                                &field_chunk,
                                end_position,
                            ) {
                                Ok(pos) => pos,
                                Err(response) => return Err(response),
                            };

                            match std::fs::File::create(&filepath) {
                                Ok(mut file) => {
                                    info!(
                                        log,
                                        "Saving DICOM file to: {} , with length:{}",
                                        filepath,
                                        field_chunk.len()
                                    );
                                    if let Err(e) =
                                        file.write_all(&field_chunk[start_position..end_position])
                                    {
                                        error!(
                                            log,
                                            "Failed to write DICOM file {}: {}", filepath, e
                                        );
                                        return Err(HttpResponse::InternalServerError()
                                            .body("Failed to save DICOM file"));
                                    }
                                    info!(log, "Saved DICOM file to: {}", filepath);
                                }
                                Err(e) => {
                                    error!(log, "Failed to create file {}: {}", filepath, e);
                                    return Err(HttpResponse::InternalServerError()
                                        .body("Failed to create storage file"));
                                }
                            }

                            field_bytes_len += field_chunk.len();
                        }
                        Ok(None) => break, // 没有更多数据块了
                        Err(e) => {
                            error!(log, "Error reading field chunk: {}", e);
                            return Err(HttpResponse::BadRequest()
                                .body(format!("Error reading field data: {}", e)));
                        }
                    }
                }

                info!(log, "Field Bytes Length: {:?}", field_bytes_len);
            }
            Ok(None) => break, // 没有更多字段了
            Err(e) => {
                error!(log, "Error parsing multipart field: {}", e);
                return Err(
                    HttpResponse::BadRequest().body(format!("Error parsing multipart data: {}", e))
                );
            }
        }
    }
    Ok(())
}

// 提取验证逻辑到独立函数
fn validate_and_find_start_position(
    field_content_type: &str,
    field_chunk: &[u8],
    end_position: usize,
) -> Result<usize, HttpResponse> {
    // 此为非必须得.用于兼容一些特殊格式 例如采用以下curl 请求会多余一个& 符号
    /*
    curl -X POST http://localhost:9000/stow-rs/v1/studies \
         -H "Content-Type: multipart/related; boundary=DICOM_BOUNDARY; type=application/json" \
         -H "Accept: application/json" \
         --data-binary $'--DICOM_BOUNDARY\r\nContent-Type: application/json\r\n\r\n' \
         --data-binary @metadata.json \
         --data-binary $'\r\n--DICOM_BOUNDARY\r\nContent-Type: application/dicom\r\n\r\n' \
         --data-binary @dcm1.dcm \
         --data-binary $'\r\n--DICOM_BOUNDARY\r\nContent-Type: application/dicom\r\n\r\n' \
         --data-binary @dcm2.dcm \
         --data-binary $'\r\n--DICOM_BOUNDARY\r\nContent-Type: application/dicom\r\n\r\n' \
         --data-binary @dcm3.dcm \
         --data-binary $'\r\n--DICOM_BOUNDARY--\r\n'
     */
    let a = field_chunk[0] == b'&' && field_chunk[end_position - 1] == b'&';
    let b = field_chunk[0] == b'$' && field_chunk[end_position - 2] == b'$';
    let c = field_chunk[0] == b'!' && field_chunk[end_position - 2] == b'!';

    match field_content_type {
        "application/dicom" => {
            // 检查标准DICOM文件（DICM在128字节偏移处）
            if field_chunk.len() >= 132 && &field_chunk[128..132] == b"DICM" {
                Ok(0)
            } else if field_chunk.len() >= 133 && (a || b || c) && &field_chunk[129..133] == b"DICM"
            {
                Ok(1)
            } else {
                Err(HttpResponse::BadRequest().body("Invalid DICOM data"))
            }
        }
        "application/json" => {
            // 验证JSON数据
            if serde_json::from_slice::<serde_json::Value>(&field_chunk).is_ok() {
                Ok(0)
            } else if field_chunk.len() >= 2
                && (a || b || c)
                && serde_json::from_slice::<serde_json::Value>(&field_chunk[1..end_position - 1])
                    .is_ok()
            {
                Ok(1)
            } else {
                Err(HttpResponse::BadRequest().body("Invalid JSON data"))
            }
        }
        _ => Ok(0),
    }
}
