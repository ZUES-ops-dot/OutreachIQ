use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use aes_gcm::aead::generic_array::GenericArray;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use std::env;

const NONCE_SIZE: usize = 12;

pub struct EncryptionService {
    cipher: Aes256Gcm,
    key_id: String,
}

impl EncryptionService {
    pub fn new() -> Result<Self, String> {
        let key_str = env::var("ENCRYPTION_KEY")
            .map_err(|_| "ENCRYPTION_KEY environment variable not set")?;
        
        let key_bytes = BASE64.decode(&key_str)
            .map_err(|e| format!("Invalid ENCRYPTION_KEY format: {}", e))?;
        
        if key_bytes.len() != 32 {
            return Err("ENCRYPTION_KEY must be 32 bytes (256 bits) when decoded".to_string());
        }
        
        let key = GenericArray::from_slice(&key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        let key_id = env::var("ENCRYPTION_KEY_ID")
            .unwrap_or_else(|_| "default-key-v1".to_string());
        
        Ok(Self { cipher, key_id })
    }

    pub fn new_with_key(key_bytes: &[u8], key_id: &str) -> Result<Self, String> {
        if key_bytes.len() != 32 {
            return Err("Key must be 32 bytes (256 bits)".to_string());
        }
        
        let key = GenericArray::from_slice(key_bytes);
        let cipher = Aes256Gcm::new(key);
        
        Ok(Self { 
            cipher, 
            key_id: key_id.to_string() 
        })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<(Vec<u8>, String), String> {
        use rand::RngCore;
        
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        
        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_bytes())
            .map_err(|e| format!("Encryption failed: {}", e))?;
        
        // Prepend nonce to ciphertext
        let mut result = nonce_bytes.to_vec();
        result.extend(ciphertext);
        
        Ok((result, self.key_id.clone()))
    }

    pub fn decrypt(&self, encrypted_data: &[u8]) -> Result<String, String> {
        if encrypted_data.len() < NONCE_SIZE {
            return Err("Encrypted data too short".to_string());
        }
        
        let (nonce_bytes, ciphertext) = encrypted_data.split_at(NONCE_SIZE);
        let nonce = Nonce::from_slice(nonce_bytes);
        
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| format!("Decryption failed: {}", e))?;
        
        String::from_utf8(plaintext)
            .map_err(|e| format!("Invalid UTF-8 in decrypted data: {}", e))
    }

    pub fn key_id(&self) -> &str {
        &self.key_id
    }
}

pub fn generate_encryption_key() -> String {
    use rand::RngCore;
    let mut key = [0u8; 32];
    OsRng.fill_bytes(&mut key);
    BASE64.encode(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 32]; // Test key - don't use in production
        let service = EncryptionService::new_with_key(&key, "test-key").unwrap();
        
        let plaintext = "my-secret-password";
        let (encrypted, key_id) = service.encrypt(plaintext).unwrap();
        
        assert_eq!(key_id, "test-key");
        assert_ne!(encrypted, plaintext.as_bytes());
        
        let decrypted = service.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_generate_key() {
        let key = generate_encryption_key();
        let decoded = BASE64.decode(&key).unwrap();
        assert_eq!(decoded.len(), 32);
    }
}
