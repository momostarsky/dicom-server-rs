use openssl::asn1::Asn1Time;
use openssl::x509::X509;
use std::fs;
use x509_parser::parse_x509_certificate;

static CA_CONTENT: &str = "-----BEGIN CERTIFICATE-----
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
TnqEtEGq35in0fiX1ai/43juYfWjj9trUmT4
-----END CERTIFICATE-----\n";

pub async fn load_ca_certificate() -> Result<X509, Box<dyn std::error::Error>> {
    X509::from_pem(CA_CONTENT.as_bytes()).map_err(|e| e.into())
}

/// 使用指定的CA证书验证客户端证书
///
/// # 参数
///
/// * `client_cert_file` - 客户端证书文件路径
///
/// # 返回值
///
/// * `Ok(())` - 验证成功
/// * `Err(e)` - 验证过程中发生的错误
pub async fn validate_client_certificate()
-> Result<(Option<String>, Option<String>), Box<dyn std::error::Error>> {
    use openssl::stack::Stack;
    let client_cert_file = "dicom-org-cn-client.crt";
    // 检查证书文件是否存在
    if !std::path::Path::new(client_cert_file).exists() {
        return Err(format!("客户端证书文件不存在: {}", client_cert_file).into());
    }

    // 读取并解析客户端证书 (OpenSSL格式)
    let cert_pem = fs::read(client_cert_file)?;
    let cert = X509::from_pem(&cert_pem)?;

    // 读取并解析CA证书

    // 读取并解析CA证书
    let ca_cert = load_ca_certificate().await?;

    // 验证证书是否由指定的CA签发
    let mut cert_store_builder = openssl::x509::store::X509StoreBuilder::new()?;
    cert_store_builder.add_cert(ca_cert)?;
    let cert_store = cert_store_builder.build();

    // 创建空的证书链栈
    let cert_chain = Stack::new()?;

    let mut cert_context = openssl::x509::X509StoreContext::new()?;
    let verify_result =
        cert_context.init(&cert_store, &cert, &cert_chain, |ctx| ctx.verify_cert())?;

    if !verify_result {
        return Err("证书验证失败：证书不是由指定CA签发".into());
    }

    // 验证证书是否在有效期内
    let now = Asn1Time::days_from_now(0)?;
    if cert.not_after() < now.as_ref() {
        return Err("证书已过期".into());
    }
    if cert.not_before() > now.as_ref() {
        return Err("证书尚未生效".into());
    }

    // 解析客户端证书中的自定义扩展
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

    // 查找hash_code扩展（OID: 1.3.6.1.4.15967132172.1）
    let mut hash_code = None;
    let client_id = cert
        .subject_name()
        .entries_by_nid(openssl::nid::Nid::COMMONNAME)
        .next()
        .map(|name| String::from_utf8_lossy(name.data().as_slice()).to_string());

    for ext in x509_cert.tbs_certificate.extensions() {
        let oid_str = ext.oid.to_string();
        if oid_str == "1.3.6.1.4.15967132172.1" {
            // 找到hash_code扩展
            hash_code = Some(String::from_utf8_lossy(ext.value).to_string());
            break;
        }
    }
    Ok((client_id, hash_code))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_load_ca_certificate() {
        let result = load_ca_certificate().await;
        assert!(
            result.is_ok(),
            "Failed to load CA certificate: {:?}",
            result.err()
        );

        let cert = result.unwrap();
        let subject = cert.subject_name();
        // 验证证书主题包含预期的信息
        let common_name = subject
            .entries_by_nid(openssl::nid::Nid::COMMONNAME)
            .next()
            .expect("Certificate should have a common name");

        let common_name_str = String::from_utf8(common_name.data().as_slice().to_vec()).unwrap();
        assert_eq!(common_name_str, "dicom.org.cn");
    }

    #[tokio::test]
    async fn test_ca_certificate_pem_format() {
        // 验证CA证书内容是有效的PEM格式
        assert!(CA_CONTENT.starts_with("-----BEGIN CERTIFICATE-----"));
        assert!(CA_CONTENT.ends_with("-----END CERTIFICATE-----\n"));

        // 验证能够成功解析PEM内容
        let result = load_ca_certificate().await;
        assert!(result.is_ok(), "CA certificate should be valid PEM format");
    }

    #[tokio::test]
    async fn test_ca_certificate_not_empty() {
        // 验证CA证书内容不为空
        assert!(!CA_CONTENT.is_empty());
        assert!(CA_CONTENT.len() > 100); // 合理的最小证书大小
    }
}
