use crate::slog::Error;
use crate::common_utils::generate_series_json;
use crate::constants::WADO_RS_TAG;
use crate::wado_rs_models::SubSeriesMeta;
use crate::{AppState, common_utils};
use actix_web::http::header::ACCEPT;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, Responder, get, web, web::Path};
use common::dicom_json_helper;
use common::redis_key::RedisHelper;
use common::server_config::{
    dicom_file_path, dicom_series_dir, dicom_study_dir, json_metadata_for_series,
    json_metadata_for_study,
};
use database::dicom_meta::DicomStateMeta;
use dicom_dictionary_std::tags;
use dicom_object::OpenFileOptions;
use permission_macros::permission_required;
use slog::info;
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
        match rh.get_study_metadata(tenant_id, study_uid) {
            Ok(metas) => {
                info!(log, "Retrieved study_info from Redis cache");
                return Ok(metas);
            }
            Err(_) => {}
        }
    }

    match app_state.db.get_state_metaes(tenant_id, study_uid).await {
        Ok(metas) => {
            info!(log, "Retrieved study_info from database");
            rh.del_study_entity_not_exists(tenant_id, study_uid);
            // 将查询结果序列化并写入 Redis 缓存，过期时间设置为2小时
            rh.set_study_metadata(tenant_id, study_uid, &metas, 2 * RedisHelper::ONE_HOUR);
            Ok(metas)
        }
        Err(e) => {
            let error_msg = format!("Failed to retrieve study info: {}", e);
            rh.set_study_entity_not_exists(tenant_id, study_uid, 5 * RedisHelper::ONE_MINULE);
            Err(HttpResponse::InternalServerError().body(error_msg))
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
#[permission_required(roles = ["doctor", "manager"], permissions = ["Read"], resource_id =["wado-rs-api"])]
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

    // 首先尝试从 Redis 缓存中获取数据
    let rh = &app_state.redis_helper;
    // 防止短期内多次访问导致数据库压力过大, 使用Redis缓存判断数据库中存在对应的实体类.
    let is_not_found = rh.get_study_entity_not_exists(tenant_id.as_str(), study_uid.as_str());

    if is_not_found.is_ok() {
        return HttpResponse::NotFound().body(format!(
            "retrieve_study_metadata Study not found in database retry after 30 seconds: {},{}",
            tenant_id, study_uid
        ));
    }
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
    let study_info = study_info.first().unwrap();
    let json_path = match json_metadata_for_study(&study_info, false) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to compute json_metadata_for_study: {}", e));
        }
    };

    let dicom_dir = match dicom_study_dir(&study_info, false) {
        Ok(v) => v,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("Failed to compute DICOM directory: {}", e));
        }
    };

    if !std::path::Path::new(&json_path).exists() {
        let dicom_path = PathBuf::from(&dicom_dir);
        let json_path = PathBuf::from(&json_path);
        info!(log, "DICOM directory: {:?}", dicom_path);
        if let Err(e) = dicom_json_helper::generate_json_file(&dicom_path, &json_path) {
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
#[get("/studies/{study_instance_uid}/subseries")]
#[permission_required(roles = ["doctor", "manager"], permissions = ["Read"], resource_id = ["wado-rs-api"])]
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

    if is_not_found.is_ok() {
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

    if is_not_found.is_ok() {
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

    if series_info.is_none() {
        return HttpResponse::NotFound()
            .body(format!("Series not found in study info: {}", series_uid));
    }
    info!(log, "Series Info: {:?}", series_info);

    let series_info = series_info.unwrap();
    let json_file_path = match json_metadata_for_series(&series_info, true) {
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

    if series_info.is_none() {
        return HttpResponse::NotFound()
            .body(format!("Series not found in study info: {}", series_uid));
    }
    info!(log, "Series Info: {:?}", series_info);

    let series_info = series_info.unwrap();

    let dicom_dir = match dicom_series_dir(&series_info, false) {
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

// 权限检查函数
fn check_user_permissions(
    req: &HttpRequest,
    required_roles: &[&str],
    required_permissions: &[&str],
    resource_id: &[&str], // 新增参数
) -> bool {
    // 从请求扩展中获取用户信息
    let extensions = req.extensions(); // 先绑定到一个变量
    let claims = match extensions.get::<Claims>() {
        Some(claims) => claims,
        None => return false, // 没有找到用户信息
    };

    println!("check_user_permissions: {:?}", &claims);

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

    // 检查角色
    if !required_roles.is_empty() {
        // let has_required_role = required_roles.iter().any(|&required_role| {
        //     // 检查realm级别角色
        //     if let Some(realm_access) = &claims.realm_access {
        //         if let Some(roles) = &realm_access.roles {
        //             return roles.iter().any(|user_role| user_role == required_role);
        //         }
        //     }
        //
        //     // 检查资源级别角色
        //     if let Some(resource_access) = &claims.resource_access {
        //         for (_, access) in resource_access {
        //             if let Some(roles) = &access.roles {
        //                 if roles.iter().any(|user_role| user_role == required_role) {
        //                     return true;
        //                 }
        //             }
        //         }
        //     }
        //     false
        // });
        let has_required_role = required_roles.iter().any(|&required_role| {
            // 如果指定了 resource_id，则只检查该 resource 下的角色
            if !resource_id.is_empty() {
                if let Some(resource_access) = &claims.resource_access {
                    // 遍历所有指定的资源ID
                    for &resource_key in resource_id {
                        if let Some(access) = resource_access.get(resource_key) {
                            if let Some(roles) = &access.roles {
                                if roles.iter().any(|user_role| user_role == required_role) {
                                    return true;
                                }
                            }
                        }
                    }
                }
            } else {
                // 检查realm级别角色
                if let Some(realm_access) = &claims.realm_access {
                    if let Some(roles) = &realm_access.roles {
                        return roles.iter().any(|user_role| user_role == required_role);
                    }
                }

                // 检查所有资源级别角色
                if let Some(resource_access) = &claims.resource_access {
                    for (_, access) in resource_access {
                        if let Some(roles) = &access.roles {
                            if roles.iter().any(|user_role| user_role == required_role) {
                                return true;
                            }
                        }
                    }
                }
            }
            false
        });
        if !has_required_role {
            return false;
        }
    }

    // 检查权限范围
    if !required_permissions.is_empty() {
        if let Some(scope) = &claims.scope {
            let user_scopes: Vec<&str> = scope.split_whitespace().collect();
            let has_required_permission = required_permissions
                .iter()
                .any(|&required_perm| user_scopes.contains(&required_perm));

            if !has_required_permission {
                return false;
            }
        } else {
            return false; // 用户没有权限范围信息
        }
    }

    true
}
