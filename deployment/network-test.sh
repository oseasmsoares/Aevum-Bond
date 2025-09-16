#!/bin/bash

# Network Connectivity Test Script
# Tests communication between all deployed nodes

set -e

echo "🌐 Aevum & Bond Network Connectivity Test"
echo "=========================================="

# Default IPs - modify as needed
BOOTSTRAP_IP=${1:-"192.168.1.100"}
MINER_IP=${2:-"192.168.1.101"} 
WALLET_IP=${3:-"192.168.1.102"}

echo "🎯 Testing network connectivity..."
echo "   Bootstrap: ${BOOTSTRAP_IP}:8000"
echo "   Miner: ${MINER_IP}:8001"
echo "   Wallet: ${WALLET_IP}:8002"
echo ""

# Function to test connectivity
test_connection() {
    local host=$1
    local port=$2
    local name=$3
    
    echo -n "   Testing ${name} (${host}:${port})... "
    
    if timeout 5 bash -c "cat < /dev/null > /dev/tcp/${host}/${port}"; then
        echo "✅ Connected"
        return 0
    else
        echo "❌ Failed"
        return 1
    fi
}

# Function to ping host
test_ping() {
    local host=$1
    local name=$2
    
    echo -n "   Ping ${name} (${host})... "
    
    if ping -c 1 -W 1 ${host} > /dev/null 2>&1; then
        echo "✅ Reachable"
        return 0
    else
        echo "❌ Unreachable"
        return 1
    fi
}

echo "🏓 ICMP Connectivity Test:"
test_ping ${BOOTSTRAP_IP} "Bootstrap"
test_ping ${MINER_IP} "Miner"
test_ping ${WALLET_IP} "Wallet"
echo ""

echo "🔌 Port Connectivity Test:"
test_connection ${BOOTSTRAP_IP} 8000 "Bootstrap"
test_connection ${MINER_IP} 8001 "Miner"
test_connection ${WALLET_IP} 8002 "Wallet"
echo ""

echo "📊 Network Status Check:"
if command -v cargo >/dev/null 2>&1; then
    echo "   Checking local network status..."
    timeout 10 cargo run --quiet -- network status 2>/dev/null || echo "   ❌ Local node not running"
    echo ""
fi

echo "🎯 Deployment Verification:"
echo "   1. Bootstrap node should start first"
echo "   2. Wait 30s for bootstrap to stabilize"
echo "   3. Start miner node with bootstrap address"
echo "   4. Wait for miner to connect"
echo "   5. Start wallet node with bootstrap address"
echo "   6. All nodes should discover each other"
echo ""

echo "📋 Next Steps:"
echo "   - Check logs for connection status"
echo "   - Monitor peer discovery"
echo "   - Test transaction propagation"
echo "   - Verify block synchronization"

echo ""
echo "Test completed! ✨"
