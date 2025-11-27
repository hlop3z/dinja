#!/bin/bash

# Run the dinja HTTP server
# Usage: ./run.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
GREEN='\033[0;32m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_info "Starting dinja HTTP server..."
log_info "Server will be available at http://127.0.0.1:8080"
log_info "Press Ctrl+C to stop the server"
echo ""

cd core
cargo run --features http

