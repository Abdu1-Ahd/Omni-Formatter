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
 *
 * Error handling contract:
 * - WASM load failure → sends { ready: false, error: <message> } and exits.
 * - Per-request failure → sends { id, error: <message> }, worker stays alive.
 * - Timeout → sends { id, error: <timeout message> }, worker stays alive.
 *   (The extension-host pool will spawn a replacement if the worker exits.)
 */

import { workerData, parentPort } from "worker_threads";
import * as fs from "fs";
import * as path from "path";

// ── Type safety ──────────────────────────────────────────────────────────

interface WorkerData {
  wasmDir: string;
}

interface IncomingMessage {
  id: number;
  requestJson: string;
}

interface OutgoingReady {
  ready: true;
}

interface OutgoingInitError {
  ready: false;
  error: string;
}

interface OutgoingResponse {
  id: number;
  responseJson: string;
}

interface OutgoingError {
  id: number;
  error: string;
}

type OutgoingMessage = OutgoingReady | OutgoingInitError | OutgoingResponse | OutgoingError;

// ── Guards ───────────────────────────────────────────────────────────────

if (!parentPort) {
  // This file must only be loaded as a Worker thread, never directly.
  throw new Error("[formatWorker] Must run inside a Worker thread — direct execution is not supported.");
}

const { wasmDir } = (workerData as WorkerData);

if (!wasmDir || typeof wasmDir !== "string") {
  parentPort.postMessage({ ready: false, error: "[formatWorker] workerData.wasmDir is missing or not a string." } satisfies OutgoingInitError);
  process.exit(1);
}

// ── WASM binding ─────────────────────────────────────────────────────────

/** The initialised wasm-bindgen API object. Set once, then immutable. */
let wasmBindgenApi: {
  format: (requestJson: string) => string;
  init_wasm?: () => void | Promise<void>;
} | null = null;

async function loadWasm(): Promise<void> {
  const wasmPath = path.join(wasmDir, "omni_core_bg.wasm");
  const jsPath   = path.join(wasmDir, "omni_core.js");

  if (!fs.existsSync(wasmPath)) {
    throw new Error(`WASM binary not found at: ${wasmPath}`);
  }
  if (!fs.existsSync(jsPath)) {
    throw new Error(`WASM JS wrapper not found at: ${jsPath}`);
  }

  let wasmBytes: Buffer;
  let jsCode: string;
  try {
    wasmBytes = fs.readFileSync(wasmPath);
  } catch (err) {
    throw new Error(`Failed to read WASM binary at ${wasmPath}: ${err instanceof Error ? err.message : String(err)}`);
  }
  try {
    jsCode = fs.readFileSync(jsPath, "utf8");
  } catch (err) {
    throw new Error(`Failed to read WASM JS wrapper at ${jsPath}: ${err instanceof Error ? err.message : String(err)}`);
  }

  // Patch the import map so both possible import keys resolve correctly.
  const patchedJsCode = jsCode.replace(
    /"\.\/omni_core_bg\.js":\s*import0,/g,
    '"./omni_core_bg.js": import0, "./core_bg.js": import0,'
  );

  // Evaluate the generated JS wrapper in this worker context.
  let getWasmBindgen: () => typeof wasmBindgenApi;
  try {
    // eslint-disable-next-line no-new-func
    getWasmBindgen = new Function(`
      ${patchedJsCode}
      return wasm_bindgen;
    `) as () => typeof wasmBindgenApi;
  } catch (err) {
    throw new Error(`Failed to evaluate WASM JS wrapper: ${err instanceof Error ? err.message : String(err)}`);
  }

  const initWasmBindgen = getWasmBindgen();
  if (typeof initWasmBindgen !== "function") {
    throw new Error("WASM JS wrapper did not export a callable wasm_bindgen function.");
  }

  try {
    await (initWasmBindgen as (bytes: Buffer) => Promise<unknown>)(wasmBytes);
  } catch (err) {
    throw new Error(`wasm_bindgen initialisation failed: ${err instanceof Error ? err.message : String(err)}`);
  }

  wasmBindgenApi = initWasmBindgen as typeof wasmBindgenApi;

  // `init_wasm()` may be synchronous or return a Promise — handle both.
  if (typeof wasmBindgenApi?.init_wasm === "function") {
    try {
      const result = wasmBindgenApi.init_wasm();
      if (result instanceof Promise) {
        await result;
      }
    } catch (err) {
      throw new Error(`init_wasm() failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }
}

// ── Initialisation ───────────────────────────────────────────────────────

async function init(): Promise<void> {
  try {
    await loadWasm();
    (parentPort!).postMessage({ ready: true } satisfies OutgoingReady);
  } catch (err) {
    const message = err instanceof Error
      ? `${err.message}${err.stack ? `\n${err.stack}` : ""}`
      : String(err);
    (parentPort!).postMessage({ ready: false, error: message } satisfies OutgoingInitError);
    // Give the parent time to receive the message before exiting.
    await new Promise<void>((resolve) => setTimeout(resolve, 50));
    process.exit(1);
  }
}

// ── Message handler ──────────────────────────────────────────────────────

function send(msg: OutgoingMessage): void {
  parentPort!.postMessage(msg);
}

parentPort.on("message", (msg: IncomingMessage) => {
  if (!wasmBindgenApi) {
    send({ id: msg.id, error: "[formatWorker] WASM is not yet initialised — request dropped." });
    return;
  }

  if (typeof msg.id !== "number" || typeof msg.requestJson !== "string") {
    // Malformed message — log and ignore (no id to reply to reliably).
    console.error("[formatWorker] Received malformed message:", msg);
    return;
  }

  // ── Per-request timeout ──────────────────────────────────────────────
  // Scales with file size: 30 s base + 1 s per 100 KB of estimated source.
  let byteEstimate = 0;
  try {
    const meta = JSON.parse(msg.requestJson) as { source_byte_length?: unknown };
    if (typeof meta.source_byte_length === "number") {
      byteEstimate = meta.source_byte_length;
    }
  } catch {
    /* ignore — timeout uses the 30 s base */
  }

  const timeoutMs = 30_000 + Math.ceil(byteEstimate / 102_400) * 1_000;

  let timedOut = false;
  const timer = setTimeout(() => {
    timedOut = true;
    send({
      id: msg.id,
      error: (
        `[formatWorker] Format timed out after ${Math.round(timeoutMs / 1000)}s ` +
        `(≈${Math.round(byteEstimate / 1024)} KB file). ` +
        `The WASM formatter may be stuck on a pathological input.`
      ),
    });
  }, timeoutMs);

  // ── Call WASM ─────────────────────────────────────────────────────────
  const sizeTag = byteEstimate > 0 ? `[≈${Math.round(byteEstimate / 1024)}KB]` : "";
  const label   = `omni:format${sizeTag}`;
  console.time(label);

  try {
    const responseJson = wasmBindgenApi!.format(msg.requestJson);

    console.timeEnd(label);
    clearTimeout(timer);

    if (!timedOut) {
      send({ id: msg.id, responseJson });
    }
  } catch (err) {
    console.timeEnd(label);
    clearTimeout(timer);

    if (!timedOut) {
      const detail = err instanceof Error
        ? `${err.message}${err.stack ? `\n${err.stack}` : ""}`
        : String(err);
      send({
        id: msg.id,
        error: `[formatWorker] WASM format() threw an exception: ${detail}`,
      });
    }
  }
});

// ── Unhandled rejection / uncaught exception safety net ─────────────────

process.on("uncaughtException", (err: Error) => {
  const message = `[formatWorker] Uncaught exception: ${err.message}\n${err.stack ?? ""}`;
  console.error(message);
  // Exit so the pool spawns a replacement worker.
  process.exit(1);
});

process.on("unhandledRejection", (reason: unknown) => {
  const message = reason instanceof Error
    ? `[formatWorker] Unhandled rejection: ${reason.message}\n${reason.stack ?? ""}`
    : `[formatWorker] Unhandled rejection: ${String(reason)}`;
  console.error(message);
  process.exit(1);
});

// ── Start ─────────────────────────────────────────────────────────────────

init();
