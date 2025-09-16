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
    signature_data: Vec<u8>,
    public_key: PublicKey,
    algorithm: SignatureAlgorithm,
    timestamp: DateTime<Utc>,
}

impl PublicKey {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(PublicKey {
            key_data: bytes,
            algorithm: SignatureAlgorithm::MLDSA65,
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.key_data
    }

    pub fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm
    }

    fn to_pqc_public_key(&self) -> dilithium5::PublicKey {
        dilithium5::PublicKey::from_bytes(&self.key_data).expect("Chave pública válida")
    }
}

impl PrivateKey {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        Ok(PrivateKey {
            key_data: bytes,
            algorithm: SignatureAlgorithm::MLDSA65,
        })
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.key_data
    }

    pub fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm
    }

    fn to_pqc_secret_key(&self) -> dilithium5::SecretKey {
        dilithium5::SecretKey::from_bytes(&self.key_data).expect("Chave privada válida")
    }
}

impl KeyPair {
    pub fn generate() -> Result<Self> {
        let (public_key_bytes, secret_key_bytes) = dilithium5::keypair();

        let public_key = PublicKey::from_bytes(public_key_bytes.as_bytes().to_vec())?;
        let private_key = PrivateKey::from_bytes(secret_key_bytes.as_bytes().to_vec())?;

        Ok(KeyPair {
            public_key,
            private_key,
        })
    }

    pub fn sign(&self, message: &[u8]) -> Result<Signature> {
        let secret_key = self.private_key.to_pqc_secret_key();
        let signed_message = dilithium5::sign(message, &secret_key);

        Ok(Signature {
            signature_data: signed_message.as_bytes().to_vec(),
            public_key: self.public_key.clone(),
            algorithm: SignatureAlgorithm::MLDSA65,
            timestamp: Utc::now(),
        })
    }
}

impl Signature {
    pub fn verify(&self, message: &[u8]) -> Result<bool> {
        let public_key = self.public_key.to_pqc_public_key();
        let signed_message = SignedMessage::from_bytes(&self.signature_data)
            .map_err(|_| BlockchainError::InvalidSignature)?;

        match dilithium5::open(&signed_message, &public_key) {
            Ok(verified_message) => Ok(verified_message == message),
            Err(_) => Ok(false),
        }
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.public_key
    }

    pub fn algorithm(&self) -> SignatureAlgorithm {
        self.algorithm
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.timestamp
    }

    pub fn size(&self) -> usize {
        self.signature_data.len()
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

/// Funções utilitárias para assinatura de transações
pub fn sign_transaction_hash(tx_hash: &Hash256, keypair: &KeyPair) -> Result<Signature> {
    keypair.sign(tx_hash.as_bytes())
}

/// Verifica uma assinatura contra um hash de transação
pub fn verify_transaction_signature(tx_hash: &Hash256, signature: &Signature) -> Result<bool> {
    signature.verify(tx_hash.as_bytes())
}

/// Cria uma chave pública a partir de bytes (para deserialização)
pub fn public_key_from_bytes(bytes: &[u8]) -> Result<PublicKey> {
    PublicKey::from_bytes(bytes.to_vec())
}

/// Cria uma assinatura a partir de bytes (para deserialização)
pub fn signature_from_bytes(
    signature_data: Vec<u8>,
    public_key: PublicKey,
    timestamp: DateTime<Utc>,
) -> Signature {
    Signature {
        signature_data,
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
