use serde::{Deserialize, Serialize};
use sha3::{Digest, Keccak256};
use std::fmt;

/// Hash de 256 bits usado para identificar blocos, transações e outros dados
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Hash256([u8; 32]);

impl Hash256 {
    /// Cria um hash zerado
    pub fn zero() -> Self {
        Hash256([0u8; 32])
    }

    /// Cria um hash a partir de bytes
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        Hash256(bytes)
    }

    /// Obtém os bytes do hash
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// Calcula o hash Keccak-256 dos dados fornecidos
    pub fn keccak256(data: &[u8]) -> Self {
        let mut hasher = Keccak256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Hash256(hash)
    }

    /// Verifica se o hash satisfaz a dificuldade especificada
    /// (número de zeros iniciais em bits)
    pub fn meets_difficulty(&self, difficulty: u32) -> bool {
        let leading_zeros = self.leading_zeros();
        leading_zeros >= difficulty
    }

    /// Conta o número de zeros iniciais no hash
    pub fn leading_zeros(&self) -> u32 {
        let mut zeros = 0;
        for &byte in &self.0 {
            if byte == 0 {
                zeros += 8;
            } else {
                zeros += byte.leading_zeros();
                break;
            }
        }
        zeros
    }
}

impl fmt::Display for Hash256 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0))
    }
}

impl From<String> for Hash256 {
    fn from(hex_string: String) -> Self {
        let bytes = hex::decode(hex_string).expect("Invalid hex string");
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Hash256(hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_creation_and_display() {
        let hash = Hash256::zero();
        assert_eq!(hash.to_string(), "0000000000000000000000000000000000000000000000000000000000000000");
    }

    #[test]
    fn test_keccak256_hashing() {
        let data = b"hello world";
        let hash = Hash256::keccak256(data);
        // Verifica se o hash não é zero (dados válidos devem produzir hash não-zero)
        assert_ne!(hash, Hash256::zero());
    }

    #[test]
    fn test_difficulty_check() {
        // Hash com muitos zeros iniciais
        let easy_hash = Hash256::from_bytes([0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        assert!(easy_hash.meets_difficulty(20));
        assert!(!easy_hash.meets_difficulty(40));
    }

    #[test]
    fn test_leading_zeros_count() {
        let hash = Hash256::from_bytes([0, 0, 0, 128, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]);
        // 3 bytes de zeros (24 bits) + 1 bit zero no quarto byte = 25 zeros
        assert_eq!(hash.leading_zeros(), 24);
    }
}
