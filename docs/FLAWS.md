# OmniFormatter Known Flaws

This document outlines the current technical debt, missing features, and known architectural flaws (F-xxx).

### F-001: Cloudflare Wrangler Local Auth Blocks CI/CD
**Description**: The registry deployment is currently blocked because `wrangler deploy` requires interactive browser authentication. There is no headless CI/CD deployment pipeline for the Cloudflare Worker.
**Impact**: High. Prevents automated deployments of the registry.
**Mitigation**: Need to configure `CLOUDFLARE_API_TOKEN` and integrate it into GitHub Actions.

### F-002: Hardcoded Registry URLs
**Description**: The `REGISTRY_BASE_URL` is hardcoded to `https://omnifmt-registry.omniformat.workers.dev` in both the VS Code extension (`moduleLoader.ts`) and the CLI (`main.rs`).
**Impact**: Medium. Prevents users from easily hosting their own enterprise registries.
**Mitigation**: Extract the registry URL into a configuration file (`.omnifmt.json` or VS Code settings).

### F-003: Tree-Sitter C-Binding Memory Leaks in WASM
**Description**: While the `talc` allocator fixed the immediate double-free corruption, long-running WASM instances might still slowly leak memory if Tree-Sitter AST nodes are not explicitly dropped before crossing the WASM boundary.
**Impact**: Medium. Format worker threads might balloon in memory over days of uptime.
**Mitigation**: Implement a memory-reset mechanism or spawn a fresh WASM instance every 100 formats.

### F-004: Lack of Module Uninstallation/Pruning
**Description**: The `ModuleLoader` downloads new WASM versions into `globalStoragePath/modules/<name>/<version>/`, but it never deletes old versions.
**Impact**: Low. Disk space usage will slowly grow over time.
**Mitigation**: Add a cache-pruning routine that deletes versions older than the latest two.

### F-005: HTML Zone Routing Overlaps
**Description**: The HTML zone router extracts `<script>` and `<style>` tags to send to the JS and CSS formatters. However, line-number offsets can become misaligned if the sub-formatter introduces multi-line breaks that cross original boundaries.
**Impact**: Medium. Can cause slight indentation drift in complex HTML documents.
**Mitigation**: Implement a more robust source-map style offset tracker during zone extraction.

### F-006: Format-on-Type Latency for Large Files
**Description**: Format-on-type currently reparses the entire document to find the intersecting node, which can exceed the 2000ms budget for extremely large files (>20k lines).
**Impact**: Low (rare edge case).
**Mitigation**: Implement incremental parsing using Tree-Sitter's `edit` capabilities instead of full reparsing.

### F-007: Fallback to D1 Blob Storage Instead of R2
**Description**: The registry currently falls back to serving WASM binaries directly from D1 SQLite blobs if R2 is unavailable. This is highly inefficient for database read quotas.
**Impact**: Medium. Will scale poorly under heavy load.
**Mitigation**: Ensure R2 buckets are properly bound and force R2 usage for blob downloads.

### F-008: Missing Windows Line Ending Normalization
**Description**: The formatter assumes `\n` line endings natively and does not dynamically respect `\r\n` (CRLF) if the editor specifically requests it, leading to mixed line endings.
**Impact**: Medium. Windows users might see git diffs showing CRLF -> LF conversions.
**Mitigation**: Read the editor's line-ending configuration and pass it through `ConfigIR`.
