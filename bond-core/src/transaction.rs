use crate::utxo::OutPoint;
use serde::{Deserialize, Serialize};
use shared::{BlockchainError, Hash256, Result};

/// Input de transação
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxInput {
    /// Referência ao UTXO sendo gasto
    pub previous_output: OutPoint,
    /// Script que prova o direito de gastar o UTXO
    /// Por enquanto usaremos um placeholder até implementar ML-DSA
    pub script_sig: Vec<u8>,
    /// Número de sequência (para controle de tempo/reposição)
    pub sequence: u32,
}

impl TxInput {
    /// Cria um novo input de transação
    #[must_use]
    pub const fn new(previous_output: OutPoint, script_sig: Vec<u8>, sequence: u32) -> Self {
        Self {
            previous_output,
            script_sig,
            sequence,
        }
    }

    /// Cria um input de coinbase (primeira transação de um bloco)
    #[must_use]
    pub const fn coinbase(script_sig: Vec<u8>) -> Self {
        Self {
            previous_output: OutPoint {
                txid: Hash256::zero(),
                vout: 0xFFFF_FFFF,
            },
            script_sig,
            sequence: 0xFFFF_FFFF,
        }
    }

    /// Verifica se este input é de uma transação coinbase
    #[must_use]
    pub fn is_coinbase(&self) -> bool {
        self.previous_output.txid == Hash256::zero() && self.previous_output.vout == 0xFFFF_FFFF
    }
}

/// Output de transação
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TxOutput {
    /// Valor em Elos
    pub value: u64,
    /// Script que define as condições para gastar este output
    pub script_pubkey: Vec<u8>,
}

impl TxOutput {
    /// Cria um novo output de transação
    #[must_use]
    pub const fn new(value: u64, script_pubkey: Vec<u8>) -> Self {
        Self {
            value,
            script_pubkey,
        }
    }
}

/// Transação na blockchain Bond
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transaction {
    /// Versão da transação
    pub version: u32,
    /// Inputs da transação
    pub inputs: Vec<TxInput>,
    /// Outputs da transação
    pub outputs: Vec<TxOutput>,
    /// Lock time (tempo mínimo para incluir em bloco)
    pub lock_time: u32,
}

impl Transaction {
    /// Cria uma nova transação
    #[must_use]
    pub const fn new(
        version: u32,
        inputs: Vec<TxInput>,
        outputs: Vec<TxOutput>,
        lock_time: u32,
    ) -> Self {
        Self {
            version,
            inputs,
            outputs,
            lock_time,
        }
    }

    /// Cria uma transação de coinbase (primeira transação de um bloco)
    #[must_use]
    pub fn coinbase(block_height: u64, reward: u64, script_pubkey: Vec<u8>) -> Self {
        // Script sig contém a altura do bloco para prevenir duplicação
        let script_sig = block_height.to_le_bytes().to_vec();

        let inputs = vec![TxInput::coinbase(script_sig)];
        let outputs = vec![TxOutput::new(reward, script_pubkey)];

        Self::new(1, inputs, outputs, 0)
    }

    /// Calcula o hash da transação (SHA3-256)
    ///
    /// # Errors
    ///
    /// Retorna erro se a serialização da transação falhar
    pub fn hash(&self) -> Result<Hash256> {
        let serialized = serde_json::to_vec(self)
            .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
        Ok(Hash256::keccak256(&serialized))
    }

    /// Verifica se é uma transação de coinbase
    #[must_use]
    pub fn is_coinbase(&self) -> bool {
        self.inputs.len() == 1 && self.inputs[0].is_coinbase()
    }

    /// Calcula o valor total dos inputs
    /// Note: Para UTXOs reais, precisaríamos consultar o UTXO set
    ///
    /// # Errors
    ///
    /// Retorna erro se alguma UTXO não for encontrada ou se houver overflow
    pub fn total_input_value(&self, utxo_set: &crate::utxo::UtxoSet) -> Result<u64> {
        let mut total = 0u64;

        for input in &self.inputs {
            if input.is_coinbase() {
                continue; // Inputs de coinbase não referenciam UTXOs existentes
            }

            let utxo = utxo_set
                .get_utxo(&input.previous_output)
                .ok_or(BlockchainError::UtxoNotFound)?;

            total = total.checked_add(utxo.value).ok_or_else(|| {
                BlockchainError::InvalidTransaction("Input value overflow".to_string())
            })?;
        }

        Ok(total)
    }

    /// Calcula o valor total dos outputs
    ///
    /// # Errors
    ///
    /// Retorna erro se houver overflow no cálculo dos valores
    pub fn total_output_value(&self) -> Result<u64> {
        let mut total = 0u64;

        for output in &self.outputs {
            total = total.checked_add(output.value).ok_or_else(|| {
                BlockchainError::InvalidTransaction("Output value overflow".to_string())
            })?;
        }

        Ok(total)
    }

    /// Calcula a taxa paga pela transação
    ///
    /// # Errors
    ///
    /// Retorna erro se os valores não forem consistentes ou se houver overflow
    pub fn fee(&self, utxo_set: &crate::utxo::UtxoSet) -> Result<u64> {
        if self.is_coinbase() {
            return Ok(0); // Transações de coinbase não pagam taxa
        }

        let input_value = self.total_input_value(utxo_set)?;
        let output_value = self.total_output_value()?;

        input_value
            .checked_sub(output_value)
            .ok_or_else(|| BlockchainError::InvalidTransaction("Negative fee".to_string()))
    }

    /// Validação básica da transação
    ///
    /// # Errors
    ///
    /// Retorna erro se a transação não atender aos critérios básicos de validação
    pub fn validate_basic(&self) -> Result<()> {
        // Verificar se não está vazia
        if self.inputs.is_empty() || self.outputs.is_empty() {
            return Err(BlockchainError::InvalidTransaction(
                "Empty inputs or outputs".to_string(),
            ));
        }

        // Verificar overflow nos outputs
        self.total_output_value()?;

        // Verificar valores positivos nos outputs
        for output in &self.outputs {
            if output.value == 0 {
                return Err(BlockchainError::InvalidTransaction(
                    "Zero value output".to_string(),
                ));
            }
        }

        // Para transações não-coinbase, verificar que não há inputs de coinbase
        if !self.is_coinbase() {
            for input in &self.inputs {
                if input.is_coinbase() {
                    return Err(BlockchainError::InvalidTransaction(
                        "Non-coinbase transaction with coinbase input".to_string(),
                    ));
                }
            }
        }

        // Para transações de coinbase, verificar que há exatamente um input
        if self.is_coinbase() && self.inputs.len() != 1 {
            return Err(BlockchainError::InvalidTransaction(
                "Coinbase transaction must have exactly one input".to_string(),
            ));
        }

        Ok(())
    }

    /// Tamanho estimado da transação em bytes
    #[must_use]
    pub const fn estimated_size(&self) -> usize {
        // Estimativa baseada na serialização JSON (simplificada)
        // Na implementação real com ML-DSA, seria muito maior
        let base_size = 4 + 4; // version + lock_time
        let inputs_size = self.inputs.len() * (32 + 4 + 100 + 4); // txid + vout + script_sig + sequence
        let outputs_size = self.outputs.len() * (8 + 100); // value + script_pubkey

        base_size + inputs_size + outputs_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utxo::{Utxo, UtxoSet};

    #[test]
    fn test_coinbase_transaction() {
        let coinbase = Transaction::coinbase(100, 5000, vec![1, 2, 3]);

        assert!(coinbase.is_coinbase());
        assert_eq!(coinbase.inputs.len(), 1);
        assert_eq!(coinbase.outputs.len(), 1);
        assert_eq!(coinbase.outputs[0].value, 5000);
    }

    #[test]
    fn test_transaction_hash() {
        let coinbase = Transaction::coinbase(100, 5000, vec![1, 2, 3]);
        let hash = coinbase.hash().unwrap();

        // Hash não deve ser zero para transação válida
        assert_ne!(hash, Hash256::zero());

        // Hash deve ser determinístico
        let hash2 = coinbase.hash().unwrap();
        assert_eq!(hash, hash2);
    }

    #[test]
    fn test_transaction_validation() {
        let coinbase = Transaction::coinbase(100, 5000, vec![1, 2, 3]);
        assert!(coinbase.validate_basic().is_ok());

        // Transação com outputs vazios deve falhar
        let empty_tx = Transaction::new(1, vec![], vec![], 0);
        assert!(empty_tx.validate_basic().is_err());
    }

    #[test]
    fn test_transaction_values() {
        let coinbase = Transaction::coinbase(100, 5000, vec![1, 2, 3]);

        // Valor total dos outputs
        assert_eq!(coinbase.total_output_value().unwrap(), 5000);

        // Coinbase não paga taxa
        let utxo_set = UtxoSet::new();
        assert_eq!(coinbase.fee(&utxo_set).unwrap(), 0);
    }

    #[test]
    fn test_regular_transaction_fee() {
        let mut utxo_set = UtxoSet::new();
        let txid = Hash256::zero();

        // Criar UTXO para gastar
        let utxo = Utxo::new(txid, 0, 1000, vec![1, 2, 3], 100);
        let outpoint = utxo.outpoint();
        utxo_set.add_utxo(utxo);

        // Criar transação que gasta o UTXO
        let input = TxInput::new(outpoint, vec![4, 5, 6], 0);
        let output = TxOutput::new(900, vec![7, 8, 9]); // 100 de taxa
        let tx = Transaction::new(1, vec![input], vec![output], 0);

        assert_eq!(tx.fee(&utxo_set).unwrap(), 100);
    }
}
