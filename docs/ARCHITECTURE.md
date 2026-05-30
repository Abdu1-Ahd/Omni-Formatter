# OmniFormatter Architecture Document

> **Status**: Phase 4 (Bundled Language Modules)
> **Last Updated**: 2026-05-30

## Table of Contents

1. [Seven Pillars](#seven-pillars)
2. [System Architecture](#system-architecture)
3. [Component Map](#component-map)
4. [Data Flow](#data-flow)
5. [Limitation Mitigations](#limitation-mitigations)
6. [Module Interface Contract](#module-interface-contract)
7. [Config Adapter Priority](#config-adapter-priority)
8. [WASM Memory Model](#wasm-memory-model)
9. [Unicode Handling](#unicode-handling)
10. [Format-on-Type Protocol](#format-on-type-protocol)

---

## Seven Pillars

| # | Pillar | Enforcement |
|---|---|---|
| 1 | Tree-sitter as the single parser | All language modules use the same Tree-sitter API |
| 2 | Per-language WASM modules, not a monolith | Separate `.wasm` file per language |
| 3 | Zone-aware embedded language formatting | `crates/core/src/zones.rs` + `stitch.rs` |
| 4 | Non-blocking Worker thread execution | `extension/src/workerPool.ts` |
| 5 | Non-destructive config reading | `extension/src/configAdapter.ts` |
| 6 | Magic comment suppression | `crates/core/src/comments.rs` |
| 7 | Idempotency guarantee | `crates/core/src/debug.rs` + weekly fuzz |

---

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│  VS Code Extension Host Process                              │
│                                                             │
│  extension.ts ──► configAdapter.ts ──► WorkerPool          │
│                       │                    │               │
│                  .prettierrc          Worker Thread 1       │
│                  pyproject.toml       Worker Thread 2       │
│                  rustfmt.toml         Worker Thread N       │
│                  .editorconfig                              │
└────────────────────────────────┬────────────────────────────┘
                                 │ postMessage (JSON)
┌────────────────────────────────▼────────────────────────────┐
│  Worker Thread (Node.js)                                     │
│                                                             │
│  formatWorker.ts                                            │
│       │                                                     │
│  WASM Core Instance (omni_core.wasm)                        │
│       │                                                     │
│  ┌────▼────────────────────────────────────────────────┐   │
│  │  format() entry point                               │   │
│  │       │                                             │   │
│  │  ZoneDetector ──► [Zone, Zone, ...]                │   │
│  │       │                                             │   │
│  │  CommentAnchor map (per zone)                      │   │
│  │       │                                             │   │
│  │  LangModule dispatch ──► WASM module per language  │   │
│  │       │                                             │   │
│  │  Stitcher (reassemble zones)                       │   │
│  │       │                                             │   │
│  │  TextEdit diff generator                           │   │
│  └────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
```

---

## Component Map

| File | Responsibility |
|---|---|
| `crates/protocol/src/lib.rs` | FormatRequest, FormatResponse, TextEdit, ByteRange |
| `crates/protocol/src/config.rs` | ConfigIR — universal config intermediate representation |
| `crates/protocol/src/zone.rs` | Zone, ZoneKind — embedded language region types |
| `crates/protocol/src/error.rs` | FormatError — all error variants |
| `crates/core/src/lib.rs` | WASM entry point — `format()` export |
| `crates/core/src/zones.rs` | Zone detector — Tree-sitter CST walk (Phase 4) |
| `crates/core/src/stitch.rs` | Zone output stitcher with re-indentation |
| `crates/core/src/comments.rs` | Comment anchor map + suppression detection |
| `crates/core/src/incremental.rs` | Format-on-type dirty region computation |
| `crates/core/src/range.rs` | Range expansion to nearest syntactic unit |
| `crates/core/src/unicode.rs` | Display column width (CJK, combining, emoji) |
| `crates/core/src/debug.rs` | Idempotency double-format check |
| `crates/core/src/arena.rs` | Bumpalo per-request arena allocator |
| `crates/core/src/memory.rs` | WASM memory constants and size guards |
| `crates/lang-js/` | JS/TS/JSX/TSX — Prettier 3.x parity |
| `crates/lang-python/` | Python — Black 24.x parity |
| `crates/lang-rust/` | Rust — rustfmt stable parity |
| `crates/lang-go/` | Go — gofmt parity |
| `crates/lang-css/` | CSS/SCSS/Less/HTML — Prettier 3.x parity |
| `extension/src/extension.ts` | VS Code activation, provider registration |
| `extension/src/workerPool.ts` | Worker thread pool |
| `extension/workers/formatWorker.ts` | Worker thread WASM host |
| `extension/src/configAdapter.ts` | Config file reader and merger |
| `extension/src/offsets.ts` | UTF-16 ↔ UTF-8 byte offset conversion |
| `extension/src/statusBar.ts` | Formatter chain status bar item |
| `extension/src/onType.ts` | Format-on-type incremental handler |
| `extension/src/moduleLoader.ts` | WASM module disk cache + registry client |
| `extension/src/wasmCompiler.ts` | Background WASM pre-compilation |
| `extension/src/conflictDetector.ts` | Competing formatter detection |
| `extension/src/chain.ts` | Post-format chain runner |

---

## Data Flow

```
1. User saves file (or triggers Format Document)
2. VS Code calls provideDocumentFormattingEdits()
3. extension.ts reads the document text → UTF-8 bytes
4. configAdapter.ts resolves ConfigIR:
      .omnifmt.json (if exists)
      → .prettierrc / pyproject.toml / rustfmt.toml
      → .editorconfig
      → module defaults
5. FormatRequest is constructed with UTF-8 byte source + ConfigIR
6. WorkerPool.dispatch() picks the least-loaded worker
7. Worker receives the request via postMessage
8. WASM format() is called:
      a. Arena allocator initialised
      b. 10MB size check
      c. Zone detection (HTML, Svelte, Vue, Astro, JS tagged templates)
      d. Comment anchor map built for each zone
      e. Each zone dispatched to its language module
      f. Language module formats the zone
      g. Zones stitched back together
      h. TextEdit diff generated (source vs formatted)
      i. Idempotency check (debug builds)
      j. Arena dropped (all parse nodes freed in one deallocation)
9. FormatResponse returned via postMessage to extension host
10. extension.ts converts UTF-8 byte offsets → VS Code UTF-16 positions
11. VS Code applies the TextEdits
12. StatusBar updated with formatter chain and timing
```

---

## Limitation Mitigations

| ID | Limitation | Mitigation |
|---|---|---|
| L-01 | Memory management in WASM | Arena allocator (bumpalo) + 10MB file limit |
| L-02 | Third-party WASM modules | SHA-256 verification on all module loads |
| L-03 | Cold WASM startup latency | Background pre-compilation on activation |
| L-04 | WASM runs synchronously | Worker thread pool (one WASM instance per worker) |
| L-05 | Multi-language files | Zone detector + stitcher |
| L-06 | Comment preservation | Comment anchor map + suppression token detection |
| L-07 | Network dependency for community modules | Disk cache (offline-first) |
| L-08 | Reference formatter parity | Weekly compat-check CI against Prettier/Black/rustfmt/gofmt |
| L-09 | Non-idempotent output | Debug double-format panic + 10k fixture fuzz suite |
| L-10 | Config migration burden | Non-destructive adapter reads existing config files |
| L-11 | Competing formatter conflict | ConflictDetector + status bar formatter chain display |
| L-12 | Opinionated vs configurable | `mode: opinionated` (default) vs `mode: advanced` |
| L-13 | Format-on-type latency | Incremental Tree-sitter re-parse + dirty region only |
| L-14 | Unicode display width | `unicode-width` crate + UTF-16↔UTF-8 conversion layer |
| L-15 | Range formatting of partial AST | Range expansion to nearest complete syntactic unit |

---

## Module Interface Contract

Every language module MUST export these five WASM functions:

```rust
// Primary format function
format_{lang}(source_bytes: &[u8], config_json: &str) -> Result<Vec<u8>, JsValue>

// Metadata functions (static, no side effects)
config_schema() -> String     // JSON Schema of this module's config
version() -> &'static str     // semver string (e.g. "0.1.0")
language_id() -> &'static str // primary VS Code languageId
aliases() -> Vec<JsValue>     // additional languageIds and file extensions
```

Violation of the interface contract blocks publishing to the registry.

---

## Config Adapter Priority

```
Priority 1 (highest): .omnifmt.json     → per-language override
Priority 2:           Native config      → .prettierrc / pyproject.toml / rustfmt.toml
Priority 3:           .editorconfig      → base layer (indent, EOL)
Priority 4 (lowest):  Module defaults    → baked into ConfigIR::default()
```

If a config file is malformed, that priority level is skipped silently.
The adapter NEVER throws. It NEVER modifies any config file.

---

## WASM Memory Model

```
WASM linear memory layout:
├── Stack (grows down from 64KB)
├── Heap (managed by bumpalo arena)
│   ├── Per-request arena (initial: 64KB, grows automatically)
│   │   ├── Tree-sitter CST nodes
│   │   ├── Zone structs
│   │   └── Comment anchor maps
│   └── Other heap allocations (wasm-bindgen, serde)
└── Data segment (string literals, static data)

Limits:
- Initial WASM memory: 16 MB
- Maximum WASM memory: 64 MB
- Maximum source file:  10 MB (enforced at extension host + WASM)
```

---

## Unicode Handling

| Context | Encoding | Unit |
|---|---|---|
| VS Code positions (API) | UTF-16 | Code units |
| WASM source input | UTF-8 | Bytes |
| Print width enforcement | Unicode display width | Display columns |
| Tree-sitter node positions | UTF-8 | Bytes |

Conversion happens ONLY at the extension host / WASM boundary:
- VS Code → WASM: `toUtf8ByteOffset()` in `extension/src/offsets.ts`
- WASM → VS Code: `toUtf16CodeUnitOffset()` in `extension/src/offsets.ts`
- Print width: `unicode-width` crate in `crates/core/src/unicode.rs`

---

## Format-on-Type Protocol

```
Target: < 16ms total round-trip

1. User presses a trigger key (space, semicolon, closing brace)
2. OnTypeHandler.handleOnType() is called
3. EditDelta is constructed (start offset, deleted bytes, inserted bytes)
4. Previous Tree-sitter tree is retrieved from cache (if available)
5. FormatRequest with previous_tree + edit is sent to worker
6. WASM core:
   a. Deserialises previous Tree-sitter tree → O(1)
   b. Calls tree.edit(delta) → O(log n) incremental re-parse
   c. Finds dirty region (smallest syntactic unit containing the edit)
   d. Formats ONLY the dirty region
   e. Returns TextEdits for the dirty region only
7. Extension host applies the edits
8. New tree serialised and stored in previousTrees cache for next keypress
```
