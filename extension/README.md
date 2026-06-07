<div align="center">
  <img src="media/logo.png" alt="OmniFormatter Logo" width="180" />
  <br/>

# OmniFormatter

[![VS Code Marketplace](https://img.shields.io/visual-studio-marketplace/v/Abdu1-Ahd.omni-formatter?style=for-the-badge&label=VS%20Code%20Marketplace&color=0066B8&logo=visualstudiocode)](https://marketplace.visualstudio.com/items?itemName=Abdu1-Ahd.omni-formatter)
[![Open VSX](https://img.shields.io/open-vsx/v/Abdu1-Ahd/omni-formatter?style=for-the-badge&label=Open%20VSX&color=952ca0&logo=eclipse)](https://open-vsx.org/extension/Abdu1-Ahd/omni-formatter)
[![Build](https://img.shields.io/github/actions/workflow/status/Abdu1-Ahd/Omni-Formatter/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white)](https://github.com/Abdu1-Ahd/Omni-Formatter/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)

[![Rust](https://img.shields.io/badge/Core-Rust-CE422B?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/Runtime-WebAssembly-654ff0?style=flat-square&logo=webassembly)](https://webassembly.org/)
[![TypeScript](https://img.shields.io/badge/Extension-TypeScript-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)
[![Cloudflare Workers](https://img.shields.io/badge/Registry-Cloudflare%20Workers-F38020?style=flat-square&logo=cloudflare)](https://workers.cloudflare.com/)

**One extension. Every language. Zero configuration.**

*A universal code formatter built in Rust, compiled to WebAssembly, and distributed via an edge registry.*

[Install for VS Code](https://marketplace.visualstudio.com/items?itemName=Abdu1-Ahd.omni-formatter) · [Install for Open VSX](https://open-vsx.org/extension/Abdu1-Ahd/omni-formatter) · [Documentation](docs/) · [Add a Language](docs/ADD_LANGUAGE_TEMPLATE.md)

</div>

---

## The Problem

A typical full-stack developer has **six separate formatter extensions** installed in VS Code — Prettier, ESLint, Black, rustfmt, gofmt, and clang-format. Each ships a full language runtime. Each has its own config format. Each registers its own `DocumentFormattingEditProvider`. Each conflicts with every other formatter installed. Each has a separate install, a separate update cycle, and a separate way to fail.

OmniFormatter eliminates this entirely.

---

## The Solution

OmniFormatter provides a **single WebAssembly binary** with a lazy-loading language module system:

- The core runtime is **under 500 KB**
- Language support is **downloaded on demand** and cached to disk
- Config is **auto-detected** from existing `.prettierrc`, `pyproject.toml`, `rustfmt.toml`, or `.editorconfig` files — no migration required
- **One status bar item** shows exactly which formatter ran and in what order
- Adding a new language requires **zero changes** to the core extension

---

## Installation

**VS Code Marketplace** (recommended):

```
ext install Abdu1-Ahd.omni-formatter
```

Or search `OmniFormatter` in the VS Code Extensions panel.

**Open VSX Registry** (for VSCodium, Gitpod, Eclipse Theia, and other compatible IDEs):

```
ext install Abdu1-Ahd.omni-formatter
```

---

## Supported Languages

OmniFormatter ships with the following language modules bundled:

| Language | Formatter Parity | File Extensions |
|---|---|---|
| JavaScript | Prettier 3.x | `.js`, `.mjs`, `.cjs` |
| TypeScript | Prettier 3.x | `.ts`, `.mts`, `.cts` |
| JSX / TSX | Prettier 3.x | `.jsx`, `.tsx` |
| Python | Black 24.x | `.py`, `.pyw` |
| Rust | rustfmt 1.x | `.rs` |
| Go | gofmt (exact) | `.go` |
| CSS | Prettier 3.x | `.css` |
| SCSS | Prettier 3.x | `.scss` |
| Less | Prettier 3.x | `.less` |
| HTML | Prettier 3.x | `.html`, `.htm` |
| Svelte | Prettier 3.x | `.svelte` |
| Vue | Prettier 3.x | `.vue` |

Additional languages are available through the [OmniFormatter Registry](https://omnifmt-registry.omniformat.workers.dev) and can be installed on demand.

---

## Features

| Feature | Detail |
|---|---|
| **Universal Language Support** | One extension handles every language via lazy-loading WASM modules |
| **Zero-Config Migration** | Reads existing `.prettierrc`, `pyproject.toml`, `rustfmt.toml`, `.editorconfig` automatically |
| **Format-on-Type** | Incremental Tree-sitter parse with dirty-region tracking — sub-16ms per keystroke |
| **Format-on-Save** | Fully configurable per-language and per-workspace |
| **Embedded Language Zones** | JSX, Svelte, Vue, `<style>` blocks, and `<script>` blocks formatted correctly |
| **Comment Preservation** | CST comment anchoring — comments never drift or disappear after formatting |
| **Conflict-Free** | Registered as `editor.defaultFormatter`; includes a conflict detector on activation |
| **Community Extensible** | Open plugin registry with SHA-256 and Ed25519 verification |
| **Idempotent Output** | `format(format(x)) === format(x)` is CI-enforced across 10,000 fixtures |
| **Status Bar Transparency** | Always shows the active formatter chain and formatting time |
| **Range Formatting** | Formats selected ranges only, expanded to the nearest valid syntax boundary |
| **Offline Support** | Modules cached to disk; formatting works without network access |

---

## Configuration

OmniFormatter is **zero-config by default**. It reads your existing native configuration files automatically.

If you need cross-language overrides, create an `.omnifmt.json` file in your project root:

```json
{
  "$schema": "https://raw.githubusercontent.com/Abdu1-Ahd/Omni-Formatter/main/docs/omnifmt.schema.json",
  "javascript": {
    "printWidth": 80,
    "tabWidth": 2,
    "singleQuote": false,
    "semi": true,
    "trailingComma": "all"
  },
  "python": {
    "lineLength": 88,
    "skipStringNormalization": false
  },
  "rust": {
    "maxWidth": 100,
    "tabSpaces": 4,
    "edition": "2021"
  }
}
```

See [`docs/omnifmt.example.json`](docs/omnifmt.example.json) for the full reference with all available options.

### VS Code Settings

You can also configure OmniFormatter directly in your `.vscode/settings.json`:

```json
{
  "editor.defaultFormatter": "Abdu1-Ahd.omni-formatter",
  "editor.formatOnSave": true,
  "editor.formatOnType": true
}
```

---

## Architecture

OmniFormatter is structured in five independent tiers. Each tier runs in its own isolation boundary.

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
└───────────────────┬───────────────────────────┬─────────────────┘
                    │ load_module(lang)          │ read_config(path)
┌───────────────────▼───┐               ┌────────▼────────────────┐
│  Language Modules      │               │  Config Adapter          │
│  (.wasm chunks)        │               │  (reads native fmts)     │
└───────────────────────┘               └─────────────────────────┘
                    │
     ┌──────────────▼────────────────┐
     │  Cloudflare Edge Registry      │
     │  D1 (metadata) · R2 (blobs)    │
     └───────────────────────────────┘
```

The WASM core activates in **under 5ms**. Language modules are independent `.wasm` files fetched from the registry and cached to `globalStoragePath` — users never download what they don't use.

For the full architectural deep dive, read [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

---

## Project Structure

```
omni-formatter/
├── Cargo.toml                     # Rust workspace root
├── package.json                   # Node.js workspace root
├── crates/
│   ├── core/                      # WASM core binary (Rust → WASM)
│   ├── protocol/                  # Shared types: Zone, ConfigIR, FormatError
│   ├── lang-js/                   # JS/TS/JSX/TSX module (Prettier 3.x parity)
│   ├── lang-python/               # Python module (Black 24.x parity)
│   ├── lang-rust/                 # Rust module (rustfmt 1.x parity)
│   ├── lang-go/                   # Go module (gofmt parity)
│   └── lang-css/                  # CSS/SCSS/Less/HTML module
├── extension/                     # VS Code extension (TypeScript)
│   ├── src/
│   │   ├── extension.ts           # Activation + provider registration
│   │   ├── workerPool.ts          # Worker thread pool manager
│   │   ├── moduleLoader.ts        # Registry client + disk cache
│   │   ├── conflictDetector.ts    # Competing formatter detection
│   │   └── chain.ts               # Post-format chain runner
│   ├── dist/modules/              # Bundled language modules (WASM)
│   └── package.json
├── registry/                      # Cloudflare Worker module registry
│   └── src/index.ts               # Hono.js router (D1 + R2)
├── cli/                           # omnifmt-cli (CI/CD standalone binary)
├── tests/                         # Idempotency, benchmarks, compatibility
├── scripts/                       # Build and packaging automation
└── docs/                          # Full documentation
    ├── ARCHITECTURE.md
    ├── ADD_LANGUAGE_TEMPLATE.md
    ├── DECISIONS.md
    └── omnifmt.example.json
```

---

## Contributing

Contributions are welcome. Please read [`.github/CONTRIBUTING.md`](.github/CONTRIBUTING.md) before opening a pull request.

### Adding a New Language

Adding language support does not require any changes to the core extension, registry, or CLI. The system is fully decoupled.

Read [`docs/ADD_LANGUAGE_TEMPLATE.md`](docs/ADD_LANGUAGE_TEMPLATE.md) for the step-by-step blueprint, including the required WASM interface, directory structure, and publishing instructions.

A community module must:
- Implement the full `OmniFormatterModule` WASM interface (5 required exports)
- Pass the idempotency test suite
- Include a config adapter for its native config format (e.g., reads `.somefmt.toml`)

Modules that do not meet these requirements will not be accepted to the registry.

---

## Security

OmniFormatter's plugin system is designed from the ground up with security as a constraint, not an afterthought.

- **Ed25519 Signatures**: Every published module is cryptographically signed by its publisher. The registry verifies the signature before storing. The extension verifies the SHA-256 hash before execution.
- **WASM Sandbox**: The formatting plugin runs inside a strict WebAssembly sandbox. It cannot access the file system, make network requests, or spawn processes.
- **Yank Protocol**: Modules are never deleted. Compromised versions are marked `yanked` and the registry returns HTTP 410 Gone, preserving the audit trail.
- **Integrity Verification**: Downloaded WASM blobs are hash-verified against the signed manifest before instantiation. A tampered binary will never execute.

---

## Changelog

See [CHANGELOG.md](CHANGELOG.md) for the full release history.

---

## License

[MIT](LICENSE) — Copyright © 2024 Abdu1-Ahd
