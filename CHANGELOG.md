# Changelog

All notable changes to OmniFormatter are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).  
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- `SECURITY.md` — vulnerability disclosure policy and security architecture documentation
- `.github/CONTRIBUTING.md` — full contributor guide with branch, commit, PR, and testing standards
- `.github/ISSUE_TEMPLATE/bug_report.md` — structured bug report template
- `.github/ISSUE_TEMPLATE/language_request.md` — language module request template
- `.github/workflows/publish.yml` — automated CI publishing to VS Code Marketplace and Open VSX Registry on version tag push
- `docs/ADD_LANGUAGE_TEMPLATE.md` — step-by-step blueprint for adding a new language module

---

## [0.1.0] — 2026-06-03

Initial public release.

### Added

- Rust workspace with `crates/protocol` and `crates/core` — WASM core with Tree-sitter parsing and Wadler document IR
- `crates/lang-js` — JavaScript / TypeScript / JSX / TSX formatter with Prettier 3.x output parity
- `crates/lang-python` — Python formatter with Black 24.x output parity
- `crates/lang-rust` — Rust formatter with rustfmt 1.x output parity
- `crates/lang-go` — Go formatter with exact gofmt output parity
- `crates/lang-css` — CSS / SCSS / Less / HTML formatter with Prettier 3.x output parity
- Zone-aware formatting engine for HTML, JSX, Svelte, Vue, and Astro embedded language zones
- Comment anchoring engine — preserves all comment positions across formatting passes
- Format-on-type incremental parse protocol — sub-16ms target on 2,000-line benchmark files
- Config adapter layer — reads `.prettierrc`, `pyproject.toml`, `rustfmt.toml`, `.editorconfig` automatically
- VS Code extension host with lazy-loading worker thread pool
- Module loader with disk caching and SHA-256 integrity verification
- Conflict detector — identifies competing formatter extensions on activation
- Status bar formatter chain display
- `omnifmt-cli` — standalone Rust binary for CI/CD usage via Wasmtime sandbox
- Cloudflare Worker module registry with Hono.js router, D1 metadata store, and R2 blob storage
- Ed25519 cryptographic signing and verification for all published modules
- 10,000-fixture idempotency CI suite
- Professional test suite — 12/12 scenario coverage

### Published

- **VS Code Marketplace**: [Abdu1-Ahd.omni-formatter](https://marketplace.visualstudio.com/items?itemName=Abdu1-Ahd.omni-formatter)
- **Open VSX Registry**: [Abdu1-Ahd.omni-formatter](https://open-vsx.org/extension/Abdu1-Ahd/omni-formatter)

---

[Unreleased]: https://github.com/Abdu1-Ahd/Omni-Formatter/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Abdu1-Ahd/Omni-Formatter/releases/tag/v0.1.0
