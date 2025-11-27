# Dinja Docker Deployment

This guide explains how to build and run the Dinja HTTP server using Docker with a minimal scratch-based image.

## Features

- **Minimal Image Size**: Uses scratch base image (~10MB final size)
- **Static Binary**: Fully static Rust binary with no dependencies
- **Multi-stage Build**: Optimized build process
- **Production Ready**: Includes health checks and resource limits

## Quick Start

### Using Docker Compose (Recommended)

```bash
# Build and start the server
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the server
docker-compose down
```

### Using Docker CLI

```bash
# Build the image
./docker-build.sh

# Run the container
docker run -d -p 8080:8080 --name dinja-server dinja:latest

# View logs
docker logs -f dinja-server

# Stop the container
docker stop dinja-server
```

## Building the Image

### Basic Build

```bash
./utils/docker-build.sh
```

### Advanced Build Options

```bash
# Build with no cache
./utils/docker-build.sh --no-cache

# Build with custom tag
./utils/docker-build.sh --tag v1.0.0

# Build and push to registry
./utils/docker-build.sh --push --tag v1.0.0

# Custom image name
./utils/docker-build.sh --name myregistry.com/dinja --tag latest
```

### Manual Build

```bash
docker build -t dinja:latest .
```

## Running the Container

### Basic Run

```bash
docker run -p 8080:8080 dinja:latest
```

### With Environment Variables

```bash
docker run -d \
  -p 8080:8080 \
  -e RUST_LOG=debug \
  -e PORT=8080 \
  --name dinja-server \
  dinja:latest
```

### With Volume Mounts (if needed)

```bash
docker run -d \
  -p 8080:8080 \
  -v $(pwd)/custom-static:/static \
  --name dinja-server \
  dinja:latest
```

## Testing the Server

### Health Check

```bash
curl http://localhost:8080/health
```

### Render Endpoint

```bash
curl -X POST http://localhost:8080/render \
  -H "Content-Type: application/json" \
  -d '{
    "mdx": {
      "test.mdx": "# Hello World\n\nThis is MDX content."
    },
    "settings": {
      "output": "html",
      "minify": false
    }
  }'
```

## Docker Image Details

### Image Layers

1. **Builder Stage**: Uses `rust:1.83-slim-bookworm`
   - Installs build dependencies
   - Compiles Rust binary with musl target for static linking
   - Builds with `--release` and `--features http`

2. **Runtime Stage**: Uses `scratch`
   - Copies only the static binary
   - Copies CA certificates for HTTPS
   - Copies static JavaScript files
   - No OS, no package manager, minimal attack surface

### Image Size

- **Builder image**: ~1.5GB (discarded after build)
- **Final image**: ~10MB (minimal!)

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `STATIC_DIR` | `/static` | Directory containing static JS files |
| `PORT` | `8080` | Server port (informational only) |
| `RUST_LOG` | - | Log level (info, debug, warn, error) |

## Docker Compose Configuration

### Basic Setup

```yaml
version: '3.8'

services:
  dinja:
    image: dinja:latest
    ports:
      - "8080:8080"
    restart: unless-stopped
```

### Production Setup

```yaml
version: '3.8'

services:
  dinja:
    image: dinja:latest
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      - RUST_LOG=info
    restart: unless-stopped
    healthcheck:
      test: ["CMD-SHELL", "wget --quiet --tries=1 --spider http://localhost:8080/health || exit 1"]
      interval: 30s
      timeout: 10s
      retries: 3
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 512M
        reservations:
          cpus: '0.5'
          memory: 128M
```

## Kubernetes Deployment

### Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: dinja
spec:
  replicas: 3
  selector:
    matchLabels:
      app: dinja
  template:
    metadata:
      labels:
        app: dinja
    spec:
      containers:
      - name: dinja
        image: dinja:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        resources:
          limits:
            cpu: "1"
            memory: "256Mi"
          requests:
            cpu: "100m"
            memory: "64Mi"
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
```

### Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: dinja
spec:
  selector:
    app: dinja
  ports:
  - port: 80
    targetPort: 8080
  type: LoadBalancer
```

## Troubleshooting

### Container Won't Start

```bash
# Check logs
docker logs dinja-server

# Check if port is already in use
lsof -i :8080

# Inspect the container
docker inspect dinja-server
```

### Image Build Fails

```bash
# Clean build without cache
docker build --no-cache -t dinja:latest .

# Check Rust toolchain
docker run --rm rust:1.83-slim-bookworm rustc --version
```

### Health Check Fails

The health check endpoint is not yet implemented. You'll need to add a `/health` endpoint to the server or modify the health check in docker-compose.yml.

## Security Considerations

- **Scratch Base**: No shell, no package manager - minimal attack surface
- **Static Binary**: No dynamic library dependencies
- **Non-Root**: Consider adding a user in future (scratch makes this tricky)
- **Read-Only Filesystem**: Can be enforced with `--read-only` flag
- **Resource Limits**: Set appropriate CPU/memory limits in production

## Performance Tuning

### Build Time

- Use BuildKit for faster builds: `DOCKER_BUILDKIT=1 docker build .`
- Enable layer caching
- Use `.dockerignore` to minimize context

### Runtime

- Adjust resource limits in docker-compose.yml
- Use multiple replicas for load balancing
- Consider using a reverse proxy (nginx, traefik) in front

## CI/CD Integration

### GitHub Actions

```yaml
name: Build and Push Docker Image

on:
  push:
    tags:
      - 'v*'

jobs:
  docker:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Login to DockerHub
        uses: docker/login-action@v2
        with:
          username: ${{ secrets.DOCKERHUB_USERNAME }}
          password: ${{ secrets.DOCKERHUB_TOKEN }}

      - name: Build and push
        uses: docker/build-push-action@v4
        with:
          context: .
          push: true
          tags: |
            yourusername/dinja:latest
            yourusername/dinja:${{ github.ref_name }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

## Automated CI/CD with GitHub Actions

### Overview

Dinja includes two GitHub Actions workflows for automated Docker image building and publishing:

1. **Manual Build** (`.github/workflows/docker-build-manual.yml`)
2. **Release Build** (`.github/workflows/docker-release.yml`)

### Manual Docker Build

Trigger manually via GitHub Actions UI with custom options.

**Workflow:** `Docker Build (Manual)`

**Options:**
- **Tag**: Custom image tag (default: `latest`)
- **Push to Registry**: Whether to push to GitHub Container Registry (ghcr.io)

**Features:**
- Builds Docker image
- Runs health check tests
- Saves image as artifact (30-day retention)
- Optionally pushes to ghcr.io

**Usage:**
1. Go to Actions tab in GitHub
2. Select "Docker Build (Manual)"
3. Click "Run workflow"
4. Set options and run

**Outputs:**
- Workflow artifact: `docker-image-{tag}-{run_number}`
- Download and load: `docker load -i docker-image-*.tar`

### Release Docker Build

Automatically triggered when a new release is published.

**Workflow:** `Docker Release`

**Triggers:**
- Automatic: On release published/created
- Manual: Via workflow_dispatch with release tag

**Features:**
- Multi-platform builds (linux/amd64, linux/arm64)
- Semantic versioning tags (e.g., `1.0.0`, `1.0`, `1`, `latest`)
- GitHub Container Registry publishing
- Image artifact with SHA256 checksum (90-day retention)
- Automatic release comment with usage instructions

**Version Tags:**
```
ghcr.io/username/dinja:1.0.0    # Specific version
ghcr.io/username/dinja:1.0      # Minor version
ghcr.io/username/dinja:1        # Major version
ghcr.io/username/dinja:latest   # Latest release
```

**Pull Published Images:**
```bash
# Pull specific version
docker pull ghcr.io/username/dinja:1.0.0

# Pull latest
docker pull ghcr.io/username/dinja:latest

# Run
docker run -p 8080:8080 ghcr.io/username/dinja:1.0.0
```

### Image Artifacts

Both workflows save Docker images to `.artifacts/` directory:

**Local Builds:**
```bash
# Build locally
./utils/docker-build.sh --tag v1.0.0

# Find saved image
ls .artifacts/
# Output: dinja-v1.0.0-20240327-123456.tar

# Load on another machine
docker load -i .artifacts/dinja-v1.0.0-20240327-123456.tar
```

**CI/CD Artifacts:**
1. Go to workflow run in GitHub Actions
2. Download artifact from "Artifacts" section
3. Extract and load:
```bash
unzip docker-image-*.zip
docker load -i *.tar
```

### Setting Up GitHub Container Registry

To use ghcr.io, ensure you have the correct permissions:

1. **Personal Access Token** (if needed):
   - Go to Settings → Developer settings → Personal access tokens
   - Create token with `write:packages` scope

2. **Login to ghcr.io:**
```bash
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
```

3. **Pull private images:**
```bash
docker pull ghcr.io/username/dinja:latest
```

## Next Steps

1. Add `/health` endpoint to the server
2. Consider adding OpenTelemetry for observability
3. Add metrics endpoint for Prometheus
4. Implement graceful shutdown
5. Add configuration file support

## Resources

- [Docker Documentation](https://docs.docker.com/)
- [Docker Compose Reference](https://docs.docker.com/compose/compose-file/)
- [Rust in Docker](https://docs.docker.com/language/rust/)
- [Multi-stage Builds](https://docs.docker.com/build/building/multi-stage/)
