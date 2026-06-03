# Adding a New Language to OmniFormatter

Blueprint & template for adding language formatter to OmniFormatter.

## 1. Architecture & Design Principles

Plugin architecture via WASM. Core logic decoupled from language formatters.

- **Isolation**: Standalone Rust crate compiled to `.wasm`. No core/extension edits.
- **Dynamic Loading**: Load on-demand from local `extension/modules/` | Cloudflare edge registry.
- **Protocol Boundary**: JSON communication via `protocol` crate.
- **WASM Exports**: Module ! expose 5 required FFI functions.

## 2. Directory Structure

Create `lang-[name]` crate in `crates/`:

```text
crates/lang-[name]/
├── Cargo.toml
├── schema.json
└── src/
    ├── adapter.rs     # Config adapter (translates ConfigIR to internal config)
    ├── format.rs      # Core formatting logic
    ├── lib.rs         # WASM entry points and FFI bindings
    └── plugin.rs      # Implementation of the LanguagePlugin trait
```

### Required Files

- **`Cargo.toml`**: ! include dependencies: `wasm-bindgen`, `serde`, `serde_json`, `protocol`. ! build output `cdylib`.
- **`schema.json`**: JSON schema for language-specific configuration options.
- **`src/lib.rs`**: WASM exports.
- **`src/adapter.rs`**: Translate `ConfigIR` → internal format config.
- **`src/format.rs`**: Core formatting algorithm.

## 3. Required WASM Interface (`src/lib.rs`)

! export exactly 5 functions. `format_[name]` derived from `language_id()`.

```rust
use protocol::ConfigIR;
use wasm_bindgen::prelude::*;

// 1. The main formatting function
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn format_[name](source_bytes: &[u8], config_json: &str) -> Result<Vec<u8>, JsValue> {
    let config: ConfigIR = serde_json::from_str(config_json).unwrap_or_default();
    match format::format(source_bytes, &config) {
        Ok(formatted) => Ok(formatted),
        Err(e) => Err(JsValue::from_str(&e.to_string())),
    }
}

// 2. Returns the contents of schema.json
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn config_schema() -> String {
    include_str!("../schema.json").to_string()
}

// 3. The current version of the module
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

// 4. The VS Code Language ID
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn language_id() -> String {
    "[name]".to_string()
}

// 5. Array of supported file extensions
#[cfg_attr(feature = "standalone", wasm_bindgen)]
pub fn aliases() -> Vec<JsValue> {
    vec![
        JsValue::from_str(".[ext]"),
    ]
}
```

## 4. Integration Workflow

Decoupled design. 0 edits to extension core.

1. **Build module**: `wasm-pack build crates/lang-[name] --target nodejs --out-name lang_[name]`
2. **Deploy**:
   - **Local bundle**: Copy `lang_[name]_bg.wasm` → `extension/modules/lang-[name]/`. Extension auto-detects & loads on save.
   - **Edge registry**: Upload to D1/R2. Extension downloads on-demand.

## 5. Development Steps Checklist

- [ ] Create `crates/lang-[name]` via `cargo new --lib`.
- [ ] `Cargo.toml` ← `crate-type = ["cdylib", "rlib"]` & dependencies.
- [ ] Write `schema.json` config settings.
- [ ] `src/adapter.rs` → parse `ConfigIR`.
- [ ] `src/format.rs` → core algorithm.
- [ ] `src/lib.rs` → export 5 WASM functions.
- [ ] `cargo test` → verify.
- [ ] Build via `wasm-pack` & load.
