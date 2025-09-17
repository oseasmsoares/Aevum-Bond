pub mod crypto;
pub mod error;
pub mod hash;
pub mod types;

pub use crypto::{
    public_key_from_bytes, sign_transaction_hash, signature_from_bytes,
    verify_transaction_signature, KeyPair, PrivateKey, PublicKey, Signature, SignatureAlgorithm,
};
pub use error::BlockchainError;
pub use hash::Hash256;
pub use types::{
    Amount, BlockHeight, BlockId, BlockchainStats, InputIndex, NetworkType, NodeConfig, OutPoint,
    OutputIndex, PeerInfo, PublicKeyHex, SignatureHex, Timestamp, TxId,
};

pub type Result<T> = std::result::Result<T, BlockchainError>;
