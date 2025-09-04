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

// 通过配置设置Redis值
pub(crate) fn set_redis_value_with_config<RV: redis::FromRedisValue>(
    config: &RedisConfig,
    key: &str,
    value: &str,
) {
    let mut redis_conn = get_redis_connection_with_config(config);
    redis_conn
        .set::<&str, &str, RV>(key, value)
        .expect("Failed to set value");
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
