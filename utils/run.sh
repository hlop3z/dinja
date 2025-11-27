#!/bin/bash

# Run the dinja HTTP server
# Usage: ./run.sh

set -e

# Get the directory where this script is located (utils/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Get the project root directory (parent of utils/)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to project root for all operations
cd "$PROJECT_ROOT"

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

