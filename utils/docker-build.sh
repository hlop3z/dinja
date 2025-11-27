#!/bin/bash
# Docker build script for Dinja HTTP Server
# Creates a minimal scratch-based Docker image

set -e

# Get the directory where this script is located (utils/)
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Get the project root directory (parent of utils/)
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to project root for docker build
cd "$PROJECT_ROOT"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}Dinja Docker Build Pipeline${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Configuration
IMAGE_NAME="${IMAGE_NAME:-dinja}"
IMAGE_TAG="${IMAGE_TAG:-latest}"
FULL_IMAGE_NAME="${IMAGE_NAME}:${IMAGE_TAG}"

# Parse command line arguments
BUILD_ARGS=""
PUSH=false
NO_CACHE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --push)
            PUSH=true
            shift
            ;;
        --no-cache)
            NO_CACHE=true
            BUILD_ARGS="$BUILD_ARGS --no-cache"
            shift
            ;;
        --tag)
            IMAGE_TAG="$2"
            FULL_IMAGE_NAME="${IMAGE_NAME}:${IMAGE_TAG}"
            shift 2
            ;;
        --name)
            IMAGE_NAME="$2"
            FULL_IMAGE_NAME="${IMAGE_NAME}:${IMAGE_TAG}"
            shift 2
            ;;
        *)
            echo -e "${YELLOW}Unknown option: $1${NC}"
            echo "Usage: $0 [--push] [--no-cache] [--tag TAG] [--name NAME]"
            exit 1
            ;;
    esac
done

echo -e "${GREEN}[INFO]${NC} Building Docker image: ${FULL_IMAGE_NAME}"
echo ""

# Build the Docker image
echo -e "${GREEN}[INFO]${NC} Running docker build..."
docker build $BUILD_ARGS -t "${FULL_IMAGE_NAME}" .

if [ $? -eq 0 ]; then
    echo ""
    echo -e "${GREEN}[SUCCESS]${NC} Docker image built successfully!"
    echo -e "${GREEN}[INFO]${NC} Image: ${FULL_IMAGE_NAME}"

    # Show image size
    SIZE=$(docker images "${FULL_IMAGE_NAME}" --format "{{.Size}}")
    echo -e "${GREEN}[INFO]${NC} Image size: ${SIZE}"
    echo ""

    # Save image to .artifacts/ directory for internal distribution
    echo -e "${GREEN}[INFO]${NC} Saving image to .artifacts/ directory..."
    ARTIFACTS_DIR="${PROJECT_ROOT}/.artifacts"
    mkdir -p "${ARTIFACTS_DIR}"

    # Generate filename with timestamp and tag
    TIMESTAMP=$(date +%Y%m%d-%H%M%S)
    SAFE_IMAGE_NAME=$(echo "${IMAGE_NAME}" | sed 's/[\/:]/-/g')
    TAR_FILENAME="${SAFE_IMAGE_NAME}-${IMAGE_TAG}-${TIMESTAMP}.tar"
    TAR_PATH="${ARTIFACTS_DIR}/${TAR_FILENAME}"

    docker save "${FULL_IMAGE_NAME}" -o "${TAR_PATH}"
    if [ $? -eq 0 ]; then
        TAR_SIZE=$(du -h "${TAR_PATH}" | cut -f1)
        echo -e "${GREEN}[SUCCESS]${NC} Image saved to: ${TAR_PATH}"
        echo -e "${GREEN}[INFO]${NC} Archive size: ${TAR_SIZE}"
        echo ""
        echo -e "${BLUE}To load this image on another machine:${NC}"
        echo -e "  docker load -i ${TAR_PATH}"
        echo ""
    else
        echo -e "${YELLOW}[WARN]${NC} Failed to save image to .artifacts/"
    fi

    # Tag as latest if not already
    if [ "${IMAGE_TAG}" != "latest" ]; then
        echo -e "${GREEN}[INFO]${NC} Tagging as latest..."
        docker tag "${FULL_IMAGE_NAME}" "${IMAGE_NAME}:latest"
    fi

    # Push if requested
    if [ "$PUSH" = true ]; then
        echo -e "${GREEN}[INFO]${NC} Pushing image to registry..."
        docker push "${FULL_IMAGE_NAME}"
        if [ "${IMAGE_TAG}" != "latest" ]; then
            docker push "${IMAGE_NAME}:latest"
        fi
    fi

    echo -e "${BLUE}========================================${NC}"
    echo -e "${GREEN}To run the container:${NC}"
    echo -e "  docker run -p 8080:8080 ${FULL_IMAGE_NAME}"
    echo ""
    echo -e "${GREEN}Or use docker-compose:${NC}"
    echo -e "  docker-compose up -d"
    echo ""
    echo -e "${GREEN}To test the server:${NC}"
    echo -e "  curl http://localhost:8080/health"
    echo -e "${BLUE}========================================${NC}"
else
    echo ""
    echo -e "${YELLOW}[ERROR]${NC} Docker build failed!"
    exit 1
fi
