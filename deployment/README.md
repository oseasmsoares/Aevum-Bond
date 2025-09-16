# Aevum & Bond - Multi-Node Deployment Guide

## Arquitetura da Rede Distribuída

### Configuração Recomendada para 3 Computadores

```
Computer 1 (Bootstrap + Full Node)
├── IP: 192.168.1.100
├── Função: Bootstrap node + Full node
├── Porta: 8000
└── Recursos: Facilita descoberta de peers

Computer 2 (Mining Node) 
├── IP: 192.168.1.101  
├── Função: Mining node (foco em mineração)
├── Porta: 8001
└── Recursos: CPU otimizada para mining

Computer 3 (Wallet Node)
├── IP: 192.168.1.102
├── Função: Wallet node (transações)  
├── Porta: 8002
└── Recursos: Interface para usuários
```

## Scripts de Deployment

### 1. Configuração Inicial
```bash
# Em cada computador, clonar e compilar
git clone <repository_url>
cd Aevum-Bond
cargo build --release
```

### 2. Executar Nodes

**Computer 1 - Bootstrap Node:**
```bash
./deployment/start-bootstrap.sh
```

**Computer 2 - Mining Node:**  
```bash
./deployment/start-miner.sh 192.168.1.100:8000
```

**Computer 3 - Wallet Node:**
```bash
./deployment/start-wallet.sh 192.168.1.100:8000
```

## Monitoramento

- **Logs**: Cada node gera logs detalhados
- **Status**: `cargo run -- network status` 
- **Peers**: `cargo run -- network peers`
- **Dashboard**: Interface web em desenvolvimento

## Teste de Conectividade

1. Iniciar bootstrap node primeiro
2. Aguardar 30s para estabilizar
3. Iniciar mining node
4. Aguardar conexão ser estabelecida  
5. Iniciar wallet node
6. Verificar comunicação entre todos os nodes

## Estrutura de Arquivos

```
deployment/
├── README.md              # Este arquivo
├── start-bootstrap.sh     # Script para bootstrap node  
├── start-miner.sh         # Script para mining node
├── start-wallet.sh        # Script para wallet node
├── network-test.sh        # Testes de conectividade
├── configs/              # Configurações específicas
│   ├── bootstrap.toml
│   ├── miner.toml  
│   └── wallet.toml
└── monitoring/           # Scripts de monitoramento
    ├── status-check.sh
    └── peer-monitor.sh
```

## Troubleshooting

### Problemas Comuns
- **Firewall**: Liberar portas 8000-8002
- **Network Discovery**: Verificar conectividade mDNS
- **Bootstrap**: Aguardar bootstrap estar online antes dos demais
- **Logs**: Verificar logs detalhados com `--log-level debug`

### Comandos de Diagnóstico  
```bash
# Verificar status da rede
cargo run -- network status

# Listar peers conectados  
cargo run -- network peers

# Teste de ping entre nodes
ping 192.168.1.100
telnet 192.168.1.100 8000
```
