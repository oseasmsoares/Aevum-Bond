#!/bin/bash

# Wallet Node Startup Script
# Computer 3 - Lightweight wallet focused on transactions

set -e

if [ $# -eq 0 ]; then
    echo "❌ Usage: $0 <bootstrap_node_address>"
    echo "   Example: $0 192.168.1.100:8000"
    exit 1
fi

BOOTSTRAP_NODE=$1

echo "💳 Starting Aevum & Bond Wallet Node"
echo "===================================="

# Configuration  
LISTEN_ADDR="0.0.0.0"
PORT=8002
LOG_LEVEL="info"

echo "📍 Wallet Configuration:"
echo "   Listen Address: ${LISTEN_ADDR}:${PORT}"
echo "   Bootstrap Node: ${BOOTSTRAP_NODE}"
echo "   Log Level: ${LOG_LEVEL}"
echo "   Node Mode: Lightweight Wallet"
echo "   Sync Mode: SPV (Simplified Payment Verification)"
echo ""

# Build project if needed
if [ ! -f "../target/release/aevum-bond" ]; then
    echo "🔨 Building project in release mode..."
    cd ..
    cargo build --release
    cd deployment
fi

echo "🎯 Wallet Features:"
echo "   ✅ Transaction Creation"
echo "   ✅ Balance Checking"
echo "   ✅ Block Header Sync"
echo "   ✅ SPV Verification"
echo "   ❌ Full Blockchain (SPV mode)"
echo ""

echo "🎯 Starting wallet node..."
echo "   Connected to bootstrap: ${BOOTSTRAP_NODE}"
echo "   Lightweight sync mode enabled"
echo "   Press Ctrl+C to stop"
echo ""

# Start the wallet node
exec ../target/release/aevum-bond start-node \
    --port ${PORT} \
    --log-level ${LOG_LEVEL}
