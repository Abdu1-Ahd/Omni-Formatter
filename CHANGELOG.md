# Changelog

All notable changes to OmniFormatter are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).  
Versioning follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

---

## [0.2.0] — 2026-06-08

### Added — Language Expansion (16 new modules, 70+ languages total)

- `crates/lang-c` — C / C++ / Objective-C / Objective-C++ formatter (clang-format style)
- `crates/lang-java` — Java / Kotlin / Scala / Groovy formatter (google-java-format / ktfmt style)
- `crates/lang-csharp` — C# / F# formatter (dotnet format style)
- `crates/lang-data` — JSON / JSON5 / YAML / TOML / XML / INI formatter (prettier / taplo style)
- `crates/lang-shell` — Bash / PowerShell / Zsh formatter (shfmt style)
- `crates/lang-markdown` — Markdown / LaTeX formatter (prettier parity / pass-through)
- `crates/lang-sql` — SQL / GraphQL formatter (sqlfluff style / prettier parity)
- `crates/lang-ruby` — Ruby / PHP / Perl / Lua formatter (rubocop / php-cs-fixer / stylua style)
- `crates/lang-swift` — Swift / Objective-C / Objective-C++ formatter (swift-format style)
- `crates/lang-mobile` — Dart formatter (dart format style)
- `crates/lang-devops` — HCL/Terraform / Dockerfile / Makefile / Nix (terraform fmt style)
- `crates/lang-functional` — Haskell / Elixir / Erlang / OCaml / Clojure / R / Julia / Lisp / Scheme
- `crates/lang-modern` — Zig / Nim / D (zig fmt style)
- `crates/lang-other` — Solidity / GDScript / AutoHotkey / COBOL / Fortran / Assembly (stubs + Solidity format)
- `crates/lang-template` — Jinja / Liquid / EJS / Handlebars / Twig (identity stubs)
- `crates/lang-sass` — Sass indented syntax (`.sass`) normalizer

### Changed

- Workspace version bumped `0.1.0` → `0.2.0`
- Extension description updated to reflect 80+ language support
- `omnifmt.formatWorkspace` glob expanded to cover all new file extensions
- `package.json` `contributes.languages` and `configurationDefaults` updated for all 70+ language IDs

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
