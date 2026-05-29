# OmniFormatter

![Build](https://img.shields.io/github/actions/workflow/status/Abdu1-Ahd/Omni-Formatter/ci.yml?style=flat-square)
![Language](https://img.shields.io/badge/language-Rust%20%2B%20TypeScript-orange?style=flat-square)
![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)
![WASM](https://img.shields.io/badge/runtime-WebAssembly-654ff0?style=flat-square)

A high-performance, universal code formatter built in Rust and compiled to WebAssembly for VS Code. A single formatting engine covers all programming languages without extension bloat, fragmented configs, or native binary dependencies.

---

## Problem Statement

Every programming language in VS Code requires its own formatter extension. Each ships a full language runtime, has its own config format, registers its own `DocumentFormattingEditProvider`, and conflicts with every other formatter installed. A typical full-stack developer runs Prettier, ESLint, Black, rustfmt, gofmt, and clang-format simultaneously — each with a separate install, separate config file, and separate update cycle.

OmniFormatter collapses this into a single WASM binary with a lazy-loading language module system. The core runtime is under 500KB. Language support is downloaded on demand and cached on disk. One config layer adapts all existing config formats. One status bar item shows exactly what ran.

---

## Architecture

The system is organized in five tiers. Each tier runs in its own isolation boundary.

```
┌─────────────────────────────────────────────────────────────────┐
│           VS Code Extension Host  (Node.js / TypeScript)         │
│   FormattingEditProvider · Worker Pool Manager · Module Loader   │
└───────────────────────────┬─────────────────────────────────────┘
                            │ postMessage (structured-clone)
┌───────────────────────────▼─────────────────────────────────────┐
│                   Worker Thread Pool                              │
│  [ Worker: JS/TS ]  [ Worker: Python ]  [ Worker: N… ]          │
│   One WASM instance per worker, pre-warmed on activation         │
└───────────────────────────┬─────────────────────────────────────┘
                            │ WASM function call (Rust ABI)
┌───────────────────────────▼─────────────────────────────────────┐
│                  WASM Core  (Rust → WASM)                        │
│  Parser (Tree-sitter) │ Zones │ Comments │ Diff Generator        │
└───────────────────────┬───────────────────────────┬─────────────┘
                        │ load_module(lang)          │ read_config(path)
┌───────────────────────▼───┐               ┌────────▼────────────┐
│  Language Modules          │               │  Config Adapter      │
│  (.wasm chunks, per-lang)  │               │  (reads native fmts) │
└────────────────────────────┘               └─────────────────────┘
```

The WASM core ships with zero language parsers. Language modules are independent `.wasm` files fetched from the registry and cached to `globalStoragePath`. The core activates in under 5ms.

For full architecture details, read [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

---

## Features

| Feature | Mechanism | Benefit |
|---|---|---|
| Universal language support | Lazy WASM module system | One extension handles any language |
| Zero config migration | Native config file adapters | Install and it works — no config changes |
| Format-on-type | Incremental Tree-sitter parse + dirty-region tracking | Under 16ms per keystroke |
| Embedded language support | Zone-Aware Formatting Engine | JSX, Svelte, Vue, styled-components work correctly |
| Comment preservation | CST comment anchoring pass | Comments never drift or disappear after formatting |
| Conflict-free | Single `FormattingEditProvider` | Replaces Prettier, Black, rustfmt, gofmt simultaneously |
| Community extensible | Open plugin registry with SHA-256 verification | Any language supported via a published module |
| Idempotent output | CI-enforced idempotency contract | `format(format(x)) === format(x)` guaranteed |
| Status bar chain display | Status bar formatter chain display | Always shows what ran and in what order |

---

## Prerequisites

| Software | Version | Purpose |
|---|---|---|
| Rust | 1.78+ | Compiles core + language module crates |
| `wasm-pack` | 0.13+ | Compiles Rust crates to `.wasm` + JS bindings |
| `wasm-bindgen-cli` | 0.2.92+ | Generates TypeScript bindings for WASM exports |
| Node.js | 20 LTS | Extension host runtime, registry server |
| pnpm | 9+ | Package manager for TypeScript workspaces |
| `cargo-fuzz` | Latest | Runs idempotency fuzz tests |
| VS Code | 1.90+ | Extension development host |

---

## Installation

```sh
# Clone the repository
git clone https://github.com/Abdu1-Ahd/Omni-Formatter.git
cd Omni-Formatter

# Install Rust WASM target
rustup target add wasm32-unknown-unknown

# Install wasm-pack
cargo install wasm-pack

# Install Node dependencies
pnpm install

# Build all WASM crates
./scripts/build-wasm.sh

# Build the extension
pnpm --filter extension build
```

---

## Quickstart

```sh
# Launch in VS Code Extension Development Host
code --extensionDevelopmentPath=./extension .

# Run all Rust tests
cargo test --workspace

# Run idempotency suite
cargo test -p lang-js --test idempotency

# Run the Node WASM smoke test
node tests/node/test-core.js
```

Expected output for smoke test:

```
[PASS] format() returned valid JSON
[PASS] source unchanged (pass-through mode)
[PASS] round-trip completed in < 1ms
```

---

## Key Design Decisions

| Decision | Choice | Rationale |
|---|---|---|
| Parser foundation | Tree-sitter | Incremental, error-tolerant, WASM-compatible, large grammar ecosystem |
| WASM threading | Worker threads (no SharedArrayBuffer) | Avoids COOP/COEP header requirements; crash isolation per language |
| Language module distribution | npm-compatible registry | Developer familiarity; SHA-256 integrity; versioned pinning |
| Config strategy | Adapter pattern (read native formats) | Zero migration cost; the hardest adoption barrier removed |
| Bundle strategy | Core + lazy modules | Core under 600KB; users never download what they don't use |
| Idempotency | CI-enforced fuzz testing | Silent idempotency bugs cause infinite format loops |
| Range formatting | Expand to nearest complete syntactic unit | Partial AST nodes produce broken output |
| Unicode handling | `unicode-width` crate + UTF-16 conversion at host boundary | Correct CJK column widths; no source position corruption |
| Status communication | Status bar chain display | Users must always know which formatters ran and in what order |

---

## Limitations

| ID | Limitation | Mitigation |
|---|---|---|
| L-01 | WASM linear memory ceiling (4GB hard cap) | Arena allocators + 10MB file size limit + 64MB WASM max |
| L-02 | Bundle size irony (language parsers inflate core) | Zero parsers in core; per-language `.wasm` modules, cached |
| L-03 | WASM startup latency (100–500ms cold compile) | Background compile on activation; serialized `.module-cache` |
| L-04 | Single-threaded WASM execution | One WASM instance per Node.js Worker thread |
| L-05 | Embedded language zones (HTML/JSX/Svelte/Vue) | Zone detector + stitcher + per-zone formatter dispatch |
| L-06 | Comment anchoring | CST comment-to-sibling anchor map, re-attached post-format |
| L-07 | Language support gaps (Zig, Gleam, WGSL, etc.) | Open plugin registry; one-click install notification |
| L-08 | Official formatter compatibility | Strict compat mode; CI diff against reference formatter output |
| L-09 | Non-idempotent output | 10,000-fixture fuzz suite in CI; debug double-format panic |
| L-10 | Config hell migration | Adapter layer reads `.prettierrc`, `pyproject.toml`, `rustfmt.toml` |
| L-11 | Extension conflicts | Registered as `editor.defaultFormatter`; conflict detector on activation |
| L-12 | Opinionated vs. configurable tension | Two module modes: Opinionated and Configurable; named presets |
| L-13 | Format-on-type latency | Incremental parse + dirty-region tracking; 16ms CI benchmark |
| L-14 | Unicode column counting | `unicode-width` crate; UTF-16 → UTF-8 conversion at host boundary |
| L-15 | Range formatting on partial AST | Expand to nearest complete syntactic unit before formatting |

For full mitigation details, read [`docs/LIMITATIONS.md`](docs/LIMITATIONS.md).

---

## Project Structure

```
omni-formatter/
├── Cargo.toml                    # Workspace root
├── package.json                  # npm workspace root (extension + cli)
├── crates/
│   ├── core/                     # WASM core binary (Rust → WASM)
│   ├── protocol/                 # Shared types (Zone, ConfigIR, FormatError)
│   ├── lang-js/                  # JS/TS/JSX/TSX module (Prettier 3.x parity)
│   ├── lang-python/              # Python module (Black 24.x parity)
│   ├── lang-rust/                # Rust module (rustfmt 1.x parity)
│   ├── lang-go/                  # Go module (gofmt parity)
│   ├── lang-css/                 # CSS/SCSS/Less/HTML module
│   └── cli/                      # omnifmt-cli (scaffold + publish + migrate)
├── extension/                    # VS Code extension (TypeScript)
├── registry/                     # Module registry server (Express)
├── tests/                        # Idempotency, benchmarks, compat, integration
├── scripts/                      # Build, package, compat-check scripts
└── docs/                         # ARCHITECTURE.md, PLUGIN_API.md, LIMITATIONS.md
```

---

## Contributing

Read [`.github/CONTRIBUTING.md`](.github/CONTRIBUTING.md) before opening a pull request.

For language module contributions, read [`docs/PLUGIN_API.md`](docs/PLUGIN_API.md). A community module that does not implement the full `OmniFormatterModule` interface, does not pass the idempotency suite, or does not include a config adapter will not be accepted to the registry.

---

## License

[MIT](LICENSE)