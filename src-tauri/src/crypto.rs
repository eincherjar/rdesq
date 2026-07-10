use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use rand::RngCore;

const APP_KEY: &[u8; 32] = b"rdesq-2024-master-key-32bytes!xy";

pub fn encrypt(plaintext: &str) -> String {
    let key = aes_gcm::aead::generic_array::GenericArray::from_slice(APP_KEY);
    let cipher = Aes256Gcm::new(key);
    let mut nonce_bytes = [0u8; 12];
    OsRng.fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);
    let ciphertext = cipher.encrypt(nonce, plaintext.as_bytes()).unwrap();
    let mut combined = nonce_bytes.to_vec();
    combined.extend_from_slice(&ciphertext);
    BASE64.encode(&combined)
}

pub fn decrypt(encoded: &str) -> Result<String, String> {
    let data = BASE64.decode(encoded).map_err(|e| e.to_string())?;
    if data.len() < 12 {
        return Err("invalid encrypted data".into());
    }
    let (nonce_bytes, ciphertext) = data.split_at(12);
    let key = aes_gcm::aead::generic_array::GenericArray::from_slice(APP_KEY);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);
    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|_| "decryption failed".to_string())?;
    String::from_utf8(plaintext).map_err(|e| e.to_string())
}
