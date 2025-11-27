# Utility Scripts

This directory contains utility scripts for building, testing, and running Dinja.

## Scripts

### `build.sh`

Main build and test script for the entire project.

**Usage:**
```bash
./utils/build.sh [command]
```

**Commands:**
- `test` - Run all tests (Rust + Python)
- `build` - Build all components
- `clean` - Clean build artifacts
- (no args) - Run full build pipeline

**Examples:**
```bash
# Run all tests
./utils/build.sh test

# Full build
./utils/build.sh

# Clean build artifacts
./utils/build.sh clean
```

### `run.sh`

Start the Dinja HTTP server for local development.

**Usage:**
```bash
./utils/run.sh
```

Starts the HTTP server on `http://127.0.0.1:8080` with the http feature enabled.

**Environment Variables:**
- `HOST` - Server host (default: `0.0.0.0`)
- `PORT` - Server port (default: `8080`)
- `STATIC_DIR` - Static files directory (default: `./core/static`)

**Example:**
```bash
# Start with default settings
./utils/run.sh

# Start with custom host and port
HOST=127.0.0.1 PORT=3000 ./utils/run.sh
```

### `docker-build.sh`

Build Docker image for the Dinja HTTP server using scratch base image.

**Usage:**
```bash
./utils/docker-build.sh [options]
```

**Options:**
- `--no-cache` - Build without using cache
- `--tag TAG` - Custom image tag (default: `latest`)
- `--name NAME` - Custom image name (default: `dinja`)
- `--push` - Push image to registry after build

**Examples:**
```bash
# Basic build
./utils/docker-build.sh

# Build with custom tag
./utils/docker-build.sh --tag v1.0.0

# Build without cache
./utils/docker-build.sh --no-cache

# Build and push
./utils/docker-build.sh --push --tag v1.0.0

# Build with custom name and tag
./utils/docker-build.sh --name myregistry.com/dinja --tag production --push
```

## Notes

- **Scripts can be run from anywhere**: The scripts automatically detect the project root directory, so you can run them from any location:
  ```bash
  # From project root
  ./utils/build.sh test

  # From subdirectory
  cd core && ../utils/build.sh test

  # From utils directory
  cd utils && ./build.sh test
  ```
- Scripts are executable (`chmod +x utils/*.sh`)
- All operations are performed relative to the project root, not the script location
- See [DOCKER.md](../DOCKER.md) for detailed Docker deployment instructions
- See [docs/development/](../docs/docs/development/) for development guides
