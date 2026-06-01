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
    __wbindgen_placeholder__: {
      __wbindgen_describe: () => {},
      __wbindgen_throw: (ptr: number, len: number) => {
        if (!wasmExports) throw new Error("WASM exports not ready");
        const mem = new Uint8Array(wasmExports.memory.buffer);
        const str = Buffer.from(mem.slice(ptr, ptr + len)).toString("utf8");
        throw new Error(str);
      }
    },
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
    console.time("format");
    const responseJson = callFormat(wasmExports, msg.requestJson);
    console.timeEnd("format");
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
 * wasm-bindgen (no-modules target) string ABI:
 *   - Input:  caller writes UTF-8 bytes via __wbindgen_malloc, passes (ptr, len).
 *   - Output: `format()` writes a (ptr, len) pair to the stack pointer location,
 *             then returns the stack pointer. Caller reads 2x i32 from that address.
 *   - Caller must free the input buffer with __wbindgen_free after the call.
 *   - The output buffer is owned by WASM and must also be freed after reading.
 */
function callFormat(exports: WasmExports, requestJson: string): string {
  // 1. Write the request JSON string into WASM memory.
  const [reqPtr, reqLen] = writeStringToWasm(exports, requestJson);

  // 2. Reserve 8 bytes on the WASM stack for the (ptr, len) return value.
  //    __wbindgen_add_to_stack_pointer(-8) allocates stack space.
  const retStackPtr = exports.__wbindgen_add_to_stack_pointer(-8);

  let responseJson: string;
  try {
    // 3. Call format(). For wasm-bindgen string return, the real function
    //    signature is: format(ret_ptr: i32, ptr: i32, len: i32) -> void
    //    and it writes the result (ptr, len) at ret_ptr.
    //    However, the exported `format` via wasm-bindgen may use a different
    //    calling convention. We call it via the raw export name.
    (exports as unknown as Record<string, (...args: any[]) => any>)["__wbg_format_or_format"](
      retStackPtr, reqPtr, reqLen
    );

    // 4. Read the output (ptr, len) from the stack.
    const mem = new Int32Array(exports.memory.buffer);
    const outPtr = mem[retStackPtr / 4];
    const outLen = mem[retStackPtr / 4 + 1];

    // 5. Read the response string from WASM memory.
    responseJson = readStringFromWasm(exports, outPtr, outLen);

    // 6. Free the output buffer (WASM owns it, we must release it).
    exports.__wbindgen_free(outPtr, outLen, 1);
  } catch (_abi_err) {
    // Fallback: try calling `format` directly as a JS-friendly export.
    // wasm-bindgen may expose it with JS shim that handles memory automatically.
    try {
      const formatFn = (exports as unknown as Record<string, (...args: any[]) => any>)["format"];
      if (typeof formatFn === "function") {
        responseJson = formatFn(requestJson) as string;
      } else {
        throw new Error("WASM export 'format' not found");
      }
    } catch (fallback_err) {
      throw new Error(`WASM format call failed: ${fallback_err}`);
    }
  } finally {
    // 7. Always restore the stack pointer and free the input buffer.
    exports.__wbindgen_add_to_stack_pointer(8);
    exports.__wbindgen_free(reqPtr, reqLen, 1);
  }

  return responseJson!;
}

init();
