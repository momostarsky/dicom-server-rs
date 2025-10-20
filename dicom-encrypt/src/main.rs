
use aes_gcm::{
    Aes256Gcm, // 或 Aes128Gcm
    aead::{Aead, KeyInit, Nonce},
};
use aes_gcm::aead::{AeadMut, AeadMutInPlace};

fn main() {


    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&[0u8; 32]); // 256-bit key
    let nonce = Nonce::<Aes256Gcm>::from_slice(&[0u8; 12]); // 96-bit nonce (recommended size)

    let cipher = Aes256Gcm::new(key);

    let plaintext = b"Hello, world!";


    // 加密（自动附加 16 字节 tag）
    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_ref())
        .expect("加密失败");

    println!("密文：{:?}", ciphertext);
    // 解密（自动验证 tag）
    let decrypted = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .expect("解密失败或数据被篡改！");

    assert_eq!(&decrypted[..], plaintext);
    println!("明文：{:?}", decrypted);
    println!("成功！");
}
