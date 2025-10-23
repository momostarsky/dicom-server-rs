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


use sha2::{Sha256, Digest};

/// 安全、标准、可读、无碰撞风险（实际意义上）
pub fn uid_hash_hex(uid: &str) -> String  {
     let hash = Sha256::digest(uid.as_bytes());
     format!("{:x}", hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uid_hash_hex_deterministic() {
        // 测试相同输入产生相同输出
        let uid = "1.2.840.10008.1.2.1";
        let hash1 = uid_hash_hex(uid);
        let hash2 = uid_hash_hex(uid);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_uid_hash_hex_different_inputs() {
        // 测试不同输入产生不同输出
        let uid1 = "1.2.840.10008.1.2.1";
        let uid2 = "1.2.840.10008.1.2.4.50";
        let hash1 = uid_hash_hex(uid1);
        let hash2 = uid_hash_hex(uid2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_uid_hash_hex_format() {
        // 测试输出格式为64字符的十六进制字符串
        let uid = "1.2.840.10008.1.2.1";
        let hash = uid_hash_hex(uid);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_uid_hash_hex_known_values() {
        // 测试已知输入的预期输出
        let uid = "1.2.840.10008.1.2.1";
        let expected = "96468535182c4185b7034dccdf950305f22125db2aa13a5fc73bce3d5c6653cc";
        let hash = uid_hash_hex(uid);
        assert_eq!(hash, expected);
    }

    #[test]
    fn test_uid_hash_hex_empty_input() {
        // 测试空字符串输入
        let uid = "";
        let hash = uid_hash_hex(uid);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}
