use crate::server_config;
use crate::server_config::DatabaseConfig;
use database::dicom_dbprovider::DbProvider;
use database::dicom_mysql::MySqlDbProvider;
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
    if !(db_type == "doris" || db_type == "postgresql") {
        return Err(DatabaseError::UnsupportedDatabase(
            "only mysql, doris or postgresql is supported".to_string(),
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

            // let pool = PgPoolOptions::new()
            //     .after_connect(|conn, _| {
            //         Box::pin(async move {
            //             let _ = conn.execute("SET timezone = 'Asia/Shanghai';").await;
            //             Ok(())
            //         })
            //     })
            //     .connect(&conn_url)
            //     .await
            //     .map_err(|e| DatabaseError::ConnectionError(format!(
            //         "database connection failed: {:?}. Connection string: {}",
            //         e, conn_url
            //     )))?;

            let pg_provider = PgDbProvider::new(conn_url);
            Ok(Arc::new(pg_provider))
        }
        "doris" => {
            let conn_url =
                server_config::generate_database_connection(&dbconfig).map_err(|_| {
                    DatabaseError::ConfigError(
                        "database connection string is not right".to_string(),
                    )
                })?;

            // let connect_options = MySqlConnectOptions::from_str(&conn_url)
            //     .map_err(|e| {
            //         DatabaseError::ConfigError(format!(
            //             "Failed to parse Doris connection options: {:?}",
            //             e
            //         ))
            //     })?
            //     .no_engine_substitution(false)
            //     .pipes_as_concat(false);
            //
            // let pool = MySqlPoolOptions::new()
            //     .after_connect(|conn, _| {
            //         Box::pin(async move {
            //             let _ = conn.execute("SET time_zone='+08:00';").await;
            //             Ok(())
            //         })
            //     })
            //     .connect_with(connect_options)
            //     .await
            //     .map_err(|e| {
            //         DatabaseError::ConnectionError(format!(
            //             "database connection failed: {:?}. Connection string: {}",
            //             e, conn_url
            //         ))
            //     })?;

            let db_provider = MySqlDbProvider::new(conn_url);
            Ok(Arc::new(db_provider))
        }
        _ => Err(DatabaseError::UnsupportedDatabase(format!(
            "Unsupported database type: {}",
            db_type
        ))),
    }
}
