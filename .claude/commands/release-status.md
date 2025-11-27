# Check Release Status

Check if a release was successfully published across all platforms.

## Arguments
- `$ARGUMENTS` - Version to check (e.g., `0.4.0`). If not provided, reads from VERSION file.

## Instructions

Check the release status for the specified version across all publishing targets:

### 1. Get Version
If no version argument provided, read from VERSION file.

### 2. Check GitHub Release & Tag
```bash
gh release view v<VERSION> --json tagName,publishedAt,assets 2>/dev/null || echo "Not found"
git tag -l "v<VERSION>"
```

### 3. Check PyPI
Use WebFetch to check: `https://pypi.org/pypi/dinja/<VERSION>/json`
- Look for the version in the response
- Report wheel availability for different platforms

### 4. Check npm
Use WebFetch to check: `https://registry.npmjs.org/dinja/<VERSION>`
- Verify the version exists
- Check dist.tarball URL

### 5. Check GitHub Container Registry (Docker)
Use WebFetch to check: `https://ghcr.io/v2/hlop3z/dinja/tags/list`
Or check via GitHub API:
```bash
gh api /users/hlop3z/packages/container/dinja/versions --jq '.[].metadata.container.tags[]' 2>/dev/null | grep -E "^<VERSION>$|^latest$" || echo "Not found"
```

### 6. Check crates.io
Use WebFetch to check: `https://crates.io/api/v1/crates/dinja-core/<VERSION>`

## Output Format

Present results in a table:

| Platform | Status | URL |
|----------|--------|-----|
| GitHub Tag | ✅/❌ | github.com/hlop3z/dinja/releases/tag/v<VERSION> |
| PyPI | ✅/❌ | pypi.org/project/dinja/<VERSION> |
| npm | ✅/❌ | npmjs.com/package/dinja/v/<VERSION> |
| Docker (GHCR) | ✅/❌ | ghcr.io/hlop3z/dinja:<VERSION> |
| crates.io | ✅/❌ | crates.io/crates/dinja-core/<VERSION> |

## Additional Checks
- If GitHub Actions is still running, report that the release is in progress
- Check `gh run list --workflow=release.yml --limit=1` for workflow status
