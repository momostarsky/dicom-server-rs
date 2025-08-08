use config::{Config, ConfigError, Environment, File};
use dotenv::dotenv;
use serde::Deserialize;
use std::env;

// 定义配置结构体
#[derive(Debug, Deserialize)]
pub(crate) struct DBConfig {
    pub(crate) url: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct ServerConfig {
    pub(crate) port: u16,
    pub(crate) host: String,
}

#[derive(Debug, Deserialize)]
pub(crate) struct AppConfig {
    pub(crate) database: DBConfig,
    pub(crate) server: ServerConfig,
}

static APP_ENV: &str = "APP_ENV";
static APP_PREFIX: &str = "DICOMWEB";

pub(crate) fn load_config() -> Result<AppConfig, ConfigError> {
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
