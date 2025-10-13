use crate::redis_helper::{get_redis_value_with_config, set_redis_value_with_expiry_and_config};
use crate::{AppState, common_utils};
use actix_web::http::header::ACCEPT;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web, web::Path};
use common::database_entities::{SeriesEntity, StudyEntity};
use common::dicom_json_helper;
use common::dicom_json_helper::walk_directory;
use common::dicom_utils::get_tag_values;
use common::server_config::{dicom_series_dir, dicom_study_dir, json_metadata_dir};
use dicom_dictionary_std::tags;
use dicom_object::OpenFileOptions;
use serde::Deserialize;
use serde_json::{Map, json};
use slog::{error, info};
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
struct StudyQueryParams {
    #[serde(rename = "charset")]
    #[warn(dead_code)]
    charset: Option<String>,
    #[serde(rename = "anonymize")]
    anonymize: Option<bool>,
    #[serde(rename = "includeField")]
    include_field: Option<Vec<String>>,
    #[serde(rename = "excludeField")]
    exclude_field: Option<Vec<String>>,
}
static WADO_CHARSET: &str = "charset";
static WADO_ANONYMIZE: &str = "anonymize";
static WADO_INCLUDE_FIELD: &str = "includeField";
static WADO_EXCLUDE_FIELD: &str = "excludeField";

static ACCEPT_DICOM_JSON_TYPE: &str = "application/dicom+json";
static ACCEPT_JSON_TYPE: &str = "application/json";
static ACCEPT_DICOM_TYPE: &str = "application/dicom";
static ACCEPT_OCTET_STREAM: &str = "application/octet-stream";
static MULIPART_ACCEPT_OCTET_STREAM: &str = "multipart/related; type=application/octet-stream";

// 检查Accept头部是否包含指定的MIME类型（不区分大小写）
fn is_accept_type_supported(accept_header: &str, expected_type: &str) -> bool {
    let accepted_types: Vec<&str> = accept_header.split(',').map(|s| s.trim()).collect();

    let expected_type_lower = expected_type.to_lowercase();
    accepted_types
        .iter()
        .any(|&t| t.to_lowercase() == expected_type_lower)
}

fn get_redis_key_for_series(tenant_id: &str, series_uid: &str) -> String {
    format!("wado:{}:series:{}", tenant_id, series_uid)
}

// 提取重复的获取study_info逻辑
async fn get_study_info_with_cache(
    tenant_id: &str,
    study_uid: &str,
    app_state: &web::Data<AppState>,
) -> Result<StudyEntity, HttpResponse> {
    let log = app_state.log.clone();
    let redis_key = format!("wado:{}:study:{}", tenant_id, study_uid);

    // 首先尝试从Redis获取study_info
    let study_info =
        match get_redis_value_with_config::<String>(&app_state.config.redis, &redis_key) {
            Some(cached_json) => {
                // 如果Redis中有缓存，尝试反序列化
                match serde_json::from_str::<StudyEntity>(&cached_json) {
                    Ok(info) => Some(info),
                    Err(e) => {
                        error!(
                            log,
                            "Failed to deserialize study_info from Redis cache: {}", e
                        );
                        None
                    }
                }
            }
            None => None,
        };

    // 如果Redis中没有缓存，则从数据库获取
    match study_info {
        Some(info) => Ok(info),
        None => {
            match app_state.db.get_study_info(tenant_id, study_uid).await {
                Ok(Some(info)) => {
                    info!(log, "Retrieved study_info from database");
                    // 将从数据库获取的数据存入Redis缓存
                    match serde_json::to_string(&info) {
                        Ok(study_info_json) => {
                            set_redis_value_with_expiry_and_config::<String>(
                                &app_state.config.redis,
                                &redis_key,
                                &study_info_json,
                                2 * 60 * 60, // 2小时过期时间
                            );
                        }
                        Err(e) => {
                            error!(
                                log,
                                "Failed to serialize study_info to JSON for caching: {}", e
                            );
                        }
                    }
                    Ok(info)
                }
                Ok(None) => {
                    let error_msg = format!("Study not found: {},{}", tenant_id, study_uid);
                    Err(HttpResponse::NotFound().body(error_msg))
                }
                Err(e) => {
                    let error_msg = format!("Failed to retrieve study info: {}", e);
                    Err(HttpResponse::InternalServerError().body(error_msg))
                }
            }
        }
    }
}

// 提取重复的获取series_info逻辑
async fn get_series_info_with_cache(
    tenant_id: &str,
    series_uid: &str,
    app_state: &web::Data<AppState>,
) -> Result<SeriesEntity, HttpResponse> {
    let log = app_state.log.clone();
    let redis_key = get_redis_key_for_series(tenant_id, series_uid);

    // 首先尝试从Redis获取series_info
    let series_info =
        match get_redis_value_with_config::<String>(&app_state.config.redis, &redis_key) {
            Some(cached_json) => {
                // 如果Redis中有缓存，尝试反序列化
                match serde_json::from_str::<SeriesEntity>(&cached_json) {
                    Ok(info) => Some(info),
                    Err(e) => {
                        error!(
                            log,
                            "Failed to deserialize series_info from Redis cache: {}", e
                        );
                        None
                    }
                }
            }
            None => None,
        };

    // 如果Redis中没有缓存，则从数据库获取
    match series_info {
        Some(info) => Ok(info),
        None => {
            match app_state.db.get_series_info(tenant_id, series_uid).await {
                Ok(Some(info)) => {
                    info!(log, "Retrieved series_info from database");
                    // 将从数据库获取的数据存入Redis缓存
                    match serde_json::to_string(&info) {
                        Ok(series_info_json) => {
                            set_redis_value_with_expiry_and_config::<String>(
                                &app_state.config.redis,
                                &redis_key,
                                &series_info_json,
                                2 * 60 * 60, // 2小时过期时间
                            );
                        }
                        Err(e) => {
                            error!(
                                log,
                                "Failed to serialize series_info to JSON for caching: {}", e
                            );
                        }
                    }
                    Ok(info)
                }
                Ok(None) => {
                    let error_msg =
                        format!("retrieve_instance not found: {},{}", tenant_id, series_uid);
                    Err(HttpResponse::NotFound().body(error_msg))
                }
                Err(e) => {
                    let error_msg =
                        format!("retrieve_instance Failed to retrieve study info: {}", e);
                    Err(HttpResponse::InternalServerError().body(error_msg))
                }
            }
        }
    }
}
#[get("/studies/{study_instance_uid}/metadata")]
async fn retrieve_study_metadata(
    study_instance_uid: Path<String>,
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let log = app_state.log.clone();
    let study_uid = study_instance_uid.into_inner();
    let tenant_id = common_utils::get_tenant_from_handler(&req);
    info!(
        log,
        "retrieve_study_metadata Tenant ID: {}  and StudyUID:{} ", tenant_id, study_uid
    );
    // let accept = req.headers().get(ACCEPT).and_then(|v| v.to_str().ok());

    // if accept != Some(ACCEPT_DICOM_JSON_TYPE) {
    //     return HttpResponse::NotAcceptable().body(format!(
    //         "retrieve_study_metadata Accept header must be {}",
    //         ACCEPT_DICOM_JSON_TYPE
    //     ));
    // }

    let study_info = match get_study_info_with_cache(&tenant_id, &study_uid, &app_state).await {
        Ok(info) => info,
        Err(response) => return response,
    };
    info!(log, "Study Info: {:?}", study_info);
    let (_study_uid_hash, dicom_dir) = match dicom_study_dir(
        tenant_id.as_str(),
        &study_info.study_date_origin,
        study_uid.as_str(),
        false,
    ) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to generate DICOM directory: {}", e));
        }
    };

    let json_dir = match json_metadata_dir(tenant_id.as_str(), &study_info.study_date_origin, true)
    {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to generate JSON directory: {}", e));
        }
    };

    let json_path = format!("{}/{}.json", json_dir, study_uid);

    if !std::path::Path::new(&json_path).exists() {
        let dicom_path = PathBuf::from(&dicom_dir);
        let json_path = PathBuf::from(&json_path);
        info!(log, "DICOM directory: {:?}", dicom_path);
        dicom_json_helper::generate_json_file(&dicom_path, &json_path)
            .expect("generate_json_file failed");
    }
    match std::fs::read_to_string(&json_path) {
        Ok(content) => HttpResponse::Ok()
            .content_type(ACCEPT_DICOM_JSON_TYPE)
            .body(content),
        Err(e) => HttpResponse::InternalServerError().body(format!(
            "retrieve_study_metadata Failed to read JSON file: {}: {}",
            json_path, e
        )),
    }
}

#[get("/studies/{study_instance_uid}/series/{series_instance_uid}/metadata")]
async fn retrieve_series_metadata(
    path: Path<(String, String)>,
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let log = app_state.log.clone();
    let (study_uid, series_uid) = path.into_inner();
    info!(
        log,
        "retrieve_series_metadata: study_instance_uid={}, series_instance_uid={}",
        study_uid,
        series_uid
    );
    let tenant_id = common_utils::get_tenant_from_handler(&req);

    // 检查 Accept 头
    let accept = req.headers().get(ACCEPT).and_then(|v| v.to_str().ok());

    if let Some(accept_str) = accept {
        if !is_accept_type_supported(accept_str, ACCEPT_DICOM_JSON_TYPE)
            && !is_accept_type_supported(accept_str, ACCEPT_JSON_TYPE)
        {
            return HttpResponse::NotAcceptable().body(format!(
                "retrieve_study_metadata Accept header must be {} and {}",
                ACCEPT_DICOM_JSON_TYPE, ACCEPT_JSON_TYPE
            ));
        }
    } else {
        return HttpResponse::NotAcceptable().body(format!(
            "retrieve_study_metadata Accept header must be {}",
            ACCEPT_DICOM_JSON_TYPE
        ));
    }
    // ... existing code ...
    let study_info = match get_study_info_with_cache(&tenant_id, &study_uid, &app_state).await {
        Ok(info) => info,
        Err(response) => return response,
    };
    info!(log, "Study Info: {:?}", study_info);
    // 获取series_info (使用提取的函数)
    let series_info = match get_series_info_with_cache(&tenant_id, &series_uid, &app_state).await {
        Ok(info) => info,
        Err(response) => return response,
    };
    info!(log, "Series Info: {:?}", series_info);
    info!(log, "Study Info: {:?}", study_info);

    let json_dir = match json_metadata_dir(tenant_id.as_str(), &study_info.study_date_origin, true)
    {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to generate JSON directory: {}", e));
        }
    };
    let json_file_path = format!("{}/{}.json", json_dir, series_uid);
    //如果json_file_path 存在,则输出json
    if std::path::Path::new(&json_file_path).exists() {
        match std::fs::read_to_string(&json_file_path) {
            Ok(json_content) => {
                info!(log, "JSON file found: {}", json_file_path);
                return HttpResponse::Ok()
                    .content_type(ACCEPT_DICOM_JSON_TYPE)
                    .body(json_content);
            }
            Err(_) => {}
        }
    }

    info!(log, "Study Info: {:?}", study_info);
    let (_study_uid_hash, _series_uid_hash, dicom_dir) = match dicom_series_dir(
        tenant_id.as_str(),
        &study_info.study_date_origin,
        study_uid.as_str(),
        series_uid.as_str(),
        false,
    ) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to generate DICOM directory: {}", e));
        }
    };

    let files = match walk_directory(dicom_dir) {
        Ok(files) => files,
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!(
                "retrieve_study_metadata Failed to walk directory: {}",
                e
            ));
        }
    };

    let mut arr = vec![];
    // 添加每个 DICOM 文件作为 multipart 中的一部分
    for file_path in &files {
        // 读取 DICOM 文件内容
        let sop_json = match OpenFileOptions::new()
            .read_until(tags::PIXEL_DATA)
            .open_file(file_path)
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
                dicom_json
            }

            Err(e) => {
                return HttpResponse::InternalServerError().body(format!(
                    "Failed to read DICOM file {}: {}",
                    file_path.display(),
                    e
                ));
            }
        };
        arr.push(sop_json);
    }
    match serde_json::to_string(&arr) {
        Ok(json_str) => {
            // 根据series_uid 将json_str 写入当前目录下面,文件路径为:./{series_uid}.json

            if let Err(e) = std::fs::write(&json_file_path, &json_str) {
                error!(log, "Failed to write JSON file {}: {}", json_file_path, e);
            }
            // series_info 序列化JSON后,调用 set_redis_value_with_expiry 写入Redis, 过期时间为2小时
            match serde_json::to_string(&series_info) {
                Ok(series_info_json) => {
                    let redis_config = &app_state.config.redis;
                    set_redis_value_with_expiry_and_config::<String>(
                        redis_config,
                        &get_redis_key_for_series(&tenant_id, &series_uid),
                        &series_info_json,
                        2 * 60 * 60,
                    );
                }
                Err(e) => {
                    error!(log, "Failed to serialize series_info to JSON: {}", e);
                }
            }
            HttpResponse::Ok()
                .content_type(ACCEPT_DICOM_JSON_TYPE)
                .body(json_str)
        }
        Err(e) => HttpResponse::InternalServerError().body(format!(
            "retrieve_study_metadata Failed to walk directory: {}",
            e
        )),
    }
}

#[get("/studies/{study_instance_uid}/series/{series_instance_uid}/instances/{sop_instance_uid}")]
async fn retrieve_instance(
    path: Path<(String, String, String)>,
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let (study_uid, series_uid, sop_uid) = path.into_inner();
    retrieve_instance_impl(study_uid, series_uid, sop_uid, 1, req, app_state).await
}

#[get(
    "/studies/{study_instance_uid}/series/{series_instance_uid}/instances/{sop_instance_uid}/frames/{frames}"
)]
async fn retrieve_instance_frames(
    path: Path<(String, String, String, u32)>,
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let (study_uid, series_uid, sop_uid, frames) = path.into_inner();
    retrieve_instance_impl(study_uid, series_uid, sop_uid, frames, req, app_state).await
}

// 通用函数处理 retrieve_instance 和 retrieve_instance_frames 的共同逻辑
async fn retrieve_instance_impl(
    study_uid: String,
    series_uid: String,
    sop_uid: String,
    frames: u32,
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> HttpResponse {
    let log = app_state.log.clone();
    if frames > 1 {
        return HttpResponse::NotImplemented().body(format!(
            "retrieve_instance_frames not implemented for frames >1: {}",
            frames
        ));
    }
    let tenant_id = common_utils::get_tenant_from_handler(&req);
    info!(
        log,
        "retrieve_instance: Tenant ID: {},study_instance_uid={}, series_instance_uid={}, sop_instance_uid={}",
        tenant_id,
        study_uid,
        series_uid,
        sop_uid
    );
    // 获取series_info (使用提取的函数)
    let study_info = match get_study_info_with_cache(&tenant_id, &study_uid, &app_state).await {
        Ok(info) => info,
        Err(response) => return response,
    };
    info!(log, "Study Info: {:?}", study_info);

    let series_info = match get_series_info_with_cache(&tenant_id, &series_uid, &app_state).await {
        Ok(info) => info,
        Err(response) => return response,
    };
    info!(log, "Series Info: {:?}", series_info);

    let (_study_uid_hash, _series_uid_hash_v, dicom_dir) = match dicom_series_dir(
        tenant_id.as_str(),
        &study_info.study_date_origin,
        study_uid.as_str(),
        series_uid.as_str(),
        false,
    ) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to generate DICOM directory: {}", e));
        }
    };

    let dicom_file = build_dicom_instance_file_path(&dicom_dir, &sop_uid);
    match OpenFileOptions::new().open_file(&dicom_file) {
        Ok(obj) => match obj.get(tags::PIXEL_DATA) {
            Some(element) => match element.to_bytes() {
                Ok(pxl_data) => HttpResponse::Ok()
                    .content_type(ACCEPT_OCTET_STREAM)
                    .body(pxl_data.into_owned()),
                Err(_) => HttpResponse::NotFound().body(format!(
                    "dicom file PixelData to_bytes failed: {}",
                    &dicom_file
                )),
            },
            None => HttpResponse::NotFound().body(format!(
                "dicom file PixelData element not found: {}",
                &dicom_file
            )),
        },
        Err(_) => HttpResponse::NotFound().body(format!("DICOM file not found: {}", &dicom_file)),
    }
}

fn build_dicom_instance_file_path(dicom_dir: &str, sop_uid: &str) -> String {
    format!("{}/{}.dcm", dicom_dir, sop_uid)
}

#[get("/echo")]
async fn echo() -> impl Responder {
    HttpResponse::Ok().body("Success")
}

pub(crate) async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
