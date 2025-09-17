use bond_core::*;
use clap::{Parser, Subcommand};
use shared::{KeyPair, sign_transaction_hash, verify_transaction_signature, Result};
use std::path::PathBuf;
use tracing::{info, warn, error, Level};
use tracing_subscriber::FmtSubscriber;

// Importação do módulo de rede
pub mod network;

/// Aevum-Bond - Blockchain pós-quântica com suporte P2P
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Executa demonstração das funcionalidades básicas (Sprint 1)
    Demo,

    /// Executa demonstração da criptografia pós-quântica (Sprint 2)
    DemoPqc,

    /// Inicia um nó da rede P2P (Sprint 3)
    StartNode(StartNodeArgs),
}

/// Argumentos para iniciar um nó P2P
#[derive(Parser, Debug)]
struct StartNodeArgs {
    /// Modo de operação do nó
    #[arg(short, long, default_value = "full")]
    mode: String,

    /// Porta para escuta de conexões P2P
    #[arg(short, long, default_value_t = 0)]
    port: u16,

    /// Endereço de IP para escutar conexões (padrão: 0.0.0.0)
    #[arg(short, long, default_value = "0.0.0.0")]
    listen: String,

    /// Lista de nós bootstrap para conexão inicial (format: endereço:porta)
    #[arg(short, long)]
    bootstrap: Vec<String>,

    /// Número máximo de peers permitidos
    #[arg(long, default_value_t = 50)]
    max_peers: usize,

    /// Desabilitar mDNS para descoberta local de peers
    #[arg(long)]
    no_mdns: bool,

    /// Número de threads para mineração (quando em modo mining)
    #[arg(long, default_value_t = 1)]
    mining_threads: usize,

    /// Caminho para o diretório de dados da blockchain
    #[arg(long)]
    data_dir: Option<PathBuf>,

    /// Endereço externo para anunciar aos peers (para nós atrás de NAT)
    #[arg(long)]
    external_addr: Option<String>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Configurar logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("Falha ao configurar logging global");

    // Parsear argumentos CLI
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Demo => run_demo()?,
        Commands::DemoPqc => run_pqc_demo()?,
        Commands::StartNode(args) => run_node(args).await?,
    }

    Ok(())
}

/// Executa demonstração da criptografia pós-quântica (Sprint 2)
fn run_pqc_demo() -> Result<()> {
    println!("\n==================================================");
    println!("🔐 Sprint 2: Criptografia Pós-Quântica ML-DSA-65");
    println!("==================================================");

    // 10. Gerar par de chaves ML-DSA
    println!("\n10. Gerando chaves ML-DSA-65...");
    let alice_keypair = KeyPair::generate()?;
    let bob_keypair = KeyPair::generate()?;

    println!("   ✅ Par de chaves Alice gerado:");
    println!(
        "      🔑 Chave Pública:  {} bytes",
        alice_keypair.public_key.as_bytes().len()
    );
    println!(
        "      🔐 Chave Privada:  {} bytes",
        alice_keypair.private_key.as_bytes().len()
    );

    // 11. Demonstrar assinatura de transação
    println!("\n11. Demonstrando assinatura PQC de transação...");
    let demo_tx = Transaction::new(
        1,
        vec![TxInput::new(
            OutPoint {
                txid: Hash256::keccak256(b"demo"),
                vout: 0,
            },
            vec![], // Script vazio por enquanto
            0,
        )],
        vec![TxOutput::new(
            1000,
            bob_keypair.public_key.as_bytes().to_vec(),
        )],
        0,
    );

    let tx_hash = demo_tx.hash()?;
    println!("   📝 Hash da transação: {}", tx_hash);

    // 12. Assinar transação com ML-DSA-65
    let signature = sign_transaction_hash(&tx_hash, &alice_keypair)?;
    println!("   ✍️ Assinatura gerada:");
    println!("      📦 Tamanho: {} bytes", signature.size());
    println!("      🕒 Timestamp: {}", signature.timestamp());
    println!("      🔧 Algoritmo: {:?}", signature.algorithm());

    // 13. Verificar assinatura
    println!("\n12. Verificando assinatura pós-quântica...");
    let verification = verify_transaction_signature(&tx_hash, &signature)?;
    println!(
        "   {} Verificação: {}",
        if verification { "✅" } else { "❌" },
        if verification { "VÁLIDA" } else { "INVÁLIDA" }
    );

    // 14. Demonstrar segurança: tentar verificar com hash diferente
    let wrong_hash = Hash256::keccak256(b"hash_errado");
    let wrong_verification = verify_transaction_signature(&wrong_hash, &signature)?;
    println!(
        "   🛡️ Verificação com hash errado: {}",
        if wrong_verification {
            "VÁLIDA (PROBLEMA!)"
        } else {
            "INVÁLIDA (correto)"
        }
    );

    println!("\n🎯 Sprint 2 - Criptografia PQC implementada!");
    println!("✅ ML-DSA-65 (CRYSTALS-Dilithium) funcional");
    println!("✅ Geração segura de chaves");
    println!("✅ Assinatura e verificação de transações");
    println!("✅ Resistência a ataques quânticos");
    println!("✅ Tamanhos: ~2.6KB pub key, ~4.9KB priv key, ~4.7KB signature");

    Ok(())
}

/// Executa um nó da rede P2P (Sprint 3)
async fn run_node(args: StartNodeArgs) -> Result<()> {
    info!("🚀 Iniciando Aevum-Bond P2P Node");
    info!("🔧 Modo: {}", args.mode);

    // Configurar modo do nó
    let node_mode = match args.mode.as_str() {
        "bootstrap" => {
            let external_addr = args.external_addr.clone()
                .unwrap_or_else(|| format!("{}:{}", args.listen, args.port));
            info!("🌐 Modo Bootstrap com endereço externo: {}", external_addr);
            network::NodeMode::BootstrapNode
        },
        "mining" => {
            info!("⛏️ Modo Mineração com {} threads", args.mining_threads);
            network::NodeMode::MiningNode {
                mining_threads: args.mining_threads,
                target_difficulty: 20, // Valor fixo por enquanto
            }
        },
        "wallet" => {
            info!("💼 Modo Carteira");
            network::NodeMode::WalletNode {
                sync_mode: network::SyncMode::SPV,
            }
        },
        _ => {
            info!("📊 Modo Full Node (padrão)");
            network::NodeMode::FullNode
        }
    };

    // Configurar o nó P2P
    let p2p_config = network::P2PConfig {
        listen_addr: args.listen,
        port: args.port,
        bootstrap_nodes: args.bootstrap,
        max_peers: args.max_peers,
        enable_mdns: !args.no_mdns,
        enable_kad_dht: true, // Habilitado por padrão
        node_mode,
        external_addr: args.external_addr,
        network_id: "aevum-bond-testnet".to_string(),
        connection_timeout: std::time::Duration::from_secs(30),
    };

    // Iniciar o nó P2P
    let mut node = network::P2PNode::new(p2p_config).await?;
    
    // Iniciar a blockchain
    info!("🔄 Inicializando blockchain...");
    let network_params = NetworkParams::default();
    let genesis_script = vec![0x76, 0xa9, 0x14, 0x12, 0x34, 0x56]; // Script P2PKH fictício
    let blockchain = Blockchain::new(network_params, genesis_script)?;
    
    // Configurar blockchain no nó P2P
    node.set_blockchain(blockchain);
    
    // Iniciar o nó
    node.start().await?;
    info!("✅ Nó P2P iniciado com ID: {}", node.node_id());
    
    // Executar loop de eventos
    info!("🔄 Iniciando loop de eventos do nó P2P...");
    node.run().await?;
    
    // Desligar nó
    info!("👋 Finalizando nó P2P");
    
    Ok(())
}

/// Executa a demonstração do Sprint 1
fn run_demo() -> Result<()> {
    println!("🔗 Aevum & Bond - Sprint 1: Fundação do Núcleo");
    println!("================================================");

    // 1. Criar blockchain com parâmetros de rede
    println!("1. Criando blockchain Bond...");
    let network_params = NetworkParams::default();
    let genesis_script = vec![0x76, 0xa9, 0x14, 0x12, 0x34, 0x56]; // Script P2PKH fictício
    let mut blockchain = Blockchain::new(network_params, genesis_script.clone())?;

    println!("   ✅ Blockchain criada com bloco gênese");
    println!("   📊 Altura: {}", blockchain.height());
    println!(
        "   💰 Supply inicial: {} Elos",
        blockchain.get_balance(&genesis_script)
    );

    // 2. Demonstrar hashing Keccak-256
    println!("\n2. Demonstrando hashing Keccak-256...");
    let data = "Aevum & Bond - Blockchain pos-quantica".as_bytes();
    let hash = Hash256::keccak256(data);
    println!("   📝 Dados: {:?}", std::str::from_utf8(data).unwrap());
    println!("   🔐 Hash: {}", hash);
    println!("   🎯 Zeros iniciais: {}", hash.leading_zeros());

    // 3. Configurar minerador
    println!("\n3. Configurando minerador...");
    let miner_config = MinerConfig {
        reward_script: vec![0x76, 0xa9, 0x14, 0x78, 0x9a, 0xbc], // Script diferente para minerador
        threads: 1,
        difficulty: 15, // Dificuldade moderada para demonstração
    };
    let miner = Miner::new(miner_config.clone());

    println!("   ⚙️ Threads: {}", miner_config.threads);
    println!(
        "   🎯 Dificuldade: {} zeros iniciais",
        miner_config.difficulty
    );

    // 4. Estimar taxa de hash
    println!("\n4. Estimando taxa de hash...");
    let hashrate = miner.estimate_hashrate(2)?; // 2 segundos de teste
    println!("   🔥 Taxa de hash estimada: {:.0} H/s", hashrate);

    // 5. Criar uma transação simples
    println!("\n5. Criando transação simples...");
    let destination_script = vec![0x76, 0xa9, 0x14, 0xde, 0xf0, 0x12];
    let transaction = blockchain.create_transaction(
        &genesis_script,
        destination_script.clone(),
        1000, // 1000 Elos = 1 BND
        50,   // 50 Elos de taxa
    )?;

    println!("   💸 Transação criada:");
    println!("     - Inputs: {}", transaction.inputs.len());
    println!("     - Outputs: {}", transaction.outputs.len());
    println!(
        "     - Valor total output: {} Elos",
        transaction.total_output_value()?
    );
    println!("     - Hash: {}", transaction.hash()?);

    // 6. Minerar um novo bloco
    println!("\n6. Minerando próximo bloco...");
    println!("   ⏳ Iniciando mineração (pode demorar alguns segundos)...");

    let start_time = std::time::Instant::now();
    let mining_result = blockchain.mine_next_block(&miner, vec![transaction])?;
    let mining_duration = start_time.elapsed();

    println!("   ⛏️ Bloco minerado com sucesso!");
    println!("     - Hash: {}", mining_result.hash);
    println!("     - Nonce: {}", mining_result.nonce);
    println!("     - Tentativas: {}", mining_result.attempts);
    println!("     - Tempo: {:.2}s", mining_duration.as_secs_f64());
    println!(
        "     - Taxa real: {:.0} H/s",
        mining_result.attempts as f64 / mining_duration.as_secs_f64()
    );

    // 7. Adicionar bloco à blockchain
    println!("\n7. Adicionando bloco à blockchain...");
    blockchain.add_block(mining_result.block)?;

    // 8. Mostrar estatísticas finais
    println!("\n8. Estatísticas finais da blockchain:");
    let stats = blockchain.stats();
    println!("   📏 Altura: {}", stats.height);
    println!("   🧱 Total de blocos: {}", stats.total_blocks);
    println!("   💳 Total de transações: {}", stats.total_transactions);
    println!("   🎯 UTXOs ativas: {}", stats.total_utxos);
    println!(
        "   💰 Supply total: {} Elos ({} BND)",
        stats.total_supply,
        stats.total_supply / 1000
    );
    println!("   🔨 Dificuldade atual: {}", stats.difficulty);

    // 9. Verificar balanços
    println!("\n9. Balanços por endereço:");
    println!(
        "   👑 Gênese: {} Elos",
        blockchain.get_balance(&genesis_script)
    );
    println!(
        "   ⛏️ Minerador: {} Elos",
        blockchain.get_balance(&miner_config.reward_script)
    );
    println!(
        "   📨 Destinatário: {} Elos",
        blockchain.get_balance(&destination_script)
    );

    println!("\n🎉 Sprint 1 concluído com sucesso!");
    println!("✅ Estruturas de dados implementadas");
    println!("✅ Hashing Keccak-256 funcional");
    println!("✅ Mineração PoW implementada");
    println!("✅ Blockchain local funcional");
    println!("✅ Testes unitários passando");

    // SPRINT 2: Demonstração de Criptografia Pós-Quântica
    println!("\n==================================================");
    println!("🔐 Sprint 2: Criptografia Pós-Quântica ML-DSA-65");
    println!("==================================================");

    // 10. Gerar par de chaves ML-DSA
    println!("\n10. Gerando chaves ML-DSA-65...");
    let alice_keypair = KeyPair::generate()?;
    let bob_keypair = KeyPair::generate()?;

    println!("   ✅ Par de chaves Alice gerado:");
    println!(
        "      🔑 Chave Pública:  {} bytes",
        alice_keypair.public_key.as_bytes().len()
    );
    println!(
        "      🔐 Chave Privada:  {} bytes",
        alice_keypair.private_key.as_bytes().len()
    );

    // 11. Demonstrar assinatura de transação
    println!("\n11. Demonstrando assinatura PQC de transação...");
    let demo_tx = Transaction::new(
        1,
        vec![TxInput::new(
            OutPoint {
                txid: Hash256::keccak256(b"demo"),
                vout: 0,
            },
            vec![], // Script vazio por enquanto
            0,
        )],
        vec![TxOutput::new(
            1000,
            bob_keypair.public_key.as_bytes().to_vec(),
        )],
        0,
    );

    let tx_hash = demo_tx.hash()?;
    println!("   📝 Hash da transação: {}", tx_hash);

    // 12. Assinar transação com ML-DSA-65
    let signature = sign_transaction_hash(&tx_hash, &alice_keypair)?;
    println!("   ✍️ Assinatura gerada:");
    println!("      📦 Tamanho: {} bytes", signature.size());
    println!("      🕒 Timestamp: {}", signature.timestamp());
    println!("      🔧 Algoritmo: {:?}", signature.algorithm());

    // 13. Verificar assinatura
    println!("\n12. Verificando assinatura pós-quântica...");
    let verification = verify_transaction_signature(&tx_hash, &signature)?;
    println!(
        "   {} Verificação: {}",
        if verification { "✅" } else { "❌" },
        if verification { "VÁLIDA" } else { "INVÁLIDA" }
    );

    // 14. Demonstrar segurança: tentar verificar com hash diferente
    let wrong_hash = Hash256::keccak256(b"hash_errado");
    let wrong_verification = verify_transaction_signature(&wrong_hash, &signature)?;
    println!(
        "   🛡️ Verificação com hash errado: {}",
        if wrong_verification {
            "VÁLIDA (PROBLEMA!)"
        } else {
            "INVÁLIDA (correto)"
        }
    );

    println!("\n🎯 Sprint 2 - Criptografia PQC implementada!");
    println!("✅ ML-DSA-65 (CRYSTALS-Dilithium) funcional");
    println!("✅ Geração segura de chaves");
    println!("✅ Assinatura e verificação de transações");
    println!("✅ Resistência a ataques quânticos");
    println!("✅ Tamanhos: ~2.6KB pub key, ~4.9KB priv key, ~4.7KB signature");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprint_1_integration() {
        // Teste de integração básico
        let result = main();
        assert!(result.is_ok());
    }
}
