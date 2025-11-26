# Dinja Documentation

This directory contains the documentation for Dinja, built with [Material for MkDocs](https://squidfunk.github.io/mkdocs-material/).

## Quick Start

### Install Dependencies

```bash
# Using uv (recommended)
uv sync

# Or using uv pip
uv pip install -r requirements.txt
```

The `docs.sh` script will automatically install dependencies if needed.

### Using the Script (Recommended)

The `docs.sh` script provides convenient commands and uses `uv` for dependency management:

```bash
# Install dependencies (if needed)
./docs.sh install

# Start development server
./docs.sh dev
# or just
./docs.sh

# Build for GitHub Pages
./docs.sh build

# Build and deploy to GitHub Pages
./docs.sh deploy
```

The script will automatically check for and install dependencies if `mkdocs` is not found.

### Manual Commands

#### Serve Locally

```bash
mkdocs serve
```

Then open http://127.0.0.1:8000 in your browser.

#### Build Documentation

```bash
mkdocs build
```

The built site will be in the `site/` directory.

## Structure

- `mkdocs.yml` - MkDocs configuration
- `docs/` - Documentation source files
  - `index.md` - Homepage
  - `getting-started/` - Installation and quick start guides
  - `python/` - Python API documentation
  - `rust/` - Rust API documentation
  - `guides/` - Usage guides
  - `development/` - Development documentation

## Customization

Edit `mkdocs.yml` to customize:
- Theme colors and appearance
- Navigation structure
- Plugins and extensions
- Site metadata

## Deployment

### GitHub Pages

The easiest way to deploy is using the script:

```bash
./docs.sh deploy
```

This will:
1. Build the documentation
2. Create/checkout the `gh-pages` branch
3. Copy the built site to the branch
4. Commit and push to GitHub

Alternatively, you can manually build and configure GitHub Pages:

1. Build the docs: `./docs.sh build`
2. Configure GitHub Pages in your repository settings to serve from the `gh-pages` branch
3. Or use GitHub Actions to auto-deploy on push

### Other Hosting

The documentation can be deployed to:
- Netlify
- Vercel
- Any static hosting service

See the [Material for MkDocs documentation](https://squidfunk.github.io/mkdocs-material/) for more deployment options.

