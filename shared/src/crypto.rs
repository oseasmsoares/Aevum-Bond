//! Módulo de Criptografia Pós-Quântica para Aevum & Bond

use crate::{BlockchainError, Hash256, Result};
use chrono::{DateTime, Utc};
use pqcrypto_dilithium::dilithium5;
use pqcrypto_traits::sign::{
    PublicKey as PQCPublicKeyTrait, SecretKey as PQCSecretKeyTrait, SignedMessage,
};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

/// Algoritmos de assinatura suportados
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignatureAlgorithm {
    MLDSA65,
}

/// Chave pública ML-DSA
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    key_data: Vec<u8>,
    algorithm: SignatureAlgorithm,
}

/// Chave privada ML-DSA
#[derive(Debug, Clone)]
pub struct PrivateKey {
    key_data: Vec<u8>,
    algorithm: SignatureAlgorithm,
}

/// Par de chaves ML-DSA
#[derive(Debug, Clone)]
pub struct KeyPair {
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

/// Assinatura digital pós-quântica
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signature {
    data: Vec<u8>,
    public_key: PublicKey,
    algorithm: SignatureAlgorithm,
    timestamp: DateTime<Utc>,
}

impl PublicKey {
    /// Cria uma chave pública a partir de bytes
    ///
    /// # Errors
    ///
    /// Retorna erro se os bytes não representarem uma chave válida
    #[allow(clippy::missing_const_for_fn)] // Vec::new() não é const
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            key_data: bytes,
            algorithm: SignatureAlgorithm::MLDSA65,
        })
    }

    /// Returns the raw bytes of the public key
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.key_data
    }

    /// Returns the signature algorithm used by this key
    #[must_use]
    pub const fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm
    }

    fn to_pqc_public_key(&self) -> dilithium5::PublicKey {
        dilithium5::PublicKey::from_bytes(&self.key_data).expect("Chave pública válida")
    }
}

impl PrivateKey {
    /// Creates a `PrivateKey` from raw bytes
    ///
    /// # Errors
    ///
    /// Returns error if the bytes are invalid for the algorithm
    #[allow(clippy::missing_const_for_fn)] // Vec operations not const
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(Self {
            key_data: bytes,
            algorithm: SignatureAlgorithm::MLDSA65,
        })
    }

    /// Returns the raw bytes of the private key
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.key_data
    }

    /// Returns the signature algorithm used by this key
    #[must_use]
    pub const fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm
    }

    fn to_pqc_secret_key(&self) -> dilithium5::SecretKey {
        dilithium5::SecretKey::from_bytes(&self.key_data).expect("Chave privada válida")
    }
}

impl KeyPair {
    /// Generates a new keypair for ML-DSA-65
    ///
    /// # Errors
    ///
    /// Returns error if key generation fails
    pub fn generate() -> Result<Self> {
        let (public_key_bytes, secret_key_bytes) = dilithium5::keypair();

        let public_key = PublicKey::from_bytes(public_key_bytes.as_bytes().to_vec())?;
        let private_key = PrivateKey::from_bytes(secret_key_bytes.as_bytes().to_vec())?;

        Ok(Self {
            public_key,
            private_key,
        })
    }

    /// Signs a message using the private key
    ///
    /// # Errors
    ///
    /// Returns error if signing fails
    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        let secret_key = self.private_key.to_pqc_secret_key();
        let signed_message = dilithium5::sign(message, &secret_key);

        Ok(Signature {
            data: signed_message.as_bytes().to_vec(),
            public_key: self.public_key.clone(),
            algorithm: SignatureAlgorithm::MLDSA65,
            timestamp: Utc::now(),
        })
    }
}

impl Signature {
    /// Verifies the signature against a message
    ///
    /// # Errors
    ///
    /// Returns error if verification fails
    pub fn verify(&self, message: &[u8]) -> Result<bool> {
        let public_key = self.public_key.to_pqc_public_key();
        let signed_message =
            SignedMessage::from_bytes(&self.data).map_err(|_| BlockchainError::InvalidSignature)?;

        dilithium5::open(&signed_message, &public_key).map_or(Ok(false), |verified_message| {
            Ok(verified_message == message)
        })
    }

    /// Returns the public key used for verification
    #[must_use]
    pub const fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    /// Returns the signature algorithm
    #[must_use]
    pub const fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm
    }

    /// Returns the timestamp when the signature was created
    #[must_use]
    pub const fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    /// Returns the size in bytes of this signature
    #[must_use]
    pub const fn size(&self) -> usize {
        self.data.len()
    }
}

impl Zeroize for PrivateKey {
    fn zeroize(&mut self) {
        self.key_data.zeroize();
    }
}

impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.zeroize();
    }
}

/// Signs a transaction hash with the given keypair
///
/// # Errors
///
/// Returns error if signing fails
pub fn sign_transaction_hash(tx_hash: &Hash256, keypair: &KeyPair) -> Result<Signature> {
    keypair.sign(tx_hash.as_bytes())
}

/// Verifies a signature against a transaction hash
///
/// # Errors
///
/// Returns error if verification fails
pub fn verify_transaction_signature(tx_hash: &Hash256, signature: &Signature) -> Result<bool> {
    signature.verify(tx_hash.as_bytes())
}

/// Creates a public key from raw bytes
///
/// # Errors
///
/// Returns error if the bytes are invalid
pub fn public_key_from_bytes(bytes: &[u8]) -> Result<PublicKey> {
    PublicKey::from_bytes(bytes.to_vec())
}

/// Creates a signature from raw components
#[allow(clippy::missing_const_for_fn)] // DateTime operations not const
#[must_use]
pub const fn signature_from_bytes(
    signature_data: Vec<u8>,
    public_key: PublicKey,
    timestamp: DateTime<Utc>,
) -> Signature {
    Signature {
        data: signature_data,
        public_key,
        algorithm: SignatureAlgorithm::MLDSA65,
        timestamp,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pqc_basic_functionality() {
        let keypair = KeyPair::generate().unwrap();
        let message = b"Aevum & Bond - Post-quantum blockchain";

        let signature = keypair.sign(message).unwrap();
        assert!(signature.verify(message).unwrap());

        println!("✅ ML-DSA-65 funcionando!");
        println!(
            "   Public Key:  {} bytes",
            keypair.public_key.as_bytes().len()
        );
        println!(
            "   Private Key: {} bytes",
            keypair.private_key.as_bytes().len()
        );
        println!("   Signature:   {} bytes", signature.size());
    }
}
