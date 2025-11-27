# GitHub Actions Workflows

This directory contains automated workflows for building, testing, and publishing Dinja.

## Workflows

### Docker Build (Manual)

**File:** `docker-build-manual.yml`

**Trigger:** Manual dispatch via GitHub Actions UI

**Purpose:** Build Docker images on-demand with custom options

**Inputs:**
- `tag` (required): Docker image tag (default: `latest`)
- `push_to_registry` (boolean): Push to GitHub Container Registry

**Outputs:**
- Docker image artifact (30-day retention)
- Optional push to `ghcr.io/owner/repo:tag`

**Usage:**
1. Navigate to Actions tab
2. Select "Docker Build (Manual)"
3. Click "Run workflow"
4. Configure options:
   - Set custom tag (e.g., `v1.0.0`, `dev`, `staging`)
   - Choose whether to push to registry
5. Download artifact from workflow run or pull from ghcr.io

---

### Docker Release

**File:** `docker-release.yml`

**Trigger:**
- Automatic: When a GitHub release is published/created
- Manual: Via workflow_dispatch with release tag

**Purpose:** Automatically build and publish Docker images for releases

**Features:**
- Multi-platform builds (linux/amd64, linux/arm64)
- Semantic version tagging (e.g., `1.0.0`, `1.0`, `1`, `latest`)
- Automatic publishing to GitHub Container Registry
- Image artifacts with SHA256 checksums (90-day retention)
- Automated release comment with usage instructions

**Version Tags Generated:**
- `ghcr.io/owner/repo:1.0.0` (exact version)
- `ghcr.io/owner/repo:1.0` (minor version)
- `ghcr.io/owner/repo:1` (major version)
- `ghcr.io/owner/repo:latest` (always points to latest release)

**Outputs:**
- Published images to ghcr.io
- Docker image tarball artifact with checksum
- Automated comment on release with pull/run instructions

---

### Python Package Publishing (Manual)

**File:** `publish-python-manual.yml`

**Purpose:** Manually publish Python packages to PyPI

---

### Release

**File:** `release.yml`

**Purpose:** Automated release workflow for publishing Python packages

---

## Image Artifacts

All Docker workflows save images to the `.artifacts/` directory and upload them as workflow artifacts.

### Local Builds

When using `./utils/docker-build.sh`:
```bash
# Images are saved to:
.artifacts/dinja-{tag}-{timestamp}.tar
```

### CI/CD Artifacts

Workflow artifacts can be downloaded from:
1. Actions tab → Select workflow run
2. Artifacts section at bottom
3. Download zip file
4. Extract and load:
```bash
unzip docker-image-*.zip
docker load -i *.tar
```

## Permissions

Workflows require the following permissions:
- `contents: read` - Read repository content
- `packages: write` - Publish to GitHub Container Registry

These are configured in each workflow file.

## Setup

### GitHub Container Registry

To publish images to ghcr.io:

1. **Public repositories**: Automatic, uses `GITHUB_TOKEN`
2. **Private repositories**: Ensure `packages: write` permission

### Pull Images

```bash
# Public images (no authentication needed)
docker pull ghcr.io/owner/repo:latest

# Private images (requires authentication)
echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin
docker pull ghcr.io/owner/repo:latest
```

## Best Practices

1. **Use semantic versioning** for releases (e.g., `v1.0.0`, `v2.1.3`)
2. **Test locally first** with `./utils/docker-build.sh`
3. **Use manual workflow** for testing before releases
4. **Let release workflow** handle production publishes
5. **Keep artifacts** for important versions (download from Actions)

## Troubleshooting

### Build Failures

1. Check workflow logs in Actions tab
2. Test locally: `./utils/docker-build.sh`
3. Verify Dockerfile syntax
4. Check static files in `core/static/`

### Registry Push Failures

1. Verify `packages: write` permission
2. Check repository settings → Actions → Workflow permissions
3. Ensure image name follows ghcr.io naming (lowercase)

### Multi-platform Build Issues

1. Ensure Docker Buildx is available
2. Check platform-specific dependencies
3. Test platforms separately if needed

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [GitHub Container Registry](https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry)
- [Docker Build Push Action](https://github.com/docker/build-push-action)
- [Semantic Versioning](https://semver.org/)
