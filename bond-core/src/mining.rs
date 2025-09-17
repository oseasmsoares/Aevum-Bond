use crate::block::{calculate_merkle_root, Block, BlockHeader};
use crate::transaction::Transaction;
use chrono::Utc;
use shared::{BlockchainError, Hash256, Result};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

/// Configuração do minerador
#[derive(Debug, Clone)]
pub struct MinerConfig {
    /// Script que receberá a recompensa de mineração
    pub reward_script: Vec<u8>,
    /// Número de threads para mineração
    pub threads: usize,
    /// Dificuldade alvo
    pub difficulty: u32,
}

impl Default for MinerConfig {
    fn default() -> Self {
        Self {
            reward_script: vec![0x76, 0xa9, 0x14], // Script P2PKH placeholder
            threads: num_cpus::get().unwrap_or(1),
            difficulty: 20, // Dificuldade inicial
        }
    }
}

/// Resultado da mineração
#[derive(Debug, Clone)]
pub struct MiningResult {
    pub block: Block,
    pub hash: Hash256,
    pub nonce: u64,
    pub attempts: u64,
}

/// Minerador de blocos
pub struct Miner {
    config: MinerConfig,
    is_mining: Arc<AtomicBool>,
}

impl Miner {
    /// Cria um novo minerador
    #[must_use]
    pub fn new(config: MinerConfig) -> Self {
        Self {
            config,
            is_mining: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Minera um bloco
    ///
    /// # Errors
    ///
    /// Retorna erro se a mineração falhar ou se não conseguir calcular o merkle root
    pub fn mine_block(
        &self,
        previous_hash: Hash256,
        transactions: Vec<Transaction>,
        block_height: u64,
        reward: u64,
    ) -> Result<MiningResult> {
        self.mine_block_with_difficulty(
            previous_hash,
            transactions,
            block_height,
            reward,
            self.config.difficulty,
        )
    }

    /// Minera um bloco com dificuldade específica
    ///
    /// # Errors
    ///
    /// Retorna erro se a mineração falhar ou se não conseguir calcular o merkle root
    pub fn mine_block_with_difficulty(
        &self,
        previous_hash: Hash256,
        transactions: Vec<Transaction>,
        block_height: u64,
        reward: u64,
        difficulty: u32,
    ) -> Result<MiningResult> {
        // Criar transação coinbase
        let coinbase =
            Transaction::coinbase(block_height, reward, self.config.reward_script.clone());

        // Combinar transações (coinbase + outras)
        let mut all_transactions = vec![coinbase];
        all_transactions.extend(transactions);

        // Calcular merkle root
        let merkle_root = calculate_merkle_root(&all_transactions)?;

        // Criar cabeçalho do bloco
        let header = BlockHeader::new(
            1,
            previous_hash,
            merkle_root,
            Utc::now(),
            difficulty, // Usar dificuldade fornecida
            0,          // nonce será incrementado durante a mineração
        );

        // Minerar com múltiplas threads
        self.mine_header_parallel(&header, &all_transactions)
    }

    /// Mineração paralela do cabeçalho
    fn mine_header_parallel(
        &self,
        header: &BlockHeader,
        transactions: &[Transaction],
    ) -> Result<MiningResult> {
        let is_mining = Arc::clone(&self.is_mining);
        is_mining.store(true, Ordering::SeqCst);

        let result = Arc::new(Mutex::new(None));
        let mut handles = vec![];

        // Dividir o espaço de nonce entre threads
        let nonce_per_thread = u64::MAX / self.config.threads as u64;

        for thread_id in 0..self.config.threads {
            let header_clone = header.clone();
            let transactions_clone = transactions.to_owned();
            let is_mining_clone = Arc::clone(&is_mining);
            let result_clone = Arc::clone(&result);

            let start_nonce = thread_id as u64 * nonce_per_thread;
            let end_nonce = if thread_id == self.config.threads - 1 {
                u64::MAX
            } else {
                (thread_id as u64 + 1) * nonce_per_thread
            };

            let handle = thread::spawn(move || {
                Self::mine_header_range(
                    header_clone,
                    &transactions_clone,
                    start_nonce,
                    end_nonce,
                    &is_mining_clone,
                    &result_clone,
                );
            });

            handles.push(handle);
        }

        // Esperar todas as threads terminarem
        for handle in handles {
            handle.join().unwrap();
        }

        // Extrair resultado
        let result = result
            .lock()
            .unwrap()
            .take()
            .ok_or(BlockchainError::NonceNotFound)?;

        Ok(result)
    }

    /// Minera um cabeçalho em um intervalo de nonce específico
    #[allow(clippy::needless_pass_by_value)] // Arc types are cheap to clone
    fn mine_header_range(
        mut header: BlockHeader,
        transactions: &[Transaction],
        start_nonce: u64,
        end_nonce: u64,
        is_mining: &Arc<AtomicBool>,
        result: &Arc<Mutex<Option<MiningResult>>>,
    ) {
        let mut attempts = 0u64;

        for nonce in start_nonce..end_nonce {
            // Verificar se outra thread já encontrou solução
            if !is_mining.load(Ordering::SeqCst) {
                break;
            }

            // Verificar se já temos resultado
            if result.lock().unwrap().is_some() {
                break;
            }

            header.nonce = nonce;
            attempts += 1;

            // Calcular hash
            if let Ok(hash) = header.hash() {
                if hash.meets_difficulty(header.difficulty) {
                    // Encontrou solução!
                    is_mining.store(false, Ordering::SeqCst);

                    let block = Block::new(header, transactions.to_owned());
                    let mining_result = MiningResult {
                        block,
                        hash,
                        nonce,
                        attempts,
                    };

                    *result.lock().unwrap() = Some(mining_result);
                    break;
                }
            }

            // Atualizar timestamp periodicamente
            if attempts % 100_000 == 0 {
                header.timestamp = Utc::now();
            }
        }
    }

    /// Para a mineração
    pub fn stop_mining(&self) {
        self.is_mining.store(false, Ordering::SeqCst);
    }

    /// Verifica se está minerando
    #[must_use]
    pub fn is_mining(&self) -> bool {
        self.is_mining.load(Ordering::SeqCst)
    }

    /// Estima a taxa de hash (hashes por segundo)
    ///
    /// # Errors
    ///
    /// Retorna erro se o cálculo do hash falhar
    #[allow(clippy::cast_precision_loss)] // Conversão intencional para cálculo de taxa
    pub fn estimate_hashrate(&self, duration_secs: u64) -> Result<f64> {
        let start_time = std::time::Instant::now();
        let mut header = BlockHeader::new(
            1,
            Hash256::zero(),
            Hash256::zero(),
            Utc::now(),
            32, // Dificuldade alta para não encontrar solução
            0,
        );

        let mut attempts = 0u64;
        while start_time.elapsed().as_secs() < duration_secs {
            header.nonce = attempts;
            let _ = header.hash()?;
            attempts += 1;
        }

        let elapsed = start_time.elapsed().as_secs_f64();
        #[allow(clippy::cast_precision_loss)] // Conversão necessária para cálculo
        let attempts_f64 = attempts as f64;
        Ok(attempts_f64 / elapsed)
    }
}

/// Ajustador de dificuldade
pub struct DifficultyAdjuster {
    /// Tempo alvo entre blocos (em segundos)
    pub target_block_time: u64,
    /// Período de ajuste em número de blocos
    pub adjustment_period: u64,
}

impl Default for DifficultyAdjuster {
    fn default() -> Self {
        Self {
            target_block_time: 600,  // 10 minutos
            adjustment_period: 2016, // ~2 semanas com blocos de 10 min
        }
    }
}

impl DifficultyAdjuster {
    /// Cria um novo ajustador de dificuldade
    #[must_use]
    pub const fn new(target_block_time: u64, adjustment_period: u64) -> Self {
        Self {
            target_block_time,
            adjustment_period,
        }
    }

    /// Calcula a nova dificuldade baseada no histórico de blocos
    ///
    /// # Errors
    ///
    /// Retorna erro se os blocos não tiverem timestamps válidos
    ///
    /// # Panics
    ///
    /// Pode entrar em pânico se o slice de blocos estiver vazio (verificado antes)
    #[allow(clippy::cast_possible_truncation)] // Conversões seguras para cálculos de dificuldade
    #[allow(clippy::cast_sign_loss)] // Conversões seguras de duração
    #[allow(clippy::cast_precision_loss)] // Conversões necessárias para cálculos
    pub fn calculate_new_difficulty(
        &self,
        current_difficulty: u32,
        blocks: &[Block],
    ) -> Result<u32> {
        let adjustment_period_usize = usize::try_from(self.adjustment_period).map_err(|_| {
            BlockchainError::InvalidBlock("Adjustment period too large".to_string())
        })?;

        if blocks.len() < adjustment_period_usize {
            return Ok(current_difficulty); // Não ajustar ainda
        }

        let recent_blocks = &blocks[blocks.len() - adjustment_period_usize..];

        // Calcular tempo real entre o primeiro e último bloco
        let first_timestamp = recent_blocks.first().unwrap().header.timestamp;
        let last_timestamp = recent_blocks.last().unwrap().header.timestamp;

        let actual_time =
            u64::try_from((last_timestamp - first_timestamp).num_seconds()).map_err(|_| {
                BlockchainError::InvalidBlock("Invalid timestamp difference".to_string())
            })?;
        let expected_time = self.target_block_time * (self.adjustment_period - 1);

        // Calcular fator de ajuste
        let adjustment_factor = actual_time as f64 / expected_time as f64;

        // Limitar ajuste a 4x para cima ou para baixo
        let clamped_factor = adjustment_factor.clamp(0.25, 4.0);

        // Nova dificuldade (inversa do fator)
        let new_difficulty = if clamped_factor > 1.0 {
            // Blocos muito lentos - diminuir dificuldade
            let decrease = (clamped_factor - 1.0) as u32;
            current_difficulty.saturating_sub(decrease).max(1)
        } else {
            // Blocos muito rápidos - aumentar dificuldade
            let increase = ((1.0 / clamped_factor) - 1.0) as u32;
            current_difficulty.saturating_add(increase)
        };

        Ok(new_difficulty.min(32)) // Limitar dificuldade máxima
    }
}

// Adicionar dependência num_cpus ao Cargo.toml seria ideal,
// mas por simplicidade, vamos usar uma implementação básica
mod num_cpus {
    pub fn get() -> Option<usize> {
        std::thread::available_parallelism()
            .map(std::num::NonZero::get)
            .ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_miner_creation() {
        let config = MinerConfig::default();
        let miner = Miner::new(config);

        assert!(!miner.is_mining());
    }

    #[test]
    fn test_mine_genesis_block() {
        let config = MinerConfig {
            reward_script: vec![1, 2, 3],
            threads: 1,
            difficulty: 1, // Dificuldade muito baixa para teste rápido
        };

        let miner = Miner::new(config);
        let result = miner
            .mine_block(
                Hash256::zero(),
                vec![], // Sem transações além da coinbase
                0,      // Altura 0 (gênese)
                5000,   // Recompensa
            )
            .unwrap();

        assert!(result.block.validate_basic().is_ok());
        assert_eq!(result.block.transactions.len(), 1);
        assert!(result.block.transactions[0].is_coinbase());
        assert!(result.hash.meets_difficulty(1));
    }

    #[test]
    fn test_difficulty_adjustment() {
        let adjuster = DifficultyAdjuster::new(600, 10); // 10 blocos para teste

        // Criar blocos simulados
        let mut blocks = vec![];
        let mut timestamp = Utc::now();

        for i in 0..10 {
            let coinbase = Transaction::coinbase(i, 5000, vec![1, 2, 3]);
            let merkle_root = calculate_merkle_root(std::slice::from_ref(&coinbase)).unwrap();

            let header = BlockHeader::new(1, Hash256::zero(), merkle_root, timestamp, 20, 0);

            blocks.push(Block::new(header, vec![coinbase]));
            timestamp += chrono::Duration::seconds(300); // Blocos de 5 min (muito rápido)
        }

        let new_difficulty = adjuster.calculate_new_difficulty(20, &blocks).unwrap();

        // Dificuldade deve aumentar pois blocos estão sendo minerados muito rapidamente
        assert!(new_difficulty > 20);
    }

    #[test]
    fn test_hashrate_estimation() {
        let config = MinerConfig {
            threads: 1,
            difficulty: 32, // Alta para não encontrar solução
            ..Default::default()
        };

        let miner = Miner::new(config);
        let hashrate = miner.estimate_hashrate(1).unwrap(); // 1 segundo

        assert!(hashrate > 0.0);
        println!("Estimated hashrate: {hashrate:.2} H/s");
    }
}
