//! Sistema de Consenso DPoS do Aevum
//!
//! Implementa o algoritmo Delegated Proof of Stake (DPoS) conforme
//! as especificações do Sprint 6. Este módulo gerencia:
//! - Eleição de validadores baseada em peso de stake
//! - Rotação de produtores de bloco
//! - Mecanismo de slashing por mau comportamento
//! - Sistema de epochs e recompensas

use crate::placeholder::{AevumState, DposConfig};
use serde::{Deserialize, Serialize};
use shared::{BlockchainError, Hash256, Result};
use std::collections::HashMap;

/// Informações sobre um slot de produção de bloco
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockSlot {
    /// Slot number (posição temporal)
    pub slot_number: u64,
    /// Validador designado para este slot
    pub validator: Hash256,
    /// Timestamp esperado para o bloco
    pub expected_time: u64,
    /// Se o bloco foi produzido
    pub block_produced: bool,
}

/// Estatísticas de performance de um validador
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorPerformance {
    /// Número total de slots atribuídos
    pub slots_assigned: u64,
    /// Número de blocos produzidos
    pub blocks_produced: u64,
    /// Número de blocos perdidos (missed)
    pub missed_blocks: u64,
    /// Taxa de aprovação (0.0 a 1.0)
    pub approval_rate: f64,
    /// Época da última atividade
    pub last_active_epoch: u64,
}

impl ValidatorPerformance {
    /// Cria nova performance zerada
    pub const fn new() -> Self {
        Self {
            slots_assigned: 0,
            blocks_produced: 0,
            missed_blocks: 0,
            approval_rate: 1.0,
            last_active_epoch: 0,
        }
    }

    /// Registra um bloco produzido
    pub fn record_block_produced(&mut self, epoch: u64) {
        self.blocks_produced += 1;
        self.last_active_epoch = epoch;
        self.update_approval_rate();
    }

    /// Registra um bloco perdido
    pub fn record_missed_block(&mut self, epoch: u64) {
        self.missed_blocks += 1;
        self.last_active_epoch = epoch;
        self.update_approval_rate();
    }

    /// Atualiza a taxa de aprovação
    fn update_approval_rate(&mut self) {
        let total_blocks = self.blocks_produced + self.missed_blocks;
        if total_blocks > 0 {
            self.approval_rate = self.blocks_produced as f64 / total_blocks as f64;
        }
    }

    /// Verifica se o validador deve ser penalizado
    pub const fn should_be_slashed(&self) -> bool {
        // Slash se taxa de aprovação menor que 50% e pelo menos 10 slots atribuídos
        self.approval_rate < 0.5 && self.slots_assigned >= 10
    }
}

impl Default for ValidatorPerformance {
    fn default() -> Self {
        Self::new()
    }
}

/// Motor de consenso DPoS
#[derive(Debug, Clone)]
pub struct DposEngine {
    /// Configuração do DPoS
    pub config: DposConfig,
    /// Performance dos validadores
    pub validator_performance: HashMap<Hash256, ValidatorPerformance>,
    /// Cronograma de slots para a época atual
    pub current_schedule: Vec<BlockSlot>,
    /// Próximo slot a ser produzido
    pub next_slot: u64,
    /// Timestamp de início da época atual
    pub epoch_start_time: u64,
}

impl DposEngine {
    /// Cria um novo motor de consenso
    pub fn new(config: DposConfig) -> Self {
        Self {
            config,
            validator_performance: HashMap::new(),
            current_schedule: Vec::new(),
            next_slot: 0,
            epoch_start_time: 0,
        }
    }

    /// Elege validadores para a próxima época baseado no stake
    pub fn elect_validators(&mut self, state: &AevumState) -> Result<Vec<Hash256>> {
        // Coleta todos os validadores e seus stakes
        let mut candidates: Vec<(Hash256, u128)> = Vec::new();
        
        for (validator_key, validator_info) in &state.validators {
            // Só considera validadores com stake mínimo
            if validator_info.stake_amount >= self.config.min_validator_stake {
                candidates.push((*validator_key, validator_info.stake_amount));
            }
        }

        // Ordena por stake (maior primeiro)
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        // Seleciona os top N validadores
        let max_validators = self.config.max_validators as usize;
        let elected: Vec<Hash256> = candidates
            .into_iter()
            .take(max_validators)
            .map(|(key, _)| key)
            .collect();

        if elected.is_empty() {
            return Err(BlockchainError::InvalidBlock(
                "Nenhum validador elegível encontrado".to_string()
            ));
        }

        Ok(elected)
    }

    /// Gera cronograma de produção de blocos para uma época
    pub fn generate_schedule(&mut self, validators: &[Hash256], epoch_start: u64) -> Result<()> {
        if validators.is_empty() {
            return Err(BlockchainError::InvalidBlock(
                "Lista de validadores vazia".to_string()
            ));
        }

        self.current_schedule.clear();
        self.epoch_start_time = epoch_start;
        self.next_slot = 0;

        // Cada slot representa um período de 3 segundos (20 blocos/min)
        const SLOT_TIME: u64 = 3; // seconds
        let total_slots = self.config.epoch_length;

        // Distribui slots entre validadores usando round-robin
        for slot in 0..total_slots {
            let validator_index = (slot % validators.len() as u64) as usize;
            let expected_time = epoch_start + (slot * SLOT_TIME);
            
            let block_slot = BlockSlot {
                slot_number: slot,
                validator: validators[validator_index],
                expected_time,
                block_produced: false,
            };
            
            self.current_schedule.push(block_slot);
        }

        Ok(())
    }

    /// Obtém o validador responsável pelo próximo bloco
    pub fn get_current_producer(&self) -> Option<Hash256> {
        if self.next_slot >= self.current_schedule.len() as u64 {
            return None;
        }

        Some(self.current_schedule[self.next_slot as usize].validator)
    }

    /// Registra que um bloco foi produzido
    pub fn record_block_produced(&mut self, validator: Hash256, epoch: u64) -> Result<()> {
        // Atualiza performance do validador
        let performance = self.validator_performance
            .entry(validator)
            .or_insert_with(ValidatorPerformance::new);
        
        performance.record_block_produced(epoch);

        // Marca slot como produzido
        if (self.next_slot as usize) < self.current_schedule.len() {
            self.current_schedule[self.next_slot as usize].block_produced = true;
        }

        // Avança para próximo slot
        self.next_slot += 1;

        Ok(())
    }

    /// Registra que um bloco foi perdido
    pub fn record_missed_block(&mut self, validator: Hash256, epoch: u64) -> Result<()> {
        let performance = self.validator_performance
            .entry(validator)
            .or_insert_with(ValidatorPerformance::new);
        
        performance.record_missed_block(epoch);

        // Avança para próximo slot mesmo com bloco perdido
        self.next_slot += 1;

        Ok(())
    }

    /// Calcula recompensas para validadores baseado em performance
    pub fn calculate_rewards(&self, total_reward: u128) -> HashMap<Hash256, u128> {
        let mut rewards = HashMap::new();
        let mut total_performance_score = 0.0;

        // Calcula score total de performance
        for performance in self.validator_performance.values() {
            total_performance_score += performance.approval_rate;
        }

        if total_performance_score == 0.0 {
            return rewards;
        }

        // Distribui recompensas proporcionalmente
        for (validator, performance) in &self.validator_performance {
            let performance_ratio = performance.approval_rate / total_performance_score;
            let reward = (total_reward as f64 * performance_ratio) as u128;
            
            if reward > 0 {
                rewards.insert(*validator, reward);
            }
        }

        rewards
    }

    /// Identifica validadores que devem ser penalizados
    pub fn identify_slashable_validators(&self) -> Vec<Hash256> {
        self.validator_performance
            .iter()
            .filter(|(_, performance)| performance.should_be_slashed())
            .map(|(validator, _)| *validator)
            .collect()
    }

    /// Aplica slashing em validadores com má performance
    pub fn apply_slashing(&mut self, state: &mut AevumState, validators_to_slash: &[Hash256]) -> Result<u128> {
        let mut total_slashed = 0u128;
        const SLASH_PERCENTAGE: u128 = 10; // 10% do stake

        for validator_key in validators_to_slash {
            if let Some(validator) = state.validators.get_mut(validator_key) {
                let slash_amount = validator.stake_amount * SLASH_PERCENTAGE / 100;
                
                if let Ok(()) = validator.remove_stake(slash_amount) {
                    total_slashed += slash_amount;
                    
                    // Desativa validador se stake ficar abaixo do mínimo
                    if validator.stake_amount < self.config.min_validator_stake {
                        validator.is_active = false;
                    }
                }
            }

            // Remove performance histórica do validador slashado
            self.validator_performance.remove(validator_key);
        }

        Ok(total_slashed)
    }

    /// Verifica se é hora de avançar época
    pub fn should_advance_epoch(&self, current_time: u64) -> bool {
        let epoch_duration = self.config.epoch_length * 3; // 3 segundos por slot
        current_time >= self.epoch_start_time + epoch_duration
    }

    /// Obtém estatísticas da época atual
    pub fn get_epoch_stats(&self) -> EpochStats {
        let total_slots = self.current_schedule.len() as u64;
        let produced_blocks = self.current_schedule
            .iter()
            .filter(|slot| slot.block_produced)
            .count() as u64;
        
        let missed_blocks = total_slots.saturating_sub(produced_blocks);
        let participation_rate = if total_slots > 0 {
            produced_blocks as f64 / total_slots as f64
        } else {
            0.0
        };

        EpochStats {
            total_slots,
            produced_blocks,
            missed_blocks,
            participation_rate,
            active_validators: self.validator_performance.len() as u32,
        }
    }
}

/// Estatísticas de uma época
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpochStats {
    /// Total de slots na época
    pub total_slots: u64,
    /// Blocos produzidos
    pub produced_blocks: u64,
    /// Blocos perdidos
    pub missed_blocks: u64,
    /// Taxa de participação (0.0 a 1.0)
    pub participation_rate: f64,
    /// Número de validadores ativos
    pub active_validators: u32,
}

/// Configurações de rede específicas do Aevum
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AevumNetworkParams {
    /// Configuração DPoS
    pub dpos_config: DposConfig,
    /// Tempo de bloco alvo (segundos)
    pub block_time: u64,
    /// Recompensa por bloco (em wei)
    pub block_reward: u128,
    /// Limite de gás por bloco
    pub gas_limit: u64,
    /// Preço mínimo de gás
    pub min_gas_price: u64,
}

impl Default for AevumNetworkParams {
    fn default() -> Self {
        Self {
            dpos_config: DposConfig::default(),
            block_time: 3, // 3 segundos por bloco
            block_reward: 1_000_000_000_000_000_000u128, // 1 AEV
            gas_limit: 8_000_000, // Similar ao Ethereum
            min_gas_price: 1_000_000_000, // 1 Gwei
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::placeholder::AevumState;

    #[test]
    fn test_dpos_engine_creation() {
        let config = DposConfig::default();
        let engine = DposEngine::new(config);
        
        assert_eq!(engine.current_schedule.len(), 0);
        assert_eq!(engine.next_slot, 0);
    }

    #[test]
    fn test_validator_election() {
        let mut engine = DposEngine::new(DposConfig::default());
        let mut state = AevumState::new();

        // Registra alguns validadores
        let val1 = Hash256::keccak256(b"validator1");
        let val2 = Hash256::keccak256(b"validator2");
        let val3 = Hash256::keccak256(b"validator3");

        state.register_validator(val1, 5000).unwrap();
        state.register_validator(val2, 3000).unwrap();
        state.register_validator(val3, 1000).unwrap();

        let elected = engine.elect_validators(&state).unwrap();
        
        assert_eq!(elected.len(), 3);
        // Deve estar ordenado por stake (maior primeiro)
        assert_eq!(elected[0], val1); // 5000 stake
        assert_eq!(elected[1], val2); // 3000 stake
        assert_eq!(elected[2], val3); // 1000 stake
    }

    #[test]
    fn test_schedule_generation() {
        let config = DposConfig {
            epoch_length: 10, // 10 slots para teste
            ..DposConfig::default()
        };
        let mut engine = DposEngine::new(config);
        
        let validators = vec![
            Hash256::keccak256(b"val1"),
            Hash256::keccak256(b"val2"),
        ];

        let epoch_start = 1000;
        engine.generate_schedule(&validators, epoch_start).unwrap();

        assert_eq!(engine.current_schedule.len(), 10);
        assert_eq!(engine.epoch_start_time, epoch_start);
        
        // Verifica distribuição round-robin
        assert_eq!(engine.current_schedule[0].validator, validators[0]);
        assert_eq!(engine.current_schedule[1].validator, validators[1]);
        assert_eq!(engine.current_schedule[2].validator, validators[0]);
    }

    #[test]
    fn test_validator_performance() {
        let mut performance = ValidatorPerformance::new();
        
        // Registra alguns blocos produzidos e perdidos
        performance.record_block_produced(1);
        performance.record_block_produced(1);
        performance.record_missed_block(1);
        
        assert_eq!(performance.blocks_produced, 2);
        assert_eq!(performance.missed_blocks, 1);
        assert!((performance.approval_rate - 0.666_666_666_666_666_7).abs() < f64::EPSILON);
    }

    #[test]
    fn test_reward_calculation() {
        let mut engine = DposEngine::new(DposConfig::default());
        
        // Adiciona performance de validadores
        let val1 = Hash256::keccak256(b"val1");
        let val2 = Hash256::keccak256(b"val2");
        
        let mut perf1 = ValidatorPerformance::new();
        perf1.approval_rate = 1.0; // 100%
        
        let mut perf2 = ValidatorPerformance::new();
        perf2.approval_rate = 0.5; // 50%
        
        engine.validator_performance.insert(val1, perf1);
        engine.validator_performance.insert(val2, perf2);
        
        let rewards = engine.calculate_rewards(1000);
        
        // val1 deve receber mais recompensa devido à melhor performance
        assert!(rewards.get(&val1).unwrap() > rewards.get(&val2).unwrap());
    }
}
