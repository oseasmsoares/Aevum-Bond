# 🏗️ Estrutura do Projeto Aevum & Bond

## 📋 Visão Geral da Arquitetura

O projeto Aevum & Bond implementa um **ecossistema blockchain dual** com criptografia pós-quântica, seguindo uma arquitetura modular em Rust baseada em workspaces.

## 📂 Estrutura de Diretórios

```
Aevum&Bond/
├── 📚 Documentação Base
│   ├── Especificações Tecnicas.txt      # Roadmap técnico completo
│   ├── White Paper Aevum & Bond.txt     # Documento conceitual
│   └── README.md                        # Documentação principal
│
├── 🦀 Código Fonte (Rust Workspace)
│   ├── Cargo.toml                       # Configuração do workspace
│   ├── src/main.rs                      # Demonstração integrada
│   │
│   ├── 🔗 shared/                       # Tipos e utilitários compartilhados
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                   # Exports públicos
│   │       ├── hash.rs                  # Hash256 com Keccak-256
│   │       └── error.rs                 # Tipos de erro da blockchain
│   │
│   ├── 🏛️ bond-core/                    # Implementação do protocolo Bond
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs                   # API pública do Bond
│   │       ├── blockchain.rs            # Lógica principal da blockchain
│   │       ├── block.rs                 # Estrutura de blocos e Merkle
│   │       ├── transaction.rs           # Sistema de transações
│   │       ├── utxo.rs                  # Sistema UTXO e gestão de estado
│   │       └── mining.rs                # Algoritmo de mineração PoW
│   │
│   └── 🌟 aevum-core/                   # Implementação do protocolo Aevum  
│       ├── Cargo.toml                   # (Sprint 6 - DPoS e contas)
│       └── src/lib.rs                   # Placeholder para futuro
│
├── 📖 docs/                             # Documentação técnica detalhada
│   ├── development-best-practices.md    # Metodologia IA + Agile
│   ├── project-structure.md            # Este arquivo
│   ├── agile-framework.md              # Framework Agile adaptado
│   ├── git-workflow.md                 # Workflow Git com IA
│   ├── security-performance.md         # Análises de segurança
│   └── copilot-claude-prompting-guide.md # Guia de prompting IA
│
├── ⚙️ .vscode/                          # Configuração VS Code
│   ├── settings.json                   # Rust + Copilot settings
│   ├── tasks.json                      # Build, test, lint tasks
│   ├── launch.json                     # Debug configurations
│   └── extensions.json                 # Extensões recomendadas
│
└── 🤖 .github/                          # GitHub automation
    ├── workflows/ci.yml                # CI/CD pipeline IA-enhanced
    └── pull_request_template.md        # Template para PRs
```

## 🧩 Arquitetura Modular

### **🔗 Shared Module**
```rust
// shared/src/lib.rs
pub mod hash;    // Hash256 wrapper para Keccak-256
pub mod error;   // Tipos de erro unificados

// Características:
// - Tipos fundamentais usados por todos os módulos
// - Zero dependências externas pesadas
// - Serialização consistente com serde
```

**Responsabilidades:**
- ✅ Hashing criptográfico Keccak-256
- ✅ Tipos de erro unificados
- ✅ Utilitários de serialização
- 🔜 Constantes de protocolo (Sprint 2+)

### **🏛️ Bond-Core Module**
```rust
// bond-core/src/lib.rs - API Principal
pub use blockchain::*;
pub use block::*;
pub use transaction::*;
pub use utxo::*;
pub use mining::*;

// Arquitetura:
// Blockchain -> Block -> Transaction -> UTXO
//     ↓           ↓         ↓          ↓
//   Mining  -> PoW Consensus -> State Management
```

**Responsabilidades:**
- ✅ **Blockchain**: Cadeia principal e validação
- ✅ **Block**: Estrutura de blocos com Merkle tree
- ✅ **Transaction**: Sistema de transações UTXO
- ✅ **UTXO**: Gestão de estado não gasto
- ✅ **Mining**: Algoritmo Proof-of-Work

### **🌟 Aevum-Core Module**
```rust
// aevum-core/src/lib.rs - Futuro DPoS
// Sprint 6: Modelo de contas + DPoS
// Sprint 7: Governança e staking
// Sprint 8: Ponte inter-ledger

// Arquitetura planejada:
// AccountState -> DPoS Consensus -> Governance
//      ↓              ↓               ↓
//   Staking -> Validator Set -> Vote Decay
```

**Responsabilidades (Futuras):**
- 🔜 **DPoS Consensus**: Algoritmo Delegated Proof-of-Stake
- 🔜 **Account Model**: Gestão de contas e balanços
- 🔜 **Governance**: Sistema de governança on-chain
- 🔜 **Bridge**: Ponte entre Bond e Aevum

## 📊 Dependências e Tecnologias

### **🦀 Core Dependencies**
```toml
[dependencies]
# 🔐 Criptografia
sha3 = "0.10"              # Keccak-256 hashing
hex = "0.4"                # Encoding hexadecimal

# 📦 Serialização  
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"         # JSON para APIs/debug

# ⏰ Temporização
chrono = { version = "0.4", features = ["serde"] }

# 🎲 Randomização
rand = "0.8"               # Para mineração e testes

# ❌ Error Handling
thiserror = "1.0"          # Macros de erro ergonômicos
```

### **🔮 Future Dependencies (por Sprint)**

**Sprint 2 - Criptografia PQC:**
```toml
pqcrypto = "0.19"          # ML-DSA (CRYSTALS-Dilithium)
# ou alternativa:
liboqs-rust = "0.9"        # liboqs bindings
```

**Sprint 3 - Rede P2P:**
```toml
libp2p = "0.53"            # Networking P2P
tokio = { version = "1.0", features = ["full"] }
prost = "0.12"             # Protocol Buffers
```

**Sprint 4 - Consenso:**
```toml
futures = "0.3"            # Async utilities
async-trait = "0.1"        # Trait objects assíncronos
```

**Sprint 5 - VM de Script:**
```toml
# Implementação própria da VM
# Stack-based, não-Turing-completa
```

## 🧪 Estrutura de Testes

### **📋 Estratégia de Testes Atual**
```
tests/
├── 🧪 Unit Tests (28 testes passando)
│   ├── shared::hash         # 4 testes
│   ├── bond-core::utxo      # 4 testes  
│   ├── bond-core::transaction # 4 testes
│   ├── bond-core::block     # 5 testes
│   ├── bond-core::mining    # 4 testes
│   ├── bond-core::blockchain # 6 testes
│   └── integration          # 1 teste
│
└── 🔮 Future Test Categories
    ├── Integration Tests    # Sprint 2+
    ├── Performance Tests    # Sprint 2+
    ├── Network Tests        # Sprint 3+
    └── End-to-End Tests     # Sprint 4+
```

### **📊 Cobertura de Testes**
```rust
// Exemplo de teste estruturado:
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_block_validation_complete() {
        // Arrange: Setup test data
        let block = create_test_block();
        let prev_block = create_genesis_block();
        
        // Act: Execute validation
        let result = validate_block(&block, &prev_block);
        
        // Assert: Verify correctness
        assert!(result.is_ok());
        
        // Edge cases testing
        test_invalid_scenarios();
    }
}
```

## 📈 Evolução Arquitetural

### **🎯 Sprint 1 (Atual) - Blockchain Local**
```rust
// Arquitetura implementada:
Blockchain {
    blocks: Vec<Block>,
    utxo_set: UTXOSet,
    network_params: NetworkParams,
}

// Funcionalidades:
✅ Hashing Keccak-256
✅ Mineração PoW  
✅ Sistema UTXO
✅ Validação de blocos
```

### **🔮 Sprint 2 - Segurança PQC**
```rust
// Arquitetura expandida:
Transaction {
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
    pqc_signatures: Vec<MLDSASignature>, // NOVO
}

MLDSASignature {
    signature_data: Vec<u8>,
    public_key: MLDSAPublicKey,
    algorithm: SignatureAlgorithm::MLDSA65,
}
```

### **🌐 Sprint 3 - Rede P2P**
```rust
// Arquitetura distribuída:
Node {
    swarm: Swarm<NetworkBehaviour>,
    blockchain: Blockchain,
    mempool: TransactionPool,
    peer_manager: PeerManager,
}
```

### **⚡ Sprint 6+ - Aevum DPoS**
```rust
// Arquitetura dual-blockchain:
AevumChain {
    state: HashMap<Address, AccountState>,
    validators: ValidatorSet,
    consensus: DPoSConsensus,
    bridge: BondBridge,
}
```

## 🔧 Configurações de Desenvolvimento

### **⚙️ VS Code Workspace**
```json
// .vscode/settings.json highlights:
{
    "rust-analyzer.checkOnSave.command": "clippy",
    "github.copilot.enable": { "rust": true },
    "editor.formatOnSave": true
}
```

### **🏗️ Build System**
```bash
# Tasks automatizadas disponíveis:
🦀 Build All      # cargo build --workspace  
🧪 Test All       # cargo test --workspace
🔍 Clippy Check   # cargo clippy --workspace
🚀 Sprint Demo    # cargo run (demonstração)
🔧 Format Code    # cargo fmt --all
```

## 📋 Pontos de Extensão

### **🔌 Plugin Architecture (Futuro)**
```rust
// Sprint 5+ - Sistema de plugins
trait ScriptOpcode {
    fn execute(&self, stack: &mut Stack) -> Result<(), VMError>;
}

// Sprint 9+ - Carteira modular
trait WalletBackend {
    fn sign_transaction(&self, tx: &Transaction) -> Result<Signature, Error>;
    fn get_balance(&self, address: &Address) -> Result<u64, Error>;
}
```

### **🌍 Internacionalização (Sprint 13+)**
```rust
// Estrutura preparada para i18n:
pub enum Language {
    English,
    Portuguese,
    // Expansível
}
```

## 🎯 Conclusão

A arquitetura do Aevum & Bond foi projetada para:

### **✅ Modularidade**
- Cada módulo tem responsabilidades bem definidas
- Acoplamento baixo, coesão alta
- Fácil adição de novas funcionalidades

### **🔒 Segurança**
- Criptografia pós-quântica desde a fundação
- Validação rigorosa em todas as camadas
- Gestão segura de memória (Rust)

### **📈 Escalabilidade**
- Arquitetura preparada para 14 sprints
- Dual-blockchain (Bond UTXO + Aevum Account)
- Rede P2P robusta

### **🧪 Testabilidade**
- Testes unitários abrangentes (28 testes passando)
- Estrutura preparada para testes de integração
- Métricas de cobertura e performance

**A arquitetura atual fornece uma base sólida para evolução até uma blockchain de produção de classe mundial! 🚀**

---

*Última atualização: Sprint 1 concluído - 15 de setembro de 2025*