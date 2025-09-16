# 🔧 Especificações Técnicas - Aevum & Bond

**Versão:** 1.1  
**Para:** Engenheiro de Software Principal  
**Data:** 14 de abril de 2025

---

## 📋 Visão Geral

Este documento detalha as especificações técnicas para a implementação do ecossistema **Aevum & Bond**. Está organizado de acordo com o roadmap de desenvolvimento e serve como um **guia de implementação sprint a sprint**. O objetivo é traduzir os conceitos do White Paper em tarefas de engenharia acionáveis, fornecendo contexto técnico e justificativas para as decisões de arquitetura.

---

## 🛠️ Pilha Tecnológica Principal

### Linguagens & Frameworks
- **Linguagem:** Rust (Edição 2021+)
  - Escolhido pelas suas garantias de segurança de memória, performance e ecossistema robusto para sistemas concorrentes
  - Cruciais para o desenvolvimento de blockchain

### Networking
- **Rede P2P:** `rust-libp2p`
  - Framework modular e agnóstico a transporte
  - Facilita descoberta de nós, pub/sub e streams de comunicação direta

### Criptografia
- **Criptografia PQC:** `pqcrypto` ou `liboqs-rust`
  - Conformidade com padrões NIST
  - Monitorização de alternativas puras em Rust recomendada

### Interface
- **CLI:** `clap`
  - Construtor de argumentos idiomático em Rust
  - Ideal para interfaces de nós e ferramentas de mineração

- **Carteira Desktop:** Tauri
  - Alternativa segura e leve ao Electron
  - Backend em Rust + Frontend nativo do sistema
  - Performance e segurança de ponta a ponta

### Financiamento
- **Venda de Gênese:** Carteira multi-assinatura (Gnosis Safe) na rede Arbitrum
  - Segurança e baixos custos para contribuidores
  - Distribuição de tokens AEV no Bloco Gênese

---

## 📅 Cronograma de Sprints e Tarefas Técnicas

**Data de Início:** 1 de maio de 2025

### 🏗️ Sprint 1: Fundação do Núcleo (1-14 mai 2025)

**Objetivo:** Construir uma simulação de blockchain local funcional com mineração (Protocolo Bond)

#### Tarefa 1.1: Implementar Estruturas de Dados Essenciais

```rust
#[derive(Serialize, Deserialize)]
struct Block {
    index: u64,
    timestamp: u64,
    prev_block_hash: Vec<u8>,
    hash: Vec<u8>,
    nonce: u64,
    transactions: Vec<Transaction>,
    merkle_root: Vec<u8>,
}

#[derive(Serialize, Deserialize)]
struct Transaction {
    id: Vec<u8>,
    inputs: Vec<TxInput>,
    outputs: Vec<TxOutput>,
}

#[derive(Serialize, Deserialize)]
struct TxInput {
    prev_tx_id: Vec<u8>,
    output_index: u32,
    unlocking_script: Vec<u8>, // assinatura + chave pública
}

#[derive(Serialize, Deserialize)]
struct TxOutput {
    value: u64,
    locking_script: Vec<u8>, // condições para gastar
}
```

- Implementar serialização binária canônica usando `bincode` ou `borsh`
- Garantir representação consistente em todos os nós

#### Tarefa 1.2: Integrar Hashing

- Utilizar crate `sha3` para implementação de Keccak-256
- Criar função `calculate_merkle_root` para transações no bloco
- Implementar função `calculate_hash` para struct Block
- Operar sobre representação binária canônica do cabeçalho

#### Tarefa 1.3: Implementar Algoritmo de Mineração PoW

```rust
fn mine_block(block: &mut Block, difficulty_target: U256)
```

- Dificuldade como alvo numérico (não número de zeros)
- Loop iterando sobre nonce até hash ≤ difficulty_target

#### Tarefa 1.4: Criar Testes Unitários Abrangentes

- Framework nativo do Rust `#[cfg(test)]`
- Testes "round-trip" para serialização
- Validação de integridade: modificação invalida hash e PoW
- Confirmação: loop encontra nonce válido para dificuldade conhecida

---

### 🔐 Sprint 2: Segurança Pós-Quântica (15-28 mai)

**Objetivo:** Integrar criptografia pós-quântica para assinatura e validação de transações

#### Tarefa 2.1: Integrar Biblioteca Criptográfica PQC

- Integrar crate `pqcrypto` 
- Esquema: **ML-DSA-65** (CRYSTALS-Dilithium Nível 3)
- Criar módulo `crypto` abstrato
- Isolar detalhes de implementação externa

#### Tarefa 2.2: Desenvolver Gestão de Chaves Segura

```rust
fn generate_keypair() -> (PublicKey, PrivateKey)

// Wrappers com trait Drop para zerar memória
impl Drop for PrivateKey {
    fn drop(&mut self) {
        // Zerar memória seguramente
    }
}
```

#### Tarefa 2.3: Implementar Lógica de Assinatura e Verificação

```rust
fn sign_transaction(transaction_id: &[u8], private_key: &PrivateKey) -> Signature
fn verify_signature(transaction_id: &[u8], signature: &Signature, public_key: &PublicKey) -> bool
```

- ID da transação: hash de todos os campos exceto assinaturas
- Validação completa: iterar sobre TxInput, extrair assinatura/chave, verificar

#### Tarefa 2.4: Criar Testes de Validação Criptográfica

- Validação de assinaturas legítimas
- Tentativas com chave pública incorreta
- Modificação de byte → verificação falha (anti-maleabilidade)
- Benchmarks básicos para operações criptográficas

---

### 🌐 Sprint 3: Fundação da Rede P2P (29 mai - 11 jun)

**Objetivo:** Rede de descoberta e comunicação entre nós

#### Tarefa 3.1: Configurar Nó P2P Básico

- `rust-libp2p` para estrutura do nó
- Swarm: transporte (TCP) + comportamento (NetworkBehaviour) + PeerId
- PeerId gerado a partir de Ed25519 (identificação, não transações)

#### Tarefa 3.2: Implementar Descoberta de Nós

- **mDNS**: descoberta "plug-and-play" em rede local
- **Futuro**: Kademlia DHT para descoberta robusta na Internet

#### Tarefa 3.3: Configurar Comunicação via Gossipsub

- Tópicos versionados:
  - `/aevum-bond/1.0/blocks`
  - `/aevum-bond/1.0/transactions`
- Estruturas de mensagem com `prost` (Protocol Buffers)
- Serialização eficiente com schemas definidos

#### Tarefa 3.4: Desenvolver CLI do Nó

```bash
# Comandos iniciais
start-node --port <PORT>
network info        # listar peers conectados
chain status        # altura da cadeia local
```

---

### ⚖️ Sprint 4: Consenso Descentralizado (12-25 jun)

**Objetivo:** Integrar blockchain na rede P2P → primeira Testnet interna

#### Tarefa 4.1: Integrar Blockchain e Rede

```rust
struct ChainState {
    blocks: Vec<Block>,
    utxos: HashMap<Vec<u8>, TxOutput>,
    mempool: Vec<Transaction>,
}
```

**Validação ao receber NewBlock via Gossipsub:**
1. PoW válido?
2. prev_block_hash corresponde ao bloco topo?
3. Todas as transações válidas (assinaturas + scripts)?
4. ✅ Aceitar bloco

#### Tarefa 4.2: Implementar Sincronização da Cadeia (IBD)

- Mecanismo request/response em libp2p
- Processo de sincronização:
  1. Perguntar altura da cadeia aos peers
  2. Se peer tem cadeia mais longa → pedir blocos em lotes (100x)
  3. **Regra:** sempre seguir cadeia válida mais longa

#### Tarefa 4.3: Iniciar Manual Técnico do Operador de Nó

**Arquivo:** `MANUAL_OPERADOR_NÓ_v0.9.md`

**Seções:**
- Compilar a partir do código-fonte
- Configurar e executar nó pela primeira vez
- Comandos da CLI e significados
- Resolução de problemas de conexão

---

## 🚀 Sprints Futuros (Resumo das Tarefas Chave)

### Sprint 5: Transações Programáveis
**VM de Script não-Turing-completa baseada em stack**

**Opcodes Essenciais:**
- `OP_DUP`, `OP_HASH256`, `OP_CHECKSIG` (ML-DSA)
- `OP_EQUALVERIFY`, `OP_PUSHDATA`

**Validação:** unlocking_script + locking_script → VM → resultado `true` na stack

### Sprint 6: Fundação do Aevum
**Blockchain Aevum com modelo de contas**

```rust
struct AccountState {
    nonce: u64,
    balance: u128,
    // Futuro: code_hash, storage_root
}
```

- **Consenso DPoS:** eleição baseada em peso de stake
- **Criptografia:** ML-DSA-44 (Nível 1) para alto rendimento

### Sprint 7: Governança e Staking do Aevum
**Transações especiais de sistema:**
- `stake`, `unstake`, `delegate`, `vote`
- **Vote Decay:** peso do stake diminui por época (governança ativa)

### Sprint 8: A Ponte Inter-Ledger
**Protocolo de ponte trust-minimized**
- Bond: pUTXO especial gasto por maioria validadores Aevum
- Aevum: cunhagem de wBND correspondente
- Explorador: backend conectado a ambas as redes via RPC

### Sprint 9: Ferramentas do Utilizador
**Carteira Desktop (Tauri)**
- Backend Rust: lógica criptográfica + comunicação com nós
- Frontend: UI via ponte IPC segura
- Seed phrase BIP39, construção/assinatura de transações

### Sprint 10: Endurecimento e Auditoria
- `cargo-fuzz` para parsing/deserialização
- Testnet privada longa duração (VMs geográficas)
- Documentação de arquitetura para auditoria externa

### Sprint 11: Empacotamento e Documentação Final
**Pipeline CI/CD (GitHub Actions)**
- Compilação multiplataforma: Windows, macOS (x86/ARM), Linux
- Testes automatizados + binários assinados
- Artefatos de release

### Sprint 12: Lançamento da Mainnet
**Script de geração do Bloco Gênese**
- Input: `genesis.json` (parâmetros + alocação Venda Gênese)
- Output: `genesis.dat` para operadores de nós iniciais

### Sprint 13: Website e Presença Online
- Infraestrutura de deploy + DNS (Cloudflare)
- Portal de documentação `mdBook` automático
- Integração com repositório principal

### Sprint 14: Marketing e Financiamento Inicial
**Carteira Multi-assinatura 3-de-5 na Arbitrum**
- Script de monitorização de depósitos em tempo real
- Export seguro: lista de transações/endereços → input Bloco Gênese

---

## 🏁 Marco de Conclusão

Ao final destes 14 sprints, teremos um **ecossistema blockchain completo e funcional** com:

✅ **Segurança Pós-Quântica** nativa  
✅ **Dual-ledger** otimizado (Bond + Aevum)  
✅ **Abstração de Contas** com recuperação social  
✅ **Ponte Inter-ledger** funcional  
✅ **Ferramentas de usuário** completas  
✅ **Documentação técnica** abrangente  
✅ **Pipeline de produção** automatizado  

**Resultado:** Base sólida para um sistema financeiro global, seguro e à prova de futuro.
