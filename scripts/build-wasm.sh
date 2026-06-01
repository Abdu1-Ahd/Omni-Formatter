#!/usr/bin/env bash
# scripts/build-wasm.sh
#
# Compiles all OmniFormatter Rust crates to WebAssembly.
# Run from the workspace root.
#
# Usage:
#   bash scripts/build-wasm.sh            # Build all crates (release)
#   bash scripts/build-wasm.sh --dev      # Build core only (debug, faster)
#
# Prerequisites:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-pack

set -euo pipefail

WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BUILD_MODE="${1:-}"

log() { echo "[build-wasm] $*"; }
err() { echo "[build-wasm] ERROR: $*" >&2; exit 1; }

# Verify prerequisites
command -v wasm-pack >/dev/null 2>&1 || err "wasm-pack not found. Install: cargo install wasm-pack"
rustup target list --installed | grep -q "wasm32-unknown-unknown" || \
    err "WASM target not installed. Run: rustup target add wasm32-unknown-unknown"

log "Building from: $WORKSPACE_ROOT"

# ─── Setup WASI SDK ─────────────────────────────────────────────────────────
WASI_SDK_VERSION="20.0"
WASI_SDK_DIR="$WORKSPACE_ROOT/wasi-sdk"

if [ ! -d "$WASI_SDK_DIR" ]; then
    log "Downloading WASI SDK..."
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        curl -L -o wasi-sdk.tar.gz "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20/wasi-sdk-${WASI_SDK_VERSION}-linux.tar.gz"
        tar xf wasi-sdk.tar.gz && mv "wasi-sdk-${WASI_SDK_VERSION}" "$WASI_SDK_DIR"
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        curl -L -o wasi-sdk.tar.gz "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20/wasi-sdk-${WASI_SDK_VERSION}-macos.tar.gz"
        tar xf wasi-sdk.tar.gz && mv "wasi-sdk-${WASI_SDK_VERSION}" "$WASI_SDK_DIR"
    elif [[ "$OSTYPE" == "msys" || "$OSTYPE" == "cygwin" || "$OSTYPE" == "win32" ]]; then
        curl -L -o wasi-sdk.tar.gz "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20/wasi-sdk-${WASI_SDK_VERSION}-mingw.tar.gz"
        tar xf wasi-sdk.tar.gz && mv "wasi-sdk-${WASI_SDK_VERSION}" "$WASI_SDK_DIR"
    else
        log "Unsupported OS for automatic WASI SDK download: $OSTYPE"
    fi
    rm -f wasi-sdk.tar.gz
fi

if [ -d "$WASI_SDK_DIR" ]; then
    export CC_wasm32_unknown_unknown="$WASI_SDK_DIR/bin/clang"
    export CFLAGS_wasm32_unknown_unknown="--sysroot=$WASI_SDK_DIR/share/wasi-sysroot -D_WASI_EMULATED_MMAN -D_WASI_EMULATED_SIGNAL -D_WASI_EMULATED_PROCESS_CLOCKS -D__wasi__ -DTREE_SITTER_FEATURE_WASM"
fi

# ─── Build core WASM ────────────────────────────────────────────────────────

build_core() {
    local mode="${1:-release}"
    log "Building crates/core (${mode})..."

    local out_dir="$WORKSPACE_ROOT/extension/dist/wasm"
    mkdir -p "$out_dir"

    if [ "$mode" = "dev" ]; then
        wasm-pack build \
            "$WORKSPACE_ROOT/crates/core" \
            --target no-modules \
            --out-dir "$out_dir" \
            --out-name "omni_core" \
            --dev
    else
        RUSTFLAGS="-C link-arg=--initial-memory=16777216 -C link-arg=--max-memory=67108864" wasm-pack build \
            "$WORKSPACE_ROOT/crates/core" \
            --target no-modules \
            --out-dir "$out_dir" \
            --out-name "omni_core" \
            --release
    fi

    local wasm_size
    wasm_size=$(wc -c < "$out_dir/omni_core_bg.wasm" 2>/dev/null || echo "0")
    log "core WASM size: ${wasm_size} bytes"

    if [ "$mode" = "release" ] && [ "$wasm_size" -gt 614400 ]; then
        log "WARNING: core WASM exceeds 600KB target (${wasm_size} bytes). Investigate size bloat."
    fi
}

# ─── Build language modules (Phase 3+) ──────────────────────────────────────

build_lang_module() {
    local name="$1"
    local crate_path="$WORKSPACE_ROOT/crates/$name"
    local out_dir="$WORKSPACE_ROOT/extension/dist/modules"

    [ -d "$crate_path" ] || { log "Skipping $name (not yet implemented)"; return; }

    log "Building $name..."
    mkdir -p "$out_dir"
    wasm-pack build \
        "$crate_path" \
        --target no-modules \
        --out-dir "$out_dir/$name" \
        --out-name "${name//-/_}" \
        --release
    log "$name built."
}

# ─── Main ────────────────────────────────────────────────────────────────────

if [ "$BUILD_MODE" = "--dev" ]; then
    build_core dev
else
    build_core release
    build_lang_module lang-js
    build_lang_module lang-python
    build_lang_module lang-rust
    build_lang_module lang-go
    build_lang_module lang-css
fi

log "All WASM builds complete."
