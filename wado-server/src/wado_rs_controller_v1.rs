// use crate::constants::WADO_RS_PERMISSONS_IMAGE_READER;
// use crate::constants::WADO_RS_ID;
// use crate::constants::WADO_RS_ROLES;
use crate::constants::WADO_RS_TAG;
use crate::wado_rs_models::SubSeriesMeta;
use crate::{AppState, common_utils};
use actix_web::http::header::ACCEPT;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, web, web::Path};
use common::dicom_json_helper;
use common::redis_key::RedisHelper;
use common::storage_config::{StorageConfig, dicom_file_path};
use database::dicom_meta::{DicomJsonMeta, DicomStateMeta};
use dicom_dictionary_std::tags;
use dicom_object::OpenFileOptions;
// use permission_macros::permission_required;
use common::dicom_json_helper::generate_series_json;
use slog::{error, info};
use std::path::PathBuf;

static ACCEPT_DICOM_JSON_TYPE: &str = "application/dicom+json";
static ACCEPT_JSON_TYPE: &str = "application/json";

static ACCEPT_DICOM_TYPE: &str = "application/dicom";
static ACCEPT_OCTET_STREAM: &str = "application/octet-stream";
//static MULIPART_ACCEPT_OCTET_STREAM: &str = "multipart/related; type=application/octet-stream";

// 检查Accept头部是否包含指定的MIME类型（不区分大小写）
fn is_accept_type_supported(accept_header: &str, expected_type: &str) -> bool {
    let accepted_types: Vec<&str> = accept_header.split(',').map(|s| s.trim()).collect();

    let expected_type_lower = expected_type.to_lowercase();
    accepted_types
        .iter()
        .any(|&t| t.to_lowercase() == expected_type_lower)
}
// 提取重复的获取study_info逻辑
async fn get_study_info_with_cache(
    tenant_id: &str,
    study_uid: &str,
    app_state: &web::Data<AppState>,
    from_cache: bool,
) -> Result<Vec<DicomStateMeta>, HttpResponse> {
    let log = app_state.log.clone();
    // 首先尝试从 Redis 缓存中获取数据
    let rh = &app_state.redis_helper;

    if from_cache {
        match rh.get_study_metadata(tenant_id, study_uid).await {
            Ok(metas) => {
                info!(log, "Retrieved study_info from Redis cache");
                if !metas.is_empty() {
                    return Ok(metas);
                }
            }
            Err(_) => {}
        }
    }

    match app_state.db.get_state_metaes(tenant_id, study_uid).await {
        Ok(metas) => {
            info!(log, "Retrieved study_info from database");
            rh.del_study_entity_not_exists(tenant_id, study_uid).await;
            // 将查询结果序列化并写入 Redis 缓存，过期时间设置为2小时
            match rh
                .set_study_metadata(tenant_id, study_uid, &metas, 2 * RedisHelper::ONE_HOUR)
                .await
            {
                Ok(_) => {
                    info!(log, "Stored study_info into Redis cache");
                }
                Err(e) => {
                    error!(log, "Failed to store study_info into Redis cache: {}", e);
                }
            }
            Ok(metas)
        }
        Err(e) => {
            let error_msg = format!("Failed to retrieve study info: {}", e);
            match rh
                .set_study_entity_not_exists(tenant_id, study_uid, 5 * RedisHelper::ONE_MINULE)
                .await
            {
                Ok(_) => {
                    info!(log, "set_study_entity_not_exists {}", study_uid);
                }
                Err(e) => {
                    error!(log, "Failed to store study_info into Redis cache: {}", e);
                }
            }
            Err(HttpResponse::InternalServerError().body(error_msg))
        }
    }
}
async fn get_series_json_meta(
    tenant_id: &str,
    study_uid: &str,
    series_uid: &str,
    app_state: &web::Data<AppState>,
) -> Option<DicomJsonMeta> {
    let log = app_state.log.clone();

    match app_state
        .db
        .get_json_meta(tenant_id, study_uid, series_uid)
        .await
    {
        Ok(metas) => {
            info!(
                log,
                "Retrieved DicomJsonMeta :{} from database  success", series_uid
            );
            Some(metas)
        }
        Err(_) => {
            info!(
                log,
                "Retrieved DicomJsonMeta :{} from database  failed ", series_uid
            );
            None
        }
    }
}
#[utoipa::path(

    get,

    params(
        ("study_instance_uid" = String, Path, description = "Study Instance UID"),
        ("x-tenant" = String, Header, description = "Tenant ID from request header"),
        ("Accept" =  String, Header, example="application/json", description = "Accept Content Type: application/dicom+json or application/json"),
        ("Authorization" = Option<String>, Header,   description = "Optional JWT Access Token in Bearer format")
    ),
    responses(
        (status = 200, description = "Study metadata retrieved successfully", content_type = "application/dicom+json"),
        (status = 404, description = "Study not found"),
        (status = 406, description = " Accept header must be application/dicom+json or application/json"),
        (status = 500, description = "Internal server error")
    ),
    tag =  WADO_RS_TAG,
    description = "Retrieve Study Metadata in DICOM JSON format",
)]
#[get("/studies/{study_instance_uid}/metadata")]

async fn retrieve_study_metadata(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    study_instance_uid: Path<String>,
) -> impl Responder {
    let log = app_state.log.clone();
    let study_uid = study_instance_uid.into_inner();
    let tenant_id = common_utils::get_tenant_from_handler(&req);
    info!(
        log,
        "retrieve_study_metadata Tenant ID: {}  and StudyUID:{} ", tenant_id, study_uid
    );

    // 检查 Accept 头
    let accept = req.headers().get(ACCEPT).and_then(|v| v.to_str().ok());

    if let Some(accept_str) = accept {
        if !is_accept_type_supported(accept_str, ACCEPT_DICOM_JSON_TYPE)
            && !is_accept_type_supported(accept_str, ACCEPT_JSON_TYPE)
        {
            return HttpResponse::NotAcceptable().body(format!(
                "retrieve_study_metadata Accept header must be {} or {}",
                ACCEPT_DICOM_JSON_TYPE, ACCEPT_JSON_TYPE
            ));
        }
    } else {
        return HttpResponse::NotAcceptable().body(format!(
            "retrieve_study_metadata Accept header must be {} or {}",
            ACCEPT_DICOM_JSON_TYPE, ACCEPT_JSON_TYPE
        ));
    }
    // 首先尝试从 Redis 缓存中获取数据
    let rh = &app_state.redis_helper;
    // 防止短期内多次访问导致数据库压力过大, 使用Redis缓存判断数据库中存在对应的实体类.
    let is_not_found = rh.get_study_entity_not_exists(tenant_id.as_str(), study_uid.as_str());

    if is_not_found.await.unwrap() == true {
        return HttpResponse::NotFound().body(format!(
            "retrieve_study_metadata Study not found in database retry after 30 seconds: {},{}",
            tenant_id, study_uid
        ));
    }

    //  从缓存中加载study_info
    let study_info = match get_study_info_with_cache(&tenant_id, &study_uid, &app_state, true).await
    {
        Ok(info) => info,
        Err(response) => return response,
    };
    if study_info.is_empty() {
        return HttpResponse::NotFound().body(format!(
            "retrieve_study_metadata Study not found in database retry after 30 seconds: {},{}",
            tenant_id, study_uid
        ));
    }

    info!(log, "Study Info: {:?}", study_info.first());
    let study_info = match study_info.first() {
        Some(v) => v,
        None => {
            return HttpResponse::NotFound().body(format!(
                "retrieve_study_metadata Study not found in database retry after 30 seconds: {},{}",
                tenant_id, study_uid
            ));
        }
    };

    let storage_config = StorageConfig::new(app_state.config.clone());

    let json_path = match storage_config.json_metadata_path_for_study(study_info, false) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to compute json_metadata_for_study: {}", e));
        }
    };

    let dicom_dir = match storage_config.dicom_study_dir(study_info, false) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to compute DICOM directory: {}", e));
        }
    };

    // 判断JSON是否生成
    let db_json = get_series_json_meta(&tenant_id, &study_uid, &study_uid, &app_state).await;
    if db_json.is_some() && std::path::Path::new(&json_path).exists() {
        let read_context = std::fs::read_to_string(&json_path);
        if read_context.is_ok() {
            return HttpResponse::Ok()
                .content_type(ACCEPT_DICOM_JSON_TYPE)
                .body(read_context.unwrap());
        }
    }
    // 重新生成JSON
    if !std::path::Path::new(&json_path).exists() {
        let dicom_path = PathBuf::from(&dicom_dir);
        let json_path = PathBuf::from(&json_path);
        info!(log, "DICOM directory: {:?}", dicom_path);
        if let Err(e) = dicom_json_helper::generate_study_json(&dicom_path, &json_path) {
            return HttpResponse::InternalServerError().body(format!(
                "retrieve_study_metadata Failed to generate JSON file: {}: {}",
                json_path.display(),
                e
            ));
        }
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
#[utoipa::path(
    get,
    params(
        ("study_instance_uid" = String, Path, description = "Study Instance UID"),
        ("x-tenant" = String, Header, example="1234567890",  description = "Tenant ID from request header"),
        ("Accept" =  String, Header,  example="application/json", description = "Accept Content Type: application/dicom+json or application/json"),
        ("Authorization" = String, Header,   description = "Optional JWT Access Token in Bearer format")
    ),
    responses(
        (status = 200, description = "Study subseries retrieved successfully", content_type = "application/dicom+json", body=[SubSeriesMeta]),
        (status = 404, description = "Study not found"),
        (status = 406, description = "Accept header must be application/dicom+json or application/json"),
        (status = 500, description = "Internal server error")
    ),
    tag =  WADO_RS_TAG,
    description = "Retrieve Study Sub-Series in DICOM JSON format",
)]
#[get("/studies/{study_instance_uid}/series")]
async fn retrieve_study_subseries(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    study_instance_uid: Path<String>,
) -> impl Responder {
    let log = app_state.log.clone();
    let study_uid = study_instance_uid.into_inner();
    let tenant_id = common_utils::get_tenant_from_handler(&req);
    info!(
        log,
        "retrieve_study_metadata Tenant ID: {}  and StudyUID:{} ", tenant_id, study_uid
    );
    // 检查 Accept 头
    let accept = req.headers().get(ACCEPT).and_then(|v| v.to_str().ok());

    if let Some(accept_str) = accept {
        if !is_accept_type_supported(accept_str, ACCEPT_DICOM_JSON_TYPE)
            && !is_accept_type_supported(accept_str, ACCEPT_JSON_TYPE)
        {
            return HttpResponse::NotAcceptable().body(format!(
                "retrieve_study_metadata Accept header must be {} or {}",
                ACCEPT_DICOM_JSON_TYPE, ACCEPT_JSON_TYPE
            ));
        }
    } else {
        return HttpResponse::NotAcceptable().body(format!(
            "retrieve_study_metadata Accept header must be {} or {}",
            ACCEPT_DICOM_JSON_TYPE, ACCEPT_JSON_TYPE
        ));
    }
    // 首先尝试从 Redis 缓存中获取数据
    let rh = &app_state.redis_helper;
    // 防止短期内多次访问导致数据库压力过大, 使用Redis缓存判断数据库中存在对应的实体类.
    let is_not_found = rh.get_study_entity_not_exists(tenant_id.as_str(), study_uid.as_str());

    if is_not_found.await.unwrap() == true {
        return HttpResponse::NotFound().body(format!(
            "retrieve_study_metadata Study not found in database retry after 30 seconds: {},{}",
            tenant_id, study_uid
        ));
    }
    //  从缓存中加载study_info
    let study_info =
        match get_study_info_with_cache(&tenant_id, &study_uid, &app_state, false).await {
            Ok(info) => info,
            Err(response) => return response,
        };
    if study_info.is_empty() {
        return HttpResponse::NotFound().body(format!(
            "retrieve_study_metadata Study not found in database retry after 30 seconds: {},{}",
            tenant_id, study_uid
        ));
    }
    let items: Vec<SubSeriesMeta> = study_info
        .iter()
        .map(|s| SubSeriesMeta::new(s).into())
        .collect::<Vec<_>>();

    match serde_json::to_string(&items) {
        Ok(content) => HttpResponse::Ok()
            .content_type(ACCEPT_DICOM_JSON_TYPE)
            .body(content),
        Err(e) => HttpResponse::InternalServerError().body(format!(
            "retrieve_study_subseries Failed to read JSON file: {} ",
            e
        )),
    }
}

#[utoipa::path(
    get,

    params(
        ("study_instance_uid" = String, Path, description = "Study Instance UID"),
        ("series_instance_uid" = String, Path, description = "Series Instance UID"),
        ("x-tenant" = String, Header, description = "Tenant ID from request header"),
        ("Accept" =  String, Header, example="application/json", description = "Accept Content Type: application/dicom+json or application/json"),
        ("Authorization" = Option<String>, Header,   description = "Optional JWT Access Token in Bearer format")

    ),
    responses(
        (status = 200, description = "Series metadata retrieved successfully", content_type = "application/dicom+json"),
        (status = 404, description = "Series or Study not found"),
        (status = 406, description = "Accept header must be application/dicom+json or application/json"),
        (status = 500, description = "Internal server error")
    ),
     tag =  WADO_RS_TAG,
    description = "Retrieve Series Metadata in DICOM JSON format"
)]
#[get("/studies/{study_instance_uid}/series/{series_instance_uid}/metadata")]
async fn retrieve_series_metadata(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    path: Path<(String, String)>,
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
                "retrieve_study_metadata Accept header must be {} or {}",
                ACCEPT_DICOM_JSON_TYPE, ACCEPT_JSON_TYPE
            ));
        }
    } else {
        return HttpResponse::NotAcceptable().body(format!(
            "retrieve_study_metadata Accept header must be {} or {}",
            ACCEPT_DICOM_JSON_TYPE, ACCEPT_JSON_TYPE
        ));
    }
    // 首先尝试从 Redis 缓存中获取数据
    let rh = RedisHelper::new(app_state.config.redis.clone());
    // 防止短期内多次访问导致数据库压力过大, 使用Redis缓存判断数据库中存在对应的实体类.
    let is_not_found = rh.get_study_entity_not_exists(tenant_id.as_str(), study_uid.as_str());

    if is_not_found.await.unwrap() == true {
        return HttpResponse::NotFound().body(format!(
            "retrieve_series_metadata Study not found in database retry after 30 seconds: {},{}",
            tenant_id, study_uid
        ));
    }
    // ... existing code ...
    let study_info = match get_study_info_with_cache(&tenant_id, &study_uid, &app_state, true).await
    {
        Ok(info) => info,
        Err(response) => return response,
    };
    if study_info.is_empty() {
        return HttpResponse::NotFound().body(format!(
            "retrieve_series_metadata Study not found in database retry after 30 seconds: {},{}",
            tenant_id, study_uid
        ));
    }

    let series_info = study_info
        .iter()
        .find(|info| info.series_uid.as_str() == series_uid)
        .cloned();

    let series_info = match series_info {
        Some(v) => v,
        None => {
            return HttpResponse::NotFound().body(format!(
                "retrieve_series_metadata seies not found in database retry after 30 seconds: {},{}",
                tenant_id, series_uid
            ));
        }
    };

    info!(log, "Series Info: {:?}", series_info);

    while rh
        .get_series_metadata_gererate(tenant_id.as_str(), series_uid.as_str())
        .await
        .is_ok()
    {
        info!(
            log,
            "get_series_metadata_gererate is Ok , sleep 100 ms to wait generating json stopped"
        );
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    let storage_config = StorageConfig::new(app_state.config.clone());

    let json_file_path = match storage_config.json_metadata_path_for_series(&series_info, true) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to generate JSON directory: {}", e));
        }
    };
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

    match generate_series_json(&series_info).await {
        Ok(json_str) => HttpResponse::Ok()
            .content_type(ACCEPT_DICOM_JSON_TYPE)
            .body(json_str),
        Err(e) => HttpResponse::InternalServerError().body(format!(
            "retrieve_study_metadata Failed to generate_series_json : {}",
            e
        )),
    }
}
#[utoipa::path(
    get,

    params(
        ("study_instance_uid" = String, Path, description = "Study Instance UID"),
        ("series_instance_uid" = String, Path, description = "Series Instance UID"),
        ("sop_instance_uid" = String, Path, description = "SOP Instance UID"),
        ("x-tenant" = String, Header, description = "Tenant ID from request header"),
        ("Accept" =  String, Header, example="application/json", description = "Accept Content Type: application/dicom  or application/octet-stream"),
        ("Authorization" = Option<String>, Header,   description = "Optional JWT Access Token in Bearer format")
    ),
    responses(
        (status = 200, description = "Instance retrieved successfully", content_type = "application/octet-stream"),
        (status = 404, description = "Instance, Series or Study not found"),
       (status = 406, description = "Accept header must be application/dicom or application/octet-stream"),
        (status = 500, description = "Internal server error")
    ),
     tag =  WADO_RS_TAG,
     description = "Retrieve Instance Pixel Data in Octet Stream format"
)]
#[get("/studies/{study_instance_uid}/series/{series_instance_uid}/instances/{sop_instance_uid}")]

async fn retrieve_instance(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    path: Path<(String, String, String)>,
) -> impl Responder {
    let (study_uid, series_uid, sop_uid) = path.into_inner();
    retrieve_instance_impl(study_uid, series_uid, sop_uid, 1, req, app_state).await
}

#[utoipa::path(
    get,

    params(
        ("study_instance_uid" = String, Path, description = "Study Instance UID"),
        ("series_instance_uid" = String, Path, description = "Series Instance UID"),
        ("sop_instance_uid" = String, Path, description = "SOP Instance UID"),
        ("frame_number" = u32, Path, description = "Frame Number"),
        ("x-tenant" = String, Header, description = "Tenant ID from request header"),
        ("Accept" =  String, Header, example="application/json", description = "Accept Content Type: application/dicom  or application/octet-stream"),
        ("Authorization" = Option<String>, Header,   description = "Optional JWT Access Token in Bearer format")
    ),
    responses(
        (status = 200, description = "Instance frame retrieved successfully"),
        (status = 404, description = "Instance frame not found"),
        (status = 406, description = "Accept header must be application/dicom or application/octet-stream"),
        (status = 500, description = "Internal server error"),
        (status = 501, description = "Not implemented")
    ),
    tag =  WADO_RS_TAG,
    description = "Retrieve Instance Frame Pixel Data in Octet Stream format"
)]
#[get(
    "/studies/{study_instance_uid}/series/{series_instance_uid}/instances/{sop_instance_uid}/frames/{frames}"
)]

async fn retrieve_instance_frames(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    path: Path<(String, String, String, u32)>,
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
    // 检查 Accept 头
    let accept = req.headers().get(ACCEPT).and_then(|v| v.to_str().ok());

    if let Some(accept_str) = accept {
        if !is_accept_type_supported(accept_str, ACCEPT_OCTET_STREAM)
            && !is_accept_type_supported(accept_str, ACCEPT_DICOM_TYPE)
        {
            return HttpResponse::NotAcceptable().body(format!(
                "retrieve_study_metadata Accept header must be {} or {}",
                ACCEPT_OCTET_STREAM, ACCEPT_DICOM_TYPE
            ));
        }
    } else {
        return HttpResponse::NotAcceptable().body(format!(
            "retrieve_study_metadata Accept header must be {} or {}",
            ACCEPT_OCTET_STREAM, ACCEPT_DICOM_TYPE
        ));
    }
    // 获取series_info (使用提取的函数)
    let study_info = match get_study_info_with_cache(&tenant_id, &study_uid, &app_state, true).await
    {
        Ok(info) => info,
        Err(response) => return response,
    };
    if study_info.is_empty() {
        return HttpResponse::NotFound().body(format!(
            "retrieve_series_metadata Study not found in database retry after 30 seconds: {},{}",
            tenant_id, study_uid
        ));
    }
    let series_info = study_info
        .iter()
        .find(|info| info.series_uid.as_str() == series_uid)
        .cloned();

    let series_info = match series_info {
        Some(v) => v,
        None => {
            return HttpResponse::NotFound().body(format!(
                "retrieve_instance_impl seies not found in database retry after 30 seconds: {},{}",
                tenant_id, series_uid
            ));
        }
    };

    info!(log, "Series Info: {:?}", series_info);

    let storage_config = StorageConfig::new(app_state.config.clone());

    let dicom_dir = match storage_config.dicom_series_dir(&series_info, false) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to generate DICOM directory: {}", e));
        }
    };

    let dicom_file = dicom_file_path(&dicom_dir, &sop_uid);
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
// Echo endpoint - 如果你也想让它出现在API文档里:
#[utoipa::path(
    get,
    responses(
        (status = 200, description = "Echo Success"),
    ),
    tag = WADO_RS_TAG,
    description = "Echo endpoint"
)]
#[get("/echo")]
async fn echo_v1() -> impl Responder {
    HttpResponse::Ok().body("Success")
}

use crate::auth_middleware_kc::Claims; // 确保能访问Claims结构

#[allow(dead_code)]
/// 用户权限检查函数
/// realm_roles: 角色列表, Realm级别到的角色, 例如"doctor", "manager", patient", "admin" 用于标示用户类别, 表示你是谁?
/// resource_roles_or_permissions: 资源级别的角色或权限列表, 例如 "Read", "Write", "Delete" 用于标示用户权限, 表示你能做什么?
/// resource_ids: 资源ID列表, 例如 "wado-rs-api", "account", 用于指定检查哪些资源下的角色
/// 身份判断：realm_access.roles.contains("role_patient")
/// 权限判断：resource_access['wado-rs-api'].roles.contains('study_viewer'')
/// *** 不要用 scope 字段做权限控制***
#[allow(dead_code)]
fn check_user_permissions(
    req: &HttpRequest,

    realm_roles: &[&str],

    resource_roles_or_permissions: &[&str],

    resource_ids: &[&str], // 新增参数
) -> bool {
    // 从请求扩展中获取用户信息
    let extensions = req.extensions(); // 先绑定到一个变量
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => return false, // 没有找到用户信息
    };
    println!(
        "claims: {:?}, roles:{:?}, permissions:{:?}, resource_ids:{:?}",
        claims, realm_roles, resource_roles_or_permissions, resource_ids
    );
    // https://www.jwt.io/ claims 示例
    //{
    //   "exp": 1762506440,
    //   "iat": 1762506140,
    //   "jti": "trrtcc:05ce0ed4-a94d-8344-4cc2-8b86f0b3469b",
    //   "iss": "https://keycloak.medical.org:8443/realms/dicom-org-cn",
    //   "aud": [
    //     "wado-rs-api",
    //     "account"
    //   ],
    //   "sub": "ac901127-ee57-4e88-89a3-fed70d4eb429",
    //   "typ": "Bearer",
    //   "azp": "wado-rs-api",
    //   "acr": "1",
    //   "allowed-origins": [
    //     "/*"
    //   ],
    //   "realm_access": {
    //     "roles": [
    //       "offline_access",
    //       "default-roles-dicom-org-cn",
    //       "uma_authorization"
    //     ]
    //   },
    //   "resource_access": {
    //     "wado-rs-api": {
    //       "roles": [
    //         "uma_protection"
    //       ]
    //     },
    //     "account": {
    //       "roles": [
    //         "manage-account",
    //         "manage-account-links",
    //         "view-profile"
    //       ]
    //     }
    //   },
    //   "scope": "profile email",
    //   "email_verified": false,
    //   "clientHost": "172.26.0.1",
    //   "preferred_username": "service-account-wado-rs-api",
    //   "clientAddress": "172.26.0.1",
    //   "client_id": "wado-rs-api"
    // }
    //

    true
}
