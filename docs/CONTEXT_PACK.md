# OmniFormatter Context Pack

This standalone document provides a high-level summary of the OmniFormatter codebase's purpose, design philosophy, and operational context. It is designed to rapidly context-load AIs or new contributors without requiring them to read every file.

## The Mission
Formatters today are slow (Python's Black is notoriously heavy), require language-specific environments (Node for Prettier, Python for Black, Go for gofmt), and suffer from dependency hell.

**OmniFormatter** solves this by compiling native formatters into strictly sandboxed WebAssembly (WASM) modules. It runs in a single VS Code extension without requiring Node.js, Python, or Go to be installed on the host machine.

## Core Pillars
1. **Zero-Config Migration**: Out of the box, OmniFormatter produces byte-for-byte identical output to the industry standard formatters (Prettier for JS/CSS, gofmt for Go).
2. **Speed**: Sub-millisecond formatting latency. Achieved via Rust, `tree-sitter`, and WASM.
3. **Security**: Sandboxed execution. The VS Code extension runs WASM without OS access. The CLI uses `wasmtime`. The registry mandates Ed25519 signatures.
4. **Resilience**: Node.js `worker_threads` isolate the formatting logic, ensuring the VS Code UI never blocks, and memory leaks in WASM are contained to the worker.

## The Codebase Topology

### 1. `crates/` (The Formatter Brains)
Written entirely in Rust. Uses `tree-sitter` for parsing source code into a Concrete Syntax Tree (CST). The CST is then mapped into a Wadler-style `Doc` Intermediate Representation (IR), which handles line-wrapping and indentation dynamically.
- The `core` and `protocol` crates provide the glue (serialization, config parsing).
- The `lang-*` crates contain the actual AST-to-Doc mapping logic for each language.

### 2. `extension/` (The VS Code Host)
Written in TypeScript. Registers as a formatter in VS Code. It uses a `ModuleLoader` to pull WASM binaries from the Cloudflare registry (and caches them on disk). Formatting requests are offloaded to `formatWorker.ts` to keep the main thread fluid.

### 3. `registry/` (The Distribution Network)
Written in TypeScript (Hono.js). Deployed to Cloudflare Workers. It acts as the package manager (`npm` equivalent) for OmniFormatter plugins. Modules are stored in R2 (blob storage) and metadata in D1 (SQLite).

### 4. `cli/` (The Native Runner)
Written in Rust. A standalone CLI (`omnifmt`) that downloads WASM binaries from the registry and executes them natively using the `wasmtime` JIT compiler. Perfect for CI/CD pipelines.

## Critical Technical Details
- **Memory Management**: WASM and C-bindings (`tree-sitter`) easily corrupt memory with default allocators like `dlmalloc`. We strictly use `talc` or `lol_alloc` in the WASM compilation targets.
- **WASM Boundary**: Data crosses the JS-to-WASM boundary as linear memory pointers. The JS side allocates memory via exported `alloc()`, writes the UTF-8 string, calls `format()`, reads the returned pointer, and calls `dealloc()`.
- **Zone Routing**: HTML and Vue files embed multiple languages. OmniFormatter extracts these embedded blocks (zones), routes them to `lang-js` or `lang-css`, and stitches the result back together seamlessly.
