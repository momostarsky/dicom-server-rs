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
pub fn uid_to_u64_deterministic_safe(uid: &str) -> u64 {
    let mut hasher = SeaHasher::new();  // 每次创建新实例
    uid.hash(&mut hasher);
    hasher.finish()
}

/// 将UID转换为确定性的u32哈希值，使用study_uid作为种子
/// 同一个StudyUID下面的SeriesUID会被hash到同一个空间范围内
/// 适用于SeriesEntity的series_uid_hash字段
/// 在单个 StudyUID 下最多100个序列的实际应用场景中，uid_to_u32_deterministic_safe 函数产生的哈希值碰撞概率极低，约为百万分之一的数量级，可以安全使用
pub fn uid_to_u32_deterministic_safe(study_uid: &str, uid: &str) -> u32 {
    let mut hasher = SeaHasher::new();
    // 先使用study_uid作为种子
    study_uid.hash(&mut hasher);
    // 再hash目标uid
    uid.hash(&mut hasher);
    // 返回u32范围内的值
    (hasher.finish() & 0xFFFFFFFF) as u32
}