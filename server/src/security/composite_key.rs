use sha2::{Sha256, Digest};
use hex;

/// Generates a composite key by hashing IP + Browser Fingerprint + Server Secret
#[derive(Clone)]
pub struct CompositeKeyGenerator {
    server_secret: String,
}

impl CompositeKeyGenerator {
    /// Create a new composite key generator with a server secret
    pub fn new(server_secret: String) -> Self {
        Self { server_secret }
    }

    /// Generate a composite key from IP address and browser fingerprint
    /// 
    /// # Arguments
    /// * `ip` - The user's IP address
    /// * `fingerprint` - The browser fingerprint from ThumbmarkJS
    /// 
    /// # Returns
    /// A hexadecimal string representing the hashed composite key
    pub fn generate(&self, ip: &str, fingerprint: &str) -> String {
        let combined = format!("{}:{}:{}", ip, fingerprint, self.server_secret);
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Validate that a composite key matches the expected format
    #[allow(dead_code)]
    pub fn is_valid_key(&self, key: &str) -> bool {
        // A SHA256 hash in hex is 64 characters
        key.len() == 64 && key.chars().all(|c| c.is_ascii_hexdigit())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_composite_key() {
        let generator = CompositeKeyGenerator::new("test_secret".to_string());
        let key = generator.generate("192.168.1.1", "fingerprint123");
        
        assert_eq!(key.len(), 64);
        assert!(generator.is_valid_key(&key));
    }

    #[test]
    fn test_same_inputs_produce_same_key() {
        let generator = CompositeKeyGenerator::new("test_secret".to_string());
        let key1 = generator.generate("192.168.1.1", "fingerprint123");
        let key2 = generator.generate("192.168.1.1", "fingerprint123");
        
        assert_eq!(key1, key2);
    }

    #[test]
    fn test_different_inputs_produce_different_keys() {
        let generator = CompositeKeyGenerator::new("test_secret".to_string());
        let key1 = generator.generate("192.168.1.1", "fingerprint123");
        let key2 = generator.generate("192.168.1.2", "fingerprint123");
        
        assert_ne!(key1, key2);
    }
}
