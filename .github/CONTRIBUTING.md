# Contributing to OmniFormatter

---

## Branch Strategy

| Branch | Purpose | Merge target |
|---|---|---|
| `main` | Protected. Releasable at all times. No direct pushes. | — |
| `feat/<scope>` | New features or language modules | `main` via PR |
| `fix/<scope>` | Bug fixes | `main` via PR |
| `chore/<scope>` | Maintenance, dependency updates, refactoring | `main` via PR |

---

## Conventional Commits

All commit messages and PR titles MUST follow the [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) specification.

Format:

```
<type>(<scope>): <short description>

[optional body]

[optional footer: Closes #<issue>]
```

Types:

| Type | When to use |
|---|---|
| `feat` | New feature or language module |
| `fix` | Bug fix |
| `build` | Build system or external dependency changes |
| `chore` | Internal maintenance (no production code change) |
| `docs` | Documentation changes only |
| `test` | Adding or updating tests |
| `perf` | Performance improvements |
| `ci` | CI/CD workflow changes |
| `refactor` | Code restructuring with no behavior change |

Examples:

```
feat(lang-js): add .prettierrc YAML variant adapter
fix(core): correct UTF-8 byte offset in incremental parse
chore(deps): bump wasm-bindgen to 0.2.93
ci: enforce 16ms format-on-type benchmark on every commit
```

Breaking changes: add `BREAKING CHANGE:` footer.

---

## Pull Request Protocol

1. Open a PR from a feature branch — never push directly to `main`.
2. PR title MUST be a valid Conventional Commits message.
3. Link the PR to an existing issue using `Closes #<number>`.
4. All status checks must pass before requesting review.
5. At least one CODEOWNERS approval is required before merging.
6. Merge via squash or rebase — no merge commits. Linear history is enforced.

---

## Pre-Flight Checks

Run all of the following before marking a PR as ready for review:

```sh
# Rust lint — zero warnings
cargo clippy --all-targets -- -D warnings

# Rust tests — all pass
cargo test --workspace

# TypeScript lint
pnpm --filter extension lint

# TypeScript tests
pnpm --filter extension test

# Format check (no modifications)
cargo fmt --all -- --check
pnpm --filter extension format:check
```

---

## Language Module Contributions

A community language module is accepted to the registry if and only if it:

1. Implements the full `OmniFormatterModule` interface (see [`docs/PLUGIN_API.md`](../docs/PLUGIN_API.md)).
2. Passes the idempotency fuzz suite (10,000 generated fixtures, zero divergences).
3. Includes a config adapter for any native config format that exists for the language.
4. Declares a valid semver version and non-empty `language_id`.
5. Is a valid WASM binary (verified on publish by the registry).

Scaffold a new module using:

```sh
omnifmt-cli new --grammar https://github.com/<org>/tree-sitter-<lang>
```

---

## Idempotency Requirement

`format(format(x)) === format(x)` is a contractual guarantee.
Any language module that fails idempotency testing is blocked from merging.
In debug builds, the WASM core double-formats every file and panics on divergence.

---

## Issue Reporting

Use the GitHub issue templates:
- **Bug report**: `.github/ISSUE_TEMPLATE/bug_report.yml`
- **Feature request**: `.github/ISSUE_TEMPLATE/feature_request.yml`

Do not open blank issues. All required template fields are mandatory.
