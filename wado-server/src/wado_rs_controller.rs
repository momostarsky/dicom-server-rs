use actix_web::http::header::ACCEPT;
use actix_web::{HttpRequest, HttpResponse, Responder, get, web, web::Path};
use serde::Deserialize;
use slog::{info};
use crate::{common_utils, AppState};
use crate::common_utils::{get_param_case_insensitive, parse_query_string_case_insensitive};

#[derive(Deserialize, Debug)]
struct StudyQueryParams {
    #[serde(rename = "charset")]
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

static ACCEPT_MIME_TYPE: &str = "application/dicom+json";
static ACCEPT_DICOM_TYPE: &str = "application/dicom";

#[get("/studies/{study_instance_uid}")]
async fn retrieve_study(
    study_instance_uid: Path<String>,
    req: HttpRequest,
    app_state : web::Data<AppState>,
) -> impl Responder {
    let study_uid = study_instance_uid.into_inner();
    let log = app_state.log.clone();
    info!(log, "retrieve_study: study_instance_uid {}", study_uid);
    let query_string = req.query_string();
    info!(log, "retrieve_study: query_string {}", query_string);
    // 手动解析查询参数
    let query_params = parse_query_string_case_insensitive(query_string);

    // 处理 charset 参数
    if let Some(charset) = get_param_case_insensitive(&query_params, WADO_CHARSET) {
        if let Some(value) = charset.first() {
            info!(log, "Charset: {}", value);
        }
    }

    // 处理 anonymize 参数
    if let Some(anonymize) = get_param_case_insensitive(&query_params, WADO_ANONYMIZE) {
        if let Some(value) = anonymize.first() {
            match value.to_lowercase().as_str() {
                "true" => info!(log, "Anonymize: true"),
                "false" => info!(log, "Anonymize: false"),
                _ => info!(log, "Anonymize: invalid value"),
            }
        }
    }

    // 处理 includeField 参数
    if let Some(include_fields) = get_param_case_insensitive(&query_params, WADO_INCLUDE_FIELD) {
        if !include_fields.is_empty() {
            info!(log, "Include Fields: {:?}", include_fields);
        }
    }

    // 处理 excludeField 参数
    if let Some(exclude_fields) = get_param_case_insensitive(&query_params, WADO_EXCLUDE_FIELD) {
        if !exclude_fields.is_empty() {
            info!(log, "Exclude Fields: {:?}", exclude_fields);
        }
    }

    let params = StudyQueryParams {
        charset: get_param_case_insensitive(&query_params, WADO_CHARSET)
            .and_then(|v| v.first().map(|s| s.trim().to_string())),
        anonymize: get_param_case_insensitive(&query_params, WADO_ANONYMIZE)
            .and_then(|v| v.first().map(|s| s.to_lowercase() == "true")),
        include_field: get_param_case_insensitive(&query_params, WADO_INCLUDE_FIELD).cloned(),
        exclude_field: get_param_case_insensitive(&query_params, WADO_EXCLUDE_FIELD).cloned(),
    };
    info!(&log, "Study Query Params: {:?}", params);

    HttpResponse::Ok().body(format!("Hello world! {}", study_uid))
}

#[get("/studies/{study_instance_uid}/metadata")]
async fn retrieve_study_metadata(
    study_instance_uid: Path<String>,
    req: HttpRequest,
    app_state : web::Data<AppState>,
) -> impl Responder {
    let log = app_state.log.clone();
    let study_uid = study_instance_uid.into_inner();
    let tenant_id = common_utils::get_tenant_from_handler(&req);
    info!(log, "retrieve_study_metadata Tenant ID: {}  and StudyUID:{} ", tenant_id, study_uid);
    let accept = req.headers().get(ACCEPT).and_then(|v| v.to_str().ok());

    if accept != Some(ACCEPT_MIME_TYPE) {
        return HttpResponse::NotAcceptable()
            .body(format!("retrieve_study_metadata Accept header must be {}", ACCEPT_MIME_TYPE));
    }

    let study_info = match app_state.db.get_study_info(&tenant_id, &study_uid).await {
        Some(info) => info,
        None => {
            return HttpResponse::NotFound().body(format!("retrieve_study_metadata not found: {},{}",tenant_id, study_uid));
        }
    };
    info!(log, "Study Info: {:?}", study_info);

    let json_file = format!("/home/dhz/jpdata/CDSS/89269/{}.json", study_uid);
    let json_text = match std::fs::read_to_string(&json_file) {
        Ok(text) => text,
        Err(_) => {
            return HttpResponse::NotFound().body(format!("JSON file not found: {}", json_file));
        }
    };

    HttpResponse::Ok()
        .content_type(ACCEPT_MIME_TYPE)
        .body(json_text)
}

#[get("/studies/{study_instance_uid}/series/{series_instance_uid}")]
async fn retrieve_series(path: Path<(String, String)>,  app_state : web::Data<AppState>,) -> impl Responder {
    let log = app_state.log.clone();
    let (study_uid, series_uid) = path.into_inner();
    info!(
        log,
        "retrieve_series: study_instance_uid={}, series_instance_uid={}", study_uid, series_uid
    );
    HttpResponse::Ok().body(format!("Hello world! {} {}", study_uid, series_uid))
}

#[get("/studies/{study_instance_uid}/series/{series_instance_uid}/metadata")]
async fn retrieve_series_metadata(
    path: Path<(String, String)>,
    req: HttpRequest,
    app_state : web::Data<AppState>,
) -> impl Responder {
    let log = app_state.log.clone();
    let (study_uid, series_uid) = path.into_inner();
    info!(
        log,
        "retrieve_series_metadata: study_instance_uid={}, series_instance_uid={}",
        study_uid,
        series_uid
    );
    // 检查 Accept 头
    let accept = req.headers().get(ACCEPT).and_then(|v| v.to_str().ok());

    if accept != Some(ACCEPT_MIME_TYPE) {
        return HttpResponse::NotAcceptable()
            .body(format!("Accept header must be {}", ACCEPT_MIME_TYPE));
    }
    HttpResponse::Ok().body(format!("Hello world! {} {}", study_uid, series_uid))
}

#[get("/studies/{study_instance_uid}/series/{series_instance_uid}/instances/{sop_instance_uid}")]
async fn retrieve_instance(
    path: Path<(String, String, String)>,
    req: HttpRequest,
    app_state : web::Data<AppState>,
) -> impl Responder {
    let log = app_state.log.clone();
    let (study_uid, series_uid, sop_uid) = path.into_inner();
    info!(
        log,
        "retrieve_instance: study_instance_uid={}, series_instance_uid={}, sop_instance_uid={}",
        study_uid,
        series_uid,
        sop_uid
    );
    // 检查 Accept 头
    // let accept = req.headers().get(ACCEPT).and_then(|v| v.to_str().ok());
    //
    // if accept != Some(ACCEPT_DICOM_TYPE) {
    //     return HttpResponse::NotAcceptable()
    //         .body(format!("Accept header must be {}", ACCEPT_DICOM_TYPE));
    // }
    let dicom_file = format!(
        "/home/dhz/jpdata/CDSS/89269/{}/{}/{}.dcm",
        study_uid, series_uid, sop_uid
    );
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

#[get(
    "/studies/{study_instance_uid}/series/{series_instance_uid}/instances/{sop_instance_uid}/metadata"
)]
async fn retrieve_instance_metadata(
    path: Path<(String, String, String)>,
    req: HttpRequest,
    app_state : web::Data<AppState>,
) -> impl Responder {
    let log = app_state.log.clone();
    let (study_uid, series_uid, sop_uid) = path.into_inner();
    info!(
        log,
        "retrieve_instance_metadata: study_instance_uid={}, series_instance_uid={}, sop_instance_uid={}",
        study_uid,
        series_uid,
        sop_uid
    );
    // 检查 Accept 头
    let accept = req.headers().get(ACCEPT).and_then(|v| v.to_str().ok());

    if accept != Some(ACCEPT_MIME_TYPE) {
        return HttpResponse::NotAcceptable()
            .body(format!("Accept header must be {}", ACCEPT_MIME_TYPE));
    }
    HttpResponse::Ok().body(format!(
        "Hello world! {} {} {}",
        study_uid, series_uid, sop_uid
    ))
}

#[get(
    "/studies/{study_instance_uid}/series/{series_instance_uid}/instances/{sop_instance_uid}/frames/{frames}"
)]
async fn retrieve_instance_frames(
    path: Path<(String, String, String, String)>,
    app_state : web::Data<AppState>,
) -> impl Responder {
    let log = app_state.log.clone();
    let (study_uid, series_uid, sop_uid, frames) = path.into_inner();
    info!(
        log,
        "retrieve_instance_frames: study_instance_uid={}, series_instance_uid={}, sop_instance_uid={}, frames={}",
        study_uid,
        series_uid,
        sop_uid,
        frames
    );
    HttpResponse::Ok().body(format!(
        "Hello world! {} {} {} {}",
        study_uid, series_uid, sop_uid, frames
    ))
}

#[get("/echo")]
async fn echo() -> impl Responder {
    HttpResponse::Ok().body("Success")
}

pub(crate) async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
