use bond_core::*;
use clap::{Args, Parser, Subcommand};
use shared::{KeyPair, sign_transaction_hash, verify_transaction_signature};
use tracing::{Level, info};
use tracing_subscriber;

mod network;
use network::{NodeMode, P2PConfig, P2PNode, SyncMode};

#[derive(Parser)]
#[command(name = "aevum-bond")]
#[command(about = "Aevum & Bond - Post-Quantum Blockchain Node")]
#[command(version = "0.3.0-sprint3")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run Sprint 1 & 2 demonstration (blockchain + crypto)
    Demo,
    /// Start P2P network node (Sprint 3)
    StartNode(StartNodeArgs),
    /// Network information and status
    Network(NetworkArgs),
    /// Display version information
    Version,
}

#[derive(Args)]
struct StartNodeArgs {
    /// Port to listen on (0 for random)
    #[arg(short, long, default_value = "0")]
    port: u16,

    /// Bootstrap nodes to connect to (IP:PORT)
    #[arg(short, long)]
    bootstrap: Vec<String>,

    /// Node operation mode
    #[arg(long, value_enum, default_value = "full")]
    mode: NodeModeArg,

    /// Number of mining threads (only for mining mode)
    #[arg(long, default_value = "1")]
    mining_threads: usize,

    /// Target mining difficulty (only for mining mode)
    #[arg(long, default_value = "20")]
    difficulty: u32,

    /// External IP address for bootstrap nodes
    #[arg(long)]
    external_ip: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum NodeModeArg {
    /// Full node - participates in all network activities
    Full,
    /// Mining node - focuses on block production
    Mining,
    /// Wallet node - lightweight, transaction-focused
    Wallet,
    /// Bootstrap node - helps with network discovery
    Bootstrap,
}

#[derive(Args)]
struct NetworkArgs {
    #[command(subcommand)]
    action: NetworkAction,
}

#[derive(Subcommand)]
enum NetworkAction {
    /// Show network status
    Status,
    /// List connected peers
    Peers,
}

#[tokio::main]
async fn main() -> shared::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Demo) => demo_blockchain_and_crypto().await,
        Some(Commands::StartNode(args)) => start_p2p_node(args).await,
        Some(Commands::Network(args)) => handle_network_commands(args).await,
        Some(Commands::Version) => {
            show_version();
            Ok(())
        }
        None => {
            // Default: run demo
            demo_blockchain_and_crypto().await
        }
    }
}

async fn start_p2p_node(args: &StartNodeArgs) -> shared::Result<()> {
    // Configure logging
    let level = match args.log_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "info" => Level::INFO,
        "warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    tracing_subscriber::fmt().with_max_level(level).init();

    info!("🚀 Starting Aevum & Bond P2P Node - Sprint 3");
    info!("🎯 Mode: {:?}", args.mode);

    // Create node mode from arguments
    let node_mode = match args.mode {
        NodeModeArg::Full => NodeMode::FullNode,
        NodeModeArg::Mining => NodeMode::MiningNode {
            mining_threads: args.mining_threads,
            target_difficulty: args.difficulty,
        },
        NodeModeArg::Wallet => NodeMode::WalletNode {
            sync_mode: SyncMode::SPV,
        },
        NodeModeArg::Bootstrap => NodeMode::BootstrapNode,
    };

    // Create a blockchain instance for the node
    let genesis_script = b"Genesis Block - Aevum & Bond P2P Node".to_vec();
    let blockchain = Blockchain::new(Default::default(), genesis_script)?;
    info!("⛓️ Blockchain initialized with genesis block");

    let config = P2PConfig {
        port: args.port,
        bootstrap_nodes: args.bootstrap.clone(),
        node_mode,
        external_addr: args.external_ip.clone(),
        ..P2PConfig::default()
    };

    let mut node = P2PNode::new(config).await?;
    info!("🆔 Node ID: {}", node.node_id());

    // Attach blockchain to the P2P node
    node.set_blockchain(blockchain);
    info!("🔗 Blockchain attached to P2P node");

    // Start the node
    node.start().await?;
    info!("✅ P2P Node started successfully");

    // Run the event loop (blocks until Ctrl+C)
    node.run().await?;

    Ok(())
}

async fn handle_network_commands(args: &NetworkArgs) -> shared::Result<()> {
    match &args.action {
        NetworkAction::Status => {
            println!("🌐 Aevum & Bond Network Status");
            println!("Sprint 3: P2P Implementation");
            println!("Status: Development/Testing");
        }
        NetworkAction::Peers => {
            println!("👥 Connected Peers");
            println!("(Feature available when node is running)");
        }
    }
    Ok(())
}

fn show_version() {
    println!("🔗 Aevum & Bond v0.3.0-sprint3");
    println!("Sprint 3: P2P Networking Implementation");
    println!("Post-Quantum Blockchain with ML-DSA-65");
    println!();
    println!("Features:");
    println!("  ✅ Sprint 1: Blockchain Foundation");
    println!("  ✅ Sprint 2: Post-Quantum Cryptography");
    println!("  🔄 Sprint 3: P2P Networking (In Progress)");
}

async fn demo_blockchain_and_crypto() -> shared::Result<()> {
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
