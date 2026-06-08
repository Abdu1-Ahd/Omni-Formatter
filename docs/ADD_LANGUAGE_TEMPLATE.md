# ➕ Add a Language to OmniFormatter

> **No core changes needed. Ever.** Your formatter ships as a standalone `.wasm` plugin. The extension, CLI, and registry are completely untouched.

---

## ⚡ The 10-Minute Overview

| What you build | How it integrates |
|---|---|
| A single Rust crate → compiled to `.wasm` | Auto-detected from `extension/dist/modules/` or pulled live from the Cloudflare edge registry |
| 5 exported WASM functions | The extension host calls them — no glue code on your side |
| One JSON schema file | Powers autocomplete in VS Code for your formatter options |
| One config adapter | Reads existing user configs (`.prettierrc`, `pyproject.toml`, etc.) — zero migration friction |

---

## 📁 Directory Layout

```
crates/lang-[name]/
├── Cargo.toml       ← crate manifest
├── schema.json      ← JSON Schema for your config options
└── src/
    ├── lib.rs       ← 5 required WASM exports
    ├── adapter.rs   ← reads native config → ConfigIR
    └── format.rs    ← Tree-sitter CST → formatted string
```

---

## 🔧 `Cargo.toml`

```toml
[package]
name = "lang-[name]"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[features]
standalone = ["wasm-bindgen"]

[dependencies]
protocol    = { path = "../../crates/protocol" }
serde       = { version = "1", features = ["derive"] }
serde_json  = "1"
wasm-bindgen = { version = "0.2", optional = true }
# tree-sitter-[name] = "x.y.z"   ← add your grammar here

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

---

## 🦀 The 5 Required WASM Exports (`src/lib.rs`)

Replace `[name]` with the VS Code language ID (e.g. `zig`, `gleam`, `wgsl`).

```rust
use protocol::ConfigIR;
use wasm_bindgen::prelude::*;

/// 1. Format the source code.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn format_[name](source_bytes: &[u8], config_json: &str) -> Result<Vec<u8>, JsValue> {
    let config: ConfigIR = serde_json::from_str(config_json).unwrap_or_default();
    format::format(source_bytes, &config)
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// 2. Return the contents of schema.json.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn config_schema() -> String {
    include_str!("../schema.json").to_string()
}

/// 3. Return the module version.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 4. Return the VS Code language ID.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn language_id() -> String {
    "[name]".to_string()
}

/// 5. Return all handled file extensions.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![JsValue::from_str(".[ext]")]
}
```

> **Contract:** `format(format(src)) == format(src)` — idempotency is mandatory, not optional.

---

## 🔌 Config Adapter (`src/adapter.rs`)

This is the key to zero-config migration. Read the language's native config file and map it to `ConfigIR`. If none exists, return the default.

```rust
use protocol::ConfigIR;
use std::path::Path;

pub fn read_config(workspace_root: &Path) -> ConfigIR {
    let config_path = workspace_root.join(".somefmt.toml");
    if !config_path.exists() {
        return ConfigIR::default();
    }
    // Parse and map to ConfigIR fields...
    ConfigIR { print_width: 80, indent_size: 4, ..ConfigIR::default() }
}
```

**Rule:** If the language has a canonical config file (e.g. `zig.zon`, `.gleam.toml`), the adapter **must** read it.

---

## 📋 `schema.json`

Documents your formatter's config options. Powers IntelliSense in the editor.

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "[Name] Formatter Options",
  "type": "object",
  "properties": {
    "printWidth": {
      "type": "integer",
      "default": 80,
      "description": "Maximum line length before wrapping."
    }
  }
}
```

---

## 🚀 Build & Ship

### Build
```sh
wasm-pack build crates/lang-[name] --target nodejs --out-name lang_[name]
```
Produces `pkg/lang_[name]_bg.wasm` + `pkg/lang_[name].js`.

### Test Locally
```sh
cp pkg/lang_[name]_bg.wasm extension/dist/modules/lang-[name]/module.wasm
# Reload the extension development host — it auto-detects the module.
```

### Publish to the Edge Registry
```sh
npx omnifmt-cli publish --module lang-[name] --version 0.1.0
```
Users get the module downloaded on demand the first time they open a file of that language.

---

## ✅ Checklist

Work through these **in order**:

- [ ] `cargo new --lib crates/lang-[name]`
- [ ] Set `crate-type = ["cdylib", "rlib"]` and add dependencies in `Cargo.toml`
- [ ] Write `schema.json`
- [ ] Implement `adapter.rs` — parse native config → `ConfigIR`
- [ ] Implement `format.rs` — Tree-sitter CST → formatted string
- [ ] Implement `lib.rs` — all 5 exports
- [ ] `cargo test -p lang-[name]` — unit tests green
- [ ] `cargo test -p lang-[name] --test idempotency` — 1,000 fixtures pass
- [ ] `wasm-pack build` — clean
- [ ] Load in dev host — formatter activates for the target language
- [ ] Compat CI — byte-for-byte parity with the reference formatter
- [ ] Publish + verify live download

---

## 🏛️ Registry Acceptance Criteria

Modules that fail any of these are **rejected automatically** by the registry:

| Criterion | Requirement |
|---|---|
| Interface | All 5 WASM exports present |
| Idempotency | Zero failures across 1,000 fixture files |
| Config adapter | Reads the language's canonical config format |
| Schema | All config options documented |
| Parity | Byte-for-byte match with the reference formatter |
| Safety | Every `unsafe` block annotated with `// SAFETY:` |
| License | MIT or Apache-2.0 |
