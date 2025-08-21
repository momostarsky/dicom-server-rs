use config::{Config, ConfigError, Environment, File};
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

// 定义配置结构体
#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub dbtype: String, //数据库类型 POSTGRES  MYSQL SQLITE
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub host: String,
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

#[derive(Debug, Deserialize)]
pub struct DicomStoreScpConfig {
    pub port: u16,
    pub ae_title: String,
}

#[derive(Debug, Deserialize)]
pub struct KafkaConfig {
    pub brokers: String,
    pub consumer_group_id: String,
    pub queue_buffering_max_messages: u32,
    pub queue_buffering_max_kbytes: u32,
    pub batch_num_messages: u32,
    pub queue_buffering_max_ms: u32,
    pub linger_ms: u32,
    pub compression_codec: String,
    pub topic_main: String,
    pub topic_change_transfer_syntax: String,
    pub topic_multi_frames: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub kafka: Option<KafkaConfig>,
    pub database: Option<DatabaseConfig>,
    pub server: Option<ServerConfig>,
    pub local_storage: Option<LocalStorageConfig>,
    pub dicom_store_scp: Option<DicomStoreScpConfig>,
}

static APP_ENV: &str = "APP_ENV";
static APP_PREFIX: &str = "DICOM";

pub fn load_config() -> Result<AppConfig, ConfigError> {
    // 1. 加载 .env 文件
    dotenv().ok();
    // 打印当前工作目录
    match env::current_dir() {
        Ok(path) => println!("Current working directory: {:?}", path),
        Err(e) => println!("Failed to get current directory: {}", e),
    }
    // 2. 从 .env 获取当前环境 (默认 dev)
    let env = env::var(APP_ENV).unwrap_or_else(|_| "dev".into());

    // 3. 动态加载配置文件 (如 application.dev.json)
    let config_path = format!("application.{}.json", env);

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
    let app_config: AppConfig = settings.try_deserialize()?;
    match &app_config {
        AppConfig {
            database: Some(database),
            ..
        } => {
            println!("database:dbtype {:?}", database.dbtype);
            println!("database:host {:?}", database.host);
            println!("database:port {:?}", database.port);
            println!("database:username {:?}", database.username);
            println!("database:password {:?}", database.password);
            println!("database:database {:?}", database.database);
        }
        AppConfig {
            server: Some(server),
            ..
        } => {
            println!("server:port {:?}", server.port);
            println!("server:host {:?}", server.host);
        }
        AppConfig {
            local_storage: Some(local_storage),
            ..
        } => {
            println!(
                "local_storage:dicom_store_path {:?}",
                local_storage.dicom_store_path
            );
            println!(
                "local_storage:json_store_path {:?}",
                local_storage.json_store_path
            );
        }
        AppConfig {
            dicom_store_scp: Some(dicom_store_scp),
            ..
        } => {
            println!("dicom_store_scp:port {:?}", dicom_store_scp.port);
            println!("dicom_store_scp:ae_title {:?}", dicom_store_scp.ae_title);
        }
        AppConfig {
            kafka: Some(kafka), ..
        } => {
            println!("kafka:brokers {:?}", kafka.brokers);
            println!("kafka:consumer_group_id {:?}", kafka.consumer_group_id);
            println!(
                "kafka:queue_buffering_max_messages {:?}",
                kafka.queue_buffering_max_messages
            );
            println!(
                "kafka:queue_buffering_max_kbytes {:?}",
                kafka.queue_buffering_max_kbytes
            );
            println!("kafka:batch_num_messages {:?}", kafka.batch_num_messages);
            println!(
                "kafka:queue_buffering_max_ms {:?}",
                kafka.queue_buffering_max_ms
            );
            println!("kafka:linger_ms {:?}", kafka.linger_ms);
            println!("kafka:compression_codec {:?}", kafka.compression_codec);
            println!("kafka:topic {:?}", kafka.topic_main);
            println!(
                "kafka:topic_change_transfer_syntax {:?}",
                kafka.topic_change_transfer_syntax
            );
            println!("kafka:topic_multi_frames {:?}", kafka.topic_multi_frames);
        }
        _ => {
            println!("other config {:?}", app_config);
        }
    }
    Ok(app_config)
}

pub fn generate_database_connection(app_config: &AppConfig) -> std::result::Result<String, String> {
    let dbconfig = &app_config.database;
    if dbconfig.is_none() {
        return Err("database config is none".to_string());
    }
    let password = dbconfig
        .as_ref()
        .unwrap()
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

    let cfg = dbconfig.as_ref().unwrap();

    let db_conn = format!(
        "mysql://{}:{}@{}:{}/{}",
        cfg.username, password, cfg.host, cfg.port, cfg.database
    );
    println!("database connection string: {}", db_conn);
    Ok(db_conn)
}
