use crate::database_provider::DbProvider;
use crate::dbprovider_mysql::MySqlProvider;
use crate::server_config;
use sqlx::Executor;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use std::str::FromStr;
use std::sync::Arc;
use tracing::error;

// common/src/database_factory.rs
// 根据配置文件生成数据库实例
pub async fn create_db_instance() -> Option<Arc<dyn DbProvider>> {
    let config = server_config::load_config();
    let config = match config {
        Ok(config) => config,
        Err(e) => {
            error!("load config failed: {:?}", e);
            std::process::exit(-2);
        }
    };
    let db_type = config.database.dbtype.to_lowercase();
    if !(db_type == "mysql" || db_type == "doris") {
        error!("only mysql  or doris is supported");
        std::process::exit(-2);
    }

    let conn_url = match server_config::generate_database_connection(&config) {
        Ok(url) => url,
        _ => {
            error!("database connection string is not right");
            std::process::exit(-2);
        }
    };
    match db_type.as_str() {
        "mysql" => {
            match MySqlPoolOptions::new()
                .after_connect(|conn, _| {
                    Box::pin(async move {
                        let _ = conn.execute("SET time_zone='+08:00';").await;
                        Ok(())
                    })
                })
                .connect(&conn_url)
                .await
            {
                Ok(my) => {
                    let db_provider = MySqlProvider::new(my);
                    Some(Arc::new(db_provider)) // 返回 Arc 而不是 Box
                }
                Err(e) => {
                    error!(
                        "database connection failed: {:?}. Connection string: {}",
                        e, conn_url
                    );
                    std::process::exit(-2);
                }
            }
        }
        "doris" => {
            let connect_options = match MySqlConnectOptions::from_str(&conn_url) {
                Ok(options) => options.no_engine_substitution(false).pipes_as_concat(false),
                Err(e) => {
                    error!("Failed to parse Doris connection options: {:?}", e);
                    std::process::exit(-2);
                }
            };

            match MySqlPoolOptions::new()
                .after_connect(|conn, _| {
                    Box::pin(async move {
                        let _ = conn.execute("SET time_zone='+08:00';").await;
                        Ok(())
                    })
                })
                .connect_with(connect_options)
                .await
            {
                Ok(my) => {
                    let db_provider = MySqlProvider::new(my);
                    Some(Arc::new(db_provider)) // 返回 Arc 而不是 Box
                }
                Err(e) => {
                    error!(
                        "database connection failed: {:?}. Connection string: {}",
                        e, conn_url
                    );
                    std::process::exit(-2);
                }
            }
        }
        _ => {
            error!("Unsupported database type: {}", db_type);
            std::process::exit(-2);
        }
    }
}
