use openssl::asn1::{Asn1Integer, Asn1Object, Asn1OctetString, Asn1Time};
use openssl::bn::BigNum;
use openssl::hash::MessageDigest;
use openssl::pkey::PKey;
use openssl::rsa::Rsa;
use openssl::x509::extension::{BasicConstraints, ExtendedKeyUsage, KeyUsage};
use openssl::x509::{X509Extension, X509NameBuilder, X509};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};
use x509_parser::der_parser::Oid;
use x509_parser::parse_x509_certificate;

pub fn read_machine_id() -> Result<String, Box<dyn std::error::Error>> {
    let path = "/etc/machine-id";
    let contents = fs::read_to_string(path)?;
    // 通常内容为一行 UUID，可能带换行，trim一下
    let id = contents.trim().to_string();
    if id.is_empty() {
        return Err("machine-id 文件为空".into());
    }
    Ok(id)
}

pub fn get_primary_mac_address() -> Result<String, Box<dyn std::error::Error>> {
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
pub fn parse_client_cert(client_cert_file: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. 读取 client.crt 文件内容
    let cert_bytes = fs::read(client_cert_file)?;

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
        println!(
            "Signature Algorithm: {:?}",
            tbs_certificate.signature.algorithm
        );

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
pub fn sign_and_verify_with_cert(
    client_cert_file: &str,
    client_key_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 读取客户端私钥 (用于签名)
    let private_key_pem = fs::read(client_key_file)?;
    let private_key = PKey::private_key_from_pem(&private_key_pem)?;

    // 要签名的数据
    let data = b"This is the data to be signed";
    println!("原始数据: {}", String::from_utf8_lossy(data));

    // 使用私钥对数据进行签名
    let mut signer =
        openssl::sign::Signer::new(openssl::hash::MessageDigest::sha256(), &private_key)?;
    signer.update(data)?;
    let signature = signer.sign_to_vec()?;

    println!("签名数据 (HEX): {}", hex::encode(&signature));

    // 读取客户端证书 (用于验证签名)
    let cert_pem = fs::read(client_cert_file)?;
    let cert = X509::from_pem(&cert_pem)?;

    // 获取证书中的公钥
    let pub_key = cert.public_key()?;

    // 使用公钥验证签名
    let mut verifier = openssl::sign::Verifier::new(MessageDigest::sha256(), &pub_key)?;
    verifier.update(data)?;
    let valid = verifier.verify(&signature)?;

    println!(
        "签名验证结果: {}",
        if valid { "✅ 有效" } else { "❌ 无效" }
    );

    // 验证篡改数据的签名（应该失败）
    let tampered_data = b"This is the tampered data";
    let mut verifier2 = openssl::sign::Verifier::new(MessageDigest::sha256(), &pub_key)?;
    verifier2.update(tampered_data)?;
    let tampered_valid = verifier2.verify(&signature)?;

    println!(
        "篡改数据签名验证结果: {}",
        if tampered_valid {
            "✅ 有效"
        } else {
            "❌ 无效"
        }
    );

    Ok(())
}

// ... existing code ...
pub fn encrypt_with_cert_and_decrypt_with_key(
    client_cert_file: &str,
    client_key_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // 读取客户端证书 (用于加密)
    let cert_pem = std::fs::read(client_cert_file)?;
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
    let private_key_pem = fs::read(client_key_file)?;
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

/// 生成客户端证书和签名
///
/// # 参数
///
/// * `client_org` - 客户端组织名称
/// * `client_id` - 客户端唯一标识符
/// * `machine_id` - 客户端机器ID
/// * `mac_address` - 客户端MAC地址
/// * `end_date` - 证书有效期结束日期（YYYYMMDD格式）
/// * `ca_root_file` - CA根证书文件路径
/// * `ca_root_key_file` - CA根私钥文件路径
///
/// # 返回值
///
/// * `Ok((client_cert, client_sign))` - 包含客户端证书和签名的元组
/// * `Err(e)` - 生成过程中发生的错误
pub fn generate_client_and_sign(
    client_org: &str,
    client_id: &str,
    machine_id: &str,  //客户端的机器ID
    mac_address: &str, //客户端的IP地址
    end_date: &str,    // 结束时间YYYYMMDD 的格式

    ca_root_file: &str,
    ca_root_key_file: &str,
) -> Result<(Vec<u8>, Vec<u8>), Box<dyn std::error::Error>> {
    // 验证输入参数不为空
    if client_org.is_empty() {
        return Err("client_org cannot be empty".into());
    }
    if client_id.is_empty() {
        return Err("client_id cannot be empty".into());
    }
    if machine_id.is_empty() {
        return Err("machine_id cannot be empty".into());
    }
    if mac_address.is_empty() {
        return Err("mac_address cannot be empty".into());
    }
    if end_date.is_empty() {
        return Err("end_date cannot be empty".into());
    }

    // 读取CA证书
    let ca_cert_pem =
        fs::read(ca_root_file).expect(format!("读取CA根证书失败!:{}", ca_root_file).as_str());
    let ca_cert = X509::from_pem(&ca_cert_pem)?;

    // 读取CA私钥
    let ca_key_pem = fs::read(ca_root_key_file)
        .expect(format!("读取CA根私钥失败!:{}", ca_root_key_file).as_str());
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
    let machine_id_oid = Asn1Object::from_str("1.3.6.1.4.15967132172.1")?;
    let machine_id_data = Asn1OctetString::new_from_bytes(machine_id.as_bytes())?;
    let machine_id_ext = X509Extension::new_from_der(
        &machine_id_oid,
        false, // critical: 是否关键扩展
        machine_id_data.as_ref(),
        // 转换为引用
    )?;
    cert_builder.append_extension(machine_id_ext)?;

    // 添加自定义扩展：mac_address
    let mac_address_oid = Asn1Object::from_str("1.3.6.1.4.15967132172.2")?;
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
    // let filename = format!("client_{}.crt", client_id);
    // std::fs::write(&filename, client_cert_pem)?;

    // 保存客户端私钥
    let client_key_pem = client_pkey.private_key_to_pem_pkcs8()?;
    // let key_filename = format!("client_{}.key", client_id);
    // std::fs::write(&key_filename, client_key_pem)?;
    //
    // println!("✅ [Client] 授权证书已由 CA 签发并保存为 {}", filename);
    Ok((client_cert_pem, client_key_pem))
}

/// 生成 CA 根证书和私钥
///
/// # 参数
///
/// * `ca_file` - CA 根证书文件路径
/// * `ca_key_file` - CA 根私钥文件路径
///
/// # 返回值
///
/// * `Ok(())` - 生成成功
/// * `Err(e)` - 生成过程中发生的错误
pub fn generate_ca_root(
    ca_file: &str,
    ca_key_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // =============================
    // 1. 生成 CA（根证书，模拟授权机构）
    // =============================

    // 生成 CA 私钥 (RSA 2048)
    let ca_rsa = Rsa::generate(4096)?;
    let ca_pkey = PKey::from_rsa(ca_rsa)?;

    // 构造 CA 证书的 Subject（主题信息）
    let mut ca_name = X509NameBuilder::new()?;
    ca_name.append_entry_by_text("C", "CN")?; // 国家代码
    ca_name.append_entry_by_text("ST", "Zhejiang")?; // 省份
    ca_name.append_entry_by_text("L", "Hangzhou")?; // 城市
    ca_name.append_entry_by_text("O", "DICOM")?; // 组织
    ca_name.append_entry_by_text("CN", "dicom.org.cn")?; // 通用名称
    ca_name.append_entry_by_text("emailAddress", "411592148@qq.com")?; // 邮箱
    ca_name.append_entry_by_text("UID", "15967132172")?; // 用户ID
    ca_name.append_entry_by_text("SN", "dai")?; // 姓
    ca_name.append_entry_by_text("GN", "hanzhang")?; // 名
    let ca_name = ca_name.build();

    // 构建 CA 证书
    let mut ca_builder = X509::builder()?;
    ca_builder.set_version(2)?;

    // 设置序列号
    let serial = BigNum::from_u32(1)?;
    let serial = Asn1Integer::from_bn(&serial)?;
    ca_builder.set_serial_number(&serial)?;

    // 设置证书的主体和颁发者（自签名证书，两者相同）
    ca_builder.set_subject_name(&ca_name)?;
    ca_builder.set_issuer_name(&ca_name)?; // 自签名
    ca_builder.set_pubkey(&ca_pkey)?; // 设置公钥

    // 设置证书有效期（当前时间到10年后）
    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(365 * 10)?; // 10 年有效期
    ca_builder.set_not_before(&not_before)?;
    ca_builder.set_not_after(&not_after)?;

    // 添加 CA 扩展：BasicConstraints 和 KeyUsage
    ca_builder.append_extension(BasicConstraints::new().critical().ca().pathlen(2).build()?)?;
    // 添加密钥用法扩展：允许证书签名和CRL签名
    ca_builder.append_extension(
        KeyUsage::new()
            .critical()
            .key_cert_sign()
            .crl_sign()
            .build()?,
    )?;
    // 使用CA私钥对证书进行签名
    ca_builder.sign(&ca_pkey, MessageDigest::sha256())?;
    // 构建最终的证书对象
    let ca_cert = ca_builder.build();
    // 将证书和私钥转换为PEM格式
    let ca_cert_pem = ca_cert.to_pem()?; // ✅ 正确
    let ca_key_pem = ca_pkey.private_key_to_pem_pkcs8()?;

    // 将证书和私钥写入文件
    match fs::write(ca_file, ca_cert_pem) {
        Ok(_) => {
            println!("✅ [CA] 授权根证书已保存");
        }
        Err(e) => panic!("❌ [CA] 授权根证书保存失败: {}", e),
    }
    match fs::write(ca_key_file, ca_key_pem) {
        Ok(_) => {
            println!("✅ [CA] 根证书私钥已保存");
            Ok(())
        }
        Err(e) => panic!("❌ [CA] 根证书私钥保存失败: {}", e),
    }
}

// ... existing code ...
pub fn validate_my_certificate(
    client_cert_file: &str,
    ca_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    use openssl::stack::Stack;
    // 检查证书文件是否存在
    if !std::path::Path::new(client_cert_file).exists() {
        return Err(format!("证书文件不存在: {}", client_cert_file).into());
    }

    // 读取并解析客户端证书 (OpenSSL格式)
    let cert_pem = fs::read(client_cert_file)?;
    let cert = X509::from_pem(&cert_pem)?;

    // 读取并解析客户端证书 (x509-parser格式)
    let cert_bytes = fs::read(client_cert_file)?;
    let parsed_cert = if cert_bytes.starts_with(b"-----BEGIN CERTIFICATE-----") {
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
    let (_, x509_cert) = parse_x509_certificate(&parsed_cert)?;

    // 读取CA证书
    let ca_cert_pem = fs::read(ca_file)?;
    let ca_cert = X509::from_pem(&ca_cert_pem)?;

    // 1. 验证证书是否由指定的CA签发
    let mut cert_store_builder = openssl::x509::store::X509StoreBuilder::new()?;
    cert_store_builder.add_cert(ca_cert.clone())?;
    let cert_store = cert_store_builder.build();

    // 创建空的证书链栈
    let cert_chain = Stack::new()?;

    let mut cert_context = openssl::x509::X509StoreContext::new()?;
    let verify_result =
        cert_context.init(&cert_store, &cert, &cert_chain, |ctx| ctx.verify_cert())?;

    if !verify_result {
        return Err("证书验证失败：证书不是由指定CA签发".into());
    }

    // 2. 验证证书是否在有效期内
    let now = Asn1Time::days_from_now(0)?;
    if cert.not_after() < now.as_ref() {
        return Err("证书已过期".into());
    }
    if cert.not_before() > now.as_ref() {
        return Err("证书尚未生效".into());
    }

    // 3. 验证自定义扩展（machine_id和mac_address）
    // 获取系统中的machine_id和mac地址用于验证
    let expected_machine_id = read_machine_id()?;
    let expected_mac_address = get_primary_mac_address()?;

    // 使用x509-parser查找machine_id扩展（OID: 1.3.6.1.4.15967132172.1）
    let mut actual_machine_id = None;
    let mut actual_mac_address = None;
    let mut has_client_auth = false;

    for ext in x509_cert.tbs_certificate.extensions() {
        let oid_str = ext.oid.to_string();
        if oid_str == "1.3.6.1.4.15967132172.1" {
            // 找到machine_id扩展
            actual_machine_id = Some(String::from_utf8_lossy(ext.value).to_string());
        } else if oid_str == "1.3.6.1.4.15967132172.2" {
            // 找到mac_address扩展
            actual_mac_address = Some(String::from_utf8_lossy(ext.value).to_string());
        } else if oid_str == "2.5.29.37" {
            // 找到扩展密钥用法(Extended Key Usage)扩展
            // OID 1.3.6.1.5.5.7.3.2 表示客户端认证
            let client_auth_oid_bytes: &[u8] = &[0x2B, 0x06, 0x01, 0x05, 0x05, 0x07, 0x03, 0x02];
            if ext
                .value
                .windows(client_auth_oid_bytes.len())
                .any(|window| window == client_auth_oid_bytes)
            {
                has_client_auth = true;
            }
        }
    }

    let actual_machine_id = actual_machine_id.ok_or("未找到machine_id扩展")?;
    if actual_machine_id != expected_machine_id {
        return Err(format!(
            "machine_id不匹配：期望{}，实际{}",
            expected_machine_id, actual_machine_id
        )
        .into());
    }

    let actual_mac_address = actual_mac_address.ok_or("未找到mac_address扩展")?;
    if actual_mac_address != expected_mac_address {
        return Err(format!(
            "mac_address不匹配：期望{}，实际{}",
            expected_mac_address, actual_mac_address
        )
        .into());
    }

    // 4. 验证扩展密钥用法是否包含客户端认证
    if !has_client_auth {
        return Err("证书未授权用于客户端认证".into());
    }

    println!("✅ 证书验证成功");
    Ok(())
}
// ... existing code ...

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use x509_parser::der_parser::Oid;

    #[test]
    fn test_is_well_known_oid() {
        // 测试已知的OID
        let known_oid = Oid::from_str("2.5.29.14").unwrap();
        assert!(is_well_known_oid(&known_oid));

        // 测试未知的OID
        let unknown_oid = Oid::from_str("1.2.3.4.5").unwrap();
        assert!(!is_well_known_oid(&unknown_oid));
    }

    #[test]
    fn test_generate_ca_root() {
        // 创建临时目录用于测试
        let temp_dir = tempfile::tempdir().expect("无法创建临时目录");
        let ca_cert_path = temp_dir.path().join("ca_test.crt");
        let ca_key_path = temp_dir.path().join("ca_test.key");

        // 调用函数生成CA根证书
        let result = generate_ca_root(
            ca_cert_path.to_str().unwrap(),
            ca_key_path.to_str().unwrap(),
        );

        // 验证函数执行成功
        assert!(result.is_ok());

        // 验证证书文件已创建
        assert!(ca_cert_path.exists());

        // 验证私钥文件已创建
        assert!(ca_key_path.exists());

        // 验证证书文件不为空
        let cert_content = fs::read(&ca_cert_path).expect("无法读取证书文件");
        assert!(!cert_content.is_empty());
        assert!(
            String::from_utf8(cert_content)
                .unwrap()
                .contains("-----BEGIN CERTIFICATE-----")
        );

        // 验证私钥文件不为空
        let key_content = fs::read(&ca_key_path).expect("无法读取私钥文件");
        assert!(!key_content.is_empty());
        assert!(
            String::from_utf8(key_content)
                .unwrap()
                .contains("-----BEGIN PRIVATE KEY-----")
        );
    }

    #[test]
    fn test_generate_client_and_sign() {
        // 创建临时目录用于测试
        let temp_dir = tempfile::tempdir().expect("无法创建临时目录");
        let ca_cert_path = temp_dir.path().join("ca_test.crt");
        let ca_key_path = temp_dir.path().join("ca_test.key");

        // 首先生成CA根证书
        let ca_result = generate_ca_root(
            ca_cert_path.to_str().unwrap(),
            ca_key_path.to_str().unwrap(),
        );
        assert!(ca_result.is_ok());

        // 测试生成客户端证书
        let client_result = generate_client_and_sign(
            "Test Organization",
            "test-client-001",
            "machine-uuid-123",
            "00:11:22:33:44:55",
            "20301231",
            ca_cert_path.to_str().unwrap(),
            ca_key_path.to_str().unwrap(),
        );

        // 验证函数执行成功
        assert!(client_result.is_ok());

        // 获取返回的证书和私钥
        let (client_cert_pem, client_key_pem) = client_result.unwrap();

        // 验证证书和私钥不为空
        assert!(!client_cert_pem.is_empty());
        assert!(!client_key_pem.is_empty());

        // 验证证书是有效的PEM格式
        let cert_string = String::from_utf8(client_cert_pem).expect("证书不是有效的UTF-8");
        assert!(cert_string.contains("-----BEGIN CERTIFICATE-----"));
        assert!(cert_string.contains("-----END CERTIFICATE-----"));

        // 验证私钥是有效的PEM格式
        let key_string = String::from_utf8(client_key_pem).expect("私钥不是有效的UTF-8");
        assert!(key_string.contains("-----BEGIN PRIVATE KEY-----"));
        assert!(key_string.contains("-----END PRIVATE KEY-----"));
    }

    #[test]
    fn test_sign_and_verify_with_cert() {
        // 创建临时目录用于测试
        let temp_dir = tempfile::tempdir().expect("无法创建临时目录");
        let ca_cert_path = temp_dir.path().join("ca_test.crt");
        let ca_key_path = temp_dir.path().join("ca_test.key");
        let client_cert_path = temp_dir.path().join("client_test.crt");
        let client_key_path = temp_dir.path().join("client_test.key");

        // 首先生成CA根证书
        let ca_result = generate_ca_root(
            ca_cert_path.to_str().unwrap(),
            ca_key_path.to_str().unwrap(),
        );
        assert!(ca_result.is_ok());

        // 生成客户端证书
        let (client_cert_pem, client_key_pem) = generate_client_and_sign(
            "Test Organization",
            "test-client-001",
            "machine-uuid-123",
            "00:11:22:33:44:55",
            "20301231",
            ca_cert_path.to_str().unwrap(),
            ca_key_path.to_str().unwrap(),
        )
        .expect("无法生成客户端证书");

        // 将证书和密钥写入文件
        fs::write(&client_cert_path, &client_cert_pem).expect("无法写入客户端证书文件");
        fs::write(&client_key_path, &client_key_pem).expect("无法写入客户端密钥文件");

        // 测试签名和验证功能
        let result = sign_and_verify_with_cert(
            client_cert_path.to_str().unwrap(),
            client_key_path.to_str().unwrap(),
        );

        // 验证函数执行成功（输出会被打印，但我们只检查是否成功执行）
        assert!(result.is_ok());
    }

    #[test]
    fn test_encrypt_with_cert_and_decrypt_with_key() {
        // 创建临时目录用于测试
        let temp_dir = tempfile::tempdir().expect("无法创建临时目录");
        let ca_cert_path = temp_dir.path().join("ca_test.crt");
        let ca_key_path = temp_dir.path().join("ca_test.key");
        let client_cert_path = temp_dir.path().join("client_test.crt");
        let client_key_path = temp_dir.path().join("client_test.key");

        // 首先生成CA根证书
        let ca_result = generate_ca_root(
            ca_cert_path.to_str().unwrap(),
            ca_key_path.to_str().unwrap(),
        );
        assert!(ca_result.is_ok());

        // 生成客户端证书
        let (client_cert_pem, client_key_pem) = generate_client_and_sign(
            "Test Organization",
            "test-client-001",
            "machine-uuid-123",
            "00:11:22:33:44:55",
            "20301231",
            ca_cert_path.to_str().unwrap(),
            ca_key_path.to_str().unwrap(),
        )
        .expect("无法生成客户端证书");

        // 将证书和密钥写入文件
        fs::write(&client_cert_path, &client_cert_pem).expect("无法写入客户端证书文件");
        fs::write(&client_key_path, &client_key_pem).expect("无法写入客户端密钥文件");

        // 测试加密和解密功能
        let result = encrypt_with_cert_and_decrypt_with_key(
            client_cert_path.to_str().unwrap(),
            client_key_path.to_str().unwrap(),
        );

        // 验证函数执行成功
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_client_cert() {
        // 创建临时目录用于测试
        let temp_dir = tempfile::tempdir().expect("无法创建临时目录");
        let ca_cert_path = temp_dir.path().join("ca_test.crt");
        let ca_key_path = temp_dir.path().join("ca_test.key");
        let client_cert_path = temp_dir.path().join("client_test.crt");

        // 首先生成CA根证书
        let ca_result = generate_ca_root(
            ca_cert_path.to_str().unwrap(),
            ca_key_path.to_str().unwrap(),
        );
        assert!(ca_result.is_ok());

        // 生成客户端证书
        let (client_cert_pem, _client_key_pem) = generate_client_and_sign(
            "Test Organization",
            "test-client-001",
            "machine-uuid-123",
            "00:11:22:33:44:55",
            "20301231",
            ca_cert_path.to_str().unwrap(),
            ca_key_path.to_str().unwrap(),
        )
        .expect("无法生成客户端证书");

        // 将证书写入文件
        fs::write(&client_cert_path, &client_cert_pem).expect("无法写入客户端证书文件");

        // 测试解析证书功能
        let result = parse_client_cert(client_cert_path.to_str().unwrap());

        // 验证函数执行成功
        assert!(result.is_ok());
    }
}
