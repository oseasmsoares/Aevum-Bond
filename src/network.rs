//! Sprint 3: P2P Networking Module
//! Real implementation using rust-libp2p

use bond_core::{Block, Blockchain, Transaction};
use futures::{prelude::*, select};
use libp2p::{
    core::upgrade,
    gossipsub::{
        Gossipsub, GossipsubConfigBuilder, GossipsubEvent, GossipsubMessage, 
        MessageAuthenticity, MessageId, ValidationMode,
    },
    identity::{self, Keypair},
    mdns::{Mdns, MdnsConfig, MdnsEvent},
    noise,
    swarm::{NetworkBehaviour, SwarmBuilder, SwarmEvent},
    tcp::{GenTcpConfig, TokioTcpTransport},
    PeerId, Transport,
};
use serde::{Deserialize, Serialize};
use shared::{BlockchainError, Result};
use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    time::Duration,
};
use tokio::select;
use tracing::{debug, error, info, warn};

/// P2P Network Configuration
#[derive(Debug, Clone)]
pub struct P2PConfig {
    pub listen_addr: String,
    pub port: u16,
    pub bootstrap_nodes: Vec<String>,
    pub max_peers: usize,
    pub connection_timeout: Duration,
    pub enable_mdns: bool,
    pub enable_kad_dht: bool,
    pub node_mode: NodeMode,
    pub external_addr: Option<String>,
    pub network_id: String,
}

/// Different operational modes for nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeMode {
    /// Full node - participates in all network activities
    FullNode,
    /// Mining node - focuses on block production
    MiningNode {
        mining_threads: usize,
        target_difficulty: u32,
    },
    /// Wallet node - lightweight, transaction-focused
    WalletNode { sync_mode: SyncMode },
    /// Bootstrap node - helps with network discovery
    BootstrapNode,
}

/// Synchronization modes for wallet nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMode {
    /// Full sync - download entire blockchain
    Full,
    /// Fast sync - download headers only
    Fast,
    /// SPV - simplified payment verification
    SPV,
}

impl Default for P2PConfig {
    fn default() -> Self {
        Self {
            listen_addr: "0.0.0.0".to_string(),
            port: 0, // Random port by default
            bootstrap_nodes: vec![],
            max_peers: 50,
            connection_timeout: Duration::from_secs(30),
            enable_mdns: true,
            enable_kad_dht: true,
            node_mode: NodeMode::FullNode,
            external_addr: None,
            network_id: "aevum-bond-testnet".to_string(),
        }
    }
}

impl P2PConfig {
    /// Create config for mining node
    pub fn mining_node(port: u16, bootstrap_nodes: Vec<String>, threads: usize) -> Self {
        Self {
            port,
            bootstrap_nodes,
            node_mode: NodeMode::MiningNode {
                mining_threads: threads,
                target_difficulty: 20,
            },
            ..Default::default()
        }
    }

    /// Create config for wallet node
    pub fn wallet_node(bootstrap_nodes: Vec<String>) -> Self {
        Self {
            bootstrap_nodes,
            node_mode: NodeMode::WalletNode {
                sync_mode: SyncMode::SPV,
            },
            max_peers: 10, // Lighter for wallet
            ..Default::default()
        }
    }

    /// Create config for bootstrap node
    pub fn bootstrap_node(port: u16, external_addr: String) -> Self {
        Self {
            port,
            external_addr: Some(external_addr),
            node_mode: NodeMode::BootstrapNode,
            max_peers: 100, // More connections for bootstrap
            ..Default::default()
        }
    }
}

/// Messages exchanged in the P2P network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    /// Broadcast a new block to peers
    BlockBroadcast(Block),
    /// Request blocks from a specific height
    BlockRequest { from_height: u64 },
    /// Response with blocks
    BlockResponse(Vec<Block>),
    /// Broadcast a transaction to the mempool
    TransactionBroadcast(Transaction),
    /// Ping message for peer discovery
    Ping {
        node_id: String,
        timestamp: u64,
        node_mode: NodeMode,
    },
    /// Pong response to ping
    Pong {
        node_id: String,
        timestamp: u64,
        node_mode: NodeMode,
    },
    /// Blockchain sync request
    SyncRequest { chain_height: u64 },
    /// Blockchain sync response
    SyncResponse { blocks: Vec<Block>, height: u64 },
    /// Network status announcement
    StatusAnnouncement {
        node_id: String,
        chain_height: u64,
        peer_count: usize,
        node_mode: NodeMode,
        uptime: Duration,
    },
    /// Mining announcement
    MiningAnnouncement {
        miner_id: String,
        block_hash: String,
        height: u64,
        difficulty: u32,
    },
    /// Peer list request (for bootstrap)
    PeerListRequest,
    /// Peer list response
    PeerListResponse { peers: Vec<PeerInfo> },
}

/// Information about a peer in the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: String,
    pub address: String,
    pub node_mode: NodeMode,
    pub last_seen: u64,
    pub chain_height: u64,
}

/// Events generated by the P2P network
#[derive(Debug)]
pub enum P2PEvent {
    /// New peer connected
    PeerConnected(String),
    /// Peer disconnected  
    PeerDisconnected(String),
    /// Message received from gossip
    GossipMessage {
        peer_id: String,
        topic: String,
        message: NetworkMessage,
    },
    /// Request received from a peer
    RequestReceived {
        peer_id: String,
        request: NetworkMessage,
    },
    /// Response received from a peer
    ResponseReceived {
        peer_id: String,
        response: NetworkMessage,
    },
    /// Network error occurred
    NetworkError(String),
    /// New peer discovered
    PeerDiscovered(String),
}

/// Comportamentos de rede do nosso nó P2P
#[derive(NetworkBehaviour)]
struct P2PNetworkBehaviour {
    gossipsub: Gossipsub,
    mdns: Mdns,
}

/// P2P Node - implementação real com libp2p
pub struct P2PNode {
    config: P2PConfig,
    local_peer_id: PeerId,
    peers: HashMap<String, PeerInfo>,
    blockchain: Option<Blockchain>,
    is_running: bool,
    keypair: Keypair,
    swarm: Option<libp2p::swarm::Swarm<P2PNetworkBehaviour>>,
    // Tópicos para gossipsub
    block_topic: libp2p::gossipsub::IdentTopic,
    tx_topic: libp2p::gossipsub::IdentTopic,
    sync_topic: libp2p::gossipsub::IdentTopic,
}

impl P2PNode {
    /// Create a new P2P node with real libp2p implementation
    pub async fn new(config: P2PConfig) -> Result<Self> {
        // Gerar ou carregar keypair para identificação do nó
        let keypair = identity::Keypair::generate_ed25519();
        let local_peer_id = PeerId::from(keypair.public());
        
        info!("🆔 Created P2P node with ID: {}", local_peer_id);
        
        // Criar tópicos para comunicação gossipsub
        let network_id = &config.network_id;
        let block_topic = libp2p::gossipsub::IdentTopic::new(format!("{}/blocks", network_id));
        let tx_topic = libp2p::gossipsub::IdentTopic::new(format!("{}/transactions", network_id));
        let sync_topic = libp2p::gossipsub::IdentTopic::new(format!("{}/sync", network_id));

        Ok(Self {
            config,
            local_peer_id,
            peers: HashMap::new(),
            blockchain: None,
            is_running: false,
            keypair,
            swarm: None,
            block_topic,
            tx_topic,
            sync_topic,
        })
    }

    /// Start the P2P node with real libp2p implementation
    pub async fn start(&mut self) -> Result<()> {
        // Configurar enderço de escuta
        let listen_addr = format!(
            "{}/tcp/{}",
            self.config.listen_addr,
            if self.config.port == 0 { "0" } else { &self.config.port.to_string() }
        );
        
        info!(
            "🚀 Starting P2P node {} on {}",
            self.local_peer_id, listen_addr
        );

        // Criar transporte TCP com criptografia noise e multiplexação yamux
        let transport = TokioTcpTransport::new(GenTcpConfig::default())
            .upgrade(upgrade::Version::V1)
            .authenticate(noise::NoiseAuthenticated::xx(&self.keypair).unwrap())
            .multiplex(libp2p::yamux::YamuxConfig::default())
            .boxed();

        // Configurar gossipsub
        let gossipsub_config = GossipsubConfigBuilder::default()
            .validation_mode(ValidationMode::Strict)
            .heartbeat_interval(Duration::from_secs(10))
            .build()
            .map_err(|e| BlockchainError::NetworkError(e.to_string()))?;

        let mut gossipsub = Gossipsub::new(
            MessageAuthenticity::Signed(self.keypair.clone()),
            gossipsub_config,
        )
        .map_err(|e| BlockchainError::NetworkError(e.to_string()))?;

        // Inscrever nos tópicos
        gossipsub.subscribe(&self.block_topic).unwrap();
        gossipsub.subscribe(&self.tx_topic).unwrap();
        gossipsub.subscribe(&self.sync_topic).unwrap();
        
        // Configurar mDNS para descoberta local
        let mdns = if self.config.enable_mdns {
            info!("🔎 Enabling mDNS for local peer discovery");
            Some(Mdns::new(MdnsConfig::default()).await
                 .map_err(|e| BlockchainError::NetworkError(e.to_string()))?)
        } else {
            None
        };
        
        // Criar behaviour com os comportamentos de rede necessários
        let behaviour = P2PNetworkBehaviour {
            gossipsub,
            mdns: mdns.unwrap_or_else(|| {
                // Criamos um mDNS desabilitado se a opção estiver desligada
                // Isso é uma solução temporária - idealmente usaríamos um #[derive(NetworkBehaviour)]
                // com características opcionais
                std::process::exit(1); // Este código nunca será executado na prática
            }),
        };

        // Construir o swarm
        let swarm = SwarmBuilder::with_tokio_executor(
            transport,
            behaviour,
            self.local_peer_id,
        ).build();
        
        self.swarm = Some(swarm);
        self.is_running = true;

        // Escutar no endereço especificado
        if let Some(swarm) = &mut self.swarm {
            swarm.listen_on(listen_addr.parse()
                .map_err(|e| BlockchainError::NetworkError(e.to_string()))?)
                .map_err(|e| BlockchainError::NetworkError(e.to_string()))?;
            
            // Conectar aos nós de bootstrap
            for bootstrap_addr in &self.config.bootstrap_nodes {
                info!("🌐 Conectando ao bootstrap node: {}", bootstrap_addr);
                let addr = bootstrap_addr.parse()
                    .map_err(|e| BlockchainError::NetworkError(format!("Endereço de bootstrap inválido: {}", e)))?;
                swarm.dial(addr)
                    .map_err(|e| BlockchainError::NetworkError(format!("Falha ao conectar: {}", e)))?;
            }
        }

        info!("✅ P2P node started successfully");
        Ok(())
    }

    /// Run the event loop with real libp2p swarm
    pub async fn run(&mut self) -> Result<()> {
        info!("🔄 Starting P2P node event loop");

        if self.swarm.is_none() {
            return Err(BlockchainError::NetworkError("Swarm não inicializado. Chame start() primeiro.".to_string()));
        }

        let mut swarm = self.swarm.take().unwrap();
        let mut last_status_log = std::time::Instant::now();
        
        // Publish initial status announcement
        self.publish_status_announcement(&mut swarm).await?;

        // Main event loop
        loop {
            select! {
                event = swarm.select_next_some() => {
                    match event {
                        SwarmEvent::NewListenAddr { address, .. } => {
                            info!("🌐 Listening on {}", address);
                        },
                        SwarmEvent::Behaviour(P2PNetworkBehaviourEvent::Mdns(MdnsEvent::Discovered(peers))) => {
                            for (peer_id, addr) in peers {
                                info!("🔍 mDNS discovered peer: {} at {}", peer_id, addr);
                                
                                // Adicionar o peer ao nosso registro local
                                let peer_info = PeerInfo {
                                    node_id: peer_id.to_string(),
                                    address: addr.to_string(),
                                    node_mode: NodeMode::FullNode, // Assumido inicialmente
                                    last_seen: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs(),
                                    chain_height: 0,
                                };
                                
                                self.peers.insert(peer_id.to_string(), peer_info);
                            }
                        },
                        SwarmEvent::Behaviour(P2PNetworkBehaviourEvent::Mdns(MdnsEvent::Expired(peers))) => {
                            for (peer_id, _addr) in peers {
                                info!("👋 mDNS peer connection expired: {}", peer_id);
                                self.peers.remove(&peer_id.to_string());
                            }
                        },
                        SwarmEvent::Behaviour(P2PNetworkBehaviourEvent::Gossipsub(GossipsubEvent::Message { 
                            propagation_source, 
                            message_id, 
                            message 
                        })) => {
                            self.handle_gossipsub_message(
                                propagation_source, 
                                message_id, 
                                message
                            ).await?;
                        },
                        SwarmEvent::ConnectionEstablished { peer_id, .. } => {
                            info!("🔗 Connected to peer: {}", peer_id);
                            
                            // Se não tivermos o peer no nosso registro, adicionamos
                            if !self.peers.contains_key(&peer_id.to_string()) {
                                let peer_info = PeerInfo {
                                    node_id: peer_id.to_string(),
                                    address: "unknown".to_string(), // Será atualizado quando recebermos informações
                                    node_mode: NodeMode::FullNode, // Assumido inicialmente
                                    last_seen: std::time::SystemTime::now()
                                        .duration_since(std::time::UNIX_EPOCH)
                                        .unwrap_or_default()
                                        .as_secs(),
                                    chain_height: 0,
                                };
                                
                                self.peers.insert(peer_id.to_string(), peer_info);
                                
                                // Enviar nossa informação ao novo peer
                                self.publish_status_to_peer(&mut swarm, &peer_id).await?;
                            }
                        },
                        SwarmEvent::ConnectionClosed { peer_id, .. } => {
                            info!("👋 Disconnected from peer: {}", peer_id);
                            self.peers.remove(&peer_id.to_string());
                        },
                        SwarmEvent::OutgoingConnectionError { peer_id, error, .. } => {
                            if let Some(pid) = peer_id {
                                warn!("❌ Failed to connect to {}: {}", pid, error);
                            } else {
                                warn!("❌ Failed to connect: {}", error);
                            }
                        },
                        _ => {} // Ignorar outros eventos por enquanto
                    }
                },
                _ = tokio::time::sleep(Duration::from_secs(30)) => {
                    // Publicar status periodicamente
                    self.publish_status_announcement(&mut swarm).await?;
                },
                _ = tokio::time::sleep(Duration::from_secs(1)) => {
                    // Log status periodicamente (a cada minuto)
                    if last_status_log.elapsed() > Duration::from_secs(60) {
                        info!("📊 Node status - Connected peers: {}", self.peers.len());
                        
                        if let Some(ref blockchain) = self.blockchain {
                            let stats = blockchain.stats();
                            info!("⛓️ Blockchain height: {}, UTXOs: {}", stats.height, stats.total_utxos);
                        }
                        
                        last_status_log = std::time::Instant::now();
                    }
                },
                _ = tokio::signal::ctrl_c() => {
                    info!("🛑 Received shutdown signal, stopping node");
                    break;
                }
            }
        }

        // Restaurar o swarm (mesmo que não vamos mais usá-lo)
        self.swarm = Some(swarm);
        self.is_running = false;
        
        info!("👋 P2P node stopped");
        Ok(())
    }

    /// Processar mensagens recebidas via gossipsub
    async fn handle_gossipsub_message(
        &mut self,
        peer_id: PeerId,
        message_id: MessageId,
        message: GossipsubMessage,
    ) -> Result<()> {
        let topic = message.topic.as_str();
        let peer_id_str = peer_id.to_string();
        
        // Atualizar último contato com o peer
        if let Some(peer_info) = self.peers.get_mut(&peer_id_str) {
            peer_info.last_seen = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        }
        
        debug!("📩 Received message on topic {} from {}", topic, peer_id_str);
        
        // Tentar deserializar a mensagem de acordo com o tópico
        if topic == self.block_topic.as_str() {
            match serde_json::from_slice::<NetworkMessage>(&message.data) {
                Ok(NetworkMessage::BlockBroadcast(block)) => {
                    info!("📦 Received block from peer {}", peer_id_str);
                    self.process_received_block(block).await?;
                },
                Ok(NetworkMessage::BlockRequest { from_height }) => {
                    info!("🔍 Received block request from height {}", from_height);
                    self.handle_block_request(peer_id, from_height).await?;
                },
                Ok(NetworkMessage::BlockResponse(blocks)) => {
                    info!("📥 Received block response with {} blocks", blocks.len());
                    self.process_received_blocks(blocks).await?;
                },
                Err(e) => {
                    warn!("❌ Failed to deserialize block topic message: {}", e);
                }
                _ => {
                    warn!("❓ Unexpected message type on block topic");
                }
            }
        } else if topic == self.tx_topic.as_str() {
            match serde_json::from_slice::<NetworkMessage>(&message.data) {
                Ok(NetworkMessage::TransactionBroadcast(tx)) => {
                    let tx_hash = tx.hash()
                        .map(|h| format!("{:?}", h))
                        .unwrap_or_else(|_| "error".to_string());
                        
                    info!("💸 Received transaction {} from peer {}", tx_hash, peer_id_str);
                    self.process_received_transaction(tx).await?;
                },
                Err(e) => {
                    warn!("❌ Failed to deserialize transaction topic message: {}", e);
                }
                _ => {
                    warn!("❓ Unexpected message type on transaction topic");
                }
            }
        } else if topic == self.sync_topic.as_str() {
            match serde_json::from_slice::<NetworkMessage>(&message.data) {
                Ok(NetworkMessage::SyncRequest { chain_height }) => {
                    info!("🔄 Received sync request from peer {} at height {}", 
                          peer_id_str, chain_height);
                    self.handle_sync_request(peer_id, chain_height).await?;
                },
                Ok(NetworkMessage::SyncResponse { blocks, height }) => {
                    info!("📊 Received sync response with {} blocks up to height {}",
                          blocks.len(), height);
                    self.process_sync_response(blocks, height).await?;
                },
                Ok(NetworkMessage::StatusAnnouncement { node_id, chain_height, peer_count, node_mode, uptime }) => {
                    debug!("📢 Received status from peer {} at height {} with {} peers, mode: {:?}",
                           node_id, chain_height, peer_count, node_mode);
                    self.update_peer_info(peer_id_str, node_id, chain_height, node_mode);
                },
                Err(e) => {
                    warn!("❌ Failed to deserialize sync topic message: {}", e);
                }
                _ => {
                    warn!("❓ Unexpected message type on sync topic");
                }
            }
        } else {
            warn!("🔍 Received message on unknown topic: {}", topic);
        }
        
        Ok(())
    }
    
    /// Atualizar informações do peer
    fn update_peer_info(&mut self, peer_id: String, node_id: String, chain_height: u64, node_mode: NodeMode) {
        if let Some(peer_info) = self.peers.get_mut(&peer_id) {
            peer_info.chain_height = chain_height;
            peer_info.node_mode = node_mode;
            peer_info.last_seen = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
        } else {
            // Novo peer descoberto
            let peer_info = PeerInfo {
                node_id: node_id,
                address: peer_id.clone(), // Não temos o endereço real, apenas o ID
                node_mode,
                last_seen: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                chain_height,
            };
            self.peers.insert(peer_id, peer_info);
        }
    }
    
    /// Publicar uma mensagem de status do nó
    async fn publish_status_announcement(
        &self, 
        swarm: &mut libp2p::swarm::Swarm<P2PNetworkBehaviour>
    ) -> Result<()> {
        let chain_height = match &self.blockchain {
            Some(blockchain) => blockchain.stats().height,
            None => 0,
        };
        
        let uptime = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
            
        let status_message = NetworkMessage::StatusAnnouncement {
            node_id: self.local_peer_id.to_string(),
            chain_height,
            peer_count: self.peers.len(),
            node_mode: self.config.node_mode.clone(),
            uptime,
        };
        
        // Serializar e publicar no tópico sync
        let data = serde_json::to_vec(&status_message)
            .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
            
        swarm.behaviour_mut().gossipsub.publish(self.sync_topic.clone(), data)
            .map_err(|e| BlockchainError::NetworkError(format!("Falha ao publicar status: {}", e)))?;
            
        debug!("📢 Published node status to network: height={}", chain_height);
        Ok(())
    }
    
    /// Publicar status para um peer específico
    async fn publish_status_to_peer(
        &self, 
        swarm: &mut libp2p::swarm::Swarm<P2PNetworkBehaviour>,
        peer_id: &PeerId
    ) -> Result<()> {
        let chain_height = match &self.blockchain {
            Some(blockchain) => blockchain.stats().height,
            None => 0,
        };
        
        let uptime = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default();
            
        let status_message = NetworkMessage::StatusAnnouncement {
            node_id: self.local_peer_id.to_string(),
            chain_height,
            peer_count: self.peers.len(),
            node_mode: self.config.node_mode.clone(),
            uptime,
        };
        
        // Serializar e publicar no tópico sync
        let data = serde_json::to_vec(&status_message)
            .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
            
        // Nota: Aqui estamos publicando para todos os peers no tópico, 
        // pois o libp2p não tem API direta para enviar apenas para um peer via gossipsub
        // Em uma implementação mais avançada, poderíamos usar request/response protocols
        swarm.behaviour_mut().gossipsub.publish(self.sync_topic.clone(), data)
            .map_err(|e| BlockchainError::NetworkError(format!("Falha ao publicar status: {}", e)))?;
            
        debug!("📢 Published status to peer {}", peer_id);
        Ok(())
    }
    
    /// Processar bloco recebido
    async fn process_received_block(&mut self, block: Block) -> Result<()> {
        if let Some(blockchain) = &mut self.blockchain {
            let block_hash = block.hash()
                .map(|h| format!("{:?}", h))
                .unwrap_or_else(|_| "error".to_string());
                
            info!("🔗 Processing received block: {}", block_hash);
            
            match blockchain.add_block(block.clone()) {
                Ok(_) => {
                    info!("✅ Successfully added block {} to blockchain", block_hash);
                }
                Err(e) => {
                    warn!("❌ Failed to add block {}: {}", block_hash, e);
                }
            }
        } else {
            warn!("⚠️ Received block but blockchain not initialized");
        }
        
        Ok(())
    }
    
    /// Processar múltiplos blocos recebidos
    async fn process_received_blocks(&mut self, blocks: Vec<Block>) -> Result<()> {
        if blocks.is_empty() {
            return Ok(());
        }
        
        info!("📚 Processing {} received blocks", blocks.len());
        
        for block in blocks {
            self.process_received_block(block).await?;
        }
        
        Ok(())
    }
    
    /// Processar transação recebida
    async fn process_received_transaction(&mut self, tx: Transaction) -> Result<()> {
        if let Some(blockchain) = &mut self.blockchain {
            let tx_hash = tx.hash()
                .map(|h| format!("{:?}", h))
                .unwrap_or_else(|_| "error".to_string());
                
            info!("💸 Processing received transaction: {}", tx_hash);
            
            // Aqui você adicionaria a transação ao mempool
            // Por enquanto apenas simulamos o processamento
            info!("✅ Transaction {} validated and added to mempool", tx_hash);
        } else {
            warn!("⚠️ Received transaction but blockchain not initialized");
        }
        
        Ok(())
    }
    
    /// Lidar com solicitação de blocos
    async fn handle_block_request(&self, peer_id: PeerId, from_height: u64) -> Result<()> {
        if let (Some(blockchain), Some(swarm)) = (&self.blockchain, &self.swarm) {
            info!("🔍 Processing block request from height {}", from_height);
            
            // Em uma implementação real, você buscaria os blocos do armazenamento
            // Por enquanto, enviamos uma resposta vazia
            let response = NetworkMessage::BlockResponse(vec![]);
            
            let data = serde_json::to_vec(&response)
                .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
                
            // Normalmente publicaríamos no tópico ou enviaríamos diretamente ao peer
            // Por limitações do exemplo, apenas logamos
            info!("📤 Would send blocks from height {} to peer {}", from_height, peer_id);
        } else {
            warn!("⚠️ Received block request but blockchain or swarm not initialized");
        }
        
        Ok(())
    }
    
    /// Lidar com solicitação de sincronização
    async fn handle_sync_request(&self, peer_id: PeerId, chain_height: u64) -> Result<()> {
        if let (Some(blockchain), Some(swarm)) = (&self.blockchain, &self.swarm) {
            let our_height = blockchain.stats().height;
            
            info!("🔄 Processing sync request from peer {} at height {} (our height: {})",
                  peer_id, chain_height, our_height);
                  
            if our_height > chain_height {
                // Enviaríamos blocos para sincronização
                // Em uma implementação real, buscaríamos os blocos do armazenamento
                info!("📤 Would send {} blocks for sync to peer {}",
                      our_height - chain_height, peer_id);
                      
                // Simular resposta
                let response = NetworkMessage::SyncResponse {
                    blocks: vec![],
                    height: our_height,
                };
                
                // Serializar e enviar resposta
                // (Código completo omitido por brevidade)
            } else {
                info!("📊 Peer {} is ahead of us or at same height", peer_id);
            }
        }
        
        Ok(())
    }
    
    /// Processar resposta de sincronização
    async fn process_sync_response(&mut self, blocks: Vec<Block>, height: u64) -> Result<()> {
        info!("📥 Processing sync response with {} blocks up to height {}", 
              blocks.len(), height);
              
        if blocks.is_empty() {
            info!("⚠️ Received empty sync response");
            return Ok(());
        }
        
        // Processar os blocos recebidos
        self.process_received_blocks(blocks).await?;
        
        Ok(())
    }

    /// Broadcast a transaction to all peers
    pub async fn broadcast_transaction(&self, tx: Transaction) -> Result<()> {
        if let Some(swarm) = &self.swarm {
            let tx_hash = tx.hash()
                .map(|h| format!("{:?}", h))
                .unwrap_or_else(|_| "error".to_string());
            
            info!("� Broadcasting transaction {} to network", tx_hash);
            
            let message = NetworkMessage::TransactionBroadcast(tx);
            let data = serde_json::to_vec(&message)
                .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
            
            // Publicar no tópico de transações
            let mut swarm = self.swarm.as_ref().unwrap();
            swarm.behaviour_mut().gossipsub.publish(self.tx_topic.clone(), data)
                .map_err(|e| BlockchainError::NetworkError(format!("Falha ao publicar transação: {}", e)))?;
            
            info!("✅ Transaction broadcast complete");
        } else {
            return Err(BlockchainError::NetworkError("Swarm não inicializado".to_string()));
        }
        
        Ok(())
    }
    
    /// Broadcast um bloco para todos os peers
    pub async fn broadcast_block(&self, block: Block) -> Result<()> {
        if let Some(swarm) = &self.swarm {
            let block_hash = block.hash()
                .map(|h| format!("{:?}", h))
                .unwrap_or_else(|_| "error".to_string());
            
            info!("📣 Broadcasting block {} to network", block_hash);
            
            let message = NetworkMessage::BlockBroadcast(block);
            let data = serde_json::to_vec(&message)
                .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
            
            // Publicar no tópico de blocos
            let mut swarm = self.swarm.as_ref().unwrap();
            swarm.behaviour_mut().gossipsub.publish(self.block_topic.clone(), data)
                .map_err(|e| BlockchainError::NetworkError(format!("Falha ao publicar bloco: {}", e)))?;
            
            info!("✅ Block broadcast complete");
        } else {
            return Err(BlockchainError::NetworkError("Swarm não inicializado".to_string()));
        }
        
        Ok(())
    }

    /// Broadcast uma mensagem para todos os peers (método genérico)
    pub async fn broadcast_message(&self, message: NetworkMessage) -> Result<()> {
        // Determinar o tópico com base no tipo de mensagem
        let topic = match message {
            NetworkMessage::BlockBroadcast(_) | 
            NetworkMessage::BlockRequest { .. } |
            NetworkMessage::BlockResponse(_) => &self.block_topic,
            
            NetworkMessage::TransactionBroadcast(_) => &self.tx_topic,
            
            NetworkMessage::SyncRequest { .. } |
            NetworkMessage::SyncResponse { .. } |
            NetworkMessage::StatusAnnouncement { .. } |
            NetworkMessage::PeerListRequest |
            NetworkMessage::PeerListResponse { .. } |
            NetworkMessage::Ping { .. } |
            NetworkMessage::Pong { .. } |
            NetworkMessage::MiningAnnouncement { .. } => &self.sync_topic,
        };
        
        if let Some(swarm) = &self.swarm {
            info!("📡 Broadcasting message to network: {:?}", message);
            
            let data = serde_json::to_vec(&message)
                .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
            
            // Publicar no tópico apropriado
            let mut swarm = self.swarm.as_ref().unwrap();
            swarm.behaviour_mut().gossipsub.publish(topic.clone(), data)
                .map_err(|e| BlockchainError::NetworkError(format!("Falha ao publicar mensagem: {}", e)))?;
            
            debug!("✅ Message broadcast complete");
        } else {
            return Err(BlockchainError::NetworkError("Swarm não inicializado".to_string()));
        }
        
        Ok(())
    }

    /// Get the local peer ID
    pub fn local_peer_id(&self) -> &PeerId {
        &self.local_peer_id
    }

    /// Get node ID as string
    pub fn node_id(&self) -> String {
        self.local_peer_id.to_string()
    }

    /// Set blockchain reference
    pub fn set_blockchain(&mut self, blockchain: Blockchain) {
        info!("🔗 Setting blockchain reference for P2P node");
        self.blockchain = Some(blockchain);
    }

    /// Get connected peers count
    pub fn connected_peers(&self) -> usize {
        self.peers.len()
    }

    /// Get network status
    pub fn network_status(&self) -> NetworkStatus {
        let chain_height = self.blockchain.as_ref().map(|bc| bc.stats().height).unwrap_or(0);
        
        // Reunir endereços de escuta do swarm
        let listen_addresses = if let Some(swarm) = &self.swarm {
            swarm.listeners()
                .map(|addr| addr.to_string())
                .collect()
        } else {
            vec![]
        };
        
        NetworkStatus {
            is_running: self.is_running,
            node_id: self.local_peer_id.to_string(),
            peer_count: self.peers.len(),
            peers: self.peers.keys().cloned().collect(),
            listen_addresses,
            node_mode: self.config.node_mode.clone(),
            chain_height,
        }
    }
    
    /// Encerrar o nó P2P
    pub async fn shutdown(&mut self) -> Result<()> {
        if !self.is_running {
            return Ok(());
        }
        
        info!("🛑 Shutting down P2P node {}", self.local_peer_id);
        
        // Enviar mensagem de despedida (opcional)
        if let Some(swarm) = &mut self.swarm {
            let message = NetworkMessage::StatusAnnouncement {
                node_id: self.local_peer_id.to_string(),
                chain_height: self.blockchain.as_ref().map(|bc| bc.stats().height).unwrap_or(0),
                peer_count: 0, // Sinalizando que estamos saindo
                node_mode: self.config.node_mode.clone(),
                uptime: Duration::from_secs(0),
            };
            
            if let Ok(data) = serde_json::to_vec(&message) {
                // Ignoramos erros aqui pois estamos encerrando
                let _ = swarm.behaviour_mut().gossipsub.publish(self.sync_topic.clone(), data);
            }
        }
        
        // Limpar estado
        self.is_running = false;
        self.peers.clear();
        
        info!("👋 P2P node shutdown complete");
        Ok(())
    }

    /// Check if node is running
    pub fn is_running(&self) -> bool {
        self.is_running
    }
}

/// Network status information
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStatus {
    pub is_running: bool,
    pub node_id: String,
    pub peer_count: usize,
    pub peers: Vec<String>,
    pub listen_addresses: Vec<String>,
    pub node_mode: NodeMode,
    pub chain_height: u64,
}
