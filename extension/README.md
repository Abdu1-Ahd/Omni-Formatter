<div align="center">

# ⬡ OmniFormatter

**One extension. Every language. Zero configuration.**

[![VS Code Marketplace](https://img.shields.io/visual-studio-marketplace/v/Abdu1-Ahd.omni-formatter?style=for-the-badge&label=VS%20Code%20Marketplace&color=0066B8&logo=visualstudiocode)](https://marketplace.visualstudio.com/items?itemName=Abdu1-Ahd.omni-formatter)
[![Open VSX](https://img.shields.io/open-vsx/v/Abdu1-Ahd/omni-formatter?style=for-the-badge&label=Open%20VSX&color=952ca0&logo=eclipse)](https://open-vsx.org/extension/Abdu1-Ahd/omni-formatter)
[![Build](https://img.shields.io/github/actions/workflow/status/Abdu1-Ahd/Omni-Formatter/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white)](https://github.com/Abdu1-Ahd/Omni-Formatter/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](LICENSE)

[![Rust](https://img.shields.io/badge/Core-Rust-CE422B?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/Runtime-WebAssembly-654ff0?style=flat-square&logo=webassembly)](https://webassembly.org/)
[![TypeScript](https://img.shields.io/badge/Extension-TypeScript-3178C6?style=flat-square&logo=typescript)](https://www.typescriptlang.org/)

[**Install for VS Code**](https://marketplace.visualstudio.com/items?itemName=Abdu1-Ahd.omni-formatter) · [**Install for Open VSX**](https://open-vsx.org/extension/Abdu1-Ahd/omni-formatter) · [**Add a Language**](docs/ADD_LANGUAGE_TEMPLATE.md)

</div>

---

## Why OmniFormatter?

> You have 6 formatter extensions installed. They conflict. They each ship a runtime. They each have their own config format. **OmniFormatter replaces them all.**

| Before | After |
|---|---|
| 6 formatter extensions | 1 extension |
| 6 config formats to maintain | 0 — reads your existing files |
| Conflicts & unpredictable order | Single source of truth |
| 200MB+ combined install size | **< 800 KB** |

---

## 🚀 Quick Install

```sh
ext install Abdu1-Ahd.omni-formatter
```

Then add to `.vscode/settings.json`:

```json
{
  "editor.defaultFormatter": "Abdu1-Ahd.omni-formatter",
  "editor.formatOnSave": true
}
```

That's it. No config migration needed.

---

## 🌐 Supported Languages

| Language | Parity | Extensions |
|---|---|---|
| JavaScript / TypeScript | Prettier 3.x | `.js` `.ts` `.mjs` `.cjs` |
| JSX / TSX | Prettier 3.x | `.jsx` `.tsx` |
| Python | Black 24.x | `.py` `.pyw` |
| Rust | rustfmt 1.x | `.rs` |
| Go | gofmt (exact) | `.go` |
| CSS / SCSS / Less | Prettier 3.x | `.css` `.scss` `.less` |
| HTML / Svelte / Vue | Prettier 3.x | `.html` `.svelte` `.vue` |

More languages available on-demand via the [OmniFormatter Registry](https://omnifmt-registry.omniformat.workers.dev).

---

## ✨ Features at a Glance

| | Feature | Detail |
|---|---|---|
| ⚡ | **WASM Core** | Boots in < 5ms, runs in strict sandbox |
| 🔍 | **Zero-Config** | Reads `.prettierrc`, `pyproject.toml`, `rustfmt.toml`, `.editorconfig` |
| 💾 | **Format on Save / Type** | Sub-16ms incremental formatting via Tree-sitter |
| 🧩 | **Embedded Zones** | JSX, Svelte, Vue `<style>` + `<script>` blocks formatted correctly |
| 💬 | **Comment Preservation** | CST anchoring — comments never drift |
| 🛡️ | **Conflict Detector** | Notifies on activation if other formatters registered |
| 🌍 | **Community Registry** | Publish your own language module — Ed25519 verified |
| ✅ | **Idempotent** | `format(format(x)) === format(x)` enforced across 10,000+ fixtures |
| 📶 | **Offline Support** | Modules cached to disk after first use |
| 🔎 | **Status Bar** | Always shows which formatter ran + timing |

---

## ⚙️ Configuration

Zero-config by default. For cross-language overrides, add `.omnifmt.json` to your project root:

```json
{
  "$schema": "https://raw.githubusercontent.com/Abdu1-Ahd/Omni-Formatter/main/docs/omnifmt.schema.json",
  "javascript": { "printWidth": 80, "singleQuote": false, "semi": true },
  "python": { "lineLength": 88 },
  "rust": { "maxWidth": 100, "edition": "2021" }
}
```

---

## 🏗️ Architecture

```
VS Code Extension (TypeScript)
        │ postMessage
 Worker Thread Pool (Node.js)
        │ WASM call (Rust ABI)
   WASM Core (Rust → WASM)
    ├── Language Modules (.wasm)
    ├── Config Adapter (native formats)
    └── Cloudflare Edge Registry (D1 + R2)
```

Each tier runs in its own isolation boundary. Language modules download once, cache forever.

---

## 🔒 Security

- **Ed25519 Signatures** — every module cryptographically signed by publisher
- **WASM Sandbox** — no file system, no network, no process access
- **SHA-256 Integrity** — hash verified before execution
- **Yank Protocol** — compromised versions marked `yanked`, audit trail preserved

---

## 🤝 Contributing

Add a language without touching the core extension. Read the [blueprint](docs/ADD_LANGUAGE_TEMPLATE.md).

A community module must:
- Implement the `OmniFormatterModule` WASM interface (5 exports)
- Pass the idempotency test suite
- Include a config adapter for its native config format

---

## License

[MIT](LICENSE) — Copyright © 2024 Abdu1-Ahd
