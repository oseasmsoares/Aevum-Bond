use serde::{Deserialize, Serialize};
use shared::{Hash256, BlockchainError, Result};

/// Representa uma saída de transação não gasta (UTXO)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Utxo {
    /// Hash da transação que criou este UTXO
    pub txid: Hash256,
    /// Índice da saída na transação
    pub vout: u32,
    /// Valor em Elos (menor unidade do Bond)
    pub value: u64,
    /// Script que define as condições para gastar este UTXO
    pub script: Vec<u8>,
    /// Altura do bloco onde foi criado (para controle de maturidade)
    pub block_height: u64,
}

impl Utxo {
    /// Cria um novo UTXO
    pub fn new(txid: Hash256, vout: u32, value: u64, script: Vec<u8>, block_height: u64) -> Self {
        Self {
            txid,
            vout,
            value,
            script,
            block_height,
        }
    }

    /// Obtém o identificador único do UTXO
    pub fn outpoint(&self) -> OutPoint {
        OutPoint {
            txid: self.txid,
            vout: self.vout,
        }
    }

    /// Verifica se o UTXO está maduro (pode ser gasto)
    /// UTXOs de coinbase precisam de 100 blocos para maturar
    pub fn is_mature(&self, current_height: u64, is_coinbase: bool) -> bool {
        if is_coinbase {
            current_height >= self.block_height + 100
        } else {
            true
        }
    }
}

/// Identificador único de um UTXO (OutPoint)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OutPoint {
    pub txid: Hash256,
    pub vout: u32,
}

impl OutPoint {
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
    pub fn add_utxo(&mut self, utxo: Utxo) {
        let outpoint = utxo.outpoint();
        self.utxos.insert(outpoint, utxo);
    }

    /// Remove um UTXO do conjunto (quando é gasto)
    pub fn remove_utxo(&mut self, outpoint: &OutPoint) -> Option<Utxo> {
        self.utxos.remove(outpoint)
    }

    /// Obtém um UTXO do conjunto
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
            .filter(|utxo| utxo.script == script)
            .map(|utxo| utxo.value)
            .sum()
    }

    /// Encontra UTXOs suficientes para cobrir um valor específico
    pub fn find_utxos_for_amount(&self, script: &[u8], amount: u64) -> Result<Vec<&Utxo>> {
        let mut selected_utxos = Vec::new();
        let mut total_value = 0u64;

        for utxo in self.utxos.values() {
            if utxo.script == script {
                selected_utxos.push(utxo);
                total_value = total_value.checked_add(utxo.value)
                    .ok_or(BlockchainError::InvalidTransaction("Overflow in UTXO selection".to_string()))?;
                
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
        let txid = Hash256::zero();
        let utxo = Utxo::new(txid, 0, 5000, vec![1, 2, 3], 100);
        
        assert_eq!(utxo.txid, txid);
        assert_eq!(utxo.vout, 0);
        assert_eq!(utxo.value, 5000);
        assert_eq!(utxo.block_height, 100);
    }

    #[test]
    fn test_utxo_maturity() {
        let utxo = Utxo::new(Hash256::zero(), 0, 5000, vec![1, 2, 3], 100);
        
        // UTXO regular sempre está maduro
        assert!(utxo.is_mature(101, false));
        
        // UTXO de coinbase precisa de 100 blocos
        assert!(!utxo.is_mature(150, true));  // Apenas 50 blocos se passaram
        assert!(utxo.is_mature(200, true));   // 100 blocos se passaram
    }

    #[test]
    fn test_utxo_set_operations() {
        let mut utxo_set = UtxoSet::new();
        let utxo = Utxo::new(Hash256::zero(), 0, 5000, vec![1, 2, 3], 100);
        let outpoint = utxo.outpoint();

        // Adicionar UTXO
        utxo_set.add_utxo(utxo.clone());
        assert!(utxo_set.contains(&outpoint));
        assert_eq!(utxo_set.len(), 1);

        // Obter UTXO
        let retrieved = utxo_set.get_utxo(&outpoint);
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), &utxo);

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
        utxo_set.add_utxo(Utxo::new(Hash256::zero(), 0, 1000, script.clone(), 100));
        utxo_set.add_utxo(Utxo::new(Hash256::zero(), 1, 2000, script.clone(), 100));
        utxo_set.add_utxo(Utxo::new(Hash256::zero(), 2, 3000, vec![4, 5, 6], 100));

        assert_eq!(utxo_set.get_balance_for_script(&script), 3000);
    }
}
