use thiserror::Error;

/// Erros relacionados à blockchain
#[derive(Error, Debug)]
pub enum BlockchainError {
    #[error("Transação inválida: {0}")]
    InvalidTransaction(String),

    #[error("Bloco inválido: {0}")]
    InvalidBlock(String),

    #[error("Hash inválido")]
    InvalidHash,

    #[error("Dificuldade insuficiente")]
    InsufficientDifficulty,

    #[error("Nonce não encontrado")]
    NonceNotFound,

    #[error("UTXO não encontrado")]
    UtxoNotFound,

    #[error("Fundos insuficientes")]
    InsufficientFunds,

    #[error("Assinatura inválida")]
    InvalidSignature,

    #[error("Tamanho de chave inválido: esperado {expected}, recebido {actual}")]
    InvalidKeySize { expected: usize, actual: usize },

    #[error("Erro criptográfico: {0}")]
    CryptographicError(String),

    #[error("Erro de serialização: {0}")]
    SerializationError(String),

    #[error("Erro de I/O: {0}")]
    IoError(String),

    #[error("Erro de rede: {0}")]
    NetworkError(String),
}
