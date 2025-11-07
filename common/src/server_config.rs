use config::{Config, ConfigError, Environment, File};
use database::dicom_dbtype::BoundedString;
use database::dicom_meta::DicomStateMeta;
use dicom_encoding::TransferSyntaxIndex;
use dicom_transfer_syntax_registry::TransferSyntaxRegistry;
use dotenv::dotenv;
use serde::Deserialize;
use std::env;
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

#[derive(Debug, Deserialize, Clone)]
pub struct OAuth2Config {
    pub issuer_url: String,
    pub audience: String,
    pub jwks_url: String,
}

// "wado_oauth2": {
// "issuer_url": "https://keycloak.medical.org:8443/realms/dicom-org-cn",
// "audience": "wado-rs-api",
// "jwks_url": "https://keycloak.medical.org:8443/realms/dicom-org-cn/protocol/openid-connect/certs"
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
}

static APP_ENV: &str = "APP_ENV";
static ISSUER_URL: &str = "ISSUER_URL";
static AUDIENCE: &str = "AUDIENCE";
static JWKS_URL: &str = "JWKS_URL";
static APP_PREFIX: &str = "DICOM";

// 全局配置实例和初始化状态
static INIT: Once = Once::new();
static mut CONFIG: Option<AppConfig> = None;

pub fn load_config() -> Result<AppConfig, ConfigError> {
    // 使用 Once 确保只初始化一次
    unsafe {
        INIT.call_once(|| {
            // 1. 加载 .env 文件
            dotenv().ok();
            // 打印当前工作目录
            let cdir = match env::current_dir() {
                Ok(path) => {
                    println!("Current working directory: {:?}", path);
                    path
                }
                Err(e) => {
                    println!("Failed to get current directory: {}", e);
                    std::path::PathBuf::from("./")
                }
            };
            // 2. 从 .env 获取当前环境 (默认 dev)
            let env = env::var(APP_ENV).unwrap_or_else(|_| "dev".into());

            // ISSUER_URL=https://keycloak.medical.org:8443/realms/dicom-org-cn
            //     AUDIENCE=wado-rs-api
            // SERVER_PORT=8080
            // JWKS_URL: "https://keycloak.medical.org:8443/realms/dicom-org-cn/protocol/openid-connect/certs",

            // 3. 动态加载配置文件 (如 application.dev.json)
            let config_path = format!("{}/application.{}.json", cdir.display(), env);

            // 4. 使用 config 库加载配置
            let settings = Config::builder()
                // 加载 JSON 配置文件 (如 application.dev.json)
                .add_source(File::with_name(&config_path).required(true))
                // 可选：允许环境变量覆盖配置 (如 DATABASE_URL=...)
                .add_source(Environment::with_prefix(APP_PREFIX).prefix_separator("_"))
                .build();
            let settings = match settings {
                Ok(settings) => settings,
                Err(err) => panic!("Error loading config: {}", err),
            };

            // 5. 解析配置到结构体
            let mut app_config: AppConfig = match settings.try_deserialize() {
                Ok(app_config) => app_config,
                Err(err) => panic!("Error parsing config: {}", err),
            };

            // 打印配置信息（只在首次加载时打印）
            println!("redis:url {:?}", app_config.redis.url);
            println!("main_database:dbtype {:?}", app_config.main_database.dbtype);
            println!("main_database:host {:?}", app_config.main_database.host);
            println!("main_database:port {:?}", app_config.main_database.port);
            println!(
                "main_database:username {:?}",
                app_config.main_database.username
            );
            println!(
                "main_database:password {:?}",
                app_config.main_database.password
            );
            println!(
                "main_database:database {:?}",
                app_config.main_database.database
            );

            println!(
                "secondary_database:dbtype {:?}",
                app_config.secondary_database.dbtype
            );
            println!(
                "secondary_database:host {:?}",
                app_config.secondary_database.host
            );
            println!(
                "secondary_database:port {:?}",
                app_config.secondary_database.port
            );
            println!(
                "secondary_database:username {:?}",
                app_config.secondary_database.username
            );
            println!(
                "secondary_database:password {:?}",
                app_config.secondary_database.password
            );
            println!(
                "secondary_database:database {:?}",
                app_config.secondary_database.database
            );

            println!("server:port {:?}", app_config.server.port);
            println!("server:host {:?}", app_config.server.host);
            println!("server:log_level {:?}", app_config.server.allow_origin);
            println!(
                "local_storage:dicm_store_path {:?}",
                app_config.local_storage.dicm_store_path
            );
            if app_config.local_storage.dicm_store_path.ends_with("/") {
                app_config.local_storage.dicm_store_path.pop();
            }

            if app_config.local_storage.dicm_store_path.len() > 64 {
                panic!("dicm_store_path length must be less than 64 characters");
            }

            match std::fs::exists(&app_config.local_storage.dicm_store_path) {
                Ok(exists) => {
                    if !exists {
                        std::fs::create_dir_all(&app_config.local_storage.dicm_store_path)
                            .unwrap_or_else(|e| {
                                panic!("Could not create dicm_store_path directory: {}", e);
                            });
                    }
                }
                Err(e) => {
                    panic!("Could not check if dicm_store_path directory exists: {}", e);
                }
            }
            // TODO :验证能否在dicom_storage_path 下面创建目录及写入文件
            let test_dir = format!(
                "{}/{}/{}/{}",
                app_config.local_storage.dicm_store_path, "1.222", "1.444", "3.5555"
            );
            std::fs::create_dir_all(&test_dir).unwrap_or_else(|e| {
                panic!("Could not create test_dir directory: {}", e);
            });
            let test_file = format!("{}/test.dcm", test_dir);
            std::fs::write(
                &test_file,
                b"903290903234092409383404903409289899889jkkallklkj",
            )
            .unwrap_or_else(|e| {
                panic!("Could not write test_file file: {}", e);
            });
            std::fs::remove_file(&test_file).unwrap_or_else(|e| {
                panic!("Could not remove test_file file: {}", e);
            });
            std::fs::remove_dir_all(&test_dir).unwrap_or_else(|e| {
                panic!("Could not remove test_dir directory: {}", e);
            });

            println!(
                "local_storage:json_store_path {:?}",
                app_config.local_storage.json_store_path
            );
            if app_config.local_storage.json_store_path.ends_with("/") {
                app_config.local_storage.json_store_path.pop();
            }
            if app_config.local_storage.json_store_path.len() > 64 {
                panic!("json_store_path length must be less than 64 characters");
            }

            match std::fs::exists(&app_config.local_storage.json_store_path) {
                Ok(exists) => {
                    if !exists {
                        std::fs::create_dir_all(&app_config.local_storage.json_store_path)
                            .unwrap_or_else(|e| {
                                panic!("Could not create json_store_path directory: {}", e);
                            });
                    }
                }
                Err(e) => {
                    panic!("Could not check if json_store_path directory exists: {}", e);
                }
            }
            // TODO :验证能否在json_store_path 下面创建目录及写入文件
            let json_test_dir = format!(
                "{}/{}/{}/{}",
                app_config.local_storage.json_store_path, "1.222", "2.444", "3.555"
            );
            std::fs::create_dir_all(&json_test_dir).unwrap_or_else(|e| {
                panic!("Could not create json_test_dir directory: {}", e);
            });
            let json_test_file = format!("{}/test.json", json_test_dir);
            std::fs::write(
                &json_test_file,
                b"903290903234092409383404903409289899889jkkallklkj",
            )
            .unwrap_or_else(|e| {
                panic!("Could not write json_test_file file: {}", e);
            });
            std::fs::remove_file(&json_test_file).unwrap_or_else(|e| {
                panic!("Could not remove json_test_file file: {}", e);
            });
            std::fs::remove_dir_all(&json_test_dir).unwrap_or_else(|e| {
                panic!("Could not remove json_test_dir directory: {}", e);
            });

            println!("dicom_store_scp:port {:?}", app_config.dicom_store_scp.port);
            println!(
                "dicom_store_scp:ae_title {:?}",
                app_config.dicom_store_scp.ae_title
            );
            println!(
                "dicom_store_scp:cornerstonejs_supported_transfer_syntax {:?}",
                app_config
                    .dicom_store_scp
                    .cornerstonejs_supported_transfer_syntax
            );
            println!(
                "dicom_store_scp:unsupported_ts_change_to {:?}",
                app_config.dicom_store_scp.unsupported_ts_change_to
            );
            if !TransferSyntaxRegistry
                .get(&app_config.dicom_store_scp.unsupported_ts_change_to)
                .is_some()
            {
                panic!(
                    "Invalid unsupported_ts_change_to transfer syntax UID: {}",
                    app_config.dicom_store_scp.unsupported_ts_change_to
                );
            }

            //TODO: 验证 scp_config.cornerstonejs_supported_transfer_syntax 是否为空
            if app_config
                .dicom_store_scp
                .cornerstonejs_supported_transfer_syntax
                .is_empty()
            {
                // 不要在这里直接返回，而是设置错误状态
                // 使用 panic! 而不是 return，因为 call_once 闭包返回 ()
                panic!("scp_config.cornerstonejs_supported_transfer_syntax is empty");
            } else {
                //TODO: 验证 scp_config.cornerstonejs_supported_transfer_syntax 中的每个元素是否为有效的传输语法UID
                for transfer_syntax in &app_config
                    .dicom_store_scp
                    .cornerstonejs_supported_transfer_syntax
                {
                    if !TransferSyntaxRegistry.get(transfer_syntax).is_some() {
                        panic!("Invalid transfer syntax UID: {}", transfer_syntax);
                    }
                }
            }

            println!("kafka:brokers {:?}", app_config.kafka.brokers);

            println!(
                "kafka:queue_buffering_max_messages {:?}",
                app_config.kafka.queue_buffering_max_messages
            );
            println!(
                "kafka:queue_buffering_max_kbytes {:?}",
                app_config.kafka.queue_buffering_max_kbytes
            );
            println!(
                "kafka:batch_num_messages {:?}",
                app_config.kafka.batch_num_messages
            );
            println!(
                "kafka:queue_buffering_max_ms {:?}",
                app_config.kafka.queue_buffering_max_ms
            );
            println!("kafka:linger_ms {:?}", app_config.kafka.linger_ms);
            println!(
                "kafka:compression_codec {:?}",
                app_config.kafka.compression_codec
            );
            println!(
                "kafka:consumer_group_id {:?}",
                app_config.message_queue.consumer_group_id
            );
            println!(
                "message_queue:topic_main {:?}",
                app_config.message_queue.topic_main
            );
            println!(
                "message_queue:topic_log {:?}",
                app_config.message_queue.topic_log
            );

            if let Some(license_server) = app_config.dicom_license_server.as_ref() {
                println!(
                    "dicom_license_server:client_id {:?}",
                    license_server.client_id
                );
                println!(
                    "dicom_license_server:license_key {:?}",
                    license_server.license_key
                );
            }
            if let Some(oa2) = app_config.wado_oauth2.as_ref() {
                println!("wado_oauth2:issuer_url {:?}", oa2.issuer_url);
                println!("wado_oauth2:audience {:?}", oa2.audience);
                println!("wado_oauth2:jwks_url {:?}", oa2.jwks_url);
            }

            CONFIG = Some(app_config);
        });

        // 返回克隆的配置实例
        if let Some(ref config) = CONFIG {
            Ok(config.clone())
        } else {
            // 这种情况理论上不应该发生
            Err(ConfigError::Message(
                "Failed to load configuration".to_string(),
            ))
        }
    }
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

/// 生成 UID 的哈希值, 对于不足20位时，定长设为20位,前置补0
pub fn hash_uid(uid: &str) -> String {
    use seahash::SeaHasher;
    use std::hash::Hasher;

    let mut hasher = SeaHasher::new();
    hasher.write(uid.as_bytes());
    let hash_value = hasher.finish();

    // 将 u64 转换为字符串，并用前导零填充到 20 位
    format!("{:020}", hash_value)
}

pub fn dicom_study_dir(
    study_info: &DicomStateMeta,
    create_not_exists: bool,
) -> Result<String, std::io::Error> {
    let app_config = load_config().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load config: {}", e),
        )
    })?;
    let dicom_store_path = &app_config.local_storage.dicm_store_path;
    let study_dir = format!(
        "{}/{}/{}/{}",
        dicom_store_path,
        study_info.tenant_id.as_str(),
        study_info.study_date_origin.as_str(),
        study_info.study_uid.as_str()
    );
    if create_not_exists {
        std::fs::create_dir_all(&study_dir).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("create dicom_study_dir failed: {} with:'{}'", study_dir, e),
            )
        })?;
    }
    Ok((study_dir))
}

pub fn dicom_series_dir(
    study_info: &DicomStateMeta,
    create_not_exists: bool,
) -> Result<String, std::io::Error> {
    let app_config = load_config().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load config: {}", e),
        )
    })?;
    let dicom_store_path = &app_config.local_storage.dicm_store_path;
    let study_dir = format!(
        "{}/{}/{}/{}/{}",
        dicom_store_path,
        study_info.tenant_id.as_str(),
        study_info.study_date_origin.as_str(),
        study_info.study_uid.as_str(),
        study_info.series_uid.as_str()
    );
    if create_not_exists {
        std::fs::create_dir_all(&study_dir).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("create dicom_series_dir failed: {} with:'{}'", study_dir, e),
            )
        })?;
    }
    Ok((study_dir))
}

pub fn make_series_dicom_dir(
    tenant_id: &str,
    study_date: &str,
    study_uid: &str,
    series_uid: &str,
    create_not_exists: bool,
) -> Result<String, std::io::Error> {
    let app_config = load_config().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load config: {}", e),
        )
    })?;
    let dicom_store_path = &app_config.local_storage.dicm_store_path;
    let study_dir = format!(
        "{}/{}/{}/{}",
        dicom_store_path, study_date, study_uid, series_uid
    );
    if create_not_exists {
        std::fs::create_dir_all(&study_dir).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "create make_series_dicom_dir failed: {} with:'{}'",
                    study_dir, e
                ),
            )
        })?;
    }
    Ok((study_dir))
}

pub fn json_metadata_for_study(
    study_info: &DicomStateMeta,
    create_not_exists: bool,
) -> Result<String, std::io::Error> {
    let app_config = load_config().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load config: {}", e),
        )
    })?;
    let json_store_path = &app_config.local_storage.json_store_path;
    let study_dir = format!(
        "{}/{}/metadata/{}",
        json_store_path,
        study_info.tenant_id.as_str(),
        study_info.study_date_origin.as_str()
    );
    let json_path = format!("{}/{}.json", study_dir, study_info.study_uid.as_str());
    if create_not_exists {
        std::fs::create_dir_all(&study_dir).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory:  '{}': {}", study_dir, e),
            )
        })?
    }
    Ok((json_path))
}

pub fn json_metadata_for_series(
    study_info: &DicomStateMeta,
    create_not_exists: bool,
) -> Result<String, std::io::Error> {
    let app_config = load_config().map_err(|e| {
        std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to load config: {}", e),
        )
    })?;
    let json_store_path = &app_config.local_storage.json_store_path;
    let study_dir = format!(
        "{}/{}/metadata/{}/{}",
        json_store_path,
        study_info.tenant_id.as_str(),
        study_info.study_date_origin.as_str(),
        study_info.study_uid.as_str()
    );
    let json_path = format!("{}/{}.json", study_dir, study_info.series_uid.as_str());
    if create_not_exists {
        std::fs::create_dir_all(&study_dir).map_err(|e| {
            std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to create directory:  '{}': {}", study_dir, e),
            )
        })?
    }
    Ok(json_path)
}

pub fn dicom_file_path(dir: &str, sop_uid: &str) -> String {
    format!("{}/{}.dcm", dir, sop_uid)
}
