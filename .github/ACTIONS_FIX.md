# GitHub Actions Fixes

## Issues Fixed

### 1. Deprecated Action Versions
All GitHub Actions workflows were failing due to deprecated action versions:
```
Error: This request has been automatically failed because it uses a deprecated version of `actions/upload-artifact: v3`.
```

### 2. Ubuntu 24.04 Package Name Change
CI was failing on Ubuntu runners with:
```
E: Unable to locate package libwebkit2gtk-4.0-dev
```

## Fixed ✅

### Action Version Updates (Commit: 0319b05)

Updated all deprecated GitHub Actions to their latest versions:

| Action | Old Version | New Version |
|--------|-------------|-------------|
| `actions/cache` | v3 | v4 |
| `actions/upload-artifact` | v3 | v4 |
| `codecov/codecov-action` | v3 | v4 |

### Ubuntu Package Updates (Commit: 945c8fe)

Updated WebKit package for Ubuntu 24.04 (Noble) compatibility:

| Package | Old Name | New Name |
|---------|----------|----------|
| WebKit GTK | `libwebkit2gtk-4.0-dev` | `libwebkit2gtk-4.1-dev` |

## Already Up-to-Date ✅

These actions were already using the latest versions:
- `actions/checkout@v4`
- `oven-sh/setup-bun@v1`
- `dtolnay/rust-toolchain@stable`

## Changes Made

### Updated 3 instances of `actions/cache@v3` → `@v4`
- Line 31: Lint job
- Line 78: Test-backend job
- Line 154: Build job

### Updated 5 instances of `actions/upload-artifact@v3` → `@v4`
- Line 95: Backend test results
- Line 121: Frontend test results
- Line 175: Linux build artifacts
- Line 184: macOS build artifacts
- Line 193: Windows build artifacts

### Updated 2 instances of `codecov/codecov-action@v3` → `@v4`
- Line 224: Backend coverage upload
- Line 243: Frontend coverage upload

### Updated 3 instances of WebKit package name
- Line 75: Backend test dependencies
- Line 151: Build job Linux dependencies
- Line 213: Coverage job dependencies

## Testing

To verify the fix works:

```bash
# Commit and push the changes
git add .github/workflows/ci.yml
git commit -m "fix(ci): Update deprecated GitHub Actions to v4"
git push
```

The CI/CD pipeline should now run successfully without deprecation errors.

## Migration Notes

### Breaking Changes in v4

**actions/upload-artifact@v4**:
- No breaking changes for our usage
- Improved performance and reliability
- Better handling of artifact uploads

**actions/cache@v4**:
- No breaking changes for our usage
- Improved cache hit rates
- Better support for large caches

**codecov/codecov-action@v4**:
- May require token for private repos (already handled in workflow)
- Improved upload reliability

### Ubuntu 24.04 (Noble) Changes

**libwebkit2gtk-4.1-dev**:
- Ubuntu 24.04 deprecated the 4.0 version of WebKit GTK
- The 4.1 version is API-compatible for Tauri v2
- No code changes required, only package name update

## References

- [GitHub Blog: Deprecation notice for v3 artifact actions](https://github.blog/changelog/2024-04-16-deprecation-notice-v3-of-the-artifact-actions/)
- [actions/upload-artifact v4 release notes](https://github.com/actions/upload-artifact/releases/tag/v4.0.0)
- [actions/cache v4 release notes](https://github.com/actions/cache/releases/tag/v4.0.0)
- [codecov/codecov-action v4 release notes](https://github.com/codecov/codecov-action/releases/tag/v4.0.0)
- [Ubuntu 24.04 WebKit changes](https://packages.ubuntu.com/noble/libwebkit2gtk-4.1-dev)
- [Tauri v2 Prerequisites](https://v2.tauri.app/start/prerequisites/)
