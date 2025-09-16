#!/bin/bash

# Bootstrap Node Startup Script
# Computer 1 - Acts as network bootstrap and full node

set -e

echo "🚀 Starting Aevum & Bond Bootstrap Node"
echo "========================================"

# Configuration
LISTEN_ADDR="0.0.0.0"
PORT=8000
LOG_LEVEL="info"

# Get local IP for external address
EXTERNAL_IP=$(ip route get 8.8.8.8 | awk '{print $7}' | head -1)

echo "📍 Network Configuration:"
echo "   Listen Address: ${LISTEN_ADDR}:${PORT}"
echo "   External IP: ${EXTERNAL_IP}"
echo "   Log Level: ${LOG_LEVEL}"
echo "   Node Mode: Bootstrap + Full Node"
echo ""

# Build project if needed
if [ ! -f "../target/release/aevum-bond" ]; then
    echo "🔨 Building project in release mode..."
    cd ..
    cargo build --release
    cd deployment
fi

echo "🎯 Starting bootstrap node..."
echo "   Peers can connect using: ${EXTERNAL_IP}:${PORT}"
echo "   Press Ctrl+C to stop"
echo ""

# Start the node
exec ../target/release/aevum-bond start-node \
    --port ${PORT} \
    --log-level ${LOG_LEVEL}
