use aes_gcm::{
    Aes256Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::Engine;
use base64::engine::general_purpose;

pub trait EncryptHelper {
    /// 加密字符串, return base64 encoded string
    fn encrypt_string(&self, plain_text: &str) -> Result<String, Box<dyn std::error::Error>>;
    /// 解密字符串,input is base64 encoded string and  return utf8 encoded text
    fn decrypt_string(&self, encrypted_text: &str) -> Result<String, Box<dyn std::error::Error>>;
}
const ENCRYPT_KEY: [u8; 32] = *b"UbwehJpq0cJDTxdEbNZ0v2Yzl#P*x92j";
const AES_GCM_NONCE: [u8; 12] = *b"lXyl!6o3A*j3";

static SALSA20_KEY_ONE: [u8; 8] = *b"1X3h0C6h";
pub struct AesGcmEncryptor {
    key: [u8; 32],
    nonce: [u8; 12],
    version: [u8; 4],
}

impl AesGcmEncryptor {
    pub fn new() -> Self {
        AesGcmEncryptor {
            key: ENCRYPT_KEY,
            nonce: AES_GCM_NONCE,
            version: *b"V001",
        }
    }
    pub fn version(&self) -> [u8; 4] {
        self.version
    }
}
impl EncryptHelper for AesGcmEncryptor {
    fn encrypt_string(&self, plain_text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let cipher = Aes256Gcm::new(&ENCRYPT_KEY.into());
        let nonce = Nonce::from_slice(&AES_GCM_NONCE);
        // 加密数据
        let ciphertext = match cipher.encrypt(nonce, plain_text.as_bytes()) {
            Ok(ct) => ct,
            Err(_e) => return Err("Invalid encrypted data".into()),
        };
        // 将密钥、nonce和密文组合成一个可传输的字符串
        let mut result = Vec::new();
        result.extend_from_slice(b"V001"); // 添加4字节版本号
        result.extend_from_slice(&ciphertext);
        use base64::{Engine as _, engine::general_purpose};

        let b64 = general_purpose::STANDARD.encode(&result);
        Ok(b64)
    }

    fn decrypt_string(&self, encrypted_text: &str) -> Result<String, Box<dyn std::error::Error>> {
        use base64::{Engine as _, engine::general_purpose};
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
        let cipher = Aes256Gcm::new_from_slice(&ENCRYPT_KEY)
            .map_err(|e| format!("Cipher init error: {:?}", e))?;
        let nonce = Nonce::from_slice(&AES_GCM_NONCE);

        // 解密数据
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {:?}", e))?;

        // 转换为字符串
        Ok(String::from_utf8(plaintext).map_err(|e| format!("Invalid UTF-8 sequence: {:?}", e))?)
    }
}

use generic_array::GenericArray;
use salsa20::{
    Salsa20,
    cipher::{KeyIvInit, StreamCipher},
};

pub struct Salsa20Encryptor {
    key: [u8; 32],
    nonce: [u8; 8],
    version: [u8; 4],
}

impl Salsa20Encryptor {
    pub fn new() -> Self {
        Salsa20Encryptor {
            key: ENCRYPT_KEY,
            nonce: SALSA20_KEY_ONE,
            version: *b"V002",
        }
    }
    pub fn version(&self) -> [u8; 4] {
        self.version
    }
}
impl EncryptHelper for Salsa20Encryptor {
    fn encrypt_string(&self, plain_text: &str) -> Result<String, Box<dyn std::error::Error>> {
        let mut data = plain_text.as_bytes().to_vec();
        let key_array = GenericArray::from_slice(&ENCRYPT_KEY);
        let nonce_array = GenericArray::from_slice(&SALSA20_KEY_ONE);

        let mut cipher = Salsa20::new(key_array, nonce_array);
        cipher.apply_keystream(&mut data);

        // 添加版本号
        let mut result = Vec::new();
        result.extend_from_slice(self.version().as_slice());
        result.extend_from_slice(&data);
        Ok(general_purpose::STANDARD.encode(&result))
    }

    fn decrypt_string(&self, encrypted_text: &str) -> Result<String, Box<dyn std::error::Error>> {
        // 解码base64
        let data = general_purpose::STANDARD.decode(encrypted_text)?;

        if data.len() < 4 {
            return Err("Invalid encrypted data".into());
        }

        // 检查版本号
        let version = &data[0..4];
        if version != self.version() {
            return Err(format!(
                "Unsupported encryption version: {:?}",
                std::str::from_utf8(version).unwrap_or("invalid")
            )
            .into());
        }

        let ciphertext = &data[4..];
        let mut decrypted_data = ciphertext.to_vec();
        let key_array = GenericArray::from_slice(&ENCRYPT_KEY);
        let nonce_array = GenericArray::from_slice(&SALSA20_KEY_ONE);

        let mut cipher = Salsa20::new(key_array, nonce_array);
        cipher.apply_keystream(&mut decrypted_data);
        Ok(String::from_utf8(decrypted_data)
            .map_err(|e| format!("Invalid UTF-8 sequence: {:?}", e))?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aes_gcm_encrypt_decrypt() {
        let encryptor = AesGcmEncryptor::new();
        let original_text = "Hello, World!";
        let encrypted = encryptor
            .encrypt_string(original_text)
            .expect("Encryption failed");
        let decrypted = encryptor
            .decrypt_string(&encrypted)
            .expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_aes_gcm_encrypt_decrypt_empty_string() {
        let encryptor = AesGcmEncryptor::new();
        let original_text = "";
        let encrypted = encryptor
            .encrypt_string(original_text)
            .expect("Encryption failed");
        let decrypted = encryptor
            .decrypt_string(&encrypted)
            .expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_aes_gcm_encrypt_decrypt_chinese_string() {
        let encryptor = AesGcmEncryptor::new();
        let original_text = "你好，世界！";
        let encrypted = encryptor
            .encrypt_string(original_text)
            .expect("Encryption failed");
        let decrypted = encryptor
            .decrypt_string(&encrypted)
            .expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_aes_gcm_encrypt_decrypt_mixed_string() {
        let encryptor = AesGcmEncryptor::new();
        let original_text = "Hello你好123!";
        let encrypted = encryptor
            .encrypt_string(original_text)
            .expect("Encryption failed");
        let decrypted = encryptor
            .decrypt_string(&encrypted)
            .expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_aes_gcm_decrypt_invalid_version() {
        let encryptor = AesGcmEncryptor::new();
        // 创建一个带有无效版本号的加密数据
        let mut invalid_data = Vec::new();
        invalid_data.extend_from_slice(b"V002"); // 无效版本号
        invalid_data.extend_from_slice(b"some_data");

        let encoded = general_purpose::STANDARD.encode(&invalid_data);

        let result = encryptor.decrypt_string(&encoded);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported encryption version")
        );
    }

    #[test]
    fn test_salsa20_encrypt_decrypt() {
        let encryptor = Salsa20Encryptor::new();
        let original_text = "Hello, World!";
        let encrypted = encryptor
            .encrypt_string(original_text)
            .expect("Encryption failed");
        let decrypted = encryptor
            .decrypt_string(&encrypted)
            .expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_salsa20_encrypt_decrypt_empty_string() {
        let encryptor = Salsa20Encryptor::new();
        let original_text = "";
        let encrypted = encryptor
            .encrypt_string(original_text)
            .expect("Encryption failed");
        let decrypted = encryptor
            .decrypt_string(&encrypted)
            .expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_salsa20_encrypt_decrypt_chinese_string() {
        let encryptor = Salsa20Encryptor::new();
        let original_text = "你好，世界！";
        let encrypted = encryptor
            .encrypt_string(original_text)
            .expect("Encryption failed");
        let decrypted = encryptor
            .decrypt_string(&encrypted)
            .expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_salsa20_encrypt_decrypt_mixed_string() {
        let encryptor = Salsa20Encryptor::new();
        let original_text = "Hello你好123!";
        let encrypted = encryptor
            .encrypt_string(original_text)
            .expect("Encryption failed");
        let decrypted = encryptor
            .decrypt_string(&encrypted)
            .expect("Decryption failed");

        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_salsa20_decrypt_invalid_version() {
        let encryptor = Salsa20Encryptor::new();
        // 创建一个带有无效版本号的加密数据
        let mut invalid_data = Vec::new();
        invalid_data.extend_from_slice(b"V001"); // 无效版本号
        invalid_data.extend_from_slice(b"some_data");

        let encoded = general_purpose::STANDARD.encode(&invalid_data);

        let result = encryptor.decrypt_string(&encoded);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Unsupported encryption version")
        );
    }

    #[test]
    fn test_encryptor_trait_objects() {
        let aes_encryptor: Box<dyn EncryptHelper> = Box::new(AesGcmEncryptor::new());
        let salsa_encryptor: Box<dyn EncryptHelper> = Box::new(Salsa20Encryptor::new());

        let original_text = "Test with trait objects";

        // 测试 AES-GCM
        let encrypted = aes_encryptor
            .encrypt_string(original_text)
            .expect("AES encryption failed");
        let decrypted = aes_encryptor
            .decrypt_string(&encrypted)
            .expect("AES decryption failed");
        assert_eq!(original_text, decrypted);

        // 测试 Salsa20
        let encrypted = salsa_encryptor
            .encrypt_string(original_text)
            .expect("Salsa20 encryption failed");
        let decrypted = salsa_encryptor
            .decrypt_string(&encrypted)
            .expect("Salsa20 decryption failed");
        assert_eq!(original_text, decrypted);
    }

    #[test]
    fn test_both_encryptors_with_same_data() {
        let aes_encryptor = AesGcmEncryptor::new();
        let salsa_encryptor = Salsa20Encryptor::new();

        let original_text = "Same data, different algorithms";

        // 使用 AES-GCM 加密解密
        let aes_encrypted = aes_encryptor
            .encrypt_string(original_text)
            .expect("AES encryption failed");
        let aes_decrypted = aes_encryptor
            .decrypt_string(&aes_encrypted)
            .expect("AES decryption failed");

        // 使用 Salsa20 加密解密
        let salsa_encrypted = salsa_encryptor
            .encrypt_string(original_text)
            .expect("Salsa20 encryption failed");
        let salsa_decrypted = salsa_encryptor
            .decrypt_string(&salsa_encrypted)
            .expect("Salsa20 decryption failed");

        // 验证两者都能正确解密回原文
        assert_eq!(original_text, aes_decrypted);
        assert_eq!(original_text, salsa_decrypted);

        // 验证加密结果不同（因为算法和版本号不同）
        assert_ne!(aes_encrypted, salsa_encrypted);
    }
}
