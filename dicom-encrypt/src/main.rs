use aes_gcm::{aead::{Aead, KeyInit, Nonce}, AeadInPlace, Aes256Gcm};

fn main() {


    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&[0u8; 32]); // 256-bit key
    let nonce = Nonce::<Aes256Gcm>::from_slice(&[0u8; 12]); // 96-bit nonce (recommended size)

    let cipher = Aes256Gcm::new(key);

    let plaintext = b"Hello, world!";

    // 加密
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


    let mut plaintext2 = b"Hello, worldxx!".to_vec(); // 改为 Vec<u8>
    let associated_data = b"metadata";
    let ciphertext = cipher
        .encrypt_in_place(nonce, associated_data, &mut plaintext2 )
        .expect("加密失败");
    // 解密时同样直接传递 &mut plaintext2
    println!("密文：{:?}", plaintext2);

    cipher
        .decrypt_in_place(nonce, associated_data, &mut plaintext2)
        .expect("解密失败或数据被篡改！");
    println!("明文2：{:?}", plaintext2);

}
