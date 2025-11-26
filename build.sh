#!/bin/bash

# Build, test, and development script for dinja
# Usage:
#   ./build.sh build      - Build Rust workspace
#   ./build.sh test       - Run Rust tests
#   ./build.sh dev        - Install Python bindings in development mode
#   ./build.sh clean      - Clean build artifacts
#   ./build.sh all        - Build, test, and install dev mode

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

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

# Check if Python is available (via uv or directly)
check_python() {
    # Prefer uv if available
    if check_uv; then
        uv python list >/dev/null 2>&1 && return 0
        # uv can install Python on demand, so consider it available
        return 0
    fi
    
    # Fall back to direct Python check
    if command_exists python3; then
        python3 --version >/dev/null 2>&1 && return 0
    fi
    if command_exists python; then
        python --version >/dev/null 2>&1 && return 0
    fi
    return 1
}

# Get Python command (uv run or direct python)
get_python_cmd() {
    if check_uv; then
        echo "uv run"
    elif command_exists python3; then
        echo "python3"
    elif command_exists python; then
        echo "python"
    else
        return 1
    fi
}

# Get Python path for PyO3 build configuration
get_python_path() {
    # Try uv python find first
    if check_uv; then
        local python_path
        if python_path=$(uv python find 2>/dev/null); then
            echo "$python_path"
            return 0
        fi
    fi
    
    # Fall back to system Python
    if command_exists python3; then
        command -v python3
        return 0
    fi
    if command_exists python; then
        command -v python
        return 0
    fi
    
    return 1
}

# Setup Python environment for PyO3 builds
setup_pyo3_env() {
    local python_path
    
    # Try to get Python path
    if python_path=$(get_python_path); then
        export PYO3_PYTHON="$python_path"
        log_info "Setting PYO3_PYTHON=$python_path"
        return 0
    fi
    
    # If not found and uv is available, try uv init (only if no pyproject.toml exists)
    if check_uv; then
        if [ ! -f "pyproject.toml" ]; then
            log_info "Python not found, running uv init..."
            if uv init --no-readme >/dev/null 2>&1; then
                # Try again after uv init
                if python_path=$(get_python_path); then
                    export PYO3_PYTHON="$python_path"
                    log_info "Setting PYO3_PYTHON=$python_path"
                    return 0
                fi
            fi
        else
            log_info "Python not found, trying uv python install..."
            if uv python install >/dev/null 2>&1; then
                # Try again after installation
                if python_path=$(get_python_path); then
                    export PYO3_PYTHON="$python_path"
                    log_info "Setting PYO3_PYTHON=$python_path"
                    return 0
                fi
            fi
        fi
    fi
    
    # If still not found, exit with error
    log_error "Python is required but not found."
    if check_uv; then
        log_error "Tried to set up Python via uv but it's still not available."
        log_error "Try running: uv python install"
    else
        log_error "Please install Python 3.13+ or install uv (https://github.com/astral-sh/uv)"
    fi
    exit 1
}

# Build core crate only (no Python required)
build_core() {
    log_info "Building core crate (no Python required)..."
    cargo build -p dinja-core
    log_info "Core build completed successfully"
}

# Build core crate in release mode
build_core_release() {
    log_info "Building core crate (release mode, no Python required)..."
    cargo build -p dinja-core --release
    log_info "Core release build completed successfully"
}

# Build Rust workspace (requires Python for python-bindings)
build_rust() {
    if ! check_python; then
        log_warn "Python not found. Building core crate only."
        if check_uv; then
            log_warn "uv is installed. To build Python bindings, run: ./build.sh dev"
        else
            log_warn "To build Python bindings, install Python (or uv) and run: ./build.sh dev"
        fi
        build_core
        return
    fi
    
    log_info "Building Rust workspace..."
    if check_uv; then
        log_info "Using uv for Python environment..."
        setup_pyo3_env
    fi
    cargo build --workspace
    log_info "Rust build completed successfully"
}

# Build Rust workspace in release mode
build_rust_release() {
    if ! check_python; then
        log_warn "Python not found. Building core crate only (release mode)."
        if check_uv; then
            log_warn "uv is installed. To build Python bindings, run: ./build.sh build-python"
        else
            log_warn "To build Python bindings, install Python (or uv) and run: ./build.sh build-python"
        fi
        build_core_release
        return
    fi
    
    log_info "Building Rust workspace (release mode)..."
    if check_uv; then
        log_info "Using uv for Python environment..."
        setup_pyo3_env
    fi
    cargo build --workspace --release
    log_info "Rust release build completed successfully"
}

# Test core crate only (no Python required)
test_core() {
    log_info "Running core crate tests (no Python required)..."
    cargo test -p dinja-core
    log_info "Core tests completed successfully"
}

# Run Rust tests (requires Python for python-bindings)
test_rust() {
    if ! check_python; then
        log_warn "Python not found. Testing core crate only."
        if check_uv; then
            log_warn "uv is installed. To test Python bindings, install Python and run: ./build.sh test"
        fi
        test_core
        return
    fi
    
    log_info "Running Rust tests..."
    if check_uv; then
        log_info "Using uv for Python environment..."
        setup_pyo3_env
    fi
    cargo test --workspace
    log_info "Rust tests completed successfully"
}

# Setup virtualenv for maturin (maturin requires VIRTUAL_ENV to be set)
setup_maturin_venv() {
    local venv_path="$SCRIPT_DIR/python-bindings/.venv"
    
    # Check if virtualenv already exists
    if [ -d "$venv_path" ]; then
        log_info "Using existing virtualenv at $venv_path"
    else
        log_info "Creating virtualenv at $venv_path..."
        if check_uv; then
            uv venv "$venv_path"
        else
            # Fall back to python -m venv
            if command_exists python3; then
                python3 -m venv "$venv_path"
            elif command_exists python; then
                python -m venv "$venv_path"
            else
                log_error "Cannot create virtualenv: Python not found"
                exit 1
            fi
        fi
    fi
    
    # Set VIRTUAL_ENV so maturin can find it
    export VIRTUAL_ENV="$venv_path"
    log_info "Set VIRTUAL_ENV=$venv_path"
}

# Install Python bindings in development mode
dev_mode() {
    if ! check_python; then
        log_error "Python is required but not found."
        if ! check_uv; then
            log_error "Please install Python 3.13+ or install uv (https://github.com/astral-sh/uv)"
        else
            log_error "uv is installed but Python setup failed. Try: uv python install"
        fi
        exit 1
    fi
    
    log_info "Installing Python bindings in development mode..."
    
    cd python-bindings
    
    # Setup virtualenv for maturin
    setup_maturin_venv
    
    if check_uv; then
        # Use uv tool run (uvx) to run maturin - it handles installation automatically
        log_info "Using uv to run maturin..."
        uv tool run maturin develop
    else
        # Check if maturin is available in system
        if ! command_exists maturin; then
            if command_exists pip; then
                log_info "Installing maturin with pip..."
                pip install maturin
            else
                log_error "maturin is not installed and pip is not available."
                log_error "Install maturin with: pip install maturin (or use uv: uv tool run maturin)"
                exit 1
            fi
        fi
        maturin develop
    fi
    
    cd ..
    
    log_info "Python bindings installed in development mode"
    if check_uv; then
        log_info "You can now use: uv run python -c 'import dinja; print(dinja.hello_py(\"World\"))'"
    else
        log_info "You can now use: python -c 'import dinja; print(dinja.hello_py(\"World\"))'"
    fi
}

# Build Python wheels
build_python() {
    if ! check_python; then
        log_error "Python is required but not found."
        if ! check_uv; then
            log_error "Please install Python 3.13+ or install uv (https://github.com/astral-sh/uv)"
        else
            log_error "uv is installed but Python setup failed. Try: uv python install"
        fi
        exit 1
    fi
    
    log_info "Building Python wheels..."
    
    cd python-bindings
    
    # Setup virtualenv for maturin
    setup_maturin_venv
    
    if check_uv; then
        # Use uv tool run (uvx) to run maturin - it handles installation automatically
        log_info "Using uv to run maturin..."
        uv tool run maturin build --release
    else
        # Check if maturin is available in system
        if ! command_exists maturin; then
            if command_exists pip; then
                log_info "Installing maturin with pip..."
                pip install maturin
            else
                log_error "maturin is not installed and pip is not available."
                log_error "Install maturin with: pip install maturin (or use uv: uv tool run maturin)"
                exit 1
            fi
        fi
        maturin build --release
    fi
    
    cd ..
    
    log_info "Python wheels built successfully"
}

# Clean build artifacts
clean() {
    log_info "Cleaning build artifacts..."
    cargo clean --workspace
    cd python-bindings
    rm -rf target/wheels
    rm -rf dist
    cd ..
    log_info "Clean completed"
}

# Run all: build, test, and dev mode
run_all() {
    log_info "Running full build, test, and dev setup..."
    build_rust
    test_rust
    dev_mode
    log_info "All tasks completed successfully!"
}

# Main command dispatcher
case "${1:-help}" in
    build)
        build_rust
        ;;
    build-core)
        build_core
        ;;
    build-release)
        build_rust_release
        ;;
    build-core-release)
        build_core_release
        ;;
    test)
        test_rust
        ;;
    test-core)
        test_core
        ;;
    dev)
        dev_mode
        ;;
    build-python)
        build_python
        ;;
    clean)
        clean
        ;;
    all)
        run_all
        ;;
    help|--help|-h)
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  build              Build Rust workspace (debug mode, falls back to core if no Python)"
        echo "  build-core         Build core crate only (no Python required)"
        echo "  build-release      Build Rust workspace (release mode, falls back to core if no Python)"
        echo "  build-core-release Build core crate only in release mode (no Python required)"
        echo "  test               Run Rust tests (falls back to core tests if no Python)"
        echo "  test-core          Run core crate tests only (no Python required)"
    echo "  dev                Install Python bindings in development mode (uses uv if available)"
    echo "  build-python       Build Python wheels (uses uv if available)"
        echo "  clean              Clean all build artifacts"
        echo "  all                Build, test, and install dev mode"
        echo "  help               Show this help message"
        ;;
    *)
        log_error "Unknown command: $1"
        echo "Run '$0 help' for usage information"
        exit 1
        ;;
esac

