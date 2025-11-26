#!/bin/bash

# Dinja Documentation Script
# Usage: ./docs.sh [dev|build|deploy|install]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check if uv is installed
check_uv() {
    if ! command -v uv &> /dev/null; then
        echo -e "${RED}Error: uv is not installed${NC}"
        echo "Install it from: https://github.com/astral-sh/uv"
        exit 1
    fi
}

# Install dependencies using uv
install() {
    check_uv
    echo -e "${GREEN}Installing documentation dependencies with uv...${NC}"
    
    if [ -f "pyproject.toml" ]; then
        echo -e "${YELLOW}Using uv sync (pyproject.toml)${NC}"
        uv sync
    elif [ -f "requirements.txt" ]; then
        echo -e "${YELLOW}Using uv pip install (requirements.txt)${NC}"
        uv pip install -r requirements.txt
    else
        echo -e "${RED}Error: No pyproject.toml or requirements.txt found${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Dependencies installed!${NC}"
}

# Get mkdocs command (use uv run if available, otherwise direct)
get_mkdocs_cmd() {
    if command -v uv &> /dev/null && [ -f "pyproject.toml" ] || [ -f "uv.lock" ]; then
        # Use uv run to execute mkdocs from the virtual environment
        echo "uv run mkdocs"
    elif command -v mkdocs &> /dev/null; then
        # Use directly installed mkdocs
        echo "mkdocs"
    else
        echo ""
    fi
}

# Check if mkdocs is available (either directly or via uv)
check_mkdocs() {
    local mkdocs_cmd=$(get_mkdocs_cmd)
    
    if [ -z "$mkdocs_cmd" ]; then
        echo -e "${YELLOW}mkdocs is not installed. Installing dependencies...${NC}"
        install
        
        # Check again after installation
        mkdocs_cmd=$(get_mkdocs_cmd)
        if [ -z "$mkdocs_cmd" ]; then
            echo -e "${RED}Error: mkdocs is still not available after installation${NC}"
            echo "Try running: uv sync"
            exit 1
        fi
    fi
}

# Function to run dev server
dev() {
    check_mkdocs
    local mkdocs_cmd=$(get_mkdocs_cmd)
    echo -e "${GREEN}Starting MkDocs development server...${NC}"
    echo -e "${YELLOW}Open http://localhost:8000 in your browser${NC}"
    echo -e "${YELLOW}Press Ctrl+C to stop the server${NC}"
    echo ""
    $mkdocs_cmd serve
}

# Function to build documentation
build() {
    check_mkdocs
    local mkdocs_cmd=$(get_mkdocs_cmd)
    echo -e "${GREEN}Building documentation for GitHub Pages...${NC}"
    
    # Clean previous build
    if [ -d "site" ]; then
        echo -e "${YELLOW}Cleaning previous build...${NC}"
        rm -rf site
    fi
    
    # Build the site
    $mkdocs_cmd build
    
    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✓ Documentation built successfully!${NC}"
        echo -e "${YELLOW}Output directory: site/${NC}"
        echo ""
        echo "To deploy to GitHub Pages:"
        echo "  1. Run: ./docs.sh deploy"
        echo "  2. Or manually commit and push the site/ directory to the gh-pages branch"
    else
        echo -e "${RED}✗ Build failed${NC}"
        exit 1
    fi
}

# Function to deploy to GitHub Pages (using gh-pages branch)
deploy() {
    echo -e "${GREEN}Building and deploying to GitHub Pages...${NC}"
    
    # Build first
    build
    
    # Check if git is available
    if ! command -v git &> /dev/null; then
        echo -e "${RED}Error: git is not installed${NC}"
        exit 1
    fi
    
    # Check if we're in a git repository
    if ! git rev-parse --git-dir > /dev/null 2>&1; then
        echo -e "${RED}Error: Not in a git repository${NC}"
        exit 1
    fi
    
    echo -e "${YELLOW}Deploying to gh-pages branch...${NC}"
    
    # Create or checkout gh-pages branch
    if git show-ref --verify --quiet refs/heads/gh-pages; then
        git checkout gh-pages
        git pull origin gh-pages 2>/dev/null || true
    else
        git checkout --orphan gh-pages
        git rm -rf . 2>/dev/null || true
    fi
    
    # Copy site contents to root
    cp -r site/* .
    
    # Add and commit
    git add .
    git commit -m "Deploy documentation to GitHub Pages" || echo "No changes to commit"
    
    echo -e "${GREEN}✓ Documentation deployed!${NC}"
    echo -e "${YELLOW}Pushing to origin/gh-pages...${NC}"
    git push origin gh-pages
    
    # Return to original branch
    CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "main")
    if [ "$CURRENT_BRANCH" != "gh-pages" ]; then
        # We're already on the branch we want
        :
    else
        # Try to return to main/master
        git checkout main 2>/dev/null || git checkout master 2>/dev/null || echo "Note: Could not return to main/master branch"
    fi
    
    echo -e "${GREEN}✓ Deployment complete!${NC}"
    echo -e "${YELLOW}Your documentation should be available at:${NC}"
    echo "  https://hlop3z.github.io/dinja/"
    echo ""
    echo "Note: It may take a few minutes for GitHub Pages to update."
}

# Function to show usage
usage() {
    echo "Dinja Documentation Script"
    echo ""
    echo "Usage: ./docs.sh [command]"
    echo ""
    echo "Commands:"
    echo "  install  Install dependencies using uv (default if mkdocs not found)"
    echo "  dev      Start development server (default)"
    echo "  build    Build documentation for production"
    echo "  deploy   Build and deploy to GitHub Pages"
    echo "  help     Show this help message"
    echo ""
    echo "Examples:"
    echo "  ./docs.sh install  # Install dependencies"
    echo "  ./docs.sh dev      # Start dev server"
    echo "  ./docs.sh build    # Build for GitHub Pages"
    echo "  ./docs.sh deploy   # Build and deploy"
    echo ""
    echo "Note: This script uses uv for dependency management"
}

# Main script logic
case "${1:-dev}" in
    install)
        install
        ;;
    dev)
        dev
        ;;
    build)
        build
        ;;
    deploy)
        deploy
        ;;
    help|--help|-h)
        usage
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        echo ""
        usage
        exit 1
        ;;
esac

