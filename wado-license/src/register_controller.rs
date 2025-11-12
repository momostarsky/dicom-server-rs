use crate::AppState;
use actix_web::{HttpRequest, HttpResponse, Responder, get, post, web};
use std::fs;

use common::cert_helper;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use slog::info;
use tokio::fs::File as TokioFile;
use tokio_util::io::ReaderStream;
#[derive(serde::Deserialize,Serialize)]
pub(crate) struct ClientRegisterParams {
    pub(crate) client_id: String,
    pub(crate) client_name: String,
    pub(crate) client_hash_code: String,
    pub(crate) end_date: String,
}

pub(crate) const   CA_FILE: &str = "/opt/dicom-server/ca_root.pem";
pub(crate) const CA_KEY_FILE: &str = "/opt/dicom-server/ca_key_root.pem";

impl ClientRegisterParams {
    pub(crate) fn validate(&self) -> Result<(), String> {
        // Validate client_id: 字母数字组合，16位
        lazy_static! {
            static ref CLIENT_ID_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9]{16}$").unwrap();
        }
        if !CLIENT_ID_REGEX.is_match(&self.client_id) {
            return Err(
                "client_id must be 16 characters long and contain only letters and numbers"
                    .to_string(),
            );
        }

        // Validate client_name: 字母数字组合并支持,. 10到64位
        lazy_static! {
             static ref CLIENT_NAME_REGEX: Regex = Regex::new(r"^[a-zA-Z0-9,.\s\u{4e00}-\u{9fff}]{10,64}$").unwrap();
        }
        if !CLIENT_NAME_REGEX.is_match(&self.client_name) {
            return Err("client_name must be between 10 and 64 characters long and contain only letters, numbers, commas, periods, and spaces".to_string());
        }

        // Validate client_machine_id: 字母数字组合，16~128位
        lazy_static! {
            static ref CLIENT_HASH_CODE_REGEX: Regex =
                Regex::new(r"^[a-zA-Z0-9]{16,128}$").unwrap();
        }
        if !CLIENT_HASH_CODE_REGEX.is_match(&self.client_hash_code) {
            return Err("client_hash_code must be between 16 and 128 characters long and contain only letters and numbers".to_string());
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

/// 处理客户端注册
///
/// # Arguments
///
/// * `req` - HTTP请求对象
/// * `app_state` - 应用状态数据
/// * `params` - 客户端注册参数
///
/// # Returns
///
/// * `impl Responder` - HTTP响应对象
///
/// # 说明
///
/// 该函数处理客户端注册请求，根据请求参数生成客户端证书并返回。
/// curl "http://116.63.110.45:8888/client/registe?client_id=HZ10000XXX1&client_name=Sky.LTD&client_machine_id=898989398398moioio2xio22332&client_mac_address=OA:IB:OC:E3:GC:8B&end_date=20261231"
///
/// # 示例
///
/// 注册一个客户端：
///
/// ```sh
/// curl "http://116.63.110.45:8888/client/registe?client_id=HZ10000XXX1&client_name=Sky.LTD&client_machine_id=898989398398moioio2xio22332&client_mac_address=OA:IB:OC:E3:GC:8B&end_date=20261231"
/// ```
#[get("/client/registe")]
async fn client_registe_get(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    params: web::Query<ClientRegisterParams>,
) -> impl Responder {
    process_client_registration(req, app_state, params.into_inner()).await
}

/// 获取CA公钥证书
///
/// # Arguments
///
/// * `app_state` - 应用状态数据
///
/// # Returns
///
/// * `impl Responder` - 包含CA公钥证书的JSON响应
///
/// # 说明
///
/// 该函数返回CA公钥证书，客户端可以使用此证书验证由该CA签发的证书。
#[get("/ca")]
async fn get_ca_certificate(app_state: web::Data<AppState>) -> impl Responder {
    let log = &app_state.log;

    // 读取CA证书内容
    // let ca_cert_content = match fs::read_to_string(CA_FILE) {
    //     Ok(content) => content,
    //     Err(e) => {
    //         slog::error!(log, "Failed to read CA certificate file: {}", e);
    //         return HttpResponse::InternalServerError()
    //             .body(format!("Failed to read CA certificate: {}", e));
    //     }
    // };

    // 尝试以 Tokio 文件方式打开
    let file = match TokioFile::open(&CA_FILE).await {
        Ok(f) => f,
        Err(e) => {
            slog::error!(log, "Failed to open CA certificate file: {}", e);
            return HttpResponse::InternalServerError()
                .body(format!("Failed to open CA certificate file: {}", e));
        }
    };

    // 将文件转换为 Stream
    let stream = ReaderStream::new(file);

    // 设置 Content-Disposition，指定下载的默认文件名为 dicom-org-cn.crt
    let content_disposition = "attachment; filename=\"dicom-org-cn.crt\"";

    HttpResponse::Ok()
        .append_header(("Content-Type", "application/octet-stream"))
        .append_header(("Content-Disposition", content_disposition))
        .streaming(stream)
}
/// 处理客户端注册
///
/// # Arguments
///
/// * `req` - HTTP请求对象
/// * `app_state` - 应用状态数据
/// * `params` - 客户端注册参数
///
/// # Returns
///
/// * `impl Responder` - HTTP响应对象
///
/// # 说明
///
/// 该函数处理客户端注册请求，根据请求参数生成客户端证书并返回。
/// 支持两种请求方式：GET和POST。
///
/// # 示例
///
/// 注册一个客户端：
///
/// ```sh
/// curl -X POST \
//   -H "Content-Type: application/x-www-form-urlencoded" \
//   -d "client_id=HZ100001&client_name=Sky.LTD&client_machine_id=898989398398moioio2xio22332&client_mac_address=OA:IB:OC:E3:GC:8B&end_date=20261231" \
//   http://116.63.110.45:8888/client/registe
/// ```
///
/// 注册成功后，服务器会返回一个包含客户端证书的响应。
///
/// 证书内容示例：
///
/// ```
/// -----BEGIN CERTIFICATE-----
/// XXXXXXXXXXXXXX
/// -----END CERTIFICATE-----
#[post("/client/registe")]
async fn client_registe_post(
    req: HttpRequest,
    app_state: web::Data<AppState>,
    params: Option<web::Form<ClientRegisterParams>>,
    json_params: Option<web::Json<ClientRegisterParams>>,
) -> impl Responder {
    // 检查Content-Type头
    let content_type = req
        .headers()
        .get("Content-Type")
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
        &params.client_hash_code,
        &params.end_date,
        &CA_FILE,
        &CA_KEY_FILE,
    ) {
        Ok(result) => result,
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("generate_client_and_sign error:{}", e));
        }
    };

    let client_cert_file_path = format!("/opt/client-cert/client_{}.crt", &params.client_id);
    let client_key_file_path = format!("/opt/client-cert/client_{}.key", &params.client_id);
    match fs::write(&client_cert_file_path, &client_cert) {
        Ok(_) => {
            info!(
                log,
                "write client cert file success:{}", client_cert_file_path
            );
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("write client cert file error:{}", e));
        }
    };

    match fs::write(&client_key_file_path, &client_seckey) {
        Ok(_) => {
            info!(
                log,
                "write client key file success:{}", client_key_file_path
            );
        }
        Err(e) => {
            return HttpResponse::InternalServerError()
                .body(format!("write client key file error:{}", e));
        }
    };

    // 尝试以 Tokio 文件方式打开
    let file = match TokioFile::open(&client_cert_file_path).await {
        Ok(f) => f,
        Err(e) => {
            slog::error!(log, "Failed to open CA certificate file: {}", e);
            return HttpResponse::InternalServerError()
                .body(format!("Failed to open CA certificate file: {}", e));
        }
    };

    // 将文件转换为 Stream
    let stream = ReaderStream::new(file);

    // 设置 Content-Disposition，指定下载的默认文件名为 dicom-org-cn.crt
    let content_disposition = format!("attachment; filename=\"client_{}.crt\"", &params.client_id);
    HttpResponse::Ok()
        .append_header(("Content-Type", "application/octet-stream"))
        .append_header(("Content-Disposition", content_disposition))
        .streaming(stream)
}

pub(crate) async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
