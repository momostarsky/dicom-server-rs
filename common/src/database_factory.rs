use crate::server_config;
use crate::server_config::DatabaseConfig;
use database::dicom_dbprovider::DbProvider;
use database::dicom_pg::PgDbProvider;
use std::sync::Arc;

// 定义自定义错误类型
#[derive(Debug)]
pub enum DatabaseError {
    ConfigError(String),
    ConnectionError(String),
    UnsupportedDatabase(String),
}

impl std::fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            DatabaseError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            DatabaseError::UnsupportedDatabase(msg) => write!(f, "Unsupported database: {}", msg),
        }
    }
}

impl std::error::Error for DatabaseError {}

// 根据配置文件生成数据库实例，返回 Result 而不是直接退出
pub async fn create_db_instance(
    dbconfig: &DatabaseConfig,
) -> Result<Arc<dyn DbProvider>, DatabaseError> {
    let db_type = dbconfig.dbtype.to_lowercase();
    if !(db_type == "postgresql") {
        return Err(DatabaseError::UnsupportedDatabase(
            "only  postgresql is supported".to_string(),
        ));
    }

    match db_type.as_str() {
        "postgresql" => {
            let conn_url =
                server_config::generate_pg_database_connection(&dbconfig).map_err(|_| {
                    DatabaseError::ConfigError(
                        "database connection string is not right".to_string(),
                    )
                })?;

            let pg_provider = PgDbProvider::new(conn_url);
            Ok(Arc::new(pg_provider))
        }

        _ => Err(DatabaseError::UnsupportedDatabase(format!(
            "Unsupported database type: {}",
            db_type
        ))),
    }
}
