//! Sistema de Transações do Aevum
//!
//! Implementa tipos de transação específicos do modelo de contas do Aevum,
//! incluindo transferências, staking, delegação e transações de governança.

use serde::{Deserialize, Serialize};
use shared::{BlockchainError, Hash256, Result};

/// Tipos de transação no Aevum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AevumTransactionType {
    /// Transferência simples entre contas
    Transfer {
        to: Hash256,
        amount: u128,
    },
    /// Stake de tokens para se tornar validador
    Stake {
        amount: u128,
    },
    /// Retirada de stake (unstake)
    Unstake {
        amount: u128,
    },
    /// Delegação de tokens para um validador
    Delegate {
        validator: Hash256,
        amount: u128,
    },
    /// Retirada de delegação
    Undelegate {
        validator: Hash256,
        amount: u128,
    },
    /// Voto em proposta de governança
    Vote {
        proposal_id: u64,
        vote: bool, // true = sim, false = não
        weight: u128, // peso do voto baseado no stake
    },
    /// Criação de proposta de governança
    CreateProposal {
        title: String,
        description: String,
        voting_period: u64, // duração em blocos
    },
    /// Reivindicação de recompensas
    ClaimRewards,
}

/// Transação do Aevum (Account Model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AevumTransaction {
    /// Endereço de origem
    pub from: Hash256,
    /// Nonce para prevenir replay attacks
    pub nonce: u64,
    /// Tipo e dados específicos da transação
    pub tx_type: AevumTransactionType,
    /// Limite de gás para execução
    pub gas_limit: u64,
    /// Preço do gás
    pub gas_price: u64,
    /// Assinatura da transação
    pub signature: Option<Vec<u8>>,
    /// Timestamp da transação
    pub timestamp: u64,
}

impl AevumTransaction {
    /// Cria uma nova transação de transferência
    pub fn transfer(
        from: Hash256,
        to: Hash256,
        amount: u128,
        nonce: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            from,
            nonce,
            tx_type: AevumTransactionType::Transfer { to, amount },
            gas_limit,
            gas_price,
            signature: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Cria uma nova transação de stake
    pub fn stake(
        from: Hash256,
        amount: u128,
        nonce: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            from,
            nonce,
            tx_type: AevumTransactionType::Stake { amount },
            gas_limit,
            gas_price,
            signature: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Cria uma nova transação de delegação
    pub fn delegate(
        from: Hash256,
        validator: Hash256,
        amount: u128,
        nonce: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            from,
            nonce,
            tx_type: AevumTransactionType::Delegate { validator, amount },
            gas_limit,
            gas_price,
            signature: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Cria uma nova transação de voto
    pub fn vote(
        from: Hash256,
        proposal_id: u64,
        vote: bool,
        weight: u128,
        nonce: u64,
        gas_limit: u64,
        gas_price: u64,
    ) -> Self {
        Self {
            from,
            nonce,
            tx_type: AevumTransactionType::Vote {
                proposal_id,
                vote,
                weight,
            },
            gas_limit,
            gas_price,
            signature: None,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    /// Calcula o hash da transação
    pub fn hash(&self) -> Result<Hash256> {
        let tx_data = serde_json::to_vec(self)
            .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
        Ok(Hash256::keccak256(&tx_data))
    }

    /// Calcula o custo total da transação (gas_limit * gas_price)
    pub fn total_cost(&self) -> u128 {
        self.gas_limit as u128 * self.gas_price as u128
    }

    /// Valida a estrutura básica da transação
    pub fn validate_basic(&self) -> Result<()> {
        // Verificar limites de gás
        if self.gas_limit == 0 {
            return Err(BlockchainError::InvalidTransaction(
                "Gas limit deve ser maior que zero".to_string(),
            ));
        }

        if self.gas_price == 0 {
            return Err(BlockchainError::InvalidTransaction(
                "Gas price deve ser maior que zero".to_string(),
            ));
        }

        // Validar campos específicos por tipo
        match &self.tx_type {
            AevumTransactionType::Transfer { amount, .. } => {
                if *amount == 0 {
                    return Err(BlockchainError::InvalidTransaction(
                        "Valor de transferência deve ser maior que zero".to_string(),
                    ));
                }
            }
            AevumTransactionType::Stake { amount } => {
                if *amount == 0 {
                    return Err(BlockchainError::InvalidTransaction(
                        "Valor de stake deve ser maior que zero".to_string(),
                    ));
                }
            }
            AevumTransactionType::Delegate { amount, .. } => {
                if *amount == 0 {
                    return Err(BlockchainError::InvalidTransaction(
                        "Valor de delegação deve ser maior que zero".to_string(),
                    ));
                }
            }
            AevumTransactionType::CreateProposal { title, description, voting_period } => {
                if title.is_empty() {
                    return Err(BlockchainError::InvalidTransaction(
                        "Título da proposta não pode estar vazio".to_string(),
                    ));
                }
                if description.is_empty() {
                    return Err(BlockchainError::InvalidTransaction(
                        "Descrição da proposta não pode estar vazia".to_string(),
                    ));
                }
                if *voting_period == 0 {
                    return Err(BlockchainError::InvalidTransaction(
                        "Período de votação deve ser maior que zero".to_string(),
                    ));
                }
            }
            _ => {} // Outros tipos não precisam validação adicional
        }

        Ok(())
    }

    /// Assina a transação (simulado por enquanto)
    pub fn sign(&mut self, _private_key: &[u8]) -> Result<()> {
        // Por enquanto, criar uma assinatura simulada
        // Em implementação completa, usaria ML-DSA-44 para Aevum
        let tx_hash = self.hash()?;
        let signature = format!("sim_sig_{}", hex::encode(tx_hash.as_bytes()))
            .into_bytes();
        
        self.signature = Some(signature);
        Ok(())
    }

    /// Verifica se a transação está assinada
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }
}

/// Informações sobre uma proposta de governança
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceProposal {
    /// ID único da proposta
    pub id: u64,
    /// Endereço do criador
    pub proposer: Hash256,
    /// Título da proposta
    pub title: String,
    /// Descrição detalhada
    pub description: String,
    /// Bloco de início da votação
    pub voting_start: u64,
    /// Bloco de fim da votação
    pub voting_end: u64,
    /// Votos favoráveis (peso total)
    pub yes_votes: u128,
    /// Votos contrários (peso total)
    pub no_votes: u128,
    /// Lista de votantes
    pub voters: Vec<Hash256>,
    /// Status da proposta
    pub status: ProposalStatus,
}

/// Status possíveis de uma proposta
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    /// Proposta ativa para votação
    Active,
    /// Proposta passou
    Passed,
    /// Proposta rejeitada
    Rejected,
    /// Proposta expirada
    Expired,
    /// Proposta executada
    Executed,
}

impl GovernanceProposal {
    /// Cria uma nova proposta
    pub fn new(
        id: u64,
        proposer: Hash256,
        title: String,
        description: String,
        voting_start: u64,
        voting_period: u64,
    ) -> Self {
        Self {
            id,
            proposer,
            title,
            description,
            voting_start,
            voting_end: voting_start + voting_period,
            yes_votes: 0,
            no_votes: 0,
            voters: Vec::new(),
            status: ProposalStatus::Active,
        }
    }

    /// Adiciona um voto à proposta
    pub fn add_vote(&mut self, voter: Hash256, vote: bool, weight: u128) -> Result<()> {
        // Verifica se o usuário já votou
        if self.voters.contains(&voter) {
            return Err(BlockchainError::InvalidTransaction(
                "Usuário já votou nesta proposta".to_string(),
            ));
        }

        // Adiciona o voto
        if vote {
            self.yes_votes += weight;
        } else {
            self.no_votes += weight;
        }

        self.voters.push(voter);
        Ok(())
    }

    /// Finaliza a proposta baseado nos votos
    pub fn finalize(&mut self, current_block: u64) -> ProposalStatus {
        if current_block < self.voting_end {
            return ProposalStatus::Active;
        }

        // Calcula resultado
        let total_votes = self.yes_votes + self.no_votes;
        if total_votes == 0 {
            self.status = ProposalStatus::Expired;
        } else if self.yes_votes > self.no_votes {
            self.status = ProposalStatus::Passed;
        } else {
            self.status = ProposalStatus::Rejected;
        }

        self.status.clone()
    }
}

/// Pool de transações pendentes (Mempool) do Aevum
#[derive(Debug, Clone)]
pub struct AevumMempool {
    /// Transações pendentes organizadas por nonce
    pub pending: std::collections::BTreeMap<Hash256, Vec<AevumTransaction>>,
    /// Limite máximo de transações no pool
    pub max_size: usize,
    /// Preço mínimo de gás aceito
    pub min_gas_price: u64,
}

impl AevumMempool {
    /// Cria um novo mempool
    pub fn new(max_size: usize, min_gas_price: u64) -> Self {
        Self {
            pending: std::collections::BTreeMap::new(),
            max_size,
            min_gas_price,
        }
    }

    /// Adiciona uma transação ao mempool
    pub fn add_transaction(&mut self, tx: AevumTransaction) -> Result<()> {
        // Validação básica
        tx.validate_basic()?;

        // Verifica assinatura
        if !tx.is_signed() {
            return Err(BlockchainError::InvalidTransaction(
                "Transação deve estar assinada".to_string(),
            ));
        }

        // Verifica preço mínimo de gás
        if tx.gas_price < self.min_gas_price {
            return Err(BlockchainError::InvalidTransaction(
                "Preço de gás abaixo do mínimo".to_string(),
            ));
        }

        // Verifica limite de tamanho
        let total_txs: usize = self.pending.values().map(|v| v.len()).sum();
        if total_txs >= self.max_size {
            return Err(BlockchainError::InvalidTransaction(
                "Mempool cheio".to_string(),
            ));
        }

        // Adiciona à lista do remetente
        let sender_txs = self.pending.entry(tx.from).or_insert_with(Vec::new);
        sender_txs.push(tx);
        
        // Ordena por nonce
        sender_txs.sort_by_key(|tx| tx.nonce);

        Ok(())
    }

    /// Remove uma transação do mempool
    pub fn remove_transaction(&mut self, from: Hash256, nonce: u64) -> Option<AevumTransaction> {
        if let Some(sender_txs) = self.pending.get_mut(&from) {
            if let Some(pos) = sender_txs.iter().position(|tx| tx.nonce == nonce) {
                return Some(sender_txs.remove(pos));
            }
        }
        None
    }

    /// Obtém as próximas transações executáveis para um endereço
    pub fn get_executable_transactions(&self, from: Hash256, current_nonce: u64) -> Vec<AevumTransaction> {
        if let Some(sender_txs) = self.pending.get(&from) {
            sender_txs
                .iter()
                .filter(|tx| tx.nonce == current_nonce)
                .cloned()
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Obtém estatísticas do mempool
    pub fn stats(&self) -> MempoolStats {
        let total_transactions: usize = self.pending.values().map(|v| v.len()).sum();
        let unique_senders = self.pending.len();
        
        MempoolStats {
            total_transactions,
            unique_senders,
            max_size: self.max_size,
        }
    }
}

/// Estatísticas do mempool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStats {
    pub total_transactions: usize,
    pub unique_senders: usize,
    pub max_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transfer_transaction_creation() {
        let from = Hash256::keccak256(b"from");
        let to = Hash256::keccak256(b"to");
        
        let tx = AevumTransaction::transfer(from, to, 1000, 1, 21000, 1000000);
        
        assert_eq!(tx.from, from);
        assert_eq!(tx.nonce, 1);
        assert_eq!(tx.gas_limit, 21000);
        assert!(!tx.is_signed());
    }

    #[test]
    fn test_transaction_validation() {
        let from = Hash256::keccak256(b"from");
        let to = Hash256::keccak256(b"to");
        
        // Transação válida
        let valid_tx = AevumTransaction::transfer(from, to, 1000, 1, 21000, 1000000);
        assert!(valid_tx.validate_basic().is_ok());
        
        // Transação com valor zero (inválida)
        let invalid_tx = AevumTransaction::transfer(from, to, 0, 1, 21000, 1000000);
        assert!(invalid_tx.validate_basic().is_err());
    }

    #[test]
    fn test_stake_transaction() {
        let from = Hash256::keccak256(b"staker");
        
        let tx = AevumTransaction::stake(from, 5000, 2, 50000, 2000000);
        
        match tx.tx_type {
            AevumTransactionType::Stake { amount } => {
                assert_eq!(amount, 5000);
            }
            _ => panic!("Tipo de transação incorreto"),
        }
    }

    #[test]
    fn test_governance_proposal() {
        let proposer = Hash256::keccak256(b"proposer");
        let voter = Hash256::keccak256(b"voter");
        
        let mut proposal = GovernanceProposal::new(
            1,
            proposer,
            "Proposta de Teste".to_string(),
            "Descrição da proposta".to_string(),
            100,
            50,
        );
        
        assert_eq!(proposal.id, 1);
        assert_eq!(proposal.voting_end, 150);
        
        // Adicionar voto
        proposal.add_vote(voter, true, 1000).unwrap();
        assert_eq!(proposal.yes_votes, 1000);
        assert_eq!(proposal.voters.len(), 1);
        
        // Tentar votar novamente (deve falhar)
        assert!(proposal.add_vote(voter, false, 500).is_err());
    }

    #[test]
    fn test_mempool_operations() {
        let mut mempool = AevumMempool::new(100, 1000000);
        let from = Hash256::keccak256(b"sender");
        let to = Hash256::keccak256(b"receiver");
        
        let mut tx = AevumTransaction::transfer(from, to, 1000, 1, 21000, 1000000);
        tx.sign(b"fake_key").unwrap();
        
        // Adicionar transação
        assert!(mempool.add_transaction(tx.clone()).is_ok());
        
        // Verificar estatísticas
        let stats = mempool.stats();
        assert_eq!(stats.total_transactions, 1);
        assert_eq!(stats.unique_senders, 1);
        
        // Remover transação
        let removed = mempool.remove_transaction(from, 1);
        assert!(removed.is_some());
        
        // Verificar que foi removida
        let stats = mempool.stats();
        assert_eq!(stats.total_transactions, 0);
    }

    #[test]
    fn test_transaction_cost() {
        let from = Hash256::keccak256(b"from");
        let to = Hash256::keccak256(b"to");
        
        let tx = AevumTransaction::transfer(from, to, 1000, 1, 21000, 1000000);
        assert_eq!(tx.total_cost(), 21_000_000_000u128);
    }
}
