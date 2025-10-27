use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};

static AES_GCM_KEY: [u8; 32] = *b"UbwehJpq0cJDTxdEbNZ0v2Yzl#P*x92j";
static AES_GCM_NONCE: [u8; 12] = *b"lXyl!6o3A*j3";
pub fn encrypt_string(plain_text: &str) -> Result<String, Box<dyn std::error::Error>> {
    let cipher = Aes256Gcm::new(&AES_GCM_KEY.into());
    let nonce = Nonce::from_slice(&AES_GCM_NONCE);
    // 加密数据
    let ciphertext = match cipher.encrypt(nonce, plain_text.as_bytes()) {
        Ok(ct) => ct,
        Err(e) => return Err("Invalid encrypted data".into()),
    };
    // 将密钥、nonce和密文组合成一个可传输的字符串
    let mut result = Vec::new();
    result.extend_from_slice(b"V001"); // 添加4字节版本号
    result.extend_from_slice(&ciphertext);
    use base64::{
        Engine as _, alphabet,
        engine::{self, general_purpose},
    };

    let b64 = general_purpose::STANDARD.encode(&result);
    Ok(b64)
}

pub fn decrypt_string(encrypted_text: &str) -> Result<String, Box<dyn std::error::Error>> {
    use base64::{
        Engine as _, alphabet,
        engine::{self, general_purpose},
    };
    // 解码base64
    let data = general_purpose::STANDARD.decode(encrypted_text)?;

    if data.len() < 4 {
        return Err("Invalid encrypted data".into());
    }
    // 检查版本号
    let version = &data[0..4];
    if version != b"V001" {
        return Err(format!(
            "Unsupported encryption version: {:?}",
            std::str::from_utf8(version).unwrap_or("invalid")
        )
        .into());
    }
    // 分离密钥、nonce和密文

    let ciphertext = &data[4..];

    // 初始化解密器
    let cipher = Aes256Gcm::new_from_slice(&AES_GCM_KEY)
        .map_err(|e| format!("Cipher init error: {:?}", e))?;
    let nonce = Nonce::from_slice(&AES_GCM_NONCE);

    // 解密数据
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("Decryption failed: {:?}", e))?;

    // 转换为字符串
    Ok(String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8 sequence: {:?}", e))?)
}

use generic_array::GenericArray;
use salsa20::{
    Salsa20, XSalsa20,
    cipher::{KeyIvInit, StreamCipher},
};
static SALSA20_KEY: [u8; 32] = *b"KyRvYBr%VYg2U!1p@5dE7t^t5ZZ8Xp&2";
static SALSA20_KEY_ONE: [u8; 8] = *b"1X3h0C6h";

pub fn encrypt_with_salsa20(plaintext: &str) -> Vec<u8> {
    let mut data = plaintext.as_bytes().to_vec();
    let key_array = GenericArray::from_slice(&SALSA20_KEY);
    let nonce_array = GenericArray::from_slice(&SALSA20_KEY_ONE);

    let mut cipher = Salsa20::new(key_array, nonce_array);
    cipher.apply_keystream(&mut data);
    data
}

pub fn decrypt_with_salsa20(ciphertext: &[u8]) -> Vec<u8> {
    let mut data = ciphertext.to_vec();
    let key_array = GenericArray::from_slice(&SALSA20_KEY);
    let nonce_array = GenericArray::from_slice(&SALSA20_KEY_ONE);

    let mut cipher = Salsa20::new(key_array, nonce_array);
    cipher.apply_keystream(&mut data);
    data
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_string() {
        let original_text = "Hello, World!";
        let encrypted = encrypt_string(original_text).expect("Encryption failed");
        let decrypted = decrypt_string(&encrypted).expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_empty_string() {
        let original_text = "";
        let encrypted = encrypt_string(original_text).expect("Encryption failed");
        let decrypted = decrypt_string(&encrypted).expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_chinese_string() {
        let original_text = "你好，世界！";
        let encrypted = encrypt_string(original_text).expect("Encryption failed");
        let decrypted = decrypt_string(&encrypted).expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_encrypt_decrypt_long_string() {
        let original_text = "This is a longer test string to verify that the encryption and decryption works correctly with larger amounts of data.";
        let encrypted = encrypt_string(original_text).expect("Encryption failed");
        let decrypted = decrypt_string(&encrypted).expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_decrypt_invalid_version() {
        // 创建一个带有无效版本号的加密数据
        let mut invalid_data = Vec::new();
        invalid_data.extend_from_slice(b"V002"); // 无效版本号
        invalid_data.extend_from_slice(b"some_data");

        use base64::{Engine as _, engine::general_purpose};
        let encoded = general_purpose::STANDARD.encode(&invalid_data);

        let result = decrypt_string(&encoded);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported encryption version")
        );
    }

    #[test]
    fn test_decrypt_invalid_base64() {
        let invalid_base64 = "invalid_base64!!!";
        let result = decrypt_string(invalid_base64);
        assert!(result.is_err());
    }

    #[test]
    fn test_decrypt_too_short_data() {
        use base64::{Engine as _, engine::general_purpose};
        let short_data = b"A";
        let encoded = general_purpose::STANDARD.encode(short_data);
        let result = decrypt_string(&encoded);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid encrypted data")
        );
    }
    #[test]
    fn test_salsa20_encrypt_decrypt() {
        let original_text = "Hello, World!";
        let encrypted = encrypt_with_salsa20(original_text);
        let decrypted = decrypt_with_salsa20(&encrypted);

        assert_eq!(original_text.as_bytes(), decrypted);
    }

    #[test]
    fn test_salsa20_encrypt_decrypt_empty_string() {
        let original_text = "";
        let encrypted = encrypt_with_salsa20(original_text);
        let decrypted = decrypt_with_salsa20(&encrypted);

        assert_eq!(original_text.as_bytes(), decrypted);
    }

    #[test]
    fn test_salsa20_encrypt_decrypt_chinese_string() {
        let original_text = "你好，世界！";
        let encrypted = encrypt_with_salsa20(original_text);
        let decrypted = decrypt_with_salsa20(&encrypted);

        assert_eq!(original_text.as_bytes(), decrypted);
    }

    #[test]
    fn test_salsa20_encrypt_decrypt_long_string() {
        let original_text = "This is a longer test string to verify that the Salsa20 encryption and decryption works correctly with larger amounts of data.";
        let encrypted = encrypt_with_salsa20(original_text);
        let decrypted = decrypt_with_salsa20(&encrypted);

        assert_eq!(original_text.as_bytes(), decrypted);
    }

    #[test]
    fn test_salsa20_encrypt_length_preservation() {
        let original_text = "Test string";
        let original_len = original_text.len();
        let encrypted = encrypt_with_salsa20(original_text);
        let decrypted = decrypt_with_salsa20(&encrypted);
        // 验证加密和解密后的长度保持不变
        assert_eq!(encrypted.len(), original_len);
        assert_eq!(decrypted.len(), original_len);
        assert_eq!(decrypted, original_text.as_bytes());
    }
}
