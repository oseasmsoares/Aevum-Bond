//! Tipos compartilhados entre módulos do Aevum & Bond

use serde::{Deserialize, Serialize};

/// Alias para identificador único de transação
pub type TxId = crate::Hash256;

/// Alias para identificador único de bloco
pub type BlockId = crate::Hash256;

/// Alias para chave pública em formato string hexadecimal
pub type PublicKeyHex = String;

/// Alias para assinatura digital em formato string hexadecimal
pub type SignatureHex = String;

/// Alias para valor monetário (satoshis)
pub type Amount = u64;

/// Alias para timestamp Unix
pub type Timestamp = i64;

/// Alias para altura do bloco na blockchain
pub type BlockHeight = u64;

/// Alias para índice de saída de transação
pub type OutputIndex = u32;

/// Alias para índice de entrada de transação
pub type InputIndex = u32;

/// Referência a uma saída de transação (UTXO)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutPoint {
    /// Hash da transação que contém a saída
    pub txid: TxId,
    /// Índice da saída na transação
    pub vout: OutputIndex,
}

impl OutPoint {
    /// Creates a new transaction output reference
    #[must_use]
    pub const fn new(txid: TxId, vout: OutputIndex) -> Self {
        Self { txid, vout }
    }
}

/// Network configurations for different environments
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkType {
    /// Main production network
    Mainnet,
    /// Test network
    Testnet,
    /// Local development network
    #[default]
    Devnet,
    /// Regression network for testing
    Regtest,
}

/// Configurações globais do nó
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Tipo de rede
    pub network: NetworkType,
    /// Porta P2P para comunicação
    pub p2p_port: u16,
    /// Porta RPC para API
    pub rpc_port: u16,
    /// Endereço de bind
    pub bind_address: String,
    /// Peers para conexão inicial
    pub bootstrap_peers: Vec<String>,
    /// Diretório de dados do nó
    pub data_dir: String,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            network: NetworkType::Devnet,
            p2p_port: 8333,
            rpc_port: 8332,
            bind_address: "127.0.0.1".to_string(),
            bootstrap_peers: vec![],
            data_dir: "./data".to_string(),
        }
    }
}

/// Estatísticas da blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    /// Altura atual da blockchain
    pub height: BlockHeight,
    /// Hash do último bloco
    pub tip_hash: BlockId,
    /// Número total de transações
    pub total_transactions: u64,
    /// Número total de UTXOs
    pub total_utxos: u64,
    /// Dificuldade atual
    pub current_difficulty: u32,
    /// Timestamp do último bloco
    pub last_block_time: Timestamp,
    /// Taxa de hash estimada da rede
    pub estimated_hash_rate: f64,
}

impl Default for BlockchainStats {
    fn default() -> Self {
        Self {
            height: 0,
            tip_hash: crate::Hash256::zero(),
            total_transactions: 0,
            total_utxos: 0,
            current_difficulty: 1,
            last_block_time: 0,
            estimated_hash_rate: 0.0,
        }
    }
}

/// Informações de peer da rede P2P
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// ID único do peer
    pub peer_id: String,
    /// Endereço do peer
    pub address: String,
    /// Altura da blockchain do peer
    pub height: BlockHeight,
    /// Versão do protocolo
    pub protocol_version: u32,
    /// Timestamp da última comunicação
    pub last_seen: Timestamp,
    /// Status da conexão
    pub connected: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_outpoint_creation() {
        let txid = crate::Hash256::zero();
        let vout = 0;
        let outpoint = OutPoint::new(txid, vout);

        assert_eq!(outpoint.txid, txid);
        assert_eq!(outpoint.vout, vout);
    }

    #[test]
    fn test_node_config_defaults() {
        let config = NodeConfig::default();

        assert_eq!(config.network, NetworkType::Devnet);
        assert_eq!(config.p2p_port, 8333);
        assert_eq!(config.rpc_port, 8332);
        assert_eq!(config.bind_address, "127.0.0.1");
    }

    #[test]
    fn test_blockchain_stats_default() {
        let stats = BlockchainStats::default();

        assert_eq!(stats.height, 0);
        assert_eq!(stats.total_transactions, 0);
        assert_eq!(stats.total_utxos, 0);
    }
}
