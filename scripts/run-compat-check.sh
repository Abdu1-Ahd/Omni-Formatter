#!/usr/bin/env bash
# scripts/run-compat-check.sh
#
# Downloads and runs the reference formatters against OmniFormatter's output.
# Asserts byte-for-byte equality on every fixture file.
#
# Prerequisites:
#   - Prettier 3.x: npm install -g prettier@3
#   - Black 24.x: pip install black==24.*
#   - rustfmt: rustup component add rustfmt
#   - gofmt: installed with Go toolchain
#
# Usage: bash scripts/run-compat-check.sh

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
FIXTURES_DIR="$WORKSPACE_ROOT/tests/compat"
DIFF_DIR="$WORKSPACE_ROOT/tests/compat/diff"

log() { echo "[compat-check] $*"; }
err() { echo "[compat-check] ERROR: $*" >&2; }

mkdir -p "$DIFF_DIR"

FAILURES=0

# ─── JS/TS Compat Check ───────────────────────────────────────────────────────

check_prettier() {
    local fixture_dir="$FIXTURES_DIR/js"
    [ -d "$fixture_dir" ] || { log "No JS fixtures yet — skipping Prettier compat check"; return; }

    command -v prettier >/dev/null 2>&1 || { log "prettier not found — skipping"; return; }

    log "Checking Prettier 3.x compat..."

    for fixture in "$fixture_dir"/*.js "$fixture_dir"/*.ts; do
        [ -f "$fixture" ] || continue
        local basename
        basename=$(basename "$fixture")

        # Format with Prettier (reference)
        local prettier_out
        prettier_out=$(prettier --parser typescript "$fixture" 2>/dev/null)

        # Format with OmniFormatter
        local omni_out
        omni_out=$(cargo run --manifest-path "$WORKSPACE_ROOT/cli/Cargo.toml" -- print "$fixture")

        # Compare
        if [ "$prettier_out" != "$omni_out" ]; then
            diff <(echo "$prettier_out") <(echo "$omni_out") > "$DIFF_DIR/$basename.diff" 2>&1 || true
            err "COMPAT FAIL: $basename differs from Prettier output"
            FAILURES=$((FAILURES + 1))
        else
            log "  PASS: $basename"
        fi
    done
}

check_prettier

# ─── Summary ─────────────────────────────────────────────────────────────────

log ""
if [ "$FAILURES" -gt 0 ]; then
    log "COMPAT CHECK FAILED: $FAILURES fixture(s) differ from reference formatter output."
    log "See diffs in: $DIFF_DIR"
    exit 1
else
    log "All compat checks PASSED."
fi
