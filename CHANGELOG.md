# Changelog

All notable changes to OmniFormatter are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added
- Initial repository scaffold with professional GitHub standards
- Rust workspace with `crates/protocol` and `crates/core` (stub WASM core)
- VS Code extension host with worker pool and pass-through format
- JS/TS/JSX/TSX language module with Prettier 3.x output parity
- Python language module with Black 24.x output parity
- Rust language module with rustfmt 1.x output parity
- Go language module with gofmt output parity
- CSS/SCSS/Less/HTML language module with Prettier 3.x output parity
- Zone-aware formatting engine for HTML, JSX, Svelte, Vue, Astro embedded languages
- Comment anchoring engine (preserves all comment positions across formatting)
- Format-on-type incremental protocol (sub-16ms on 2000-line files)
- Config adapter layer (reads `.prettierrc`, `pyproject.toml`, `rustfmt.toml`, `.editorconfig`)
- `omnifmt-cli` scaffold, publish, and migrate commands
- npm-compatible module registry server at `registry/`

---

## [0.1.0] - TBD

_First release. See `## [Unreleased]` for contents._

---

[Unreleased]: https://github.com/Abdu1-Ahd/Omni-Formatter/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/Abdu1-Ahd/Omni-Formatter/releases/tag/v0.1.0
