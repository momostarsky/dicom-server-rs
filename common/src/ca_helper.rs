use crate::server_config::LicenseServerConfig;
use regex::Regex;
use serde_json::Value;


/// 客户端注册错误类型
#[derive(Debug)]
pub enum ClientRegisterError {
    ValidationError(String),
    NetworkError(reqwest::Error),
    JsonParseError(reqwest::Error),
    FileWriteError(std::io::Error),
    MissingClientCert,
    HttpError(reqwest::StatusCode),
}

impl std::fmt::Display for ClientRegisterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientRegisterError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ClientRegisterError::NetworkError(err) => write!(f, "Network error: {}", err),
            ClientRegisterError::JsonParseError(err) => write!(f, "JSON parse error: {}", err),
            ClientRegisterError::FileWriteError(err) => write!(f, "File write error: {}", err),
            ClientRegisterError::MissingClientCert => write!(f, "Missing client_cert in response"),
            ClientRegisterError::HttpError(status) => write!(f, "HTTP error: {}", status),
        }
    }
}

impl std::error::Error for ClientRegisterError {}

/// 验证客户端注册参数
///
/// # 参数
///
/// * `license_server` - 许可证服务器配置
///
/// # 返回值
///
/// 返回验证结果，如果验证通过则返回Ok(())，否则返回Err(错误信息)
fn validate_client_register_params(license_server: &LicenseServerConfig) -> Result<(), String> {
    // Validate client_id: 字母数字组合，16位
    let client_id_regex = Regex::new(r"^[a-zA-Z0-9]{16}$").unwrap();
    if !client_id_regex.is_match(&license_server.client_id) {
        return Err(
            "client_id must be 16 characters long and contain only letters and numbers".to_string(),
        );
    }

    // Validate client_name: 字母数字组合并支持,. 10到64位
    let client_name_regex = Regex::new(r"^[a-zA-Z0-9,.\s]{10,64}$").unwrap();
    if !client_name_regex.is_match(&license_server.client_name) {
        return Err("client_name must be between 10 and 64 characters long and contain only letters, numbers, commas, periods, and spaces".to_string());
    }

    // Validate client_machine_id: 字母数字组合，16~128位
    let client_machine_id_regex = Regex::new(r"^[a-zA-Z0-9]{16,128}$").unwrap();
    if !client_machine_id_regex.is_match(&license_server.machine_id) {
        return Err("client_machine_id must be between 16 and 128 characters long and contain only letters and numbers".to_string());
    }

    // Validate client_mac_address: 网卡地址格式 (MAC地址格式)
    let mac_address_regex = Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap();
    if !mac_address_regex.is_match(&license_server.mac_address) {
        return Err("client_mac_address must be in MAC address format (e.g., 00:1A:2B:3C:4D:5E or 00-1A-2B-3C-4D-5E)".to_string());
    }

    // Validate end_date: YYYYMMDD 格式
    let end_date_regex = Regex::new(r"^\d{8}$").unwrap();
    if !end_date_regex.is_match(&license_server.end_date) {
        return Err("end_date must be in YYYYMMDD format".to_string());
    }

    // Additional validation for end_date to ensure it's a valid date
    if let Err(_) = chrono::NaiveDate::parse_from_str(&license_server.end_date, "%Y%m%d") {
        return Err("end_date must be a valid date in YYYYMMDD format".to_string());
    }

    Ok(())
}
 // ... existing code ...

/// CA证书获取错误类型
#[derive(Debug)]
pub enum CaCertificateError {
    NetworkError(reqwest::Error),
    JsonParseError(reqwest::Error),
    MissingCaCert,
    HttpError(reqwest::StatusCode),
    FileWriteError(std::io::Error),
}

impl std::fmt::Display for CaCertificateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CaCertificateError::NetworkError(err) => write!(f, "Network error: {}", err),
            CaCertificateError::JsonParseError(err) => write!(f, "JSON parse error: {}", err),
            CaCertificateError::MissingCaCert => write!(f, "Missing ca_cert in response"),
            CaCertificateError::HttpError(status) => write!(f, "HTTP error: {}", status),
            CaCertificateError::FileWriteError(err) => write!(f, "File write error: {}", err),
        }
    }
}

impl std::error::Error for CaCertificateError {}

/// 从DICOM许可证服务器获取公共CA证书
///
/// # 参数
///
/// * `dicom_license_server` - DICOM许可证服务器地址
/// * `public_ca_file_path` - 公共CA证书文件路径
///
/// # 返回值
///
/// 返回获取结果，如果获取成功则返回Ok(())，否则返回Err(错误信息)
pub async fn get_public_ca_from_server(
    dicom_license_server: &str,
    public_ca_file_path: &str,
) -> Result<(), CaCertificateError> {
    // 构建完整的URL
    let url = if dicom_license_server.ends_with("/ca") {
        dicom_license_server.to_string()
    } else if dicom_license_server.ends_with("/") {
        format!("{}ca", dicom_license_server)
    } else {
        format!("{}/ca", dicom_license_server)
    };

    // 创建HTTP客户端
    let client = reqwest::Client::new();

    // 发送GET请求
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(CaCertificateError::NetworkError)?;

    // 检查响应状态
    if !response.status().is_success() {
        return Err(CaCertificateError::HttpError(response.status()));
    }

    // 解析JSON响应
    let json_value = response
        .json::<Value>()
        .await
        .map_err(CaCertificateError::JsonParseError)?;

    // 提取ca_cert字段
    let ca_cert = json_value
        .get("ca_cert")
        .and_then(|v| v.as_str())
        .ok_or(CaCertificateError::MissingCaCert)?
        .to_string();

    // 将CA证书写入指定文件
    std::fs::write(public_ca_file_path, &ca_cert)
        .map_err(CaCertificateError::FileWriteError)?;

    println!(
        "Successfully wrote CA certificate to {}",
        public_ca_file_path
    );

    Ok(())
}

// ... existing code ...

pub async fn client_register(
    license_server: &LicenseServerConfig,
    dicom_license_server: &str,
) -> Result<String, ClientRegisterError> {
    // 首先验证参数
    if let Err(validation_error) = validate_client_register_params(license_server) {
        eprintln!("Parameter validation failed: {}", validation_error);
        return Err(ClientRegisterError::ValidationError(validation_error));
    }

    // curl -X POST \
    //   -H "Content-Type: application/x-www-form-urlencoded" \
    //   -d "client_id=HZ100001&client_name=Sky.LTD&client_machine_id=898989398398moioio2xio22332&client_mac_address=OA:IB:OC:E3:GC:8B&end_date=20261231" \
    //   http://116.63.110.45:8888/client/register
    //TODO:  模拟HTTP POST 实现客户端注册

    // 构建完整的URL
    let url = format!(
        "{}/client/registe",
        dicom_license_server.trim_end_matches('/')
    );

    // 构造表单数据
    let form_data = [
        ("client_id", license_server.client_id.clone()),
        ("client_name", license_server.client_name.clone()),
        ("client_machine_id", license_server.machine_id.clone()),
        ("client_mac_address", license_server.mac_address.clone()),
        ("end_date", license_server.end_date.clone()),
    ];

    // 创建HTTP客户端
    let client = reqwest::Client::new();

    // 发送POST请求
    let response = client
        .post(&url)
        .form(&form_data)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .map_err(ClientRegisterError::NetworkError)?;

    // 检查响应状态
    if !response.status().is_success() {
        return Err(ClientRegisterError::HttpError(response.status()));
    }

    // 解析JSON响应
    let json_value = response
        .json::<serde_json::Value>()
        .await
        .map_err(ClientRegisterError::JsonParseError)?;

    // 提取client_cert字段
    let client_cert = json_value
        .get("client_cert")
        .and_then(|v| v.as_str())
        .ok_or(ClientRegisterError::MissingClientCert)?
        .to_string();

    // 将client_cert内容写入license_key指定的路径
    std::fs::write(&license_server.license_key, &client_cert)
        .map_err(ClientRegisterError::FileWriteError)?;

    println!(
        "Successfully wrote client certificate to {}",
        license_server.license_key
    );

    // 返回client_cert内容
    Ok(client_cert)
}

 // ... existing code ...

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server_config::LicenseServerConfig;


    #[tokio::test]
    async fn test_get_public_ca_from_server_success() {
        // 使用实际的服务器地址进行测试
        let server_url = "http://116.63.110.45:8888";
        let ca_file_path = "/tmp/test_ca_cert.pem";

        // 调用函数
        let result = get_public_ca_from_server(server_url, ca_file_path).await;

        // 验证结果成功
        assert!(result.is_ok(), "Getting CA certificate should succeed");

        // 验证CA证书文件是否已创建
        assert!(
            std::path::Path::new(ca_file_path).exists(),
            "CA certificate file should be created at specified path"
        );

        // 读取写入的CA证书文件内容
        let ca_cert_content = std::fs::read_to_string(ca_file_path).unwrap();

        // 验证内容包含证书的标志性内容
        assert!(
            ca_cert_content.contains("-----BEGIN CERTIFICATE-----"),
            "CA certificate should contain start marker"
        );
        assert!(
            ca_cert_content.contains("-----END CERTIFICATE-----"),
            "CA certificate should contain end marker"
        );

        // 清理测试文件
        let _ = std::fs::remove_file(ca_file_path);
    }

    #[tokio::test]
    async fn test_get_public_ca_from_server_with_trailing_slash() {
        // 使用带斜杠的实际服务器地址进行测试
        let server_url = "http://116.63.110.45:8888/";
        let ca_file_path = "/tmp/test_ca_cert.pem";

        // 调用函数
        let result = get_public_ca_from_server(server_url, ca_file_path).await;

        // 验证结果成功
        assert!(result.is_ok(), "Getting CA certificate should succeed");

        // 验证CA证书文件是否已创建
        assert!(
            std::path::Path::new(ca_file_path).exists(),
            "CA certificate file should be created at specified path"
        );

        // 读取写入的CA证书文件内容
        let ca_cert_content = std::fs::read_to_string(ca_file_path).unwrap();

        // 验证内容包含证书的标志性内容
        assert!(
            ca_cert_content.contains("-----BEGIN CERTIFICATE-----"),
            "CA certificate should contain start marker"
        );
        assert!(
            ca_cert_content.contains("-----END CERTIFICATE-----"),
            "CA certificate should contain end marker"
        );

        // 清理测试文件
        let _ = std::fs::remove_file(ca_file_path);
    }

    #[tokio::test]
    async fn test_get_public_ca_from_server_with_existing_path() {
        // 使用已包含路径的实际服务器地址进行测试
        let server_url = "http://116.63.110.45:8888/ca";
        let ca_file_path = "/tmp/test_ca_cert.pem";

        // 调用函数
        let result = get_public_ca_from_server(server_url, ca_file_path).await;

        // 验证结果成功
        assert!(result.is_ok(), "Getting CA certificate should succeed");

        // 验证CA证书文件是否已创建
        assert!(
            std::path::Path::new(ca_file_path).exists(),
            "CA certificate file should be created at specified path"
        );

        // 读取写入的CA证书文件内容
        let ca_cert_content = std::fs::read_to_string(ca_file_path).unwrap();

        // 验证内容包含证书的标志性内容
        assert!(
            ca_cert_content.contains("-----BEGIN CERTIFICATE-----"),
            "CA certificate should contain start marker"
        );
        assert!(
            ca_cert_content.contains("-----END CERTIFICATE-----"),
            "CA certificate should contain end marker"
        );

        // 清理测试文件
        let _ = std::fs::remove_file(ca_file_path);
    }

    #[tokio::test]
    async fn test_get_public_ca_from_server_invalid_url() {
        // 使用无效的服务器地址进行测试
        let server_url = "http://invalid-server-address.local:1234";
        let ca_file_path = "/tmp/test_ca_cert.pem";

        // 调用函数
        let result = get_public_ca_from_server(server_url, ca_file_path).await;

        // 验证结果失败
        assert!(result.is_err(), "Getting CA certificate should fail with invalid URL");
    }


    #[tokio::test]
    async fn test_client_register() {
        // 创建测试用的LicenseServerConfig实例，参数格式参考register_controller.rs中的验证规则
        let license_server = LicenseServerConfig {
            url: "http://116.63.110.45:8888".to_string(),
            client_id: "HZ100001ABCDEFGH".to_string(), // 16位字母和数字，符合^[a-zA-Z0-9]{16}$规则
            client_name: "hz xiasha momoStarsky Techlogy.LTD".to_string(), // 10-64个字符，包含字母、数字、逗号、句号和空格
            machine_id: "898989398398moioio2xio22332".to_string(), // 16-128位字母数字组合
            mac_address: "0A:1B:2C:3D:4E:5F".to_string(), // MAC地址格式，符合^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$规则
            end_date: "20261231".to_string(), // YYYYMMDD格式日期
            license_key: "/tmp/test_client_cert.pem".to_string(), // 用于测试的临时路径
        };

        // 测试客户端注册
        let server_url = "http://116.63.110.45:8888";
        let result = client_register(&license_server, server_url).await;

        // 验证结果是成功的
        assert!(result.is_ok(), "Client register should succeed");

        let result = result.unwrap();

        // 验证结果不为空
        assert!(
            !result.is_empty(),
            "Client register should return a non-empty result"
        );

        // 验证结果包含证书的标志性内容
        assert!(
            result.contains("-----BEGIN CERTIFICATE-----"),
            "Response should contain certificate start marker"
        );
        assert!(
            result.contains("-----END CERTIFICATE-----"),
            "Response should contain certificate end marker"
        );

        // 验证证书文件是否已创建
        assert!(
            std::path::Path::new(&license_server.license_key).exists(),
            "Certificate file should be created at specified path"
        );

        // 读取写入的证书文件内容
        let written_cert = std::fs::read_to_string(&license_server.license_key).unwrap();
        assert_eq!(
            written_cert, result,
            "Written certificate file should match returned certificate"
        );

        // 清理测试文件
        let _ = std::fs::remove_file(&license_server.license_key);
    }
}

