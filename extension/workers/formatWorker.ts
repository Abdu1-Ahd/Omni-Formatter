/**
 * Format Worker Thread Entry Point
 *
 * Each worker runs in its own isolated Node.js Worker thread with its own
 * WASM instance. No SharedArrayBuffer. No shared state. (L-04 mitigation)
 *
 * Lifecycle:
 * 1. Worker starts, reads `wasmDir` from workerData.
 * 2. Loads and instantiates the core WASM module.
 * 3. Sends { ready: true } to the parent thread.
 * 4. For each incoming { id, requestJson } message:
 *    a. Calls the WASM `format()` export.
 *    b. Sends { id, responseJson } back.
 */

import { workerData, parentPort } from "worker_threads";
import * as fs from "fs";
import * as path from "path";

if (!parentPort) {
  throw new Error("formatWorker.ts must run inside a Worker thread");
}

const { wasmDir } = workerData as { wasmDir: string };

/** WASM core exports interface */
interface WasmExports {
  format: (requestJsonPtr: number, requestJsonLen: number) => number;
  memory: WebAssembly.Memory;
  // wasm-bindgen generated helpers
  __wbindgen_malloc: (size: number, align: number) => number;
  __wbindgen_realloc: (ptr: number, oldSize: number, newSize: number, align: number) => number;
  __wbindgen_free: (ptr: number, size: number, align: number) => void;
  __wbindgen_add_to_stack_pointer: (delta: number) => number;
}

let wasmExports: WasmExports | null = null;

/**
 * Write a JavaScript string into the WASM linear memory.
 * Returns [ptr, len] in bytes.
 */
function writeStringToWasm(exports: WasmExports, str: string): [number, number] {
  const encoded = Buffer.from(str, "utf8");
  const len = encoded.length;
  const ptr = exports.__wbindgen_malloc(len, 1);
  const mem = new Uint8Array(exports.memory.buffer);
  mem.set(encoded, ptr);
  return [ptr, len];
}

/**
 * Read a null-terminated string from WASM linear memory at ptr.
 * wasm-bindgen passes return values as a (ptr, len) pair written
 * to the stack pointer location.
 */
function readStringFromWasm(exports: WasmExports, ptr: number, len: number): string {
  const mem = new Uint8Array(exports.memory.buffer);
  return Buffer.from(mem.slice(ptr, ptr + len)).toString("utf8");
}

async function loadWasm(): Promise<void> {
  const wasmPath = path.join(wasmDir, "omni_core_bg.wasm");

  if (!fs.existsSync(wasmPath)) {
    throw new Error(`WASM binary not found at: ${wasmPath}. Run: bash scripts/build-wasm.sh`);
  }

  const wasmBytes = fs.readFileSync(wasmPath);
  const wasmModule = await WebAssembly.compile(wasmBytes);

  // wasm-bindgen no-modules target requires an imports object
  const instance = await WebAssembly.instantiate(wasmModule, {
    // wasm-bindgen may inject __wbindgen_placeholder__ imports in future versions
    __wbindgen_placeholder__: {},
  });

  wasmExports = instance.exports as unknown as WasmExports;
}

async function init(): Promise<void> {
  try {
    await loadWasm();
    parentPort!.postMessage({ ready: true });
  } catch (err) {
    parentPort!.postMessage({ ready: false, error: String(err) });
    process.exit(1);
  }
}

parentPort.on("message", (msg: { id: number; requestJson: string }) => {
  if (!wasmExports) {
    parentPort!.postMessage({
      id: msg.id,
      error: "WASM not initialised",
    });
    return;
  }

  try {
    // The wasm-bindgen no-modules build wraps the format function.
    // For the Phase 1 stub, we call a simplified interface.
    // In Phase 3+, the full wasm-bindgen JS glue handles memory management.
    const responseJson = callFormat(wasmExports, msg.requestJson);
    parentPort!.postMessage({ id: msg.id, responseJson });
  } catch (err) {
    parentPort!.postMessage({
      id: msg.id,
      error: `Worker error: ${err}`,
    });
  }
});

/**
 * Call the WASM `format()` function with a JSON string request.
 *
 * This is a simplified direct call for Phase 1 (stub WASM).
 * Phase 3+ uses the full wasm-bindgen JS glue which handles all
 * memory management automatically.
 */
function callFormat(exports: WasmExports, requestJson: string): string {
  // For Phase 1, the stub format() takes and returns JS strings via wasm-bindgen.
  // The actual low-level ABI is managed by wasm-bindgen generated code.
  // Here we simulate the call for the pass-through stub.
  const request = JSON.parse(requestJson);
  const response = {
    edits: [],
    formatter_chain: `OmniFormatter core (stub) for ${request.language_id}`,
    is_noop: true,
  };
  return JSON.stringify(response);
}

init();
