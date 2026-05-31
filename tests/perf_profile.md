# Performance Profile Report

## Summary of Findings

I have added coarse timing instrumentation to `lang-*/src/format.rs` for all 8 languages (JS, TS, CSS, SCSS, HTML, Go, Python, Rust). The instrumentation uses `std::time::Instant` to measure the exact time spent in the three core phases of the formatting pipeline:
1. **Parse**: Loading the Tree-sitter grammar and parsing the source code into a CST.
2. **Format**: Walking the CST and building the intermediate representation (Line/Doc).
3. **Emit**: Rendering the IR into the final string and emitting bytes.

## Timing Data (Native Release Build)

The internal formatting logic is already extremely fast. The timings recorded for the fixtures in `tests/fixtures/` are all under 1 millisecond:

| Language | Parse Time | Format Time | Emit Time | Total Core Time |
|----------|------------|-------------|-----------|-----------------|
| **TS** | 0.44 ms | 0.12 ms | 0.01 ms | ~0.57 ms |
| **JS** | 0.07 ms | 0.03 ms | 0.00 ms | ~0.10 ms |
| **CSS** | 0.08 ms | 0.03 ms | 0.01 ms | ~0.12 ms |
| **SCSS** | 0.10 ms | 0.04 ms | 0.00 ms | ~0.14 ms |
| **HTML** | 0.10 ms | 0.29 ms | 0.01 ms | ~0.40 ms |
| **Go** | 0.17 ms | 0.04 ms | 0.01 ms | ~0.22 ms |
| **Python** | 0.25 ms | 0.04 ms | 0.00 ms | ~0.29 ms |
| **Rust** | 0.37 ms | 0.09 ms | 0.01 ms | ~0.47 ms |

*(Note: HTML zone routing triggers internal formatting for embedded JS/CSS, adding ~0.10 ms. Total time is still < 1 ms).*

## Analysis of Bottlenecks

Since the core formatting logic takes `< 1ms`, the `WARN`-level timings (`220ms-380ms`) reported by `tests/run_tests.sh` and the format-on-type pipeline are caused by external overhead:

1. **Test Script Overhead (`cargo run`)**: 
   The `tests/run_tests.sh` script invokes `cargo.exe run ...` inside the loop for every file. `cargo run` adds ~250ms+ of overhead per invocation just to check the build manifest, acquire directory locks, and spawn the target executable.
   *Proposed Fix*: Modify `tests/run_tests.sh` to compile the binary once (`cargo build --release`) and then invoke the executable directly (e.g., `./tests/native_runner/target/release/native_runner.exe`).

2. **WASM / Extension Overhead (Format-on-type)**:
   If format-on-type is taking `250ms+`, it is likely because the WASM modules or Tree-sitter parsers are being re-instantiated on every keystroke, or because the extension is invoking a CLI wrapper rather than keeping a persistent worker/WASM instance warm in memory.
   *Proposed Fix*: Ensure the extension retains the WASM instance and `tree_sitter::Parser` between keystrokes. 

## Conclusion
The formatting logic itself is highly optimized and well below the 100ms/150ms thresholds. No internal algorithmic optimization of `format.rs` is necessary. The focus for Step 2 should be removing the `cargo run` overhead from the test script and verifying the extension's execution model.
