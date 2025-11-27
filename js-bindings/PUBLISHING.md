# Publishing @dinja/core to npm

This document describes how to publish the `@dinja/core` package to npm using GitHub Actions with **Trusted Publishing** (OIDC).

## Why Trusted Publishing?

Trusted Publishing uses OpenID Connect (OIDC) to authenticate with npm, which is **more secure** than using long-lived access tokens:

✅ **No secrets stored** - No tokens in GitHub secrets
✅ **Short-lived credentials** - Tokens expire automatically
✅ **Scoped access** - Limited to specific workflows and repositories
✅ **Audit trail** - Better visibility into who published what
✅ **No token leaks** - Can't be accidentally committed or exposed

## Prerequisites

### 1. npm Account Setup

**Create an account:**

- Sign up at [npmjs.com](https://www.npmjs.com/)
- Request access to the `@dinja` scope (or use your own scope)

**Configure Trusted Publishing:**

1. **Go to your package page** (or create it first):
   - <https://www.npmjs.com/package/@dinja/core>

2. **Navigate to Publishing Access settings**:
   - Click on your package
   - Go to "Settings" tab
   - Scroll to "Publishing Access" section

3. **Add GitHub Actions as a publisher**:
   - Click "Add publisher"
   - Select "GitHub Actions" as the provider
   - Fill in the form:
     - **Repository owner:** `yourusername` (or organization)
     - **Repository name:** `dinja`
     - **Workflow name:** `npm-publish-release.yml` (or `npm-publish-manual.yml`)
     - **Environment:** Leave empty (we don't use GitHub environments)

4. **Repeat for both workflows** (if using both):
   - Add another publisher for `npm-publish-manual.yml`

**Example configuration:**

```
Provider: GitHub Actions
Repository: yourusername/dinja
Workflow: npm-publish-release.yml
Environment: (leave empty)
```

### 2. First-Time Package Publishing

⚠️ **Important:** For the first version of a package, you need to publish manually once:

```bash
cd js-bindings

# Login to npm
npm login

# Publish first version
npm publish --access public

# Now Trusted Publishing is configured and workflows will work
```

This is an npm requirement - Trusted Publishing only works for packages that already exist.

## Publishing Methods

### 1. Automated Release (Recommended)

The automated workflow triggers when you create a GitHub release.

**Steps:**

1. **Create a git tag:**

   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **Create a GitHub release:**
   - Go to: Repository → Releases → Draft a new release
   - Choose the tag you just created (e.g., `v0.1.0`)
   - Fill in the release title and description
   - Check "Set as a pre-release" if publishing alpha/beta/rc versions
   - Click "Publish release"

3. **Workflow automatically:**
   - Builds native bindings for all platforms (Windows, macOS, Linux - x64, ARM64, musl)
   - Publishes to npm with the appropriate tag
   - Updates the release notes with npm installation instructions

**Version tag detection:**

- Stable releases: `v1.0.0` → npm tag: `latest`
- Pre-releases: `v1.0.0-beta.1` → npm tag: `beta`
- Pre-releases: `v1.0.0-alpha.1` → npm tag: `alpha`
- Pre-releases: `v1.0.0-rc.1` → npm tag: `rc`
- Other pre-releases: → npm tag: `next`

### 2. Manual Release

Use the manual workflow for ad-hoc releases or testing.

**Steps:**

1. **Trigger the workflow:**
   - Go to: Actions → "Publish to npm (Manual)" → Run workflow

2. **Configure options:**
   - **Version:** Enter the version number (e.g., `0.1.0`)
   - **npm tag:** Choose `latest`, `beta`, `alpha`, or `next`
   - **Dry run:** Check this to build without publishing (for testing)

3. **Click "Run workflow"**

The workflow will:

- Build bindings for all platforms
- Update package.json version
- Publish to npm (or just show what would be published in dry run mode)
- Create a GitHub release (unless dry run)

## npm Tags

npm tags allow users to install specific versions:

- **latest** (default): Stable production releases

  ```bash
  npm install @dinja/core
  npm install @dinja/core@latest
  ```

- **beta**: Beta pre-releases

  ```bash
  npm install @dinja/core@beta
  ```

- **alpha**: Alpha pre-releases

  ```bash
  npm install @dinja/core@alpha
  ```

- **next**: Canary/nightly builds

  ```bash
  npm install @dinja/core@next
  ```

## Platform Support

The workflows build native bindings for:

| Platform | Architecture | Target Triple |
|----------|-------------|---------------|
| macOS | x64 | `x86_64-apple-darwin` |
| macOS | ARM64 | `aarch64-apple-darwin` |
| Windows | x64 | `x86_64-pc-windows-msvc` |
| Windows | ARM64 | `aarch64-pc-windows-msvc` |
| Linux (GNU) | x64 | `x86_64-unknown-linux-gnu` |
| Linux (GNU) | ARM64 | `aarch64-unknown-linux-gnu` |
| Linux (musl) | x64 | `x86_64-unknown-linux-musl` |
| Linux (musl) | ARM64 | `aarch64-unknown-linux-musl` |

## Versioning

Follow [Semantic Versioning](https://semver.org/):

- **MAJOR.MINOR.PATCH** (e.g., `1.2.3`)
- **MAJOR**: Breaking changes
- **MINOR**: New features (backward compatible)
- **PATCH**: Bug fixes (backward compatible)

Pre-release versions:

- **alpha**: `1.0.0-alpha.1` - Early testing
- **beta**: `1.0.0-beta.1` - Feature complete, testing
- **rc**: `1.0.0-rc.1` - Release candidate

## Testing Before Publishing

**Dry run test:**

```bash
# Use the manual workflow with "Dry run" checked
# Or run locally:
cd js-bindings
npm publish --dry-run
```

**Local test:**

```bash
cd js-bindings
npm pack
# This creates a .tgz file you can test installing
npm install ../dinja-core-0.1.0.tgz
```

## Troubleshooting

### Build fails for a specific platform

- Check the GitHub Actions logs for the failing platform
- Ensure Rust toolchain supports the target
- Verify dependencies are available for that platform

### npm publish fails with authentication error

**Error:** `npm error code ENEEDAUTH` or `npm error code E401`

**Solutions:**

1. **Verify Trusted Publishing is configured** on npm:
   - Go to <https://www.npmjs.com/package/@dinja/core/access>
   - Check that GitHub Actions publisher is listed
   - Ensure repository and workflow names match exactly

2. **Check workflow permissions**:
   - Verify the workflow has `id-token: write` permission
   - Check that the job has proper permissions block

3. **Ensure package exists**:
   - Trusted Publishing only works for existing packages
   - Publish the first version manually (see "First-Time Package Publishing")

### npm publish fails with 403 Forbidden

**Error:** `npm error code E403`

**Possible causes:**

- Package name is already taken by someone else
- You don't have access to the `@dinja` scope
- Trusted Publishing configuration doesn't match your repository

**Solutions:**

- Verify you own the `@dinja` scope on npm
- Check Trusted Publishing settings match your repository exactly
- Try using a different package name or scope you control

### Version already published

- npm doesn't allow republishing the same version
- Increment the version number and try again
- Use a pre-release version for testing (e.g., `0.1.0-test.1`)

### Provenance attestation fails

**Error:** `npm error Failed to generate provenance`

**Solutions:**

- Ensure the workflow has `id-token: write` permission
- Check that you're using npm v9.5.0 or later
- Verify the package name and version are valid

### OIDC token exchange fails

**Error:** `Error: Unable to get ACTIONS_ID_TOKEN_REQUEST_URL`

**Solutions:**

- Verify repository settings allow workflow permissions
- Go to: Settings → Actions → General → Workflow permissions
- Ensure "Read and write permissions" is enabled

## Post-Publish Checklist

After publishing:

- [ ] Verify the package on npm: <https://www.npmjs.com/package/@dinja/core>
- [ ] Test installation: `npm install @dinja/core`
- [ ] Check that all platform binaries are included
- [ ] Verify provenance attestation is present (see below)
- [ ] Update documentation if needed
- [ ] Announce the release (if major version)

## Provenance & Supply Chain Security

With Trusted Publishing and `--provenance` flag, your package includes:

### What is Provenance?

Provenance is a cryptographically signed attestation that links your npm package to its source code and build process. It proves:

- ✅ **Source:** Which GitHub repository the package came from
- ✅ **Commit:** The exact commit SHA that was built
- ✅ **Workflow:** Which GitHub Actions workflow built it
- ✅ **Integrity:** The package hasn't been tampered with

### Viewing Provenance

**On npm website:**

1. Go to <https://www.npmjs.com/package/@dinja/core>
2. Click on a specific version
3. Look for the "Provenance" badge
4. Click it to see the full attestation

**Using npm CLI:**

```bash
npm view @dinja/core@latest --json | grep provenance -A 10
```

**Verify during installation:**

```bash
npm install @dinja/core --foreground-scripts
# npm will verify the provenance automatically
```

### Benefits for Users

Users of your package get:

- **Transparency:** They can verify exactly what source code was used
- **Trust:** Cryptographic proof the package came from your repository
- **Security:** Detection of supply chain attacks or tampering
- **Audit trail:** Complete history of what was built and when

### Example Provenance Info

```json
{
  "predicateType": "https://slsa.dev/provenance/v1",
  "subject": {
    "name": "@dinja/core",
    "digest": { "sha512": "..." }
  },
  "predicate": {
    "buildType": "https://github.com/actions/...",
    "builder": {
      "id": "https://github.com/yourusername/dinja/actions/runs/..."
    },
    "invocation": {
      "configSource": {
        "uri": "git+https://github.com/yourusername/dinja",
        "digest": { "sha1": "commit-sha" }
      }
    }
  }
}
```

This makes your package more trustworthy and secure for users!
