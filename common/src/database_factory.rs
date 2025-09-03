use crate::database_provider::DbProvider;
use crate::mysql_provider::MySqlProvider;
use crate::server_config;
use sqlx::Executor;
use sqlx::mysql::MySqlPoolOptions;
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
    if db_type != "mysql" {
        error!("only mysql is supported");
        std::process::exit(-2);
    }


    let conn_url = match server_config::generate_database_connection(&config) {
        Ok(url) => url,
        _ => {
            error!("database connection string is not right");
            std::process::exit(-2);
        }
    };
    let pool_connection = match MySqlPoolOptions::new()
        .after_connect(|conn, _| {
            Box::pin(async move {
                let _ = conn.execute("SET time_zone='+08:00';").await;
                Ok(())
            })
        })
        .connect(&conn_url)
        .await
    {
        Ok(my) => my,
        Err(e) => {
            error!("database connection failed: {:?}", e);
            std::process::exit(-2);
        }
    };

    let db_provider = MySqlProvider::new(pool_connection);
    Some(Arc::new(db_provider)) // 返回 Arc 而不是 Box
}
