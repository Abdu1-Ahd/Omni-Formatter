# Contributing to OmniFormatter

Thank you for your interest in contributing. This document establishes the standards and workflow expected of all contributors. Please read it in full before opening a pull request or filing an issue.

---

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Before You Contribute](#before-you-contribute)
- [Development Setup](#development-setup)
- [Branch and Commit Standards](#branch-and-commit-standards)
- [Pull Request Process](#pull-request-process)
- [Adding a New Language Module](#adding-a-new-language-module)
- [Testing Requirements](#testing-requirements)
- [Code Style](#code-style)
- [Reporting Security Vulnerabilities](#reporting-security-vulnerabilities)

---

## Code of Conduct

This project follows a simple standard: be direct, be constructive, be respectful. Contributions that are discriminatory, abusive, or made in bad faith will be closed without response.

---

## Before You Contribute

1. **Search existing issues and pull requests** before opening a new one. Duplicate effort wastes everyone's time.
2. **For significant changes**, open a discussion issue first and describe what you intend to do and why. Large PRs submitted without prior discussion are likely to be rejected.
3. **For bug fixes**, link the PR to the issue it resolves using `Fixes #<issue-number>` in the PR description.
4. **Do not open a PR that changes unrelated files**. Keep scope narrow. A PR that fixes a bug in `lang-python` should not touch `lang-js`, `extension/`, or documentation unless directly necessary.

---

## Development Setup

Refer to [`DEVELOPERS.md`](../DEVELOPERS.md) for the complete local setup guide, including all required tool versions and build commands.

The minimum requirements are:

| Tool | Version |
|---|---|
| Rust | 1.78+ |
| `wasm-pack` | 0.13+ |
| Node.js | 20 LTS |
| pnpm | 9+ |
| VS Code | 1.90+ |

---

## Branch and Commit Standards

### Branches

| Pattern | Purpose |
|---|---|
| `feat/<short-description>` | New features |
| `fix/<short-description>` | Bug fixes |
| `docs/<short-description>` | Documentation changes only |
| `refactor/<short-description>` | Code restructuring with no behavior change |
| `test/<short-description>` | Adding or fixing tests only |
| `chore/<short-description>` | Build scripts, CI, dependency updates |

All branches must target `main`.

### Commit Messages

Follow the [Conventional Commits](https://www.conventionalcommits.org/) specification.

```
<type>(<scope>): <short imperative summary>

[optional body]

[optional footer]
```

**Types**: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`, `perf`  
**Scope**: The affected component — e.g., `lang-js`, `extension`, `registry`, `cli`, `core`

**Examples:**

```
feat(lang-python): add support for PEP 695 type alias syntax
fix(extension): prevent double-format on save in multi-root workspaces
docs(readme): update installation instructions for Open VSX
test(core): add idempotency fixtures for nested ternary expressions
```

---

## Pull Request Process

1. **Fork the repository** and create your branch from `main`.
2. **Run all tests locally** and confirm they pass before pushing.
3. **Keep the PR description clear**: state the problem, the approach, and what was tested.
4. **Request a review** from a maintainer by tagging `@Abdu1-Ahd`.
5. PRs must pass all CI checks before they will be reviewed.
6. At least **one approval from a maintainer** is required to merge.
7. Maintainers squash-merge all PRs. Your commit history does not need to be clean.

### PR Checklist

Before marking a PR ready for review, confirm the following:

- [ ] All existing tests pass (`cargo test --workspace` and `pnpm --filter extension test`)
- [ ] New behavior is covered by new tests
- [ ] `cargo clippy --all-targets -- -D warnings` produces zero warnings
- [ ] `cargo fmt --all -- --check` passes without changes
- [ ] The `CHANGELOG.md` `[Unreleased]` section has been updated
- [ ] Documentation has been updated if behavior has changed

---

## Adding a New Language Module

Language support is fully decoupled from the core extension. Adding a new language requires zero changes to the core codebase.

Read [`docs/ADD_LANGUAGE_TEMPLATE.md`](../docs/ADD_LANGUAGE_TEMPLATE.md) for the complete step-by-step blueprint.

**Registry Acceptance Requirements:**

A community language module submitted to the OmniFormatter registry must meet all of the following requirements:

1. **Interface compliance**: All five required WASM exports must be present (`format_<name>`, `config_schema`, `version`, `language_id`, `aliases`).
2. **Idempotency**: The module must pass the standard idempotency test suite with zero failures across 1,000 fixture files.
3. **Config adapter**: A `src/adapter.rs` must be present that reads the language's canonical native config format (e.g., `.black.toml`, `rustfmt.toml`) and converts it to `ConfigIR`.
4. **Schema**: A `schema.json` must be provided that documents all language-specific configuration options.
5. **No unsafe code without justification**: Any `unsafe` block requires a comment explaining why it is necessary and why it is sound.

Modules that do not meet these requirements will not be accepted.

---

## Testing Requirements

| Layer | Command | Requirement |
|---|---|---|
| Rust unit + integration | `cargo test --workspace` | All tests must pass |
| Rust lints | `cargo clippy --all-targets -- -D warnings` | Zero warnings |
| Rust formatting | `cargo fmt --all -- --check` | No changes |
| WASM smoke test | `node tests/node/test-core.js` | All assertions pass |
| Extension TypeScript | `pnpm --filter extension test` | All tests pass |
| Idempotency | `cargo test -p lang-<name> --test idempotency` | Zero failures |

Performance regressions in format-on-type (above 16ms per keystroke on the 2,000-line benchmark fixture) are treated as bugs and must be resolved before merging.

---

## Code Style

### Rust

- Follow `rustfmt` defaults. Run `cargo fmt --all` before committing.
- All public items must have a doc comment (`///`).
- Avoid `unwrap()` and `expect()` in production code paths. Propagate errors using `?`.
- `unsafe` blocks are permitted only where necessary and must include a `// SAFETY:` comment explaining the invariant being upheld.

### TypeScript

- Follow the existing ESLint configuration in `extension/.eslintrc.json`.
- All exported functions and classes must have JSDoc comments.
- Do not introduce runtime dependencies. The extension ships zero production dependencies beyond the VS Code API and Node.js built-ins.

### Documentation

- Use present tense in prose ("Returns a pointer" not "Will return a pointer").
- Code blocks in Markdown must specify a language identifier.
- Tables must have headers.

---

## Reporting Security Vulnerabilities

**Do not file a public GitHub issue for security vulnerabilities.**

Report vulnerabilities by emailing the maintainer directly or using GitHub's private vulnerability reporting feature:  
`Security` → `Report a vulnerability` on the repository page.

Include a description of the vulnerability, the affected component, and a proof of concept if available. You will receive a response within 72 hours.

See [`SECURITY.md`](../SECURITY.md) for the full disclosure policy.
