use crate::{AppState, cert_helper};
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use slog::info;
use regex::Regex;
use lazy_static::lazy_static;

#[derive(serde::Deserialize)]
struct ClientRegisterParams {
    client_id: String,
    client_name: String,
    client_machine_id: String,
    client_mac_address: String,
    end_date: String,
}

impl ClientRegisterParams {
    fn validate(&self) -> Result<(), String> {
        // Validate client_id: 字母数字组合，16位
        lazy_static! {
            static ref CLIENT_ID_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9]{16}$").unwrap();
        }
        if !CLIENT_ID_REGEX.is_match(&self.client_id) {
            return Err("client_id must be 16 characters long and contain only letters and numbers".to_string());
        }

        // Validate client_name: 字母数字组合并支持,. 10到64位
        lazy_static! {
            static ref CLIENT_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9,.\s]{10,64}$").unwrap();
        }
        if !CLIENT_NAME_REGEX.is_match(&self.client_name) {
            return Err("client_name must be between 10 and 64 characters long and contain only letters, numbers, commas, periods, and spaces".to_string());
        }

        // Validate client_machine_id: 字母数字组合，16~128位
        lazy_static! {
            static ref CLIENT_MACHINE_ID_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9]{16,128}$").unwrap();
        }
        if !CLIENT_MACHINE_ID_REGEX.is_match(&self.client_machine_id) {
            return Err("client_machine_id must be between 16 and 128 characters long and contain only letters and numbers".to_string());
        }

        // Validate client_mac_address: 网卡地址格式 (MAC地址格式)
        lazy_static! {
            static ref MAC_ADDRESS_REGEX: Regex = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap();
        }
        if !MAC_ADDRESS_REGEX.is_match(&self.client_mac_address) {
            return Err("client_mac_address must be in MAC address format (e.g., 00:1A:2B:3C:4D:5E or 00-1A-2B-3C-4D-5E)".to_string());
        }

        // Validate end_date: YYYYMMDD 格式
        lazy_static! {
            static ref END_DATE_REGEX: Regex = Regex::new(r"^\d{8}$").unwrap();
        }
        if !END_DATE_REGEX.is_match(&self.end_date) {
            return Err("end_date must be in YYYYMMDD format".to_string());
        }

        // Additional validation for end_date to ensure it's a valid date
        if let Err(_) = chrono::NaiveDate::parse_from_str(&self.end_date, "%Y%m%d") {
            return Err("end_date must be a valid date in YYYYMMDD format".to_string());
        }

        Ok(())
    }
}

#[get("/client/registe")]
async fn client_registe_get(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    params: web::Query<ClientRegisterParams>,
) -> impl Responder {
    process_client_registration(req, app_state, params.into_inner()).await
}

#[post("/client/registe")]
async fn client_registe_post(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    params: Option<web::Form<ClientRegisterParams>>,
    json_params: Option<web::Json<ClientRegisterParams>>,
) -> impl Responder {

    // 检查Content-Type头
    let content_type = req.headers().get("Content-Type")
        .and_then(|ct| ct.to_str().ok())
        .map(|ct| ct.to_string())
        .unwrap_or_default();

    // 根据Content-Type选择参数来源
    let params = if content_type.starts_with("application/x-www-form-urlencoded") {
        match params {
            Some(form_params) => form_params.into_inner(),
            None => return HttpResponse::BadRequest().body("Missing form parameters"),
        }
    } else if content_type.starts_with("application/json") {
        match json_params {
            Some(json_data) => json_data.into_inner(),
            None => return HttpResponse::BadRequest().body("Missing JSON parameters"),
        }
    } else {
        return HttpResponse::BadRequest().body(
            "Unsupported Content-Type. Supported types: application/x-www-form-urlencoded, application/json"
        );
    };

    process_client_registration(req, app_state, params).await
}

async fn process_client_registration(
    _req: HttpRequest,
    app_state: web::Data<AppState>,
    params: ClientRegisterParams,
) -> HttpResponse {
    let log = &app_state.log;

    // 验证参数
    if let Err(validation_error) = params.validate() {
        return HttpResponse::BadRequest().body(validation_error);
    }

    info!(
        log,
        "client_registe  client_id:{} , client:{}, endDate:{}",
        params.client_id,
        params.client_name,
        params.end_date
    );

    let (client_cert, client_seckey) = match cert_helper::generate_client_and_sign(
        &params.client_name,
        &params.client_id,
        &params.client_machine_id,
        &params.client_mac_address,
        &params.end_date,
    ) {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("generate_client_and_sign error:{}", e));
        }
    };

    let cert_file_path = format!("/opt/client-cert/client_{}.crt", &params.client_id);
    let key_file_path = format!("/opt/client-cert/client_{}.key", &params.client_id);
    match std::fs::write(&cert_file_path, &client_cert) {
        Ok(_) => {
            info!(log, "write cert file success:{}", cert_file_path);
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("write cert file error:{}", e));
        }
    };

    match std::fs::write(&key_file_path, &client_seckey) {
        Ok(_) => {
            info!(log, "write key file success:{}", key_file_path);
        }
        Err(e) => {
            return HttpResponse::InternalServerError().body(format!("write key file error:{}", e));
        }
    };

    let filename = std::path::Path::new(&cert_file_path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("client.crt");

    HttpResponse::Ok()
        .append_header(("Content-Type", "application/octet-stream"))
        .append_header(("Content-Disposition", format!("attachment; filename=\"{}\"", filename)))
        .body(client_cert)
}

#[get("/client/validate")]
async fn client_validate(
    client_id: String,     // 客户端ID
    client_seckey: String, // 结束时间
    req: HttpRequest,
    app_state: web::Data<AppState>,
) -> impl Responder {
    let log = &app_state.log;
    info!(log, "retrieve_study_metadata  client_id:{} ", client_id);
    return HttpResponse::NotAcceptable().body(format!(
        "retrieve_study_metadata Accept header must be {}",
        "application/json"
    ));
}
pub(crate) async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
