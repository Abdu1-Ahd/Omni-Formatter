# Changelog

All notable changes to the OmniFormatter project will be documented in this file.

## [0.2.32] - 2026-07-07
### Changed
- **Format on Save**: OmniFormatter now strictly only runs on manual saves (Ctrl+S / Cmd+S). This prevents annoying visual disruption and cursor jumping for users with "Auto Save" (afterDelay) enabled.

### Fixed
- **CSS/LESS/SCSS**: Fixed a critical idempotency and corruption bug caused by broken tree-sitter AST nodes for CSS extensions.
- **LESS Variables**: Restored formatting support for LESS variables containing advanced expressions (like functions or arithmetic) without losing code.
- **SCSS Missing Punctuation**: Corrected an issue where unclosed blocks in `@mixin` and `@if` caused the formatter to lose commas and parentheses in function calls (e.g. `rgba()`).
- **Idempotency**: Prevented comments trailing unclosed blocks in SCSS from shifting down the file with every formatting pass.
- **Test Suite**: Fully resolved all remaining regressions and achieved a 100% test pass rate for all 72 supported languages.
