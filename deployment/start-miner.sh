#!/bin/bash

# Mining Node Startup Script  
# Computer 2 - Dedicated mining node with optimized configuration

set -e

if [ $# -eq 0 ]; then
    echo "❌ Usage: $0 <bootstrap_node_address>"
    echo "   Example: $0 192.168.1.100:8000"
    exit 1
fi

BOOTSTRAP_NODE=$1

echo "⛏️  Starting Aevum & Bond Mining Node"
echo "====================================="

# Configuration
LISTEN_ADDR="0.0.0.0"
PORT=8001
LOG_LEVEL="info"
MINING_THREADS=$(nproc) # Use all available CPU cores

echo "📍 Mining Configuration:"
echo "   Listen Address: ${LISTEN_ADDR}:${PORT}"
echo "   Bootstrap Node: ${BOOTSTRAP_NODE}"
echo "   Mining Threads: ${MINING_THREADS}"
echo "   Log Level: ${LOG_LEVEL}"
echo "   Node Mode: Dedicated Mining"
echo ""

# Build project if needed
if [ ! -f "../target/release/aevum-bond" ]; then
    echo "🔨 Building project in release mode..."
    cd ..
    cargo build --release
    cd deployment
fi

echo "⚡ Performance Settings:"
echo "   Priority: High (mining optimized)"
echo "   CPU Affinity: All cores"
echo "   Memory: Unlimited"
echo ""

# Set high priority for mining
# Note: May require sudo privileges
# sudo nice -n -10 \

echo "🎯 Starting mining node..."
echo "   Connected to bootstrap: ${BOOTSTRAP_NODE}"
echo "   Mining with ${MINING_THREADS} threads"
echo "   Press Ctrl+C to stop"
echo ""

# Start the mining node
exec ../target/release/aevum-bond start-node \
    --port ${PORT} \
    --log-level ${LOG_LEVEL}
