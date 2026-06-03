# OmniFormatter Decision Log

This document records architectural and technical decisions made during the development of OmniFormatter. Each decision is standalone and provides context for future agents or developers.

## D-001: Rust Core for WASM Compilation
**Context**: The formatter needs to run consistently across VS Code (Node.js/Browser), a standalone CLI (Native), and potentially the web.
**Decision**: Core logic and formatting pipelines are written in Rust, compiled to `wasm32-unknown-unknown`.
**Consequences**: Guarantees bit-for-bit identical formatting across all platforms. Eliminates the need to ship Node.js or Python environments to users. Requires managing a memory allocator compatible with WASM (e.g., `talc` or `lol_alloc`).

## D-002: Prettier 3.x Parity for JS/TS/CSS
**Context**: Developers are resistant to adopting new formatting styles.
**Decision**: For web languages (JS, TS, CSS, SCSS, HTML), the default output is strictly byte-for-byte compatible with Prettier 3.x.
**Consequences**: Eases migration. Requires exact replication of Prettier's AST traversal and Wadler-style document IR layout algorithm.

## D-003: Tree-Sitter for Parsing
**Context**: Regex-based formatters are fragile and context-blind.
**Decision**: Use `tree-sitter` grammars compiled via C-bindings to WASM for all language parsing.
**Consequences**: Provides robust, error-tolerant CSTs (Concrete Syntax Trees). Introduces complexity in WASM compilation because `tree-sitter` C dependencies require stubs for libc functions like `iswalpha` and `iswdigit` in the `wasm32-unknown-unknown` target.

## D-004: Cloudflare Workers for Registry
**Context**: The plugin registry must be highly available, globally distributed, and low-latency.
**Decision**: Host the registry on Cloudflare Workers, utilizing Hono.js for routing.
**Consequences**: Ultra-fast edge resolution. Serverless execution minimizes cost. Imposes limits on payload sizes and execution times, but perfectly suits serving lightweight metadata and redirecting to WASM binaries.

## D-005: Ed25519 Signature Verification
**Context**: Arbitrary WASM execution is a security risk if the registry is compromised.
**Decision**: Every published module must be signed with an Ed25519 private key. The registry verifies this signature before storing, and the extension verifies the SHA-256 hash before execution.
**Consequences**: Cryptographic guarantee of module provenance. Yanked or tampered modules are instantly rejected.

## D-006: Node.js Worker Threads without SharedArrayBuffer
**Context**: The VS Code extension needs to format code without blocking the main UI thread.
**Decision**: Spawn Node.js `worker_threads` for formatting. Pass data via standard message passing; do NOT use `SharedArrayBuffer`.
**Consequences**: Avoids complex synchronization locks and potential deadlocks. Maximizes compatibility with environments where `SharedArrayBuffer` is disabled for security reasons (e.g., some web extensions).

## D-007: Cloudflare D1 for Registry Metadata
**Context**: The registry needs to track modules, versions, and publishers.
**Decision**: Use Cloudflare D1 (Serverless SQLite) to store manifest metadata, while WASM blobs go to R2 or D1 blob storage.
**Consequences**: Fast relational queries for version resolution. Easy to backup and query.

## D-008: CLI with Wasmtime Sandbox
**Context**: Users need a way to run OmniFormatter outside of VS Code (e.g., in CI pipelines).
**Decision**: Build a Rust CLI that downloads WASM modules and executes them using the `wasmtime` JIT engine.
**Consequences**: The CLI provides a completely sandboxed execution environment. The WASM module has zero access to the host file system or network, mitigating supply-chain attacks.

## D-009: Global Allocator Selection (lol_alloc / talc)
**Context**: The standard `dlmalloc` implementation caused memory corruption in WASM when interacting with `tree-sitter`'s C-bindings.
**Decision**: Replace the default global allocator with `talc` (and occasionally testing `lol_alloc`).
**Consequences**: Fixed double-free and memory corruption bugs during heavy AST traversals. Ensured stable WASM module execution without memory leaks.

## D-010: Idempotency by Design
**Context**: Formatting the same code twice must result in zero changes on the second pass.
**Decision**: Enforce strict idempotency checks in the test suite (`cargo run -- format` twice must yield no diff).
**Consequences**: Prevents "format loops" where the editor continuously formats on save. Guarantees stable AST mapping.
