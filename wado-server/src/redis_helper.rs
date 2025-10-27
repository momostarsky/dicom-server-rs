use common::server_config::RedisConfig;
use redis::Commands;

// 通过配置创建Redis连接
pub fn connect_with_config(config: &RedisConfig) -> redis::Connection {
    redis::Client::open(config.url.clone())
        .expect("Invalid Redis connection URL")
        .get_connection()
        .expect("Failed to connect to Redis")
}

// 通过配置获取Redis连接
pub fn get_redis_connection_with_config(config: &RedisConfig) -> redis::Connection {
    connect_with_config(config)
}

// 通过配置设置带过期时间的Redis值
pub(crate) fn set_redis_value_with_expiry_and_config<RV: redis::FromRedisValue>(
    config: &RedisConfig,
    key: &str,
    value: &str,
    expiry_seconds: u64,
) {
    let mut redis_conn = get_redis_connection_with_config(config);
    redis_conn
        .set_ex::<&str, &str, RV>(key, value, expiry_seconds)
        .expect("Failed to set value with expiry");
}

// 通过配置获取Redis值
pub(crate) fn get_redis_value_with_config<RV: redis::FromRedisValue>(
    config: &RedisConfig,
    key: &str,
) -> Option<RV> {
    let mut redis_conn = get_redis_connection_with_config(config);
    redis_conn.get(key).ok()
}

pub(crate) fn db_study_entity_is_not_found<RV: redis::FromRedisValue>(
    config: &RedisConfig,
    study_uid: &str,
    expiry_seconds: u64,
) {
    let mut redis_conn = get_redis_connection_with_config(config);
    let key_for_not_found = format!("db:study:{}", study_uid);
    let _ = redis_conn.set_ex::<&str, &str, RV>(&key_for_not_found, "1", expiry_seconds);
}
pub(crate) fn db_study_entity_remove<RV: redis::FromRedisValue>(
    config: &RedisConfig,
    study_uid: &str,
) {
    let mut redis_conn = get_redis_connection_with_config(config);
    let key_for_not_found = format!("db:study:{}", study_uid);
    let _ = redis_conn.del::<&str, RV>(&key_for_not_found);
}
pub(crate) fn db_study_entity_is_not_exists<RV: redis::FromRedisValue>(
    config: &RedisConfig,
    study_uid: &str,
) -> Option<RV> {
    let mut redis_conn = get_redis_connection_with_config(config);
    let key_for_not_found = format!("db:study:{}", study_uid);
    redis_conn.get(&key_for_not_found).ok()
}
