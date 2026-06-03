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

let wasmBindgenApi: any = null;

async function loadWasm(): Promise<void> {
  const wasmPath = path.join(wasmDir, "omni_core_bg.wasm");
  const jsPath = path.join(wasmDir, "omni_core.js");

  if (!fs.existsSync(wasmPath) || !fs.existsSync(jsPath)) {
    throw new Error(`WASM binary or JS wrapper not found in: ${wasmDir}`);
  }

  const wasmBytes = fs.readFileSync(wasmPath);
  const jsCode = fs.readFileSync(jsPath, "utf8");

  const patchedJsCode = jsCode.replace(
    /"\.\/omni_core_bg\.js": import0,/g,
    '"./omni_core_bg.js": import0, "./core_bg.js": import0,'
  );

  // Evaluate the generated JS wrapper in this worker context
  const getWasmBindgen = new Function(`
    ${patchedJsCode}
    return wasm_bindgen;
  `);
  
  const initWasmBindgen = getWasmBindgen();
  await initWasmBindgen(wasmBytes);
  
  wasmBindgenApi = initWasmBindgen;
  wasmBindgenApi.init_wasm();
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
  if (!wasmBindgenApi) {
    parentPort!.postMessage({
      id: msg.id,
      error: "WASM not initialised",
    });
    return;
  }

  try {
    console.time("format");
    const responseJson = wasmBindgenApi.format(msg.requestJson);
    console.timeEnd("format");
    parentPort!.postMessage({ id: msg.id, responseJson });
  } catch (err) {
    parentPort!.postMessage({
      id: msg.id,
      error: `Worker error: ${err}`,
    });
  }
});

init();
