mod register_controller;

use crate::register_controller::{client_registe, client_validate, manual_hello};
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use openssl::asn1::{Asn1Integer, Asn1Time};
use openssl::bn::BigNum;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage};
use openssl::x509::{X509, X509NameBuilder, X509Req, X509Ref};
use openssl::x509::X509VerifyResult;
use slog::Drain;
use slog::{Logger, o};
use std::time::{SystemTime, UNIX_EPOCH};
// 定义应用状态

#[derive(Clone)]
struct AppState {
    log: Logger,
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    generate_ca_root().expect("生成CA根证书失败!");
    generate_client_and_sign("HzMomoStarsky.LTD","HZ109999").expect("生成客户端证书失败!");
    let log = configure_log();
    let app_state = AppState { log: log.clone() };
    HttpServer::new(move || {
        let mut cors = Cors::default()
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);

        cors = cors.allow_any_origin();

        App::new()
            // 使用.wrap()方法添加Compress中间件
            .wrap(middleware::Compress::default())
            .wrap(cors)
            .app_data(web::Data::new(app_state.clone()))
            .service(client_registe)
            .service(client_validate)
            .route("/hey", web::get().to(manual_hello))
    })
    .bind(("0.0.0.0", 8888))?
    .run()
    .await
}

fn generate_client_and_sign(
    client_org: &str,
    client_id: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 读取CA证书
    let ca_cert_pem = std::fs::read("./ca.pem").expect("读取CA证书失败!");
    let ca_cert = X509::from_pem(&ca_cert_pem)?;

    // 读取CA私钥
    let ca_key_pem = std::fs::read("./ca_key.pem").expect("读取CA私钥失败!");
    let ca_pkey = PKey::private_key_from_pem(&ca_key_pem)?;

    // =============================
    // 2. 生成客户端密钥 & CSR
    // =============================

    let client_rsa = Rsa::generate(2048)?;
    let client_pkey = PKey::from_rsa(client_rsa)?;

    let mut client_name = X509NameBuilder::new()?;
    client_name.append_entry_by_text("C", "CN")?;
    client_name.append_entry_by_text("O", client_org)?;
    client_name.append_entry_by_text("CN", client_id)?; // 客户端唯一标识
    let client_name = client_name.build();

    // 创建证书请求
    let mut req_builder = X509Req::builder()?;
    req_builder.set_version(0)?;
    req_builder.set_subject_name(&client_name)?;
    req_builder.set_pubkey(&client_pkey)?;
    req_builder.sign(&client_pkey, MessageDigest::sha256())?;
    let client_req = req_builder.build();

    println!("✅ [Client] 客户端 CSR 已生成");

    // =============================
    // 3. 用 CA 签发客户端证书（授权证书）
    // =============================
    let mut cert_builder = X509::builder()?;
    cert_builder.set_version(2)?;

    // 设置序列号
    let serial = BigNum::from_u32(1)?;
    let serial = Asn1Integer::from_bn(&serial)?;
    cert_builder.set_serial_number(&serial)?;

    // 正确设置主题名和公钥
    cert_builder.set_subject_name(&client_name)?; // 使用之前创建的client_name
    cert_builder.set_issuer_name(ca_cert.subject_name())?; // 由 CA 签发
    cert_builder.set_pubkey(&client_pkey)?; // 直接使用client_pkey

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let not_before = Asn1Time::from_unix(now as i64)?;
    let not_after = Asn1Time::from_unix((now + 365 * 24 * 60 * 60) as i64)?; // 1 年有效期
    cert_builder.set_not_before(&not_before)?;
    cert_builder.set_not_after(&not_after)?;

    // 关键：添加扩展 => 表明该证书用于客户端身份认证
    cert_builder.append_extension(
        ExtendedKeyUsage::new()
            .client_auth() // 表示该证书用于客户端 TLS 认证
            .build()?,
    )?;

    // 用 CA 私钥签名客户端证书
    cert_builder.sign(&ca_pkey, MessageDigest::sha256())?;
    let client_cert = cert_builder.build();

    // 保存客户端证书
    let client_cert_pem = client_cert.to_pem()?;
    let filename = format!("client_{}.crt", client_id);
    std::fs::write(&filename, client_cert_pem)?;

    // 保存客户端私钥
    let client_key_pem = client_pkey.private_key_to_pem_pkcs8()?;
    let key_filename = format!("client_{}.key", client_id);
    std::fs::write(&key_filename, client_key_pem)?;

    println!("✅ [Client] 授权证书已由 CA 签发并保存为 {}", filename);
    Ok(())
}

fn generate_ca_root() -> Result<(), Box<dyn std::error::Error>> {
    // =============================
    // 1. 生成 CA（根证书，模拟授权机构）
    // =============================

    // 生成 CA 私钥 (RSA 2048)
    let ca_rsa = Rsa::generate(4096)?;
    let ca_pkey = PKey::from_rsa(ca_rsa)?;

    // 构造 CA 证书的 Subject
    let mut ca_name = X509NameBuilder::new()?;
    ca_name.append_entry_by_text("C", "CN")?;
    ca_name.append_entry_by_text("O", register_controller::ORG_NAME)?;
    ca_name.append_entry_by_text("CN", register_controller::ROOT_NAME)?;
    let ca_name = ca_name.build();

    // 构建 CA 证书
    let mut ca_builder = X509::builder()?;
    ca_builder.set_version(2)?;

    // 设置序列号
    let serial = BigNum::from_u32(1)?;
    let serial = Asn1Integer::from_bn(&serial)?;
    ca_builder.set_serial_number(&serial)?;

    ca_builder.set_subject_name(&ca_name)?;
    ca_builder.set_issuer_name(&ca_name)?; // 自签名
    ca_builder.set_pubkey(&ca_pkey)?;

    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(365 * 10)?; // 10 年有效期
    ca_builder.set_not_before(&not_before)?;
    ca_builder.set_not_after(&not_after)?;

    // 添加 CA 扩展：BasicConstraints 和 KeyUsage
    ca_builder.append_extension(BasicConstraints::new().critical().ca().pathlen(2).build()?)?;

    ca_builder.append_extension(
        KeyUsage::new()
            .critical()
            .key_cert_sign()
            .crl_sign()
            .build()?,
    )?;

    ca_builder.sign(&ca_pkey, MessageDigest::sha256())?;

    let ca_cert = ca_builder.build();
    // 然后才可以调用：
    let ca_cert_pem = ca_cert.to_pem()?; // ✅ 正确
    let ca_key_pem = ca_pkey.private_key_to_pem_pkcs8()?;

    match std::fs::write("./ca.pem", ca_cert_pem) {
        Ok(_) => {
            println!("✅ [CA] 授权根证书已保存");
        }
        Err(e) => panic!("❌ [CA] 授权根证书保存失败: {}", e),
    }
    match std::fs::write("./ca_key.pem", ca_key_pem) {
        Ok(_) => {
            println!("✅ [CA] 根证书私钥已保存");
            Ok(())
        }
        Err(e) => panic!("❌ [CA] 根证书私钥保存失败: {}", e),
    }
}

fn configure_log() -> Logger {
    let decorator = slog_term::TermDecorator::new().build();
    let console_drain = slog_term::FullFormat::new(decorator).build().fuse();

    // It is used for Synchronization
    let console_drain = slog_async::Async::new(console_drain).build().fuse();

    // Root logger
    Logger::root(console_drain, o!("v"=>env!("CARGO_PKG_VERSION")))
}
