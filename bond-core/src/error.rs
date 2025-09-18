use std::fmt;
use shared::{BlockchainError, Result};

/// Tipos de erro específicos do Bond
#[derive(Debug, Clone, PartialEq)]
pub enum BondError {
    /// Erro de script
    ScriptError(String),
    /// Erro de transação não encontrada
    TransactionNotFound(String),
    /// Erro de validação de transação
    InvalidTransaction(String),
    /// Erro de validação de bloco
    InvalidBlock(String),
    /// Erro criptográfico
    CryptoError(String),
    /// Erro de validação
    ValidationError(String),
    /// Erro de serialização
    SerializationError(String),
    /// Erro genérico
    Other(String),
}

impl fmt::Display for BondError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BondError::ScriptError(msg) => write!(f, "Script error: {}", msg),
            BondError::TransactionNotFound(msg) => write!(f, "Transaction not found: {}", msg),
            BondError::InvalidTransaction(msg) => write!(f, "Invalid transaction: {}", msg),
            BondError::InvalidBlock(msg) => write!(f, "Invalid block: {}", msg),
            BondError::CryptoError(msg) => write!(f, "Crypto error: {}", msg),
            BondError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            BondError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            BondError::Other(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for BondError {}

impl From<BlockchainError> for BondError {
    fn from(err: BlockchainError) -> Self {
        match err {
            BlockchainError::InvalidTransaction(msg) => BondError::InvalidTransaction(msg),
            BlockchainError::InvalidBlock(msg) => BondError::InvalidBlock(msg),
            BlockchainError::CryptographicError(msg) => BondError::CryptoError(msg),
            BlockchainError::SerializationError(msg) => BondError::SerializationError(msg),
            BlockchainError::InsufficientFunds => BondError::InvalidTransaction("Insufficient funds".to_string()),
            _ => BondError::Other(err.to_string()),
        }
    }
}

/// Alias para Result com BondError
pub type BondResult<T> = std::result::Result<T, BondError>;
