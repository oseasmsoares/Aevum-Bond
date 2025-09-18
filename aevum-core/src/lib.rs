//! # Aevum Core - Sistema de Blockchain com Modelo de Contas
//! 
//! Este crate implementa o protocolo Aevum, uma blockchain baseada em contas
//! com consenso Delegated Proof of Stake (DPoS) e criptografia pós-quântica.
//! 
//! ## Visão Geral
//! 
//! O Aevum é a segunda blockchain do ecossistema Aevum & Bond, projetada para:
//! - **Alto throughput** com consenso DPoS
//! - **Modelo de contas** similar ao Ethereum
//! - **Transações especiais** para governança e staking
//! - **Criptografia pós-quântica** ML-DSA-44 (Nível 1)
//! - **Smart contracts** (planejado para versões futuras)
//! 
//! ## Arquitetura
//! 
//! ### Consenso DPoS
//! O Aevum utiliza Delegated Proof of Stake onde:
//! - Validadores são eleitos baseado em stake total
//! - Slots de produção são distribuídos em round-robin
//! - Performance ruim resulta em slashing automático
//! - Recompensas proporcionais à taxa de aprovação
//! 
//! ### Modelo de Contas
//! Cada conta possui:
//! - `nonce`: contador para prevenir replay attacks
//! - `balance`: saldo em tokens AEV
//! - `code_hash`: hash do código (para contratos futuros)
//! - `storage_root`: root da árvore de storage (para contratos futuros)
//! 
//! ### Transações Especiais
//! - **Transfer**: transferência básica entre contas
//! - **Stake**: stake de tokens para validação
//! - **Delegate**: delegação para validadores
//! - **Vote**: participação em governança
//! - **Governança**: criação e votação de propostas
//! 
//! ## Exemplo de Uso
//! 
//! ```rust
//! use aevum_core::{AevumState, DposEngine, DposConfig};
//! use aevum_core::transaction::{AevumTransaction, AevumTransactionType};
//! use shared::Hash256;
//! 
//! // Criar estado inicial
//! let mut state = AevumState::new();
//! 
//! // Configurar consenso DPoS
//! let config = DposConfig::default();
//! let mut consensus = DposEngine::new(config);
//! 
//! // Criar conta
//! let user_addr = Hash256::keccak256(b"user1");
//! state.create_account(user_addr, 10000); // 10k AEV inicial
//! 
//! // Registrar como validador primeiro
//! state.register_validator(user_addr, 5000).unwrap();
//! 
//! // Fazer stake adicional
//! let stake_tx = AevumTransaction::stake(user_addr, 5000, 1, 50000, 2000000);
//! 
//! // Eleger validadores (agora deve funcionar)
//! let validators = consensus.elect_validators(&state).unwrap();
//! assert!(!validators.is_empty());
//! ```
//! 
//! ## Módulos
//! 
//! - [`placeholder`] - Estruturas básicas (AccountState, AevumState, ValidatorInfo)
//! - [`consensus`] - Motor de consenso DPoS completo
//! - [`transaction`] - Sistema de transações e mempool
//! 
//! ## Especificações Técnicas
//! 
//! - **Block time**: 3 segundos
//! - **TPS estimado**: ~3000 transações por segundo
//! - **Validadores máximos**: 21 (configurável)
//! - **Stake mínimo**: 1000 AEV (configurável)
//! - **Epoch length**: 2160 blocos (~1.8 horas)
//! - **Unstake delay**: 7 epochs (~12.6 horas)

pub mod placeholder;
pub mod consensus;
pub mod transaction;

// Re-exports principais para facilitar o uso
pub use placeholder::{
    AccountState, AevumState, ValidatorInfo, DposConfig,
};
pub use consensus::{
    DposEngine, ValidatorPerformance, AevumNetworkParams, EpochStats,
};
pub use transaction::{
    AevumTransaction, AevumTransactionType, AevumMempool,
    GovernanceProposal, ProposalStatus,
};

/// Versão do protocolo Aevum
pub const AEVUM_VERSION: &str = "0.1.0";

/// ID da rede Aevum (para separar de outras redes)
pub const AEVUM_NETWORK_ID: u32 = 2; // Bond = 1, Aevum = 2

/// Constantes do protocolo
pub mod constants {
    /// Número mágico para identificar blocos Aevum
    pub const MAGIC_BYTES: [u8; 4] = [0x41, 0x45, 0x56, 0x4D]; // "AEVM"
    
    /// Tamanho máximo de um bloco (1MB)
    pub const MAX_BLOCK_SIZE: usize = 1_000_000;
    
    /// Número máximo de transações por bloco
    pub const MAX_TRANSACTIONS_PER_BLOCK: usize = 1000;
    
    /// Limite de gás padrão por transação
    pub const DEFAULT_GAS_LIMIT: u64 = 21_000;
    
    /// Preço mínimo de gás (1 Gwei)
    pub const MIN_GAS_PRICE: u64 = 1_000_000_000;
    
    /// Recompensa base por bloco (1 AEV = 10^18 wei)
    pub const BASE_BLOCK_REWARD: u128 = 1_000_000_000_000_000_000;
    
    /// Tempo de vida padrão de uma proposta (1 semana em blocos)
    pub const DEFAULT_PROPOSAL_LIFETIME: u64 = 201_600; // ~7 dias
    
    /// Chain ID do Aevum
    pub const AEVUM_CHAIN_ID: u32 = 1001;
    
    /// Block time em segundos
    pub const AEVUM_BLOCK_TIME: u64 = 3;
}

/// Utilitários para o Aevum
pub mod utils {
    use shared::Hash256;
    
    /// Converte um endereço público para formato de exibição
    pub fn format_address(addr: &Hash256) -> String {
        let hex = hex::encode(addr.as_bytes());
        format!("0x{}", &hex[0..40]) // Primeiros 20 bytes como hex
    }
    
    /// Converte wei (menor unidade) para AEV
    pub fn wei_to_aev(wei: u128) -> f64 {
        wei as f64 / 1_000_000_000_000_000_000.0
    }
    
    /// Converte AEV para wei
    pub fn aev_to_wei(aev: f64) -> u128 {
        (aev * 1_000_000_000_000_000_000.0) as u128
    }
    
    /// Valida se um endereço tem formato correto
    pub fn is_valid_address(addr: &Hash256) -> bool {
        // Por enquanto, qualquer hash não-zero é válido
        !addr.as_bytes().iter().all(|&b| b == 0)
    }
    
    /// Calcula a taxa de transação baseada em complexidade
    pub fn calculate_transaction_fee(gas_used: u64, gas_price: u64) -> u128 {
        gas_used as u128 * gas_price as u128
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::utils::*;
    
    #[test]
    fn test_version() {
        assert_eq!(AEVUM_VERSION, "0.1.0");
        assert_eq!(AEVUM_NETWORK_ID, 2);
    }
    
    #[test]
    fn test_constants() {
        assert_eq!(constants::MAGIC_BYTES, [0x41, 0x45, 0x56, 0x4D]);
        assert_eq!(constants::MAX_BLOCK_SIZE, 1_000_000);
        assert_eq!(constants::BASE_BLOCK_REWARD, 1_000_000_000_000_000_000);
        assert_eq!(constants::AEVUM_CHAIN_ID, 1001);
        assert_eq!(constants::AEVUM_BLOCK_TIME, 3);
    }
    
    #[test]
    fn test_address_formatting() {
        let addr = shared::Hash256::keccak256(b"test_address");
        let formatted = format_address(&addr);
        assert!(formatted.starts_with("0x"));
        assert_eq!(formatted.len(), 42); // 0x + 40 chars
    }
    
    #[test]
    fn test_currency_conversion() {
        assert_eq!(aev_to_wei(1.0), 1_000_000_000_000_000_000);
        assert_eq!(wei_to_aev(1_000_000_000_000_000_000), 1.0);
        
        assert_eq!(aev_to_wei(0.5), 500_000_000_000_000_000);
        assert_eq!(wei_to_aev(500_000_000_000_000_000), 0.5);
    }
    
    #[test]
    fn test_address_validation() {
        let valid_addr = shared::Hash256::keccak256(b"valid");
        assert!(is_valid_address(&valid_addr));
        
        let zero_addr = shared::Hash256::zero();
        assert!(!is_valid_address(&zero_addr));
    }
    
    #[test]
    fn test_transaction_fee_calculation() {
        let fee = calculate_transaction_fee(21000, 1_000_000_000);
        assert_eq!(fee, 21_000_000_000_000);
    }
    
    #[test]
    fn test_aevum_constants() {
        // Testa se as constantes estão dentro de limites razoáveis
        assert!(constants::MAX_TRANSACTIONS_PER_BLOCK >= 100);
        assert!(constants::DEFAULT_GAS_LIMIT >= 21_000);
        assert!(constants::MIN_GAS_PRICE > 0);
    }
    
    #[test]
    fn test_bridge_transaction() {
        // Exemplo de transação que seria usada na ponte inter-ledger
        let from = shared::Hash256::keccak256(b"bridge_user");
        let amount = aev_to_wei(10.0); // 10 AEV
        
        // Simular criação de wBND no Aevum
        let mut state = AevumState::new();
        state.create_account(from, amount);
        
        assert_eq!(state.get_account(&from).unwrap().balance, amount);
        assert_eq!(wei_to_aev(amount), 10.0);
    }
}
