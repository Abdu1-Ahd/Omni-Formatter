# OmniFormatter — Developer Guide

Internal engineering reference. Covers local setup, build commands, architecture overview, and platform-specific notes.

---

## Table of Contents

- [Minimum Required Versions](#minimum-required-versions)
- [Local Setup](#local-setup)
- [Build Commands](#build-commands)
- [Test Execution](#test-execution)
- [Lint Commands](#lint-commands)
- [Docker Build](#docker-build)
- [Directory → Architectural Function Matrix](#directory--architectural-function-matrix)
- [Build Phases](#build-phases)
- [Platform-Specific Notes](#platform-specific-notes)
- [Release Procedure](#release-procedure)

---

## Minimum Required Versions

| Software | Version | Installation |
|---|---|---|
| Rust | 1.78+ | `rustup update stable` |
| `wasm-pack` | 0.13+ | `cargo install wasm-pack` |
| `wasm-bindgen-cli` | 0.2.92+ | `cargo install wasm-bindgen-cli` |
| `wasm32-unknown-unknown` target | any | `rustup target add wasm32-unknown-unknown` |
| Node.js | 20 LTS | https://nodejs.org |
| pnpm | 9+ | `npm install -g pnpm@9` |
| `cargo-fuzz` | latest | `cargo install cargo-fuzz` |
| VS Code | 1.90+ | https://code.visualstudio.com |

Framework exclusions: **No framework wraps the core Rust logic.** The TypeScript extension uses zero frameworks beyond the VS Code API and Node.js built-ins.

---

## Local Setup

```sh
# 1. Clone
git clone https://github.com/Abdu1-Ahd/Omni-Formatter.git
cd Omni-Formatter

# 2. Install Rust WASM target
rustup target add wasm32-unknown-unknown

# 3. Install wasm-pack and wasm-bindgen-cli
cargo install wasm-pack wasm-bindgen-cli

# 4. Install Node dependencies (extension + registry + cli)
pnpm install

# 5. Build all WASM crates
./scripts/build-wasm.sh

# 6. Build extension TypeScript
pnpm --filter extension build
```

---

## Build Commands

| Command | Purpose |
|---|---|
| `cargo build --workspace` | Build all Rust crates (native targets) |
| `bash scripts/build-wasm.sh` | Build all language module WASM binaries |
| `wasm-pack build crates/lang-js --target nodejs` | Build a single language module |
| `pnpm --filter extension build` | Compile extension TypeScript to `dist/` |
| `pnpm --filter extension package` | Package the extension to a `.vsix` file |
| `npx vsce package` | Package and produce `.vsix` for manual install |

---

## Test Execution

```sh
# Run all Rust unit + integration tests
cargo test --workspace

# Run idempotency fuzz suite (requires cargo-fuzz and nightly on Windows)
cargo fuzz run idempotency_js -- -max_len=65536 -runs=10000

# Run Node.js WASM smoke test
node tests/node/test-core.js

# Run TypeScript extension tests
pnpm --filter extension test

# Run the professional workspace test suite
npm test --prefix tests/professional_workspace

# Run format-on-type latency benchmark (16ms target)
cargo bench -p core --bench format_on_type
```

---

## Lint Commands

```sh
# Rust — zero warnings policy
cargo clippy --all-targets -- -D warnings

# TypeScript
pnpm --filter extension lint

# Rust format check (does not modify files)
cargo fmt --all -- --check

# TypeScript format check
pnpm --filter extension format:check
```

---

## Docker Build

```sh
docker build -t omni-formatter:dev .
docker run --rm omni-formatter:dev cargo test --workspace
```

---

## Directory → Architectural Function Matrix

| Directory / File | Role | Input | Output |
|---|---|---|---|
| `crates/protocol/` | Shared type definitions | — | `Zone`, `ConfigIR`, `FormatError`, `FormatRequest`, `FormatResponse` |
| `crates/core/` | WASM core binary | `FormatRequest` JSON | `FormatResponse` JSON |
| `crates/core/src/zones.rs` | Embedded language zone detector | Tree-sitter CST | `Vec<Zone>` |
| `crates/core/src/comments.rs` | Comment anchoring engine | CST + comment nodes | Anchor map |
| `crates/core/src/stitch.rs` | Zone output stitcher | Per-zone formatted bytes | Full file bytes |
| `crates/core/src/incremental.rs` | Format-on-type incremental parse | Edit delta + previous Tree | Dirty region + CST update |
| `crates/core/src/range.rs` | Range expansion | Selection range | Nearest complete CST unit |
| `crates/core/src/unicode.rs` | Display column counting | UTF-8 bytes | Display column widths |
| `crates/lang-*/` | Language module (one per language) | Zone bytes + `ConfigIR` | Formatted bytes |
| `crates/lang-*/src/adapter.rs` | Config file reader + translator | Native config files | `ConfigIR` |
| `extension/src/extension.ts` | Extension activation + provider | VS Code events | `DocumentFormattingEditProvider` registration |
| `extension/src/workerPool.ts` | Worker thread pool | Format requests | Dispatched `FormatResponse` |
| `extension/workers/formatWorker.ts` | Worker thread entry point | `postMessage FormatRequest` | `postMessage FormatResponse` |
| `extension/src/offsets.ts` | UTF-16 ↔ UTF-8 conversion | VS Code positions | Byte offsets |
| `extension/src/moduleLoader.ts` | Registry client + disk cache | Language ID | Loaded WASM module |
| `extension/src/conflictDetector.ts` | Competing formatter detection | Installed extensions | Conflict notification |
| `extension/src/chain.ts` | Post-format chain runner | `"postFormat"` config + formatted doc | Final formatted doc |
| `registry/src/index.ts` | Cloudflare Worker router | HTTP requests | Module metadata + WASM blobs |
| `tests/idempotency/` | 10,000-fixture fuzz idempotency suite | Generated source files | Pass/fail assertion |
| `tests/benchmarks/` | Format-on-type latency benchmarks | 2,000-line fixture + 1-char edit | Latency measurement |
| `tests/compat/` | Reference formatter output comparison | Source fixture corpus | Byte diff vs. reference formatter |
| `scripts/` | Build + package automation | — | `.wasm` artifacts, `.vsix` package |

---

## Build Phases

The project is structured into five sequential build phases. Each phase must be complete and all tests passing before the next phase begins.

| Phase | Goal | Key Deliverable |
|---|---|---|
| 1 | Core protocol + WASM scaffold | `format()` stub passes Node smoke test |
| 2 | Extension host + worker pool | Pass-through format works in VS Code |
| 3 | JS/TS language module | Prettier 3.x parity + 16ms format-on-type |
| 4 | Python / Rust / Go / CSS modules | All 4 modules pass compat CI |
| 5 | Registry + CLI + release | `.vsix` published; `omnifmt-cli` scaffolds a module |

---

## Platform-Specific Notes

### Windows (PowerShell)

- Use `;` as command separator, not `&&`.
- `build-wasm.sh` requires Git Bash or WSL2. Run via: `bash scripts/build-wasm.sh`
- Line endings: `.gitattributes` enforces LF. If files arrive with CRLF, run: `git add --renormalize .`
- `cargo-fuzz` requires a nightly Rust toolchain on Windows: `rustup install nightly`

### macOS

- No known issues beyond standard Rust + Node setup.

### Linux

- Ensure `libssl-dev` and `pkg-config` are installed for any crate requiring TLS.

---

## Release Procedure

OmniFormatter uses [Semantic Versioning](https://semver.org/). The publish workflow is fully automated via GitHub Actions.

### Steps

1. Update the version in `extension/package.json` (e.g., `"version": "0.1.1"`).
2. Update `CHANGELOG.md` — move all items from `[Unreleased]` into a new `[0.1.1] — YYYY-MM-DD` section.
3. Commit the changes:
   ```sh
   git commit -am "chore: release v0.1.1"
   git push
   ```
4. Tag the release:
   ```sh
   git tag v0.1.1
   git push origin v0.1.1
   ```
5. The [Publish workflow](.github/workflows/publish.yml) will automatically:
   - Build and package the `.vsix`
   - Publish to VS Code Marketplace
   - Publish to Open VSX Registry

6. Create a GitHub Release at `https://github.com/Abdu1-Ahd/Omni-Formatter/releases/new` using the tag and paste the relevant CHANGELOG section as the body.
