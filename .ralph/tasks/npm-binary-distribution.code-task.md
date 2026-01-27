---
status: completed
started: 2026-01-16
completed: 2026-01-16
---

# Code Task: npm Binary Distribution for Ralph

## Summary

Enable `npm install -g @ralph-orchestrator/cli` and `npx @ralph-orchestrator/cli` as installation methods by publishing platform-specific npm packages.

## Context

Ralph is currently distributed via:
- Source compilation (`cargo build`)
- GitHub Releases (pre-built binaries)
- crates.io (`cargo install ralph-cli`)

Many developers expect CLI tools to be installable via npm (like biome, turbo, esbuild). This task adds npm as a distribution channel.

## Approach Options

### Option A: cargo-dist Native npm Support (Recommended)

cargo-dist v0.30+ has built-in npm installer support. This requires minimal code changes.

**Changes required:**
1. Update `Cargo.toml` metadata to enable npm installer
2. Create npm scope on npmjs.com
3. Add `NPM_TOKEN` secret to GitHub repo
4. Regenerate CI workflow with `cargo dist init`

### Option B: Manual npm Package Setup

Create npm packages manually for full control over the distribution.

**Changes required:**
1. Create `npm/` directory structure with platform packages
2. Write JS wrapper script for binary detection
3. Add npm publish job to release workflow
4. Handle cross-platform binary naming

## Acceptance Criteria

- [ ] Users can install via `npm install -g @ralph-orchestrator/cli`
- [ ] Users can run via `npx @ralph-orchestrator/cli --version`
- [ ] All 4 platforms supported: darwin-arm64, darwin-x64, linux-arm64, linux-x64
- [ ] npm packages published automatically on GitHub release tag
- [ ] Version numbers synchronized between Cargo.toml and npm packages
- [ ] README updated with npm installation instructions

## Technical Details

### Package Structure (if manual approach)

```
@ralph-orchestrator/cli           <- Main package (JS wrapper)
├── @ralph-orchestrator/darwin-arm64   <- macOS Apple Silicon
├── @ralph-orchestrator/darwin-x64     <- macOS Intel
├── @ralph-orchestrator/linux-arm64    <- Linux ARM
└── @ralph-orchestrator/linux-x64      <- Linux x64
```

### Platform Package package.json Example

```json
{
  "name": "@ralph-orchestrator/darwin-arm64",
  "version": "2.0.0",
  "os": ["darwin"],
  "cpu": ["arm64"],
  "main": "bin/ralph"
}
```

### JS Wrapper Script (bin/ralph)

```javascript
#!/usr/bin/env node
const { execFileSync } = require('child_process');

const PLATFORMS = {
  'darwin-arm64': '@ralph-orchestrator/darwin-arm64',
  'darwin-x64': '@ralph-orchestrator/darwin-x64',
  'linux-arm64': '@ralph-orchestrator/linux-arm64',
  'linux-x64': '@ralph-orchestrator/linux-x64',
};

const key = `${process.platform}-${process.arch}`;
const pkg = PLATFORMS[key];
if (!pkg) {
  console.error(`Unsupported platform: ${key}`);
  process.exit(1);
}

const bin = require.resolve(`${pkg}/bin/ralph`);
execFileSync(bin, process.argv.slice(2), { stdio: 'inherit' });
```

## Gotchas to Handle

1. **Publishing order**: Platform packages must publish before main package
2. **Executable permissions**: `chmod +x` binaries before packaging
3. **npm spam detection**: Must use scoped packages (`@scope/name`)
4. **Version sync**: All packages need identical versions

## Resources

- [cargo-dist npm installer docs](https://opensource.axo.dev/cargo-dist/book/installers/npm.html)
- [Orhun's Blog: Packaging Rust for npm](https://blog.orhun.dev/packaging-rust-for-npm/)
- [esbuild npm implementation](https://github.com/evanw/esbuild/tree/main/npm)
- [@biomejs/biome package.json](https://github.com/biomejs/biome/blob/main/packages/@biomejs/biome/package.json)

## Estimated Effort

- **Option A (cargo-dist)**: ~1-2 hours
- **Option B (manual)**: ~4-6 hours

## Recommendation

Start with Option A (cargo-dist native support). If it doesn't meet requirements, fall back to Option B for full control.
