use crate::transaction::Transaction;
use crate::utxo::{Utxo, UtxoSet};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use shared::{BlockchainError, Hash256, Result};

/// Cabeçalho do bloco
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlockHeader {
    /// Versão do bloco
    pub version: u32,
    /// Hash do bloco anterior
    pub previous_hash: Hash256,
    /// Merkle root das transações
    pub merkle_root: Hash256,
    /// Timestamp do bloco
    pub timestamp: DateTime<Utc>,
    /// Dificuldade alvo (número de zeros iniciais requeridos)
    pub difficulty: u32,
    /// Nonce para mineração
    pub nonce: u64,
}

impl BlockHeader {
    /// Cria um novo cabeçalho de bloco
    #[must_use]
    pub const fn new(
        version: u32,
        previous_hash: Hash256,
        merkle_root: Hash256,
        timestamp: DateTime<Utc>,
        difficulty: u32,
        nonce: u64,
    ) -> Self {
        Self {
            version,
            previous_hash,
            merkle_root,
            timestamp,
            difficulty,
            nonce,
        }
    }

    /// Calcula o hash do cabeçalho do bloco
    ///
    /// # Errors
    ///
    /// Retorna erro se a serialização falhar
    pub fn hash(&self) -> Result<Hash256> {
        let serialized = serde_json::to_vec(self)
            .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
        Ok(Hash256::keccak256(&serialized))
    }

    /// Verifica se o hash do cabeçalho atende à dificuldade
    ///
    /// # Errors
    ///
    /// Retorna erro se o cálculo do hash falhar
    pub fn meets_difficulty(&self) -> Result<bool> {
        let hash = self.hash()?;
        Ok(hash.meets_difficulty(self.difficulty))
    }
}

/// Bloco completo da blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// Cabeçalho do bloco
    pub header: BlockHeader,
    /// Transações no bloco
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Cria um novo bloco
    #[must_use]
    pub const fn new(header: BlockHeader, transactions: Vec<Transaction>) -> Self {
        Self {
            header,
            transactions,
        }
    }

    /// Cria o bloco gênese (primeiro bloco da blockchain)
    ///
    /// # Errors
    ///
    /// Retorna erro se a mineração ou cálculo do merkle root falharem
    pub fn genesis(genesis_reward: u64, genesis_script: Vec<u8>) -> Result<Self> {
        let coinbase = Transaction::coinbase(0, genesis_reward, genesis_script);
        let merkle_root = calculate_merkle_root(std::slice::from_ref(&coinbase))?;

        let mut header = BlockHeader::new(
            1,
            Hash256::zero(), // Bloco gênese não tem antecessor
            merkle_root,
            Utc::now(),
            1, // Dificuldade muito baixa para gênese
            0,
        );

        // Minerar o bloco gênese
        loop {
            let hash = header.hash()?;
            if hash.meets_difficulty(header.difficulty) {
                break;
            }
            header.nonce += 1;
        }

        Ok(Self::new(header, vec![coinbase]))
    }

    /// Obter o hash do bloco
    ///
    /// # Errors
    ///
    /// Retorna erro se o cálculo do hash do cabeçalho falhar
    pub fn hash(&self) -> Result<Hash256> {
        self.header.hash()
    }

    /// Obtém a altura do bloco (extraída da transação coinbase)
    ///
    /// # Errors
    ///
    /// Retorna erro se não houver transação coinbase válida ou se a altura não puder ser decodificada
    pub fn height(&self) -> Result<u64> {
        if self.transactions.is_empty() || !self.transactions[0].is_coinbase() {
            return Err(BlockchainError::InvalidBlock(
                "Missing coinbase transaction".to_string(),
            ));
        }

        let coinbase = &self.transactions[0];
        if coinbase.inputs.is_empty() {
            return Err(BlockchainError::InvalidBlock(
                "Invalid coinbase transaction".to_string(),
            ));
        }

        let script_sig = &coinbase.inputs[0].script_sig;
        if script_sig.len() < 8 {
            return Err(BlockchainError::InvalidBlock(
                "Invalid coinbase height encoding".to_string(),
            ));
        }

        let height_bytes: [u8; 8] = script_sig[0..8]
            .try_into()
            .map_err(|_| BlockchainError::InvalidBlock("Invalid height bytes".to_string()))?;

        Ok(u64::from_le_bytes(height_bytes))
    }

    /// Validação básica do bloco
    ///
    /// # Errors
    ///
    /// Retorna erro se o bloco não atender aos critérios de validação básica
    pub fn validate_basic(&self) -> Result<()> {
        // Verificar se tem pelo menos uma transação (coinbase)
        if self.transactions.is_empty() {
            return Err(BlockchainError::InvalidBlock(
                "Block has no transactions".to_string(),
            ));
        }

        // Verificar se a primeira transação é coinbase
        if !self.transactions[0].is_coinbase() {
            return Err(BlockchainError::InvalidBlock(
                "First transaction is not coinbase".to_string(),
            ));
        }

        // Verificar se não há outras transações coinbase
        for (i, tx) in self.transactions.iter().enumerate() {
            if i > 0 && tx.is_coinbase() {
                return Err(BlockchainError::InvalidBlock(
                    "Multiple coinbase transactions".to_string(),
                ));
            }
        }

        // Validar cada transação
        for tx in &self.transactions {
            tx.validate_basic()?;
        }

        // Verificar merkle root
        let calculated_merkle = calculate_merkle_root(&self.transactions)?;
        if calculated_merkle != self.header.merkle_root {
            return Err(BlockchainError::InvalidBlock(
                "Invalid merkle root".to_string(),
            ));
        }

        // Verificar se atende à dificuldade
        if !self.header.meets_difficulty()? {
            return Err(BlockchainError::InsufficientDifficulty);
        }

        Ok(())
    }

    /// Aplica o bloco ao conjunto de UTXOs
    ///
    /// # Errors
    ///
    /// Retorna erro se alguma UTXO requerida não for encontrada ou se ocorrer erro na aplicação
    pub fn apply_to_utxo_set(&self, utxo_set: &mut UtxoSet) -> Result<()> {
        let block_height = self.height()?;

        for tx in &self.transactions {
            let txid = tx.hash()?;

            // Remover UTXOs gastos
            if !tx.is_coinbase() {
                for input in &tx.inputs {
                    if !utxo_set.contains(&input.previous_output) {
                        return Err(BlockchainError::UtxoNotFound);
                    }
                    utxo_set.remove_utxo(&input.previous_output);
                }
            }

            // Criar novos UTXOs
            for (output_index, output) in tx.outputs.iter().enumerate() {
                let utxo = Utxo::new(
                    txid,
                    output_index.try_into().map_err(|_| {
                        BlockchainError::InvalidBlock("Output index overflow".to_string())
                    })?,
                    output.value,
                    output.script_pubkey.clone(),
                    block_height,
                );
                utxo_set.add_utxo(utxo);
            }
        }

        Ok(())
    }

    /// Calcula o tamanho do bloco em bytes
    #[must_use]
    pub fn size(&self) -> usize {
        // Estimativa simplificada
        const HEADER_SIZE: usize = 200; // Estimativa para cabeçalho serializado
        let transactions_size: usize = self
            .transactions
            .iter()
            .map(Transaction::estimated_size)
            .sum();

        HEADER_SIZE + transactions_size
    }

    /// Verifica se o bloco excede o tamanho máximo (4MB)
    #[must_use]
    pub fn exceeds_max_size(&self) -> bool {
        const MAX_BLOCK_SIZE: usize = 4_000_000; // 4MB
        self.size() > MAX_BLOCK_SIZE
    }
}

/// Calcula a merkle root de uma lista de transações
///
/// # Errors
///
/// Retorna erro se o cálculo do hash das transações falhar
pub fn calculate_merkle_root(transactions: &[Transaction]) -> Result<Hash256> {
    if transactions.is_empty() {
        return Ok(Hash256::zero());
    }

    let mut hashes: Vec<Hash256> = transactions
        .iter()
        .map(Transaction::hash)
        .collect::<Result<Vec<_>>>()?;

    // Se há apenas uma transação, retorna seu hash
    if hashes.len() == 1 {
        return Ok(hashes[0]);
    }

    // Construir árvore merkle
    while hashes.len() > 1 {
        let mut next_level = Vec::new();

        for chunk in hashes.chunks(2) {
            let mut data = Vec::new();
            data.extend_from_slice(chunk[0].as_bytes());

            if chunk.len() == 2 {
                // Combinar dois hashes
                data.extend_from_slice(chunk[1].as_bytes());
            } else {
                // Hash ímpar - combina consigo mesmo
                data.extend_from_slice(chunk[0].as_bytes());
            }

            let combined = Hash256::keccak256(&data);
            next_level.push(combined);
        }

        hashes = next_level;
    }

    Ok(hashes[0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_genesis_block_creation() {
        let genesis = Block::genesis(5000, vec![1, 2, 3]).unwrap();

        assert_eq!(genesis.transactions.len(), 1);
        assert!(genesis.transactions[0].is_coinbase());
        assert_eq!(genesis.height().unwrap(), 0);

        // Mostrar o erro específico se a validação falhar
        match genesis.validate_basic() {
            Ok(()) => (),
            Err(e) => {
                println!("Erro na validação do bloco gênese: {e:?}");
                let hash = genesis.hash().unwrap();
                println!("Hash do bloco: {hash}");
                println!("Zeros iniciais: {}", hash.leading_zeros());
                println!("Dificuldade: {}", genesis.header.difficulty);
                println!(
                    "Atende dificuldade: {}",
                    hash.meets_difficulty(genesis.header.difficulty)
                );
                panic!("Validação do bloco gênese falhou");
            }
        }
    }

    #[test]
    fn test_block_hash() {
        let genesis = Block::genesis(5000, vec![1, 2, 3]).unwrap();
        let hash = genesis.hash().unwrap();

        // Hash deve ser determinístico
        let hash2 = genesis.hash().unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_merkle_root_calculation() {
        let tx1 = Transaction::coinbase(0, 5000, vec![1, 2, 3]);
        let tx2 = Transaction::coinbase(1, 5000, vec![4, 5, 6]);

        let single_tx_root = calculate_merkle_root(std::slice::from_ref(&tx1)).unwrap();
        let double_tx_root = calculate_merkle_root(&[tx1, tx2]).unwrap();

        assert_ne!(single_tx_root, double_tx_root);
        assert_ne!(single_tx_root, Hash256::zero());
    }

    #[test]
    fn test_utxo_set_application() {
        let mut utxo_set = UtxoSet::new();
        let genesis = Block::genesis(5000, vec![1, 2, 3]).unwrap();

        // Aplicar bloco gênese
        genesis.apply_to_utxo_set(&mut utxo_set).unwrap();

        assert_eq!(utxo_set.len(), 1);
        assert_eq!(utxo_set.get_balance_for_script(&[1, 2, 3]), 5000);
    }

    #[test]
    fn test_block_size_limits() {
        let genesis = Block::genesis(5000, vec![1, 2, 3]).unwrap();

        // Bloco gênese não deve exceder tamanho máximo
        assert!(!genesis.exceeds_max_size());

        // Verificar que o tamanho é razoável
        assert!(genesis.size() > 0);
        assert!(genesis.size() < 1000); // Deve ser pequeno para um bloco simples
    }
}
