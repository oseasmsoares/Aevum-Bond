use serde::{Deserialize, Serialize};
use shared::{BlockchainError, Hash256, Result};
use crate::transaction::TxOutput;

/// Representa uma saída de transação não gasta (UTXO)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Utxo {
    /// A saída original da transação
    pub output: TxOutput,
    /// Altura do bloco onde foi criado (para controle de maturidade)
    pub height: u64,
}

impl Utxo {
    /// Cria um novo UTXO
    pub fn new(txid: Hash256, vout: u32, value: u64, script_pubkey: Vec<u8>, height: u64) -> Self {
        Self {
            output: TxOutput {
                value,
                script_pubkey,
            },
            height,
        }
    }

    /// Obtém o identificador único do UTXO
    pub fn outpoint(&self) -> OutPoint {
        // Note: Em um sistema real, precisaríamos armazenar o txid e vout
        // Por agora, vamos usar um placeholder
        OutPoint {
            txid: Hash256::zero(),
            vout: 0,
        }
    }

    /// Verifica se o UTXO está maduro (pode ser gasto)
    /// UTXOs de coinbase precisam de 100 blocos para maturar
    pub fn is_mature(&self, current_height: u64, is_coinbase: bool) -> bool {
        if is_coinbase {
            current_height >= self.height + 100
        } else {
            true
        }
    }
}

/// Identificador único de um UTXO (`OutPoint`)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: Hash256,
    pub vout: u32,
}

impl OutPoint {
    /// Cria um novo `OutPoint`
    pub fn new(txid: Hash256, vout: u32) -> Self {
        Self { txid, vout }
    }
}

/// Conjunto de UTXOs para controle de estado
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoSet {
    utxos: std::collections::HashMap<OutPoint, Utxo>,
}

impl UtxoSet {
    /// Cria um novo conjunto vazio de UTXOs
    pub fn new() -> Self {
        Self {
            utxos: std::collections::HashMap::new(),
        }
    }

    /// Adiciona um UTXO ao conjunto
    pub fn add(&mut self, outpoint: OutPoint, utxo: Utxo) {
        self.utxos.insert(outpoint, utxo);
    }

    /// Adiciona um UTXO ao conjunto (método legado)
    pub fn add_utxo(&mut self, utxo: Utxo) {
        let outpoint = utxo.outpoint();
        self.utxos.insert(outpoint, utxo);
    }

    /// Remove um UTXO do conjunto (quando é gasto)
    pub fn remove_utxo(&mut self, outpoint: &OutPoint) -> Option<Utxo> {
        self.utxos.remove(outpoint)
    }

    /// Obtém um UTXO do conjunto
    pub fn get(&self, outpoint: &OutPoint) -> Option<&Utxo> {
        self.utxos.get(outpoint)
    }

    /// Obtém um UTXO do conjunto (método legado)
    pub fn get_utxo(&self, outpoint: &OutPoint) -> Option<&Utxo> {
        self.utxos.get(outpoint)
    }

    /// Verifica se um UTXO existe
    pub fn contains(&self, outpoint: &OutPoint) -> bool {
        self.utxos.contains_key(outpoint)
    }

    /// Obtém o valor total de UTXOs controlados por um script específico
    pub fn get_balance_for_script(&self, script: &[u8]) -> u64 {
        self.utxos
            .values()
            .filter(|utxo| utxo.output.script_pubkey == script)
            .map(|utxo| utxo.output.value)
            .sum()
    }

    /// Encontra UTXOs suficientes para cobrir um valor específico
    pub fn find_utxos_for_amount(&self, script: &[u8], amount: u64) -> Result<Vec<&Utxo>> {
        let mut selected_utxos = Vec::new();
        let mut total_value = 0u64;

        for utxo in self.utxos.values() {
            if utxo.output.script_pubkey == script {
                selected_utxos.push(utxo);
                total_value = total_value.checked_add(utxo.output.value).ok_or_else(|| {
                    BlockchainError::InvalidTransaction("Overflow in UTXO selection".to_string())
                })?;

                if total_value >= amount {
                    return Ok(selected_utxos);
                }
            }
        }

        if total_value < amount {
            return Err(BlockchainError::InsufficientFunds);
        }

        Ok(selected_utxos)
    }

    /// Retorna o número total de UTXOs
    pub fn len(&self) -> usize {
        self.utxos.len()
    }

    /// Verifica se o conjunto está vazio
    pub fn is_empty(&self) -> bool {
        self.utxos.is_empty()
    }
}

impl Default for UtxoSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_utxo_creation() {
        let utxo = Utxo::new(Hash256::zero(), 0, 5000, vec![1, 2, 3], 100);

        assert_eq!(utxo.output.value, 5000);
        assert_eq!(utxo.output.script_pubkey, vec![1, 2, 3]);
        assert_eq!(utxo.height, 100);
    }

    #[test]
    fn test_utxo_maturity() {
        let utxo = Utxo::new(Hash256::zero(), 0, 5000, vec![1, 2, 3], 100);

        // UTXO regular sempre está maduro
        assert!(utxo.is_mature(101, false));

        // UTXO de coinbase precisa de 100 blocos
        assert!(!utxo.is_mature(150, true)); // Apenas 50 blocos se passaram
        assert!(utxo.is_mature(200, true)); // 100 blocos se passaram
    }

    #[test]
    fn test_utxo_set_operations() {
        let mut utxo_set = UtxoSet::new();
        let utxo = Utxo::new(Hash256::zero(), 0, 5000, vec![1, 2, 3], 100);
        let outpoint = OutPoint::new(Hash256::zero(), 0);

        // Adicionar UTXO
        utxo_set.add(outpoint, utxo.clone());
        assert!(utxo_set.contains(&outpoint));
        assert_eq!(utxo_set.len(), 1);

        // Obter UTXO
        let retrieved = utxo_set.get(&outpoint);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().output.value, utxo.output.value);

        // Remover UTXO
        let removed = utxo_set.remove_utxo(&outpoint);
        assert!(removed.is_some());
        assert!(!utxo_set.contains(&outpoint));
        assert_eq!(utxo_set.len(), 0);
    }

    #[test]
    fn test_balance_calculation() {
        let mut utxo_set = UtxoSet::new();
        let script = vec![1, 2, 3];

        // Adicionar UTXOs com o mesmo script
        let outpoint1 = OutPoint::new(Hash256::zero(), 0);
        let outpoint2 = OutPoint::new(Hash256::zero(), 1);
        let outpoint3 = OutPoint::new(Hash256::zero(), 2);

        utxo_set.add(outpoint1, Utxo::new(Hash256::zero(), 0, 1000, script.clone(), 100));
        utxo_set.add(outpoint2, Utxo::new(Hash256::zero(), 1, 2000, script.clone(), 100));
        utxo_set.add(outpoint3, Utxo::new(Hash256::zero(), 2, 3000, vec![4, 5, 6], 100));

        assert_eq!(utxo_set.get_balance_for_script(&script), 3000);
    }
}
