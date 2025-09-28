use config::{Config, ConfigError, Environment, File};
use dotenv::dotenv;
use serde::Deserialize;
use std::env;
use std::sync::Once;

#[derive(Debug, Deserialize, Clone)]
pub struct RedisConfig {
    pub url: String,            //连接地址
    pub passwd: Option<String>, //密码
    pub is_lts: Option<bool>,   //是否启动TLS
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
// "local_storage":{
// "type": "DISK",
// "dicom_store_path": "/home/dhz/jpdata/CDSS",
// "json_store_path": "/home/dhz/jpdata/CDSS/store"
// }
#[derive(Debug, Deserialize, Clone)]
pub struct LocalStorageConfig {
    pub dicom_store_path: String,
    pub json_store_path: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct DicomStoreScpConfig {
    pub port: u16,
    pub ae_title: String,
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
    pub topic_change_transfer_syntax: String ,

}

#[derive(Debug, Deserialize, Clone)]
pub struct LicenseServerConfig {
    /// DICOM 许可服务器的 API 密钥  16位字母或是数字字符串
    pub client_id: String,
    /// DICOM 许可密钥的HASHCODE
    pub license_key: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub redis: RedisConfig,
    pub kafka: KafkaConfig,
    pub database: DatabaseConfig,
    pub server: ServerConfig,
    pub local_storage: LocalStorageConfig,
    pub dicom_store_scp: DicomStoreScpConfig,
    pub message_queue: MessageQueueConfig,
    pub dicom_license_server: Option<LicenseServerConfig>,
}

static APP_ENV: &str = "APP_ENV";
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
            let app_config: AppConfig = match settings.try_deserialize() {
                Ok(app_config) => app_config,
                Err(err) => panic!("Error parsing config: {}", err),
            };

            // 打印配置信息（只在首次加载时打印）
            println!("redis:url {:?}", app_config.redis.url);
            println!("database:dbtype {:?}", app_config.database.dbtype);
            println!("database:host {:?}", app_config.database.host);
            println!("database:port {:?}", app_config.database.port);
            println!("database:username {:?}", app_config.database.username);
            println!("database:password {:?}", app_config.database.password);
            println!("database:database {:?}", app_config.database.database);
            println!("server:port {:?}", app_config.server.port);
            println!("server:host {:?}", app_config.server.host);
            println!("server:log_level {:?}", app_config.server.allow_origin);
            println!(
                "local_storage:dicom_store_path {:?}",
                app_config.local_storage.dicom_store_path
            );
            println!(
                "local_storage:json_store_path {:?}",
                app_config.local_storage.json_store_path
            );
            println!("dicom_store_scp:port {:?}", app_config.dicom_store_scp.port);
            println!(
                "dicom_store_scp:ae_title {:?}",
                app_config.dicom_store_scp.ae_title
            );
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
                "message_queue:topic_change_transfer_syntax {:?}",
                app_config.message_queue.topic_change_transfer_syntax
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
pub fn generate_database_connection(app_config: &AppConfig) -> Result<String, String> {
    let dbconfig = &app_config.database;
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
