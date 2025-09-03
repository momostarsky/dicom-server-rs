use crate::common_utils::{get_param_case_insensitive, parse_query_string_case_insensitive};
use crate::{AppState, common_utils};
use actix_web::cookie::time::macros::time;
use actix_web::http::header::ACCEPT;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web, web::Path};
use chrono::Utc;
use common::change_file_transfer::ChangeStatus;
use common::dicom_json_helper;
use common::dicom_json_helper::walk_directory;
use common::dicom_utils::get_tag_values;
use dicom_dictionary_std::tags;
use serde::Deserialize;
use serde_json::{Map, json};
use slog::{error, info};
use std::path::PathBuf;
use uuid::{NoContext, Timestamp, Uuid};

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
static ACCEPT_OCTET_STREAM_TYPE: &str = "multipart/related; type=application/octet-stream";

// 检查Accept头部是否包含指定的MIME类型（不区分大小写）
fn is_accept_type_supported(accept_header: &str, expected_type: &str) -> bool {
    let accepted_types: Vec<&str> = accept_header.split(',').map(|s| s.trim()).collect();

    let expected_type_lower = expected_type.to_lowercase();
    accepted_types
        .iter()
        .any(|&t| t.to_lowercase() == expected_type_lower)
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
    let accept = req.headers().get(ACCEPT).and_then(|v| v.to_str().ok());

    if accept != Some(ACCEPT_DICOM_JSON_TYPE) {
        return HttpResponse::NotAcceptable().body(format!(
            "retrieve_study_metadata Accept header must be {}",
            ACCEPT_DICOM_JSON_TYPE
        ));
    }

    let study_info = match app_state.db.get_study_info(&tenant_id, &study_uid).await {
        Ok(Some(info)) => info,
        Ok(None) => {
            return HttpResponse::NotFound().body(format!(
                "retrieve_instance not found: {},{}",
                tenant_id, study_uid
            ));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!(
                "retrieve_instance Failed to retrieve study info: {}",
                e
            ));
        }
    };
    info!(log, "Study Info: {:?}", study_info);
    let dicom_dir = format!(
        "{}/{}/{}/{}",
        app_state.local_storage_config.dicom_store_path,
        tenant_id,
        study_info.patient_id,
        study_uid
    );
    if !std::path::Path::new(&dicom_dir).exists() {
        return HttpResponse::NotFound().body(format!("DICOM directory not found: {}", dicom_dir));
    }

    let json_dir = format!(
        "{}/{}/{:?}",
        app_state.local_storage_config.json_store_path, tenant_id, study_info.study_date
    );
    if !std::path::Path::new(&json_dir).exists() {
        std::fs::create_dir_all(&json_dir).expect("create_dir_all failed for JSON directory");
    }
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

    let series_info = match app_state.db.get_series_info(&tenant_id, &series_uid).await {
        Ok(Some(info)) => info,
        Ok(None) => {
            return HttpResponse::NotFound().body(format!(
                "retrieve_instance not found: {},{}",
                tenant_id, study_uid
            ));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!(
                "retrieve_instance Failed to retrieve study info: {}",
                e
            ));
        }
    };
    info!(log, "Study Info: {:?}", series_info);

    let dicom_dir = format!(
        "{}/{}/{}/{}/{}",
        app_state.local_storage_config.dicom_store_path,
        tenant_id,
        series_info.patient_id,
        series_info.study_instance_uid,
        series_info.series_instance_uid,
    );
    if !std::path::Path::new(&dicom_dir).exists() {
        return HttpResponse::NotFound().body(format!("DICOM directory not found: {}", dicom_dir));
    }

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
        let sop_json = match dicom_object::OpenFileOptions::new()
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
        Ok(json_str) => HttpResponse::Ok()
            .content_type(ACCEPT_DICOM_JSON_TYPE)
            .body(json_str),
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
            "retrieve_instance_frames not implemented for more than one frame: {}",
            frames
        ));
    }

    info!(
        log,
        "retrieve_instance: study_instance_uid={}, series_instance_uid={}, sop_instance_uid={}",
        study_uid,
        series_uid,
        sop_uid
    );

    let tenant_id = common_utils::get_tenant_from_handler(&req);
    info!(
        log,
        "retrieve_instance Tenant ID: {}  and StudyUID:{} ", tenant_id, study_uid
    );

    // 获取 series_info (用于两个 endpoint)
    let series_info = match app_state.db.get_series_info(&tenant_id, &series_uid).await {
        Ok(Some(info)) => info,
        Ok(None) => {
            return HttpResponse::NotFound().body(format!(
                "retrieve_instance not found: {},{}",
                tenant_id, study_uid
            ));
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!(
                "retrieve_instance Failed to retrieve study info: {}",
                e
            ));
        }
    };

    info!(log, "series_info  : {:?}", series_info);

    let dicom_dir = format!(
        "{}/{}/{}/{}",
        app_state.local_storage_config.dicom_store_path,
        tenant_id,
        series_info.patient_id,
        study_uid
    );

    if !std::path::Path::new(&dicom_dir).exists() {
        return HttpResponse::NotFound().body(format!("DICOM directory not found: {}", dicom_dir));
    }

    let dicom_file = format!("{}/{}/{}.dcm", dicom_dir, series_uid, sop_uid);
    let dicom_bytes = match std::fs::read(&dicom_file) {
        Ok(bytes) => bytes,
        Err(_) => {
            return HttpResponse::NotFound().body(format!("DICOM file not found: {}", dicom_file));
        }
    };

    HttpResponse::Ok()
        .content_type(ACCEPT_DICOM_TYPE)
        .body(dicom_bytes)
}

#[get("/echo")]
async fn echo() -> impl Responder {
    HttpResponse::Ok().body("Success")
}

pub(crate) async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
