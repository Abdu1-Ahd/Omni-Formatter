# Adding a New Language to OmniFormatter

This document is the authoritative blueprint for implementing and publishing a new language module. OmniFormatter is designed so that adding a language requires **zero changes** to the core extension, registry, or CLI. All new language support is delivered as a standalone WASM plugin.

---

## 1. Design Principles

| Principle | Implementation |
|---|---|
| **Isolation** | A language module is a standalone Rust crate compiled to `.wasm`. It has no access to the filesystem, network, or OS. |
| **Dynamic Loading** | Modules are loaded on demand from the local `extension/dist/modules/` bundle or the Cloudflare edge registry. |
| **Protocol Boundary** | All communication between the host and the module uses the JSON-serializable `ConfigIR` type from the `protocol` crate. |
| **Parity Contract** | The module must produce byte-for-byte identical output to the canonical formatter for the language (e.g., `zig fmt`, `gleam format`). |
| **Idempotency** | `format(format(source)) === format(source)` must hold for all valid inputs. |

---

## 2. Required Directory Structure

Create a new crate in `crates/` following this layout:

```
crates/lang-[name]/
├── Cargo.toml           # Crate manifest — must declare cdylib output and required deps
├── schema.json          # JSON Schema for language-specific config options
└── src/
    ├── lib.rs           # WASM entry points and FFI exports (5 required functions)
    ├── adapter.rs       # ConfigIR translator — reads native config format
    ├── format.rs        # Core formatting algorithm (Tree-sitter CST → formatted string)
    └── plugin.rs        # (optional) LanguagePlugin trait impl for shared logic
```

---

## 3. `Cargo.toml` Requirements

```toml
[package]
name = "lang-[name]"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib", "rlib"]

[features]
standalone = ["wasm-bindgen"]  # Only expose wasm-bindgen exports in standalone WASM builds

[dependencies]
protocol = { path = "../../crates/protocol" }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
wasm-bindgen = { version = "0.2", optional = true }
# Add your Tree-sitter grammar here:
# tree-sitter-[name] = "x.y.z"

[dev-dependencies]
wasm-bindgen-test = "0.3"
```

---

## 4. Required WASM Interface (`src/lib.rs`)

Your module **must** export exactly five functions. Replace `[name]` with the VS Code language ID (e.g., `zig`, `gleam`, `wgsl`).

```rust
use protocol::ConfigIR;
use wasm_bindgen::prelude::*;

/// 1. Main formatting entry point.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn format_[name](source_bytes: &[u8], config_json: &str) -> Result<Vec<u8>, JsValue> {
    let config: ConfigIR = serde_json::from_str(config_json).unwrap_or_default();
    match format::format(source_bytes, &config) {
        Ok(formatted) => Ok(formatted),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

/// 2. Returns the contents of schema.json as a string.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn config_schema() -> String {
    include_str!("../schema.json").to_string()
}

/// 3. Returns the module's version string (from Cargo.toml).
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// 4. Returns the VS Code language ID this module handles.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn language_id() -> String {
    "[name]".to_string()
}

/// 5. Returns a list of all file extensions this module handles.
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str(".[ext]"),
        // Add additional extensions here.
    ]
}
```

---

## 5. Config Adapter (`src/adapter.rs`)

The adapter reads the language's canonical native config file and translates it into `ConfigIR`. This is what enables zero-config migration — users do not need to change or delete their existing formatter configuration.

```rust
use protocol::ConfigIR;
use std::path::Path;

/// Reads the native config file for this language (e.g., `.somefmt.toml`)
/// and returns a populated ConfigIR. Falls back to ConfigIR::default() if
/// no config file is found.
pub fn read_config(workspace_root: &Path) -> ConfigIR {
    let config_path = workspace_root.join(".somefmt.toml");

    if !config_path.exists() {
        return ConfigIR::default();
    }

    // Parse the native config and map fields to ConfigIR.
    // ...

    ConfigIR {
        print_width: 80,
        indent_size: 4,
        ..ConfigIR::default()
    }
}
```

**Required native config files to support:**

Look at the language's official tooling. If the language has a canonical config file (e.g., `zig.zon`, `.gleam.toml`), the adapter must read it. If the language has no standard config, `adapter.rs` may return `ConfigIR::default()` directly.

---

## 6. `schema.json`

Document all language-specific configuration options as a JSON Schema object:

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "[Name] Formatter Options",
  "type": "object",
  "properties": {
    "printWidth": {
      "type": "integer",
      "default": 80,
      "description": "Maximum line length before the formatter wraps."
    }
  }
}
```

---

## 7. Integration Workflow

The module integrates with OmniFormatter without touching any core files.

### Step 1 — Build

```sh
wasm-pack build crates/lang-[name] --target nodejs --out-name lang_[name]
```

This produces:
- `pkg/lang_[name]_bg.wasm` — the WASM binary
- `pkg/lang_[name].js` — generated JS bindings

### Step 2a — Local Bundle (for development and testing)

Copy the `.wasm` file into the extension's local module directory:

```sh
cp pkg/lang_[name]_bg.wasm extension/dist/modules/lang-[name]/module.wasm
```

The extension auto-detects bundled modules and loads them without hitting the registry. Reload the extension development host to pick up the new module.

### Step 2b — Publish to the Edge Registry

```sh
# Sign and publish your module to the OmniFormatter registry
npx omnifmt-cli publish --module lang-[name] --version 0.1.0
```

Once published, the extension will download the module on demand the first time a user opens a file of that language.

---

## 8. Development Checklist

Work through these steps in order. Do not proceed to the next step until the current one is fully passing.

- [ ] Create `crates/lang-[name]` via `cargo new --lib`
- [ ] Add `crate-type = ["cdylib", "rlib"]` and all required dependencies to `Cargo.toml`
- [ ] Write `schema.json` documenting all config options
- [ ] Implement `src/adapter.rs` — parse native config and return `ConfigIR`
- [ ] Implement `src/format.rs` — Tree-sitter CST walk → Wadler Doc IR → formatted string
- [ ] Implement `src/lib.rs` — export all 5 required WASM functions
- [ ] Run `cargo test -p lang-[name]` — all unit tests pass
- [ ] Run `cargo test -p lang-[name] --test idempotency` — 1,000 idempotency fixtures pass
- [ ] Build via `wasm-pack build` — no errors
- [ ] Load into extension development host — formatter activates for the target language
- [ ] Run compat CI against reference formatter output — byte-for-byte parity verified
- [ ] Publish to registry and test live download flow

---

## 9. Registry Acceptance Criteria

A module submitted to the public OmniFormatter Registry will be rejected if any of the following is not met:

| Criterion | Requirement |
|---|---|
| Interface compliance | All 5 required WASM exports present |
| Idempotency | Zero failures across 1,000 fixture files |
| Config adapter | Reads the language's canonical native config format |
| Schema | `schema.json` documents all config options |
| Parity | Output matches the reference formatter on the parity corpus |
| Safety | All `unsafe` blocks annotated with `// SAFETY:` comments |
| License | Module source is MIT or Apache-2.0 licensed |
