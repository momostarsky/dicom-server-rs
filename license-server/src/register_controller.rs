use crate::AppState;
use actix_web::{get, post, web, HttpRequest, HttpResponse, Responder};

use regex::Regex;
use lazy_static::lazy_static;
use slog::info;
use common::cert_helper;

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
// MIIE0zCCArugAwIBAgIBATANBgkqhkiG9w0BAQsFADCBwjELMAkGA1UEBhMCQ04x
// ETAPBgNVBAgMCFpoZWppYW5nMREwDwYDVQQHDAhIYW5nemhvdTEXMBUGA1UECgwO
// TGljZW5zZSBTZXJ2ZXIxFTATBgNVBAMMDGRpY29tLm9yZy5jbjEfMB0GCSqGSIb3
// DQEJARYQNDExNTkyMTQ4QHFxLmNvbTEbMBkGCgmSJomT8ixkAQEMCzE1OTY3MTMy
// MTcyMQwwCgYDVQQEDANkYWkxETAPBgNVBCoMCGhhbnpoYW5nMB4XDTI1MDkxMTAz
// NTcwNloXDTI2MDkxMTAzNTcwNlowMjELMAkGA1UEBhMCQ04xEDAOBgNVBAoMB1Nr
// eS5MVEQxETAPBgNVBAMMCEhaMTAwMDAxMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8A
// MIIBCgKCAQEAzOw0DG0O+TIzn8b6TrB14DA7FkQ1+l7q8G9xZI787O0Oj7FBw5VO
// B2KHUABbslpSH086sRak3EgzuiQUmMtTbdSVhckuT1Pp4nOVU0u+9toSBHJpGkRZ
// B2kdUR4fhQBJB+HGo745vDsmnPsk5s4VmpBCkb7lwbpKL8zpE6owRjhf1B6JDVV7
// TOKTMFv2/1/Am62kYC71vqtYjdIFmtrPehvvyeUyhjS+Utpi3pxCvoS//ocbVhWm
// /L07w7VZ0CN6JqIDUTtmAGqA+pJvmwpVI8VucKskR4CK90pzk1xpLZeWYxhQnnFl
// CVAGIcPhyyNCc7I97RbhVVe3ZjaiI/uKzQIDAQABo2MwYTATBgNVHSUEDDAKBggr
// BgEFBQcDAjApBgorBgEEu73ctAwBBBs4OTg5ODkzOTgzOThtb2lvaW8yeGlvMjIz
// MzIwHwYKKwYBBLu93LQMAgQRT0E6SUI6T0M6RTM6R0M6OEIwDQYJKoZIhvcNAQEL
// BQADggIBAGyO/gqGNl9ywUc+GVh0N4t2ts4nvw+uX1MQHxCOWZwzs3DafMY6qoG5
// wb5/OObHIJAKDjvC7aPIKtVY90pO2CRmaMas9Cuf6sdnt41LQrQO5V32wgg6AjaJ
// ilZhAuFREBdNAUgAr+xcfS8Ob5y6qtSPcpgSKBSp2kVCxepxQxHo9zt7mmzAhFq9
// Om4YhzC67PDwC1/96Bh/w8PYeNw1Fs4e9MJl4aAQPt/zgJjEs2BG+kBHumk2/WvI
// DK7vRxFayLD7AclKYstW8roITOPvZW12aL1yZE2ggUSuWmcKwdH3VKXm95Y5qhEB
// /O1+29lxE8QwBqTKZrjJgodXLWRHit3b8bsgzCk+nMKakznt/RMXL19IVful5opx
// LfuhhWmBRBxnurusnytYWCTgPkkfwaIRGtFQoTLZ52YcrQrp1WALVL2aqgnHIfkq
// +/hy4asqkqZbypIz//aexavdZytOQcgwmqbWn6Glp0RirrpYVvZah+bPyja0mC5Z
// Ia00R+biBySj+ZdNJY/9RwuzOsxVukYEbYwiMo64bEsalIYrSVl7XEIRRwDjnYP1
// PeLK2igwTS32KfHQh6LfJGv9ozW2trx2r/A3yIUHZx6xfIkui3iE+O1Ru6Dyp9qW
// HuJIB6TAV0KcmXlk8J0wODpl01GCV1fwQ6Z/mNsQfawBLP0Pg8QE
/// -----END CERTIFICATE-----
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

pub(crate) async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}
