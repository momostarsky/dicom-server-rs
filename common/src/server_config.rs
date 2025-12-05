use config::{Config, ConfigError, Environment, File};
use dicom_encoding::TransferSyntaxIndex;
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use dotenv::dotenv;
use serde::Deserialize;
use std::{env, fs};
use std::path::PathBuf;
use std::sync::Once;

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,              //连接地址
    pub password: Option<String>, //密码
    pub is_lts: Option<bool>,     //是否启动TLS
}

// 定义配置结构体
#[derive(Debug, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub dbtype: String, //数据库类型 POSTGRES  MYSQL SQLITE
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
    pub allow_origin: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LocalStorageConfig {
    pub dicm_store_path: String,
    pub json_store_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DicomStoreScpConfig {
    pub port: u16,
    pub ae_title: String,
    pub unsupported_ts_change_to: String,
    pub cornerstonejs_supported_transfer_syntax: Vec<String>,
    pub tenant_group: String,   // "0x1211",
    pub tenant_element: String, // "0x1217",
}

#[derive(Debug, Deserialize, Clone)]
pub struct KafkaConfig {
    pub brokers: String,

    pub queue_buffering_max_messages: u32,
    pub queue_buffering_max_kbytes: u32,
    pub batch_num_messages: u32,
    pub queue_buffering_max_ms: u32,
    pub linger_ms: u32,
    pub compression_codec: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct MessageQueueConfig {
    pub consumer_group_id: String,
    pub topic_main: String,
    pub topic_log: String,

    pub topic_dicom_state: String,
    pub topic_dicom_image: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct LicenseServerConfig {
    /// DICOM 许可服务器的 API 密钥  16位字母或是数字字符串
    pub client_id: String,
    /// DICOM 许可密钥的HASHCODE
    pub license_key: String,
}

// --- 配置结构 ---
#[derive(Debug, Clone, Deserialize)]
pub struct RoleRule {
    #[serde(rename = "from")]
    pub json_path: String,
    #[serde(rename = "values")]
    pub required_values: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuth2Config {
    pub issuer_url: String,
    pub audience: String,
    pub jwks_url: String,
    #[serde(default)]
    pub roles: Option<RoleRule>,
    #[serde(default)]
    pub permissions: Option<RoleRule>,
}
#[derive(Debug, Deserialize, Clone)]
pub struct WebWorkerConfig {
    ///series_lastUpdateTime + X 分钟内没有更新
    pub interval_minute: u16,
    /// 最大cpu 使用率
    pub cpu_usage: u16,
    /// 最大内存使用率
    pub memory_usage: u16,
}

// "webworker": {
// "interval_minute": 5,
// "cpu_usage": 40,
// "memory_usage": 70
// }

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
    pub main_database: DatabaseConfig,
    pub secondary_database: DatabaseConfig,
    pub server: ServerConfig,
    pub local_storage: LocalStorageConfig,
    pub dicom_store_scp: DicomStoreScpConfig,
    pub message_queue: MessageQueueConfig,
    pub dicom_license_server: Option<LicenseServerConfig>,
    pub wado_oauth2: Option<OAuth2Config>,
    pub webworker: Option<WebWorkerConfig>,
}

static APP_ENV: &str = "APP_ENV";
static APP_PREFIX: &str = "DICOM";

// 全局配置实例和初始化状态
static INIT: Once = Once::new();
static mut CONFIG: Option<AppConfig> = None;
 fn validate_and_create_path(path: &str, path_name: &str) -> Result<(), ConfigError> {
    if path.len() > 64 {
        return Err(ConfigError::Message(format!(
            "{} length must be less than 64 characters", path_name
        )));
    }

    match fs::exists(path) {
        Ok(exists) => {
            if !exists {
                fs::create_dir_all(path)
                    .map_err(|e| ConfigError::Message(format!(
                        "Could not create {} directory: {}", path_name, e
                    )))?;
            }
        }
        Err(e) => {
            return Err(ConfigError::Message(format!(
                "Could not check if {} directory exists: {}", path_name, e
            )));
        }
    }

    // 测试写入权限
    let test_dir = format!("{}/{}/{}/{}", path, "test", "sub", "dir");
    fs::create_dir_all(&test_dir)
        .map_err(|e| ConfigError::Message(format!(
            "Could not create test directory in {}: {}", path_name, e
        )))?;

    let test_file = format!("{}/test.tmp", test_dir);
    fs::write(&test_file, b"test")
        .map_err(|e| ConfigError::Message(format!(
            "Could not write test file in {}: {}", path_name, e
        )))?;

    fs::remove_file(&test_file).ok();
    fs::remove_dir_all(&test_dir).ok();

    Ok(())
}



 pub fn load_config() -> Result<AppConfig, ConfigError> {
    unsafe {
        INIT.call_once(|| {
            match load_config_internal() {
                Ok(app_config) => {
                    CONFIG = Some(app_config);
                }
                Err(e) => {
                    eprintln!("Failed to load configuration: {}", e);
                    CONFIG = None;
                }
            }
        });

        if let Some(ref config) = CONFIG {
            Ok(config.clone())
        } else {
            Err(ConfigError::Message(
                "Failed to load configuration".to_string(),
            ))
        }
    }
}

fn load_config_internal() -> Result<AppConfig, ConfigError> {
    // 1. 加载 .env 文件
    dotenv().ok();
    // 打印当前工作目录
    let cdir = {
        // try current_dir, then CARGO_MANIFEST_DIR, then fallback to "."
        let candidate = env::current_dir()
            .or_else(|_| env::var("CARGO_MANIFEST_DIR").map(PathBuf::from))
            .map_err(|e| ConfigError::Message(format!(
                "Failed to determine current dir: {}", e
            )))?;
        // try to canonicalize (make absolute / normalize). If canonicalize fails, keep candidate.
        fs::canonicalize(&candidate).unwrap_or(candidate)
    };
    // 2. 从 .env 获取当前环境 (默认 dev)
    let env = env::var(APP_ENV).unwrap_or_else(|_| "dev".into());
    // 3. 动态加载配置文件 (如 application.dev.json)
    let config_path = format!("{}/application.{}.json", cdir.display(), env);
    println!("application configuration file path :{}", config_path);
    // 4. 使用 config 库加载配置
    let settings = Config::builder()
        // 加载 JSON 配置文件 (如 application.dev.json)
        .add_source(File::with_name(&config_path).required(true))
        // 可选：允许环境变量覆盖配置 (如 DATABASE_URL=...)
        .add_source(Environment::with_prefix(APP_PREFIX).prefix_separator("_"))
        .build()
        .map_err(|err| ConfigError::Message(format!("Error loading config: {}", err)))?;

    // 5. 解析配置到结构体
    let mut app_config: AppConfig = settings
        .try_deserialize()
        .map_err(|err| ConfigError::Message(format!("Error parsing config: {}", err)))?;

    // 验证本地存储路径
    validate_and_create_path(&app_config.local_storage.dicm_store_path, "dicm_store_path")?;

    if app_config.local_storage.dicm_store_path.ends_with('/') {
        app_config.local_storage.dicm_store_path.pop();
    }

    validate_and_create_path(&app_config.local_storage.json_store_path, "json_store_path")?;

    if app_config.local_storage.json_store_path.ends_with('/') {
        app_config.local_storage.json_store_path.pop();
    }

    // 验证传输语法
    if !TransferSyntaxRegistry
        .get(&app_config.dicom_store_scp.unsupported_ts_change_to)
        .is_some()
    {
        return Err(ConfigError::Message(format!(
            "Invalid unsupported_ts_change_to transfer syntax UID: {}",
            app_config.dicom_store_scp.unsupported_ts_change_to
        )));
    }

    if app_config
        .dicom_store_scp
        .cornerstonejs_supported_transfer_syntax
        .is_empty()
    {
        return Err(ConfigError::Message(
            "scp_config.cornerstonejs_supported_transfer_syntax is empty".to_string(),
        ));
    } else {
        for transfer_syntax in &app_config
            .dicom_store_scp
            .cornerstonejs_supported_transfer_syntax
        {
            if !TransferSyntaxRegistry.get(transfer_syntax).is_some() {
                return Err(ConfigError::Message(format!(
                    "Invalid transfer syntax UID: {}",
                    transfer_syntax
                )));
            }
        }
    }

    // TODO: 输出配置信息，注意不要打印敏感信息
    println!("Configuration loaded successfully: {:?}", app_config);

    Ok(app_config)
}

pub fn generate_database_connection(dbconfig: &DatabaseConfig) -> Result<String, String> {
    let password = dbconfig
        .password
        .replace("@", "%40")
        .replace(":", "%3A")
        .replace("/", "%2F")
        .replace("?", "%3F")
        .replace("&", "%26")
        .replace("#", "%23")
        .replace("[", "%5B")
        .replace("]", "%5D")
        .replace("{", "%7B")
        .replace("}", "%7D")
        .replace("|", "%7C")
        .replace("<", "%3C")
        .replace(">", "%3E")
        .replace("\\", "%5C")
        .replace("^", "%5E")
        .replace("`", "%60");

    let db_conn = format!(
        "mysql://{}:{}@{}:{}/{}?allowPublicKeyRetrieval=true&characterEncoding=UTF-8&serverTimezone=Asia/Shanghai&useSSL=false",
        dbconfig.username, password, dbconfig.host, dbconfig.port, dbconfig.database
    );
    println!("database connection string: {}", db_conn);

    Ok(db_conn)
}

pub fn generate_pg_database_connection(dbconfig: &DatabaseConfig) -> Result<String, String> {
    let password = dbconfig
        .password
        .replace("@", "%40")
        .replace(":", "%3A")
        .replace("/", "%2F")
        .replace("?", "%3F")
        .replace("&", "%26")
        .replace("#", "%23")
        .replace("[", "%5B")
        .replace("]", "%5D")
        .replace("{", "%7B")
        .replace("}", "%7D")
        .replace("|", "%7C")
        .replace("<", "%3C")
        .replace(">", "%3E")
        .replace("\\", "%5C")
        .replace("^", "%5E")
        .replace("`", "%60");

    let db_conn = format!(
        "postgresql://{}:{}@{}:{}/{}",
        dbconfig.username, password, dbconfig.host, dbconfig.port, dbconfig.database
    );
    println!("postgresql database connection string: {}", db_conn);

    Ok(db_conn)
}
