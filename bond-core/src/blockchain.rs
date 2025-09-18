use crate::block::Block;
use crate::mining::{DifficultyAdjuster, Miner, MiningResult};
use crate::transaction::Transaction;
use crate::utxo::UtxoSet;
use serde::{Deserialize, Serialize};
use shared::{BlockchainError, Hash256, Result};
use std::collections::HashMap;

/// Estado da blockchain Bond
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    /// Cadeia de blocos
    blocks: Vec<Block>,
    /// Conjunto atual de UTXOs
    utxo_set: UtxoSet,
    /// Índice de hash para bloco (para busca rápida)
    block_index: HashMap<Hash256, usize>,
    /// Parâmetros da rede
    network_params: NetworkParams,
}

/// Parâmetros da rede Bond
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkParams {
    /// Recompensa inicial por bloco em Elos
    pub initial_reward: u64,
    /// Dificuldade inicial
    pub initial_difficulty: u32,
    /// Tempo alvo entre blocos (segundos)
    pub target_block_time: u64,
    /// Período de ajuste de dificuldade (blocos)
    pub difficulty_adjustment_period: u64,
    /// Tamanho máximo do bloco (bytes)
    pub max_block_size: usize,
}

impl Default for NetworkParams {
    fn default() -> Self {
        Self {
            initial_reward: 5000,               // 5000 Elos = 5 BND
            initial_difficulty: 1,              // Dificuldade muito baixa para desenvolvimento
            target_block_time: 600,             // 10 minutos
            difficulty_adjustment_period: 2016, // ~2 semanas
            max_block_size: 4_000_000,          // 4MB
        }
    }
}

impl Blockchain {
    /// Cria uma nova blockchain com bloco gênese
    ///
    /// # Errors
    ///
    /// Retorna erro se não conseguir criar o bloco gênese ou aplicá-lo ao conjunto UTXO
    pub fn new(network_params: NetworkParams, genesis_script: Vec<u8>) -> Result<Self> {
        let genesis_block = Block::genesis(network_params.initial_reward, genesis_script)?;
        let genesis_hash = genesis_block.hash()?;

        let mut utxo_set = UtxoSet::new();
        genesis_block.apply_to_utxo_set(&mut utxo_set)?;

        let mut block_index = HashMap::new();
        block_index.insert(genesis_hash, 0);

        Ok(Self {
            blocks: vec![genesis_block],
            utxo_set,
            block_index,
            network_params,
        })
    }

    /// Adiciona um bloco à blockchain
    ///
    /// # Errors
    ///
    /// Retorna erro se o bloco não for válido ou não puder ser aplicado
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        // Validar bloco
        self.validate_block(&block)?;

        // Aplicar ao UTXO set
        let mut new_utxo_set = self.utxo_set.clone();
        block.apply_to_utxo_set(&mut new_utxo_set)?;

        // Adicionar à cadeia
        let block_hash = block.hash()?;
        let block_index = self.blocks.len();

        self.blocks.push(block);
        self.utxo_set = new_utxo_set;
        self.block_index.insert(block_hash, block_index);

        Ok(())
    }

    /// Valida um bloco antes de adicioná-lo
    ///
    /// # Errors
    ///
    /// Retorna erro se o bloco não atender aos critérios de validação
    pub fn validate_block(&self, block: &Block) -> Result<()> {
        // Validação básica do bloco
        block.validate_basic()?;

        // Verificar se não excede tamanho máximo
        if block.size() > self.network_params.max_block_size {
            return Err(BlockchainError::InvalidBlock(
                "Block exceeds maximum size".to_string(),
            ));
        }

        // Verificar se referencia o último bloco
        let last_block = self.get_latest_block();
        let last_hash = last_block.hash()?;

        if block.header.previous_hash != last_hash {
            return Err(BlockchainError::InvalidBlock(
                "Invalid previous hash".to_string(),
            ));
        }

        // Verificar altura do bloco
        let expected_height = self.height() + 1;
        let actual_height = block.height()?;
        if actual_height != expected_height {
            return Err(BlockchainError::InvalidBlock(
                "Invalid block height".to_string(),
            ));
        }

        // Verificar dificuldade
        let expected_difficulty = self.get_next_difficulty();
        if block.header.difficulty != expected_difficulty {
            return Err(BlockchainError::InvalidBlock(
                "Invalid difficulty".to_string(),
            ));
        }

        // Verificar recompensa de coinbase
        let expected_reward = self.calculate_block_reward(expected_height);
        let coinbase = &block.transactions[0];
        let total_fees = self.calculate_total_fees(&block.transactions[1..])?;
        let expected_coinbase_value = expected_reward + total_fees;

        if coinbase.total_output_value()? != expected_coinbase_value {
            return Err(BlockchainError::InvalidBlock(
                "Invalid coinbase reward".to_string(),
            ));
        }

        // Validar todas as transações não-coinbase
        for tx in &block.transactions[1..] {
            self.validate_transaction(tx)?;
        }

        Ok(())
    }

    /// Valida uma transação
    ///
    /// # Errors
    ///
    /// Retorna erro se a transação não for válida
    pub fn validate_transaction(&self, tx: &Transaction) -> Result<()> {
        // Validação básica
        tx.validate_basic()?;

        // Verificar se todos os inputs referenciam UTXOs existentes
        for input in &tx.inputs {
            if !self.utxo_set.contains(&input.previous_output) {
                return Err(BlockchainError::UtxoNotFound);
            }
        }

        // Verificar se os valores batem (inputs >= outputs)
        let input_value = tx.total_input_value(&self.utxo_set)?;
        let output_value = tx.total_output_value()?;

        if input_value < output_value {
            return Err(BlockchainError::InsufficientFunds);
        }

        // TODO: Verificar assinaturas (será implementado no Sprint 2 com ML-DSA)

        Ok(())
    }

    /// Minera o próximo bloco
    ///
    /// # Errors
    ///
    /// Retorna erro se as transações não forem válidas ou se a mineração falhar
    pub fn mine_next_block(
        &self,
        miner: &Miner,
        transactions: Vec<Transaction>,
    ) -> Result<MiningResult> {
        // Validar transações
        for tx in &transactions {
            self.validate_transaction(tx)?;
        }

        let previous_hash = self.get_latest_block().hash()?;
        let block_height = self.height() + 1;
        let reward = self.calculate_block_reward(block_height);
        let total_fees = self.calculate_total_fees(&transactions)?;
        let total_reward = reward + total_fees;
        let difficulty = self.get_next_difficulty();

        miner.mine_block_with_difficulty(
            previous_hash,
            transactions,
            block_height,
            total_reward,
            difficulty,
        )
    }

    /// Obtém o último bloco
    ///
    /// # Panics
    ///
    /// Nunca deve entrar em pânico pois o bloco gênese sempre existe
    #[must_use]
    pub fn get_latest_block(&self) -> &Block {
        self.blocks.last().unwrap() // Genesis sempre existe
    }

    /// Obtém um bloco pelo hash
    #[must_use]
    pub fn get_block_by_hash(&self, hash: &Hash256) -> Option<&Block> {
        self.block_index.get(hash).map(|&index| &self.blocks[index])
    }

    /// Obtém um bloco pela altura
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // Validação de altura controlada
    pub fn get_block_by_height(&self, height: u64) -> Option<&Block> {
        let height_usize = usize::try_from(height).ok()?;
        if height_usize < self.blocks.len() {
            Some(&self.blocks[height_usize])
        } else {
            None
        }
    }

    /// Altura atual da blockchain
    #[must_use]
    #[allow(clippy::cast_possible_truncation)] // Altura nunca será maior que usize::MAX
    pub const fn height(&self) -> u64 {
        (self.blocks.len() - 1) as u64
    }

    /// Obtém o conjunto de UTXOs
    #[must_use]
    pub const fn utxo_set(&self) -> &UtxoSet {
        &self.utxo_set
    }

    /// Calcula a recompensa para um bloco na altura especificada
    #[must_use]
    pub const fn calculate_block_reward(&self, _height: u64) -> u64 {
        // Implementação simplificada - recompensa constante
        // Na versão final, implementará inflação adaptativa
        self.network_params.initial_reward
    }

    /// Calcula o total de taxas de um conjunto de transações
    ///
    /// # Errors
    ///
    /// Retorna erro se não conseguir calcular as taxas das transações
    pub fn calculate_total_fees(&self, transactions: &[Transaction]) -> Result<u64> {
        let mut total_fees = 0u64;

        for tx in transactions {
            let fee = tx.fee(&self.utxo_set)?;
            total_fees = total_fees
                .checked_add(fee)
                .ok_or_else(|| BlockchainError::InvalidTransaction("Fee overflow".to_string()))?;
        }

        Ok(total_fees)
    }

    /// Obtém a dificuldade para o próximo bloco
    #[must_use]
    pub fn get_next_difficulty(&self) -> u32 {
        let adjuster = DifficultyAdjuster::new(
            self.network_params.target_block_time,
            self.network_params.difficulty_adjustment_period,
        );

        let current_difficulty = self.get_latest_block().header.difficulty;

        adjuster
            .calculate_new_difficulty(current_difficulty, &self.blocks)
            .unwrap_or(current_difficulty)
    }

    /// Obtém o balanço de um script específico
    #[must_use]
    pub fn get_balance(&self, script: &[u8]) -> u64 {
        self.utxo_set.get_balance_for_script(script)
    }

    /// Cria uma transação simples
    ///
    /// # Errors
    ///
    /// Retorna erro se não houver UTXOs suficientes ou se a transação não puder ser criada
    pub fn create_transaction(
        &self,
        from_script: &[u8],
        to_script: Vec<u8>,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction> {
        let total_needed = amount + fee;

        // Encontrar UTXOs suficientes
        let utxos = self
            .utxo_set
            .find_utxos_for_amount(from_script, total_needed)?;

        // Calcular valor total dos UTXOs selecionados
        let total_input: u64 = utxos.iter().map(|utxo| utxo.output.value).sum();

        // Criar inputs
        let inputs: Vec<_> = utxos
            .iter()
            .map(|utxo| {
                crate::transaction::TxInput::new(
                    utxo.outpoint(),
                    vec![], // Script sig vazio por enquanto (TODO: ML-DSA)
                    0,
                )
            })
            .collect();

        // Criar outputs
        let mut outputs = vec![crate::transaction::TxOutput::new(amount, to_script)];

        // Adicionar troco se necessário
        let change = total_input - total_needed;
        if change > 0 {
            outputs.push(crate::transaction::TxOutput::new(
                change,
                from_script.to_vec(),
            ));
        }

        Ok(Transaction::new(1, inputs, outputs, 0))
    }

    /// Estatísticas da blockchain
    #[must_use]
    pub fn stats(&self) -> BlockchainStats {
        let total_supply = self
            .blocks
            .iter()
            .map(|block| self.calculate_block_reward(block.height().unwrap_or(0)))
            .sum();

        #[allow(clippy::cast_possible_truncation)] // Conversões controladas
        BlockchainStats {
            height: self.height(),
            total_blocks: self.blocks.len() as u64,
            total_transactions: self
                .blocks
                .iter()
                .map(|b| b.transactions.len() as u64)
                .sum(),
            total_utxos: self.utxo_set.len() as u64,
            total_supply,
            difficulty: self.get_latest_block().header.difficulty,
        }
    }
}

/// Estatísticas da blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockchainStats {
    pub height: u64,
    pub total_blocks: u64,
    pub total_transactions: u64,
    pub total_utxos: u64,
    pub total_supply: u64,
    pub difficulty: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mining::MinerConfig;

    #[test]
    fn test_blockchain_creation() {
        let network_params = NetworkParams::default();
        let blockchain = Blockchain::new(network_params, vec![1, 2, 3]).unwrap();

        assert_eq!(blockchain.height(), 0);
        assert_eq!(blockchain.blocks.len(), 1);
        assert!(blockchain.get_latest_block().transactions[0].is_coinbase());
    }

    #[test]
    fn test_block_mining_and_addition() {
        let network_params = NetworkParams::default();
        let mut blockchain = Blockchain::new(network_params, vec![1, 2, 3]).unwrap();

        let miner_config = MinerConfig {
            reward_script: vec![4, 5, 6],
            threads: 1,
            difficulty: 1, // Muito baixa para teste
        };
        let miner = Miner::new(miner_config);

        // Minerar próximo bloco
        let result = blockchain.mine_next_block(&miner, vec![]).unwrap();

        // Adicionar à blockchain
        blockchain.add_block(result.block).unwrap();

        assert_eq!(blockchain.height(), 1);
        assert_eq!(blockchain.blocks.len(), 2);
    }

    #[test]
    fn test_balance_tracking() {
        let network_params = NetworkParams::default();
        let genesis_script = vec![1, 2, 3];
        let blockchain = Blockchain::new(network_params, genesis_script.clone()).unwrap();

        // Gênese deve ter o balanço inicial
        let balance = blockchain.get_balance(&genesis_script);
        assert_eq!(balance, 5000); // Recompensa inicial
    }

    #[test]
    fn test_transaction_creation() {
        let network_params = NetworkParams::default();
        let genesis_script = vec![1, 2, 3];
        let blockchain = Blockchain::new(network_params, genesis_script.clone()).unwrap();

        // Criar transação simples
        let to_script = vec![4, 5, 6];
        let tx = blockchain
            .create_transaction(
                &genesis_script,
                to_script,
                1000, // Valor
                100,  // Taxa
            )
            .unwrap();

        assert_eq!(tx.inputs.len(), 1);
        assert_eq!(tx.outputs.len(), 2); // Destinatário + troco
        assert_eq!(tx.outputs[0].value, 1000);
        assert_eq!(tx.outputs[1].value, 3900); // 5000 - 1000 - 100
    }

    #[test]
    fn test_blockchain_stats() {
        let network_params = NetworkParams::default();
        let blockchain = Blockchain::new(network_params, vec![1, 2, 3]).unwrap();

        let stats = blockchain.stats();
        assert_eq!(stats.height, 0);
        assert_eq!(stats.total_blocks, 1);
        assert_eq!(stats.total_transactions, 1); // Coinbase gênese
        assert_eq!(stats.total_utxos, 1);
        assert_eq!(stats.total_supply, 5000);
    }
}
