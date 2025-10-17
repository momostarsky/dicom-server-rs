use seahash::SeaHasher;
use std::hash::{Hash, Hasher};
// use std::sync::LazyLock;

// pub static UID_HASHER: LazyLock<SeaHasher> =   LazyLock::new(|| SeaHasher::new());
//
//
// /// 将UID转换为确定性的u64哈希值
// pub fn uid_to_u64_deterministic(uid: &str) -> u64 {
//     let mut hasher = UID_HASHER.clone();
//     uid.hash(&mut hasher);
//     hasher.finish()
// }
/// 将UID转换为确定性的u64哈希值,线程安全的版本
pub fn uid_to_u64(uid: &str) -> u64 {
    let mut hasher = SeaHasher::new();  // 每次创建新实例
    uid.hash(&mut hasher);
    hasher.finish()
}