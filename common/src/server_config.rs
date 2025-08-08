use config::{Config, ConfigError, Environment, File};
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

// 定义配置结构体
#[derive(Debug, Deserialize)]
pub struct DBConfig {
    pub type_: String, //数据库类型 POSTGRES  MYSQL SQLITE
    pub url: String,   //数据库连接字符串
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
#[derive(Debug, Deserialize)]
pub struct LocalStorageConfig {
    pub dicom_store_path: String,
    pub json_store_path: String,
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub database: Option<DBConfig>,
    pub server: Option<ServerConfig>,
    pub local_storage: Option<LocalStorageConfig>,
}

static APP_ENV: &str = "APP_ENV";
static APP_PREFIX: &str = "DICOM";

pub fn load_config() -> Result<AppConfig, ConfigError> {
    // 1. 加载 .env 文件
    dotenv().ok();

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
        .build()?;

    // 5. 解析配置到结构体
    let app_config: AppConfig = settings.try_deserialize()?;
    Ok(app_config)
}
