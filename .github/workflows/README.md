# GitHub Actions Workflows

## Build Windows Executable

This workflow builds the Tauri Station app for Windows x86_64.

### Triggers
- Push to `main` or `master` branch
- Push tags starting with `v*` (e.g., `v1.0.0`)
- Pull requests to `main` or `master`
- Manual trigger via `workflow_dispatch`

### Artifacts
- **NSIS installer** (`.exe`): Standalone installer executable
- **MSI installer** (`.msi`): Windows MSI installer package

Artifacts are uploaded to GitHub Actions artifacts and can be downloaded from the workflow run page.

### Releases
When you push a tag (e.g., `v1.0.0`), the workflow will:
1. Build the Windows executable
2. Create a draft GitHub release
3. Attach the installers to the release

### Usage

To build for Windows, simply push your code to the repository:

```bash
git push origin main
```

To create a release:

```bash
git tag v1.0.0
git push origin v1.0.0
```

The workflow will automatically:
- Build on Windows runners (x86_64)
- Create installers (MSI and NSIS)
- Upload artifacts
- Create a draft release (if tag was pushed)

