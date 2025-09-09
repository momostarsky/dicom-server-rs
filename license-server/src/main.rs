mod register_controller;

use crate::register_controller::{client_registe, client_validate, manual_hello};
use actix_cors::Cors;
use actix_web::{App, HttpServer, middleware, web};
use openssl::asn1::{
    Asn1Integer, Asn1Object, Asn1OctetString, Asn1OctetStringRef, Asn1String, Asn1StringRef,
    Asn1Time,
};
use openssl::bn::BigNum;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;

use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage};
use openssl::x509::{X509, X509NameBuilder, X509Ref, X509Req};
use openssl::x509::{X509Extension, X509VerifyResult};
use slog::Drain;
use slog::{Logger, o};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use x509_parser::der_parser::Oid;
use x509_parser::parse_x509_certificate;
// 定义应用状态

#[derive(Clone)]
struct AppState {
    log: Logger,
}
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let machine_id = read_machine_id().expect("Read机器的ID失败");
    // let mac_address = get_primary_mac_address().expect("Read机器的ID失败");
    // println!("Machine ID: {}", machine_id);
    // println!("mac_address ID: {}", mac_address);
    // generate_ca_root().expect("生成CA根证书失败!");
    // generate_client_and_sign(
    //     "HzMomoStarsky.LTD",
    //     "HZ109999",
    //     "c417aa4802b3441d931c135cbdaab367",
    //     "18:c0:4d:fa:c0:a8",
    // )
    // .expect("生成客户端证书失败!");
    parse_client_cert("./client_HZ109999.crt").expect("解析客户端证书失败!");
    encrypt_with_cert_and_decrypt_with_key( ).expect("加密解密失败!");
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
fn read_machine_id() -> Result<String, Box<dyn std::error::Error>> {
    let path = "/etc/machine-id";
    let contents = fs::read_to_string(path)?;
    // 通常内容为一行 UUID，可能带换行，trim一下
    let id = contents.trim().to_string();
    if id.is_empty() {
        return Err("machine-id 文件为空".into());
    }
    Ok(id)
}

fn get_primary_mac_address() -> Result<String, Box<dyn std::error::Error>> {
    let net_dir = std::path::Path::new("/sys/class/net");
    let mut mac = None;

    for entry in std::fs::read_dir(net_dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy().to_lowercase(); // 转小写便于比较

        // ===== 排除虚拟/容器/回环网络接口 =====
        if name_str == "lo"
            || name_str.starts_with("docker")
            || name_str.starts_with("veth")
            || name_str.starts_with("virbr")
            || name_str.starts_with("tun")
            || name_str.starts_with("tap")
            || name_str.starts_with("br-")
            || name_str.starts_with("cni")
            || name_str.starts_with("kube")
            || name_str.starts_with("flannel")
        {
            continue;
        }

        let mac_path = entry.path().join("address");
        if !mac_path.exists() {
            continue;
        }

        let content = std::fs::read_to_string(mac_path)?;
        let mac_trimmed = content.trim().to_string();

        if mac_trimmed.contains(':') {
            mac = Some(mac_trimmed);
            break;
        }
    }

    match mac {
        Some(m) => Ok(m),
        None => Err("未找到有效的 MAC 地址".into()),
    }
}

// ... existing code ...
fn parse_client_cert(cert_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 读取 client.crt 文件内容
    let cert_bytes = fs::read(cert_file)?;

    // 检查是否是 PEM 格式
    // 2. 尝试解析 PEM 格式；如果不是 PEM，可能需要先转换为 DER，或者使用 parse_x509_der_directly
    let parsed = if cert_bytes.starts_with(b"-----BEGIN CERTIFICATE-----") {
        // 如果是 PEM 格式，需要先解码
        let pem_str = std::str::from_utf8(&cert_bytes)?;
        let certs = pem::parse_many(pem_str)?;
        if certs.is_empty() {
            return Err("PEM文件中未找到证书".into());
        }
         certs[0].contents.to_vec()

    } else {
        // 如果是 DER 格式，直接解析
        cert_bytes
    };
    let result = parse_x509_certificate(&parsed);
    if let Ok((remaining, cert)) = result {
        if !remaining.is_empty() {
            eprintln!("警告：证书文件可能包含多余数据");
        }

        let tbs_certificate = &cert.tbs_certificate;

        println!("Subject: {:?}", tbs_certificate.subject.to_string());
        println!("Issuer: {:?}", tbs_certificate.issuer.to_string());
        println!("Not Before: {:?}", tbs_certificate.validity.not_before);
        println!("Not After: {:?}", tbs_certificate.validity.not_after);
        println!("Serial Number: {:?}", tbs_certificate.serial);
        println!("Signature Algorithm: {:?}", tbs_certificate.signature.algorithm);

        println!(
            "Found {} extension(s):",
            &tbs_certificate.extensions().len()
        );
        // 4. 遍历所有扩展

        for ext in tbs_certificate.extensions() {
            let oid = &ext.oid;
            let oid_str = oid.to_string(); // 比如 "1.3.6.1.5.5.7.3.2"
            let critical = ext.critical;
            let value = &ext.value; // 原始 DER 编码的字节

            // 判断是否是"私有扩展"：通常没有广泛认知的 OID，或者不在标准列表里
            // 注意：这里没有内置的"标准 OID 列表"，所以只能根据常见情况判断，或者你自行维护一个标准 OID 列表
            let is_private = !is_well_known_oid(oid);

            println!("  OID: {}", oid_str);
            println!("  Critical: {}", critical);
            println!("  Is Private (assumed): {}", is_private);
            println!("  Raw Value (hex): {:02X?}", value);
            // 尝试将值显示为字符串（如果可能）
            match std::str::from_utf8(value) {
                Ok(s) => println!("  Value as String: {}", s),
                Err(_) => println!("  Value as String: <无法解析为UTF-8字符串>"),
            }

            // 如果你想进一步解析标准扩展，可以在这里使用 x509-parser 提供的扩展解析工具
            // 但对于私有扩展，通常只能打印其 OID 和原始值
        }
    } else {
        eprintln!("证书解析失败: {:?}", parsed);
        return Err("证书解析失败".into());
    }
    Ok(())
}

fn encrypt_with_cert_and_decrypt_with_key() -> Result<(), Box<dyn std::error::Error>> {
    // 读取客户端证书 (用于加密)
    let cert_pem = std::fs::read("./client_HZ109999.crt")?;
    let cert = X509::from_pem(&cert_pem)?;

    // 获取证书中的公钥
    let pub_key = cert.public_key()?;

    // 要加密的数据
    let data = b"Hello, this is a secret message!";
    println!("原始数据: {}", String::from_utf8_lossy(data));

    // 使用公钥加密数据
    let mut encrypter = openssl::encrypt::Encrypter::new(&pub_key)?;
    encrypter.set_rsa_padding(openssl::rsa::Padding::PKCS1)?;

    // 计算加密后数据的长度
    let buffer_len = encrypter.encrypt_len(data)?;
    let mut encrypted = vec![0; buffer_len];

    // 执行加密
    let encrypted_len = encrypter.encrypt(data, &mut encrypted)?;
    encrypted.truncate(encrypted_len);

    println!("加密后的数据 (HEX): {}", hex::encode(&encrypted));

    // 读取客户端私钥 (用于解密)
    let private_key_pem = std::fs::read("./client_HZ109999.key")?;
    let private_key = PKey::private_key_from_pem(&private_key_pem)?;

    // 使用私钥解密数据
    let mut decrypter = openssl::encrypt::Decrypter::new(&private_key)?;
    decrypter.set_rsa_padding(openssl::rsa::Padding::PKCS1)?;

    // 计算解密后数据的长度
    let buffer_len = decrypter.decrypt_len(&encrypted)?;
    let mut decrypted = vec![0; buffer_len];

    // 执行解密
    let decrypted_len = decrypter.decrypt(&encrypted, &mut decrypted)?;
    decrypted.truncate(decrypted_len);

    println!("解密后的数据: {}", String::from_utf8_lossy(&decrypted));

    // 验证解密是否成功
    if data == decrypted.as_slice() {
        println!("✅ 加密/解密成功!");
    } else {
        println!("❌ 加密/解密失败!");
    }

    Ok(())
}

// ... existing code ...
// ... existing code ...
/// 简单示例函数：判断一个 OID 是否是“众所周知的标准 OID”
/// 注意：这是一个非常简化的判断，实际项目中你可能需要维护一个更全面的 OID 列表
/// 或者使用第三方 crate 如 `oid-registry`（如果有）
fn is_well_known_oid(oid: &Oid) -> bool {
    let oid_str = oid.to_string();

    // 列出一些常见的标准扩展 OID，仅作示例
    let known_oids = [
        "2.5.29.14",         // Subject Key Identifier
        "2.5.29.15",         // Key Usage
        "2.5.29.17",         // Subject Alternative Name
        "2.5.29.18",         // Issuer Alternative Name
        "2.5.29.19",         // Basic Constraints
        "2.5.29.30",         // Name Constraints
        "2.5.29.31",         // CRL Distribution Points
        "2.5.29.32",         // Certificate Policies
        "2.5.29.33",         // Policy Mappings
        "2.5.29.35",         // Authority Key Identifier
        "2.5.29.36",         // Policy Constraints
        "2.5.29.37",         // ExtKeyUsageSyntax (Extended Key Usage)
        "2.5.29.54",         // Inhibit AnyPolicy
        "1.3.6.1.5.5.7.3.1", // Server Auth
        "1.3.6.1.5.5.7.3.2", // Client Auth
        "1.3.6.1.5.5.7.3.3", // Code Signing
        "1.3.6.1.5.5.7.3.4", // Email Protection
        "1.3.6.1.5.5.7.3.8", // Time Stamping
        "1.3.6.1.5.5.7.3.9", // OCSP Signing
    ];

    known_oids.contains(&oid_str.as_str())
}
fn generate_client_and_sign(
    client_org: &str,
    client_id: &str,
    machine_id: &str,  //客户端的MAC地址
    mac_address: &str, //客户端的IP地址
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

    // ========== 添加自定义扩展：machine_id 和 mac_address ==========
    // 注意：这里使用自定义 OID，比如：
    // - 1.2.3.4.1 表示 client_machine_id
    // - 1.2.3.4.2 表示 client_mac_address

    // OID 格式是字符串，比如 "1.2.3.4.1"

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

    // 添加自定义扩展：machine_id
    let machine_id_oid = Asn1Object::from_str("1.2.3.4.1")?;
    let machine_id_data = Asn1OctetString::new_from_bytes(machine_id.as_bytes())?;
    let machine_id_ext = X509Extension::new_from_der(
        &machine_id_oid,
        false, // critical: 是否关键扩展
        machine_id_data.as_ref(),
        // 转换为引用
    )?;
    cert_builder.append_extension(machine_id_ext)?;

    // 添加自定义扩展：mac_address
    let mac_address_oid = Asn1Object::from_str("1.2.3.4.2")?;
    let mac_address_data = Asn1OctetString::new_from_bytes(mac_address.as_bytes())?;
    let mac_address_ext = X509Extension::new_from_der(
        &mac_address_oid,
        false,                     // critical: 是否关键扩展
        mac_address_data.as_ref(), // 转换为引用
    )?;
    cert_builder.append_extension(mac_address_ext)?;

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
