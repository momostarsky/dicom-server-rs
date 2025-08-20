use std::sync::Arc;
use crate::database_provider::DbProvider;
use crate::mysql_provider::MySqlProvider;
use crate::server_config;
use sqlx::MySqlPool;
use tracing::error;

// common/src/database_factory.rs
// 根据配置文件生成数据库实例
pub async fn create_db_instance() ->  Option<Arc<dyn DbProvider>>  {
    let config = server_config::load_config();
    let config = match config {
        Ok(config) => config,
        Err(e) => {
            error!("load config failed: {:?}", e);
            std::process::exit(-2);
        }
    };
    let db_type = match &config.database {
        Some(database) => database.dbtype.to_lowercase(),
        None => {
            error!("database config not found");
            std::process::exit(-2);
        }
    };
    if db_type != "mysql" {
        error!("only mysql is supported");
        None
    } else {
        match server_config::generate_database_connection(&config) {
            Ok(url) => {
                let pool = MySqlPool::connect(&url).await;
                if pool.is_err() {
                    error!("connect to database failed: {:?}", pool.err());
                    None
                } else {
                    let db_provider = MySqlProvider::new(pool.unwrap());
                    Some(Arc::new(db_provider)) // 返回 Arc 而不是 Box
                }
            }
            Err(e) => {
                error!("generate database connection failed: {:?}", e);
                None
            }
        }
    }
}
