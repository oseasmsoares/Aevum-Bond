//! Fundacao do Aevum - Blockchain com Modelo de Contas e DPoS
//! 
//! Este modulo implementa a estrutura basica do protocolo Aevum,
//! que sera a segunda blockchain do ecosistema Aevum & Bond.
//! Planejado para implementacao completa no Sprint 6.

use serde::{Deserialize, Serialize};
use shared::{BlockchainError, Hash256, Result};
use std::collections::HashMap;

/// Estado de uma conta no Aevum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    /// Numero sequencial para prevenir replay attacks
    pub nonce: u64,
    /// Saldo da conta (em wei/aevum minimo)
    pub balance: u128,
    /// Hash do codigo (para contratos futuros)
    pub code_hash: Option<Hash256>,
    /// Root do storage trie (para contratos futuros)
    pub storage_root: Option<Hash256>,
}

impl AccountState {
    /// Cria uma nova conta com saldo inicial
    pub fn new(balance: u128) -> Self {
        Self {
            nonce: 0,
            balance,
            code_hash: None,
            storage_root: None,
        }
    }

    /// Verifica se a conta tem saldo suficiente
    pub fn has_sufficient_balance(&self, amount: u128) -> bool {
        self.balance >= amount
    }

    /// Transfere valor para outra conta
    pub fn transfer(&mut self, amount: u128) -> Result<()> {
        if !self.has_sufficient_balance(amount) {
            return Err(BlockchainError::InsufficientFunds);
        }
        self.balance -= amount;
        self.nonce += 1;
        Ok(())
    }

    /// Recebe valor de transferencia
    pub fn receive(&mut self, amount: u128) {
        self.balance += amount;
    }
}

/// Informacoes de um validador DPoS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Chave publica do validador
    pub public_key: Hash256,
    /// Total de tokens em stake
    pub stake_amount: u128,
    /// Numero de delegadores
    pub delegator_count: u32,
    /// Status ativo/inativo
    pub is_active: bool,
    /// Epoca de ativacao
    pub activation_epoch: u64,
}

impl ValidatorInfo {
    /// Cria informacoes de um novo validador
    pub fn new(public_key: Hash256, stake_amount: u128) -> Self {
        Self {
            public_key,
            stake_amount,
            delegator_count: 0,
            is_active: false,
            activation_epoch: 0,
        }
    }

    /// Adiciona stake ao validador
    pub fn add_stake(&mut self, amount: u128) {
        self.stake_amount += amount;
    }

    /// Remove stake do validador
    pub fn remove_stake(&mut self, amount: u128) -> Result<()> {
        if self.stake_amount < amount {
            return Err(BlockchainError::InsufficientFunds);
        }
        self.stake_amount -= amount;
        Ok(())
    }
}

/// Estado global do Aevum (WorldState)
#[derive(Debug, Clone)]
pub struct AevumState {
    /// Mapeamento de endereco para estado da conta
    pub accounts: HashMap<Hash256, AccountState>,
    /// Mapeamento de validadores ativos
    pub validators: HashMap<Hash256, ValidatorInfo>,
    /// Numero da epoca atual
    pub current_epoch: u64,
    /// Altura do bloco atual
    pub block_height: u64,
}

impl AevumState {
    /// Cria um novo estado inicial
    pub fn new() -> Self {
        Self {
            accounts: HashMap::new(),
            validators: HashMap::new(),
            current_epoch: 0,
            block_height: 0,
        }
    }

    /// Obtem o estado de uma conta
    pub fn get_account(&self, address: &Hash256) -> Option<&AccountState> {
        self.accounts.get(address)
    }

    /// Obtem o estado mutavel de uma conta
    pub fn get_account_mut(&mut self, address: &Hash256) -> Option<&mut AccountState> {
        self.accounts.get_mut(address)
    }

    /// Cria uma nova conta
    pub fn create_account(&mut self, address: Hash256, initial_balance: u128) {
        self.accounts.insert(address, AccountState::new(initial_balance));
    }

    /// Executa uma transferencia entre contas
    pub fn transfer(&mut self, from: Hash256, to: Hash256, amount: u128) -> Result<()> {
        // Verificar se a conta de origem existe e tem saldo
        let from_account = self.accounts.get_mut(&from)
            .ok_or(BlockchainError::InvalidTransaction("Conta de origem nao encontrada".to_string()))?;
        
        from_account.transfer(amount)?;

        // Criar conta de destino se nao existir
        if !self.accounts.contains_key(&to) {
            self.create_account(to, 0);
        }

        // Receber na conta de destino
        let to_account = self.accounts.get_mut(&to).unwrap();
        to_account.receive(amount);

        Ok(())
    }

    /// Registra um novo validador
    pub fn register_validator(&mut self, validator_key: Hash256, stake_amount: u128) -> Result<()> {
        if self.validators.contains_key(&validator_key) {
            return Err(BlockchainError::InvalidTransaction("Validador ja registrado".to_string()));
        }

        let validator = ValidatorInfo::new(validator_key, stake_amount);
        self.validators.insert(validator_key, validator);
        Ok(())
    }

    /// Obtem lista de validadores ativos
    pub fn get_active_validators(&self) -> Vec<&ValidatorInfo> {
        self.validators.values()
            .filter(|v| v.is_active)
            .collect()
    }

    /// Avança para a próxima epoca
    pub fn advance_epoch(&mut self) {
        self.current_epoch += 1;
        // TODO: Implementar eleicao de validadores
    }
}

impl Default for AevumState {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuração do consenso DPoS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DposConfig {
    /// Numero maximo de validadores ativos
    pub max_validators: u32,
    /// Stake minimo para ser validador
    pub min_validator_stake: u128,
    /// Duracao da epoca em blocos
    pub epoch_length: u64,
    /// Tempo de bloqueio do unstake (em epocas)
    pub unstake_delay: u64,
}

impl Default for DposConfig {
    fn default() -> Self {
        Self {
            max_validators: 21,        // Inspirado em EOS
            min_validator_stake: 1000, // 1000 tokens minimos
            epoch_length: 2160,        // ~6 horas com 10s/bloco
            unstake_delay: 7,          // 7 epocas (~2 dias)
        }
    }
}

/// Funcoes de utilidade para o Aevum
pub mod utils {
    use super::*;

    /// Calcula o poder de voto de um validador baseado no stake
    pub fn calculate_voting_power(stake_amount: u128, total_stake: u128) -> f64 {
        if total_stake == 0 {
            0.0
        } else {
            stake_amount as f64 / total_stake as f64
        }
    }

    /// Verifica se um endereco e valido (formato basico)
    pub fn is_valid_address(address: &Hash256) -> bool {
        // Implementacao basica - todos os hashes sao validos por enquanto
        !address.as_bytes().iter().all(|&b| b == 0)
    }

    /// Gera endereco a partir de chave publica
    pub fn address_from_public_key(public_key: &Hash256) -> Hash256 {
        // Implementacao simplificada - usar os ultimos 20 bytes do hash
        Hash256::keccak256(public_key.as_bytes())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_account_creation() {
        let account = AccountState::new(1000);
        assert_eq!(account.balance, 1000);
        assert_eq!(account.nonce, 0);
        assert!(account.has_sufficient_balance(500));
        assert!(!account.has_sufficient_balance(1500));
    }

    #[test]
    fn test_account_transfer() {
        let mut account = AccountState::new(1000);
        
        // Transferencia valida
        assert!(account.transfer(300).is_ok());
        assert_eq!(account.balance, 700);
        assert_eq!(account.nonce, 1);

        // Transferencia invalida (saldo insuficiente)
        assert!(account.transfer(800).is_err());
    }

    #[test]
    fn test_validator_creation() {
        let pub_key = Hash256::zero();
        let validator = ValidatorInfo::new(pub_key, 5000);
        
        assert_eq!(validator.stake_amount, 5000);
        assert_eq!(validator.delegator_count, 0);
        assert!(!validator.is_active);
    }

    #[test]
    fn test_aevum_state() {
        let mut state = AevumState::new();
        let addr1 = Hash256::keccak256(b"addr1");
        let addr2 = Hash256::keccak256(b"addr2");

        // Criar conta inicial
        state.create_account(addr1, 1000);
        assert_eq!(state.get_account(&addr1).unwrap().balance, 1000);

        // Transferencia para conta inexistente (deve criar automaticamente)
        assert!(state.transfer(addr1, addr2, 300).is_ok());
        assert_eq!(state.get_account(&addr1).unwrap().balance, 700);
        assert_eq!(state.get_account(&addr2).unwrap().balance, 300);
    }

    #[test]
    fn test_dpos_config() {
        let config = DposConfig::default();
        assert_eq!(config.max_validators, 21);
        assert_eq!(config.min_validator_stake, 1000);
        assert_eq!(config.epoch_length, 2160);
    }

    #[test]
    fn test_voting_power() {
        let power = utils::calculate_voting_power(5000, 20000);
        assert_eq!(power, 0.25); // 25% do stake total
    }
}
