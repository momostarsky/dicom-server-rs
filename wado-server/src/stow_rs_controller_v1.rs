use actix_web::{HttpRequest, HttpResponse, Result, http::header, post, web};
use chrono::Datelike;
use std::fs;

use futures_util::StreamExt as _;
// use dicom_object::open_file; // 如果需要解析 DICOM，取消注释
use crate::AppState;
use crate::constants::STOW_RS_TAG;
use common::change_file_transfer::{convert_ts_with_gdcm_conv};
use common::dicom_file_handler::{classify_and_publish_dicom_messages, process_dicom_memobject};
use common::dicom_utils::{get_date_value_dicom, get_text_value};
use common::message_sender_kafka::KafkaMessagePublisher;
use common::storage_config::{StorageConfig, dicom_file_path};
use database::dicom_meta::DicomStoreMeta;
use dicom_dictionary_std::tags;
use dicom_object::DefaultDicomObject;
use futures_util::io::Cursor;
use multer::Multipart;
use slog::{error, info, warn};

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
        ("Accept" =  String, Header, example="application/dicom", description = "Accept Content Type: application/dicom"),
        ("Content-Type" =  String, Header, example="multipart/related; boundary=6c17d7b275f94d93f0b2a8c3d9xj; type=application/dicom", description = "Accept Content Type: application/dicom"),
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
    // TODO: 获取请求头的Content-Length 和 x-tenant 没有这两个参数则返回HTTP 请求格式不对的错误
    // 处理请求并保存实例
    process_and_store_instances(req, payload, app_state, None).await
}

/// 处理 /studies/{study_instance_uid} 端点的 POST 请求
/// 接收 multipart/related 数据流并保存 DICOM 实例到指定 Study
#[utoipa::path(
    post,
    params(
        ("study_instance_uid" = String, Path, description = "Study Instance UID"),
        ("x-tenant" = String, Header, description = "Tenant ID from request header"),
        ("Content-Type" =  String, Header, example="multipart/related; boundary=6c17d7b275f94d93f0b2a8c3d9xj; type=application/dicom", description = "Accept Content Type: application/dicom"),
        ("Content-Length" = u32, Header, example="5120000", description = "Content Length of the request body"),
        ("Accept" =  String, Header, example="application/json", description = "Accept Content Type: application/dicom"),
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
    let study_instance_uid = path.into_inner(); // 提取路径参数
    // 处理请求并保存实例到指定study
    process_and_store_instances(req, payload, app_state, Some(study_instance_uid)).await
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
    app_state: web::Data<AppState>,
    study_instance_uid: Option<String>,
) -> Result<HttpResponse> {
    // 记录开始时间
    let start_time = std::time::Instant::now();
    let log = app_state.log.clone();
    // 使用 as_ref() 获取引用而不是消耗值
    if let Some(uid) = &study_instance_uid {
        info!(log, "Received POST request on /studies/{}", uid);
    } else {
        // 处理并保存实例
        info!(log, "Received POST request on /studies");
    }

    // 获取并验证 x-tenant 头
    let tenant_id = match req.headers().get("x-tenant") {
        Some(header_value) => match header_value.to_str() {
            Ok(value) => value.to_string(),
            Err(_) => {
                error!(log, "Failed to convert x-tenant header to string");
                return Ok(HttpResponse::BadRequest().body("Malformed x-tenant header"));
            }
        },
        None => {
            error!(log, "Missing x-tenant header");
            return Ok(HttpResponse::BadRequest().body("x-tenant header is required"));
        }
    };

    // 修改为必须存在的 Content-Length
    let content_length = match req.headers().get(header::CONTENT_LENGTH) {
        Some(header_value) => match header_value.to_str() {
            Ok(value_str) => match value_str.parse::<usize>() {
                Ok(length) => length + 4096, // 添加 4096 字节,以防止边界情况
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
    // 解析 boundary 信息
    let boundary = match parse_boundary_info(&req, &log).await {
        Ok(boundary) => boundary,
        Err(response) => return Ok(response),
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
        // 处理大文件: 使用临时文件和内存映射
        let temp_file = match tempfile::NamedTempFile::new() {
            Ok(file) => file,
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

        let file = match temp_file.reopen() {
            Ok(f) => f,
            Err(e) => {
                error!(log, "Failed to reopen temporary file: {}", e);
                return Ok(HttpResponse::InternalServerError().body("Failed to reopen file"));
            }
        };

        // 设置文件大小
        if let Err(e) = file.set_len(content_length as u64) {
            error!(log, "Failed to set temporary file size: {}", e);
            return Ok(HttpResponse::InternalServerError().body("Failed to set file size"));
        }

        // 创建内存映射并在作用域内处理
        let result = {
            let mmap_result = unsafe { memmap2::MmapMut::map_mut(&file) };
            match mmap_result {
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
                                    return Ok(HttpResponse::BadRequest()
                                        .body("Payload larger than Content-Length"));
                                }
                            }
                            Err(e) => {
                                error!(log, "Error reading payload chunk: {}", e);
                                return Ok(
                                    HttpResponse::InternalServerError().body("Error reading data")
                                );
                            }
                        }
                    }

                    // 注意：这里不调用 flush()，因为我们只需要读取数据，不需要持久化到磁盘
                    let readonly_mmap = match mmap.make_read_only() {
                        Ok(mmap) => mmap,
                        Err(_) => unsafe {
                            match memmap2::Mmap::map(&file) {
                                Ok(mmap) => mmap,
                                Err(e) => {
                                    error!(log, "Failed to create readonly memory mapping: {}", e);
                                    return Ok(HttpResponse::InternalServerError()
                                        .body("Failed to create memory mapping"));
                                }
                            }
                        },
                    };

                    // 在 mmap 作用域内处理 multipart 字段
                    let mut multipart =
                        Multipart::with_reader(&readonly_mmap[..], boundary.as_str());
                    let process_result = process_multipart_fields(
                        &mut multipart,
                        &app_state,
                        &study_instance_uid,
                        &tenant_id,
                    )
                    .await;

                    // 显式释放资源
                    drop(multipart);
                    drop(readonly_mmap);

                    process_result
                }
                Err(e) => {
                    error!(log, "Failed to create memory mapping: {}", e);
                    Err(HttpResponse::InternalServerError().body("Failed to create memory mapping"))
                }
            }
        }; // mmap 的作用域结束，确保在这里释放

        // 处理结果
        if let Err(err) = result {
            return Ok(err);
        }

        // temp_file 会在离开作用域时自动清理
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
        if let Err(err) =
            process_multipart_fields(&mut multipart, &app_state, &study_instance_uid, &tenant_id)
                .await
        {
            return Ok(err);
        }
    }

    // 计算执行时间
    let duration = start_time.elapsed();

    // 构造响应
    if let Some(uid) = study_instance_uid {
        info!(
            log,
            "STOW-RS request for study {} processed successfully (simplified)", uid;
            "execution_time_ms" => duration.as_millis(),
            "content_length" => content_length,
            "use_memory_mapping" => use_memory_mapping
        );
    } else {
        info!(
            log,
            "STOW-RS request processed successfully (simplified)";
            "execution_time_ms" => duration.as_millis(),
            "content_length" => content_length,
            "use_memory_mapping" => use_memory_mapping
        );
    }

    Ok(HttpResponse::Ok()
        .body("<NativeDicomModel><Message><Status>Success</Status></Message></NativeDicomModel>"))
}

// 抽离处理 multipart 字段的逻辑
async fn process_multipart_fields(
    multipart: &mut Multipart<'_>,
    app_state: &web::Data<AppState>,
    study_instance_uid: &Option<String>,
    tenant_id: &String,
) -> Result<(), HttpResponse> {
    let log = &app_state.log;
    let storage_confg = StorageConfig::make_storage_config(&app_state.config);
    let mut files: Vec<String> = vec![];
    // 遍历所有已经处理的文件
    let mut metas: Vec<DicomStoreMeta> = Vec::with_capacity(150);
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

                if field_content_type != "application/dicom" {
                    error!(
                        log,
                        "Unsupported field content type: {}, only  application/dicom is supported",
                        field_content_type
                    );
                    return Err(HttpResponse::BadRequest()
                        .body(format!("Unsupported content type: {}", field_content_type)));
                }

                let mut field_bytes_len = 0;
                loop {
                    match field.chunk().await {
                        Ok(Some(field_chunk)) => {
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
                            field_bytes_len += field_chunk.len();
                            // 使用 std::io::Cursor 包装字节流，使其实现 Read trait
                            let cursor = Cursor::new(&field_chunk[start_position..end_position]);
                            // 用于写入磁盘
                            let datax = Cursor::new(&field_chunk[start_position..end_position]);
                            // 使用 DicomObject::read_from() 从实现了 Read trait 的源加载对象
                            let loaded_object =
                                match DefaultDicomObject::from_reader(cursor.into_inner()) {
                                    Ok(obj) => Some(obj),
                                    Err(_e) => None,
                                };
                            if loaded_object.is_none() {
                                continue;
                            }
                            let mut loaded_object = loaded_object.unwrap();
                            let tag_study_uid =
                                get_text_value(&loaded_object, tags::STUDY_INSTANCE_UID)
                                    .map(|v| v.to_string());
                            if let Some(expected_uid) = study_instance_uid {
                                if expected_uid != tag_study_uid.as_ref().unwrap() {
                                    warn!(
                                        log,
                                        "Study instance UID mismatch, excepted: {} and actual :{:?}",
                                        expected_uid,
                                        tag_study_uid
                                    );
                                }
                            }
                            let sop_inst_uid =
                                get_text_value(&loaded_object, tags::SOP_INSTANCE_UID);
                            let seris_instance_uid =
                                get_text_value(&loaded_object, tags::SERIES_INSTANCE_UID);
                            let study_date =
                                match get_date_value_dicom(&loaded_object, tags::STUDY_DATE) {
                                    Some(date) => {
                                        // 将日期格式化为 YYYYMMDD 形式
                                        Some(format!(
                                            "{:04}{:02}{:02}",
                                            date.year(),
                                            date.month(),
                                            date.day()
                                        ))
                                    }
                                    None => {
                                        warn!(log, "Failed to get study date");
                                        None
                                    }
                                };
                            if sop_inst_uid.is_none()
                                || seris_instance_uid.is_none()
                                || study_date.is_none()
                                || tag_study_uid.is_none()
                            {
                                warn!(
                                    log,
                                    "Some required tags are missing, sop_inst_uid: {:?}, seris_instance_uid: {:?}, study_date: {:?}, tag_study_uid: {:?}",
                                    sop_inst_uid,
                                    seris_instance_uid,
                                    study_date,
                                    &tag_study_uid
                                );
                            }

                            let dir_path = match storage_confg.make_series_dicom_dir(
                                tenant_id,
                                &study_date.unwrap().to_string(),
                                tag_study_uid.unwrap().as_str(),
                                seris_instance_uid.unwrap().as_str(),
                                true,
                            ) {
                                Ok(path) => path,
                                Err(_e) => {
                                    return Err(HttpResponse::InternalServerError()
                                        .body("Failed to create series directory"));
                                }
                            };

                            let filepath =
                                dicom_file_path(&dir_path, sop_inst_uid.unwrap().as_str());
                            fs::write(&filepath, datax.into_inner())
                                .expect("write data to disk failed !");


                            info!(log, "Saved DICOM file to {}", &filepath);
                            match process_dicom_memobject(
                                &mut loaded_object,
                                &filepath,
                                tenant_id,
                                &storage_confg,
                            )
                            .await
                            {
                                Ok(dicom_meta) => {
                                    info!(
                                        log,
                                        "process_dicom_memobject get DICOM metadata: {:?}",
                                        dicom_meta
                                    );
                                    metas.push(dicom_meta);
                                    files.push(filepath);
                                }
                                Err(e) => {
                                    warn!(
                                        log,
                                        "process_dicom_memobject failed: {} with :{}", filepath, e
                                    );
                                }
                            }
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

    warn!(log, "process_multipart_fields {} files", files.len());
    // if !files.is_empty() {
    //     for vf in files {
    //
    //             match dicom_object::OpenFileOptions::new()
    //                 .charset_override(CharacterSetOverride::AnyVr)
    //                 .read_until(tags::PIXEL_DATA)
    //                 .open_file(String::from(vf.as_str())){
    //                 Ok(_) => {
    //                     info!(log,"Open DICOM is OK");
    //                 },
    //                 Err(e) =>{
    //                     error!(log,"Open DICOM is Error:{}" ,e );
    //                 }
    //
    //             }
    //
    //     }
    // }
    if !metas.is_empty() {
        info!(
            log,
            "Publishing STOW-RS DICOM messages to Kafka:{}",
            metas.len()
        );
        let queue_config = &app_state.config.message_queue;
        let queue_topic_main = &queue_config.topic_main.as_str();
        let queue_topic_log = &queue_config.topic_log.as_str();

        let storage_producer = KafkaMessagePublisher::new(queue_topic_main.parse().unwrap());
        let log_producer = KafkaMessagePublisher::new(queue_topic_log.parse().unwrap());

        match classify_and_publish_dicom_messages(&metas, &storage_producer, &log_producer).await {
            Ok(_) => {
                info!(&log, "Successfully published DICOM messages");
            }
            Err(e) => {
                warn!(&log, "Failed to publish DICOM messages: {}", e);
            }
        };
        metas.clear();
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
         -H "Content-Type: multipart/related; boundary=DICOM_BOUNDARY; type=application/dicom" \
         -H "Accept: application/json" \
         --data-binary $'--DICOM_BOUNDARY\r\nContent-Type: application/dicom\r\n\r\n' \
         --data-binary @dcm1.dcm \
         --data-binary $'\r\n--DICOM_BOUNDARY\r\nContent-Type: application/dicom\r\n\r\n' \
         --data-binary @dcm2.dcm \
         --data-binary $'\r\n--DICOM_BOUNDARY\r\nContent-Type: application/dicom\r\n\r\n' \
         --data-binary @dcm3.dcm \
         --data-binary $'\r\n--DICOM_BOUNDARY--\r\n'
     */
    // 检查是否是特殊格式（首尾有相同特殊字符）
    match field_content_type {
        "application/dicom" => {
            // 检查是否有特殊包装符
            let has_special_wrapper = field_chunk.len() >= 2
                && matches!(
                    (field_chunk[0], field_chunk[end_position - 1]),
                    (b'&', b'&') | (b'$', b'$') | (b'~', b'~') | (b'!', b'!')
                );

            // 计算 DICM 偏移位置
            let offset = if has_special_wrapper { 1 } else { 0 };

            // 验证 DICM 标志是否存在
            if field_chunk.len() >= (132 + offset)
                && &field_chunk[128 + offset..132 + offset] == b"DICM"
            {
                Ok(offset)
            } else {
                Err(HttpResponse::BadRequest().body("Invalid DICOM data"))
            }
        }
        _ => Ok(0),
    }
}
