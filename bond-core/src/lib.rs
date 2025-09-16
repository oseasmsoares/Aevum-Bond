pub mod utxo;
pub mod transaction;
pub mod block;
pub mod mining;
pub mod blockchain;

// Re-exports principais
pub use blockchain::{Blockchain, NetworkParams, BlockchainStats};
pub use block::{Block, BlockHeader};
pub use transaction::{Transaction, TxInput, TxOutput};
pub use utxo::{Utxo, UtxoSet, OutPoint};
pub use mining::{Miner, MinerConfig, MiningResult, DifficultyAdjuster};

// Re-exports de tipos compartilhados
pub use shared::{Hash256, BlockchainError, Result};
