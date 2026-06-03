# OmniFormatter Developer Onboarding

Welcome to OmniFormatter! This is a single WASM-binary code formatter that supports multiple languages with zero-config migration (targeting Prettier/Gofmt/Black parity).

This standalone guide will get you running the codebase immediately.

## 1. System Requirements
- Node.js >= 20.0
- Rust >= 1.75
- `wasm-pack` (for building the WebAssembly targets)
- `wasm32-unknown-unknown` target installed (`rustup target add wasm32-unknown-unknown`)
- VS Code (for extension development)

## 2. Codebase Layout
- **`crates/`**: The Rust core.
  - `protocol/`: Shared structs (ConfigIR, FormatRequest, FormatResponse).
  - `core/`: The WASM entry points and serialization bounds.
  - `lang-*/`: Language-specific formatting logic (AST traversal).
- **`extension/`**: The VS Code extension (TypeScript).
  - `src/`: Extension host logic (Module loader, Config adapter).
  - `workers/`: Node.js `worker_threads` that instantiate the WASM modules.
- **`registry/`**: The Cloudflare Workers application (TypeScript / Hono.js) that distributes the WASM binaries.
- **`cli/`**: The standalone Rust CLI that runs formatters via `wasmtime`.

## 3. Building the Project

### Building the WASM Modules
The Rust modules must be compiled to WebAssembly. We use `wasm-pack` or raw `cargo build`.
```bash
cargo build --release --target wasm32-unknown-unknown -p lang-js
```

### Building the VS Code Extension
Navigate to the `extension/` directory and install dependencies:
```bash
cd extension
npm install
npm run build:all
```
This uses `esbuild` to bundle both the extension host (`extension.ts`) and the worker thread (`formatWorker.ts`).

## 4. Running the Tests

### Rust Unit Tests (Idempotency)
```bash
cargo test
```
Tests in Rust ensure that formatting is strictly idempotent (`format(format(code)) == format(code)`).

### Extension Integration Tests
```bash
cd extension
npm run test
```

## 5. Architectural Mental Model
When writing code for OmniFormatter, keep this lifecycle in mind:
1. VS Code fires a `provideDocumentFormattingEdits` event.
2. The extension delegates to `formatWorker.ts` via message passing.
3. The worker invokes the WASM `format` function, passing a pointer to the UTF-8 source string.
4. The Rust WASM module (`crates/lang-js/src/lib.rs`) parses the string using `tree-sitter`.
5. Rust builds a Wadler `Doc` intermediate representation (`crates/lang-js/src/format.rs`).
6. Rust runs the layout algorithm and returns a serialized JSON response containing the formatted string.
7. The worker passes it back to the extension host, which applies a single text edit.

## 6. How to Add a New Language
1. Create a new crate in `crates/lang-<name>`.
2. Depend on `tree-sitter-<name>`.
3. Implement `format(source, config)`.
4. Register the module in the `Cargo.toml` workspace.
5. Publish to the registry via `POST /publish` with Ed25519 signing.
