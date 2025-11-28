#!/bin/bash

# Build, test, and development script for dinja
# Usage:
#   ./build.sh build         - Build Rust core
#   ./build.sh build-release - Build Rust core (release mode)
#   ./build.sh test          - Run Rust tests
#   ./build.sh test-python   - Run Python HTTP client tests
#   ./build.sh clean         - Clean build artifacts

set -e

# Get the directory where this script is located (utils/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Get the project root directory (parent of utils/)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to project root for all operations
cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if uv is available
check_uv() {
    command_exists uv && return 0
    return 1
}

# Build core crate (debug mode)
build_core() {
    log_info "Building core crate..."
    cargo build -p dinja-core
    log_info "Core build completed successfully"
}

# Build core crate (release mode)
build_core_release() {
    log_info "Building core crate (release mode)..."
    cargo build -p dinja-core --release
    log_info "Core release build completed successfully"
}

# Run core tests
test_core() {
    log_info "Running core crate tests..."
    cargo test -p dinja-core
    log_info "Core tests completed successfully"
}

# Run Python HTTP client tests (requires service running)
test_python() {
    if ! check_uv; then
        log_warn "uv is required to run Python tests. Install uv: https://github.com/astral-sh/uv"
        return 1
    fi

    log_info "Running Python HTTP client tests..."
    log_warn "Note: Dinja service must be running on http://localhost:8080"

    cd python-bindings
    uv sync --dev
    uv run pytest tests
    cd ..

    log_info "Python tests completed successfully"
}

# Run JavaScript tests
test_js() {
    log_info "Running JavaScript tests..."
    cd js-bindings
    npm test
    cd ..
    log_info "JavaScript tests completed successfully"
}

# Build JavaScript client
build_js() {
    log_info "Building JavaScript client..."
    cd js-bindings
    npm install
    npm run build
    cd ..
    log_info "JavaScript build completed successfully"
}

# Clean build artifacts
clean() {
    log_info "Cleaning build artifacts..."
    cargo clean -p dinja-core
    rm -rf python-bindings/dist
    rm -rf js-bindings/dist
    log_info "Clean completed"
}

# Run all tests
run_all_tests() {
    log_info "Running all tests..."
    test_core
    test_python
    test_js
    log_info "All tests completed successfully!"
}

# Main command dispatcher
case "${1:-help}" in
    build)
        build_core
        ;;
    build-core)
        build_core
        ;;
    build-release)
        build_core_release
        ;;
    build-core-release)
        build_core_release
        ;;
    build-js)
        build_js
        ;;
    test)
        test_core
        ;;
    test-core)
        test_core
        ;;
    test-python)
        test_python
        ;;
    test-js)
        test_js
        ;;
    test-all)
        run_all_tests
        ;;
    clean)
        clean
        ;;
    help|--help|-h)
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  build              Build Rust core (debug mode)"
        echo "  build-release      Build Rust core (release mode)"
        echo "  build-js           Build JavaScript client"
        echo "  test               Run Rust core tests"
        echo "  test-core          Run Rust core tests"
        echo "  test-python        Run Python HTTP client tests (requires service running)"
        echo "  test-js            Run JavaScript tests"
        echo "  test-all           Run all tests"
        echo "  clean              Clean build artifacts"
        echo "  help               Show this help message"
        echo ""
        echo "Note: Python and TypeScript clients are pure HTTP clients."
        echo "      Start the service with: docker run -p 8080:8080 ghcr.io/hlop3z/dinja:latest"
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac
