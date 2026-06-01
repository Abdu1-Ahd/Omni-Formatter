#!/bin/bash
set -e
export WASI_SDK_DIR="$PWD/wasi-sdk-20.0"
if [ ! -d "$WASI_SDK_DIR" ]; then
  curl -L -o wasi-sdk.tar.gz "https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-20/wasi-sdk-20.0-mingw.tar.gz"
  tar xf wasi-sdk.tar.gz
fi
export CC_wasm32_unknown_unknown="$WASI_SDK_DIR/bin/clang.exe"
export CFLAGS_wasm32_unknown_unknown="--sysroot=$WASI_SDK_DIR/share/wasi-sysroot -D_WASI_EMULATED_MMAN -D_WASI_EMULATED_SIGNAL -D_WASI_EMULATED_PROCESS_CLOCKS -D__wasi__ -Ddup(x)=(x)"
cargo install wasm-pack --version "^0.13"
wasm-pack build crates/core --target nodejs --release
