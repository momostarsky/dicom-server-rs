use std::str::FromStr;
use common::mysql_provider::MySqlProvider;
use dicom_dictionary_std::tags;
use dicom_object::collector::CharacterSetOverride;
use sqlx::Executor;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use tracing::error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let password = "hzjp#123";
    let password = password
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

    let db_conn =          "mysql://dicomstore:hzjp%23123@192.168.1.14:9030/dicomdb";
    println!("database connection string: {}", db_conn);
    let connect_options = MySqlConnectOptions::from_str(db_conn )?
        .no_engine_substitution(false)
        .pipes_as_concat(false);

    let pool_connection = match MySqlPoolOptions::new()
        .after_connect(|conn, _| {
            Box::pin(async move {
                let _ = conn.execute("SET time_zone='+08:00';").await;
                Ok(())
            })
        })
        .connect_with(connect_options)
        .await
    {
        Ok(my) => my,
        Err(e) => {
            error!(
                "database connection failed: {:?}. Connection string: {}",
                e, db_conn
            );

            std::process::exit(-2);
        }
    };

    let db_provider = MySqlProvider::new(pool_connection);

    Ok(())
}
