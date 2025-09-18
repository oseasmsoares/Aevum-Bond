pub mod block;
pub mod blockchain;
pub mod error;
pub mod mining;
pub mod script;
pub mod transaction;
pub mod utxo;

// Re-exports principais
pub use block::{Block, BlockHeader};
pub use blockchain::{Blockchain, BlockchainStats, NetworkParams};
pub use mining::{DifficultyAdjuster, Miner, MinerConfig, MiningResult};
pub use transaction::{Transaction, TxInput, TxOutput};
pub use utxo::{OutPoint, Utxo, UtxoSet};

// Re-exports de tipos compartilhados
pub use shared::{BlockchainError, Hash256, Result};
