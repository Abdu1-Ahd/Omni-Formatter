## Summary

<!-- One sentence: what does this PR do? -->

Closes #

---

## Checklist

- [ ] Linked to an existing issue (e.g., `Closes #123`)
- [ ] Added or updated unit tests for all changed logic
- [ ] Updated `CHANGELOG.md` under `## [Unreleased]`
- [ ] Local build passes (`cargo test --workspace` + `pnpm test`)
- [ ] Lint passes with zero warnings (`cargo clippy -- -D warnings` + `pnpm lint`)
- [ ] Linear commit history maintained (squash or rebase — no merge commits)
- [ ] For language modules: idempotency suite passes
- [ ] For language modules: compat CI passes (if reference formatter exists)
- [ ] For extension changes: tested in VS Code Extension Development Host
- [ ] For registry/CLI changes: integration test added

---

## Type of Change

- [ ] `feat` — New feature or language module
- [ ] `fix` — Bug fix
- [ ] `perf` — Performance improvement
- [ ] `refactor` — Code restructuring (no behavior change)
- [ ] `docs` — Documentation only
- [ ] `ci` — CI/CD change
- [ ] `chore` — Maintenance

---

## Testing

Describe what tests were added or updated and how to reproduce the test scenario manually if applicable.
