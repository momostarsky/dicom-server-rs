use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct Claims {
    pub(crate) iss: String, //签发方（issuer），明确这个 JWT 是哪个认证系统生成的	必须（标准）
    pub(crate) sub: Option<String>, //主题（subject），指用户唯一标识（通常为用户 ID）	必须（标准）
    pub(crate) aud: Value,  //受众（audience），JWT 颁发给哪个客户端/应用	必须（强烈建议)
    pub(crate) exp: usize,  //过期时间（expiration），用于 token 有效期控制	必须（强烈建议）
    pub(crate) azp: Option<String>,
    pub(crate) email: Option<String>,
    pub(crate) name: Option<String>,
    pub(crate) username: Option<String>,
    pub(crate) preferred_username: Option<String>,
    pub(crate) given_name: Option<String>,
    pub(crate) family_name: Option<String>,
    pub(crate) realm_access: Option<RealmAccess>, // realm 级别权限
    pub(crate) resource_access: Option<std::collections::HashMap<String, ResourceAccess>>, // 资源级别权限
    pub(crate) scope: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct RealmAccess {
    pub(crate) roles: Option<Vec<String>>, // realm 角色
}
#[allow(dead_code)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct ResourceAccess {
    pub(crate) roles: Option<Vec<String>>, // 资源角色
}
