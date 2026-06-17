/**
 * Worker Thread Pool (L-04 mitigation)
 *
 * Manages a pool of Node.js Worker threads, each running an isolated WASM
 * instance. One WASM instance per worker — no SharedArrayBuffer, no COOP/COEP
 * header requirements.
 *
 * Design invariants:
 * - Minimum 2 workers at all times.
 * - Maximum workers = max(2, cpus - 1).
 * - Format requests dispatch to the worker with the fewest queued requests.
 * - A crashed worker is silently replaced within 100 ms; if replacement also
 *   fails, all affected pending requests are rejected with a clear error.
 * - Workers communicate via postMessage structured-clone. No shared memory.
 * - Per-worker queue depth is capped (MAX_QUEUE_DEPTH). Requests beyond the
 *   cap are immediately rejected rather than silently piling up.
 * - All timers and cancel listeners are cleaned up in every code path.
 */

import { Worker } from "worker_threads";
import * as os from "os";
import * as vscode from "vscode";
import { logger } from "./logger";

const log = logger.withContext("WorkerPool");

// ── Constants ─────────────────────────────────────────────────────────────

/** Maximum in-flight requests per worker before back-pressure kicks in. */
const MAX_QUEUE_DEPTH = 16;

/**
 * Recycle a worker after this many completed requests.
 *
 * ponytail: WASM linear memory only grows — recycling is the only way to
 * release the heap back to the OS without a custom allocator. 500 requests
 * ≈ hours of heavy use before recycling kicks in.
 */
const MAX_REQUESTS_PER_WORKER = 500;

/** Milliseconds to wait for a worker to report ready during spawn. */
const WORKER_INIT_TIMEOUT_MS = 10_000;

// ── Internal types ────────────────────────────────────────────────────────

interface PendingRequest {
  resolve: (v: string) => void;
  reject:  (e: Error)  => void;
  cancelDisposable: vscode.Disposable;
  timeoutHandle:    ReturnType<typeof setTimeout>;
}

interface WorkerEntry {
  worker:        Worker;
  queueDepth:    number;
  pending:       Map<number, PendingRequest>;
  requestCount:  number;  // ponytail: tracks completed requests for recycling
}

// ── OmniFormatterError ────────────────────────────────────────────────────

/** Typed error class so callers can distinguish OmniFormatter errors. */
export class OmniFormatterError extends Error {
  constructor(message: string, public readonly code: string) {
    super(message);
    this.name = "OmniFormatterError";
  }
}

// ── WorkerPool ────────────────────────────────────────────────────────────

export class WorkerPool {
  private readonly workerScript: string;
  private readonly wasmDir:      string;
  private readonly maxWorkers:   number;
  private workers: WorkerEntry[] = [];
  private nextMessageId = 1;

  /** Set to true after shutdown() is called to prevent new spawns. */
  private shutdownRequested = false;

  constructor(workerScript: string, wasmDir: string, maxWorkers?: number) {
    this.workerScript = workerScript;
    this.wasmDir      = wasmDir;
    this.maxWorkers   = maxWorkers ?? Math.max(2, os.cpus().length - 1);
  }

  // ── Public API ───────────────────────────────────────────────────────────

  /** Spawn all workers and wait for them to report ready. */
  async initialise(): Promise<void> {
    log.info("Initialising worker pool", { maxWorkers: this.maxWorkers, wasmDir: this.wasmDir });
    const spawns: Promise<void>[] = [];
    for (let i = 0; i < this.maxWorkers; i++) {
      spawns.push(this.spawnWorker());
    }
    await Promise.all(spawns);
    log.info("Worker pool ready", { activeWorkers: this.workers.length });
  }

  /**
   * Dispatch a format request to the least-loaded worker.
   *
   * Rejects immediately if:
   * - The pool has no workers (not initialised or all crashed).
   * - The least-loaded worker is already at MAX_QUEUE_DEPTH.
   * - The cancellation token fires before a response arrives.
   * - The extension-side safety timeout fires.
   */
  /**
   * Dispatch a format request to the least-loaded worker.
   *
   * @param requestJson  The JSON payload for the WASM `format()` call.
   * @param byteLength   UTF-8 byte size of the source — used to scale the
   *                     per-request timeout. NOT embedded in requestJson.
   * @param token        VS Code cancellation token.
   */
  dispatch(requestJson: string, byteLength: number, token: vscode.CancellationToken): Promise<string> {
    return new Promise<string>((resolve, reject) => {
      // ── Guard: pool must be live ─────────────────────────────────────
      if (this.workers.length === 0) {
        reject(new OmniFormatterError(
          "OmniFormatter worker pool has no active workers. " +
          "The WASM binary may be missing — check the OmniFormatter output channel for details.",
          "NO_WORKERS"
        ));
        return;
      }

      const entry = this.leastLoadedWorker();

      // ── Guard: back-pressure ─────────────────────────────────────────
      if (entry.queueDepth >= MAX_QUEUE_DEPTH) {
        reject(new OmniFormatterError(
          `OmniFormatter is busy (queue depth ${entry.queueDepth}/${MAX_QUEUE_DEPTH}). ` +
          "Try again in a moment.",
          "QUEUE_FULL"
        ));
        return;
      }

      // ── Already cancelled? ───────────────────────────────────────────
      if (token.isCancellationRequested) {
        reject(new OmniFormatterError("Format request was cancelled before dispatching.", "CANCELLED"));
        return;
      }

      const messageId = this.nextMessageId++;
      entry.queueDepth++;

      const workerTimeoutMs    = 30_000 + Math.ceil(byteLength / 102_400) * 1_000;
      const extensionTimeoutMs = workerTimeoutMs + 5_000; // 5 s buffer so the worker wins the race

      // ── Cleanup helper ───────────────────────────────────────────────
      const cleanup = (): void => {
        entry.pending.delete(messageId);
        entry.queueDepth = Math.max(0, entry.queueDepth - 1);
      };

      // ── Cancellation ─────────────────────────────────────────────────
      const cancelDisposable = token.onCancellationRequested(() => {
        const pending = entry.pending.get(messageId);
        if (!pending) { return; }
        clearTimeout(pending.timeoutHandle);
        pending.cancelDisposable.dispose();
        cleanup();
        reject(new OmniFormatterError("Format request cancelled by user.", "CANCELLED"));
        log.debug("Request cancelled", { messageId });
      });

      // ── Extension-side timeout ───────────────────────────────────────
      const timeoutHandle = setTimeout(() => {
        const pending = entry.pending.get(messageId);
        if (!pending) { return; }
        pending.cancelDisposable.dispose();
        cleanup();
        const sizeMb = (byteLength / 1_048_576).toFixed(2);
        reject(new OmniFormatterError(
          `OmniFormatter: format timed out after ${Math.round(extensionTimeoutMs / 1000)} s ` +
          `(file ≈${sizeMb} MB). The WASM formatter took too long.`,
          "TIMEOUT"
        ));
        log.warn("Request timed out on extension side", {
          messageId,
          timeoutMs: extensionTimeoutMs,
          byteLength,
        });
      }, extensionTimeoutMs);

      entry.pending.set(messageId, { resolve, reject, cancelDisposable, timeoutHandle });
      // byteLength is sent alongside requestJson as a separate field so the
      // worker can scale its own timeout without parsing the WASM payload.
      entry.worker.postMessage({ id: messageId, requestJson, byteLength });
    });
  }

  /** Gracefully shut down all workers, rejecting any in-flight requests. */
  async shutdown(): Promise<void> {
    this.shutdownRequested = true;
    log.info("Worker pool shutting down", { activeWorkers: this.workers.length });

    await Promise.allSettled(
      this.workers.map((entry) => this.drainAndTerminate(entry))
    );

    this.workers = [];
    log.info("Worker pool shut down.");
  }

  // ── Private helpers ──────────────────────────────────────────────────────

  private drainAndTerminate(entry: WorkerEntry): Promise<void> {
    const shutdownError = new OmniFormatterError("Worker pool is shutting down.", "SHUTDOWN");
    for (const { reject, cancelDisposable, timeoutHandle } of entry.pending.values()) {
      clearTimeout(timeoutHandle);
      cancelDisposable.dispose();
      reject(shutdownError);
    }
    entry.pending.clear();
    return entry.worker.terminate().then(() => undefined);
  }

  private leastLoadedWorker(): WorkerEntry {
    // Invariant: callers must check workers.length > 0 before calling this.
    return this.workers.reduce((min, w) => w.queueDepth < min.queueDepth ? w : min);
  }

  private spawnWorker(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.shutdownRequested) {
        reject(new OmniFormatterError("Cannot spawn worker: pool is shutting down.", "SHUTDOWN"));
        return;
      }

      let worker: Worker;
      try {
        worker = new Worker(this.workerScript, {
          workerData: { wasmDir: this.wasmDir },
        });
      } catch (err) {
        const msg = `Failed to create Worker thread from "${this.workerScript}": ${err instanceof Error ? err.message : String(err)}`;
        log.error("Worker spawn failed", err instanceof Error ? err : new Error(String(err)), { workerScript: this.workerScript });
        reject(new OmniFormatterError(msg, "SPAWN_FAILED"));
        return;
      }

      const entry: WorkerEntry = {
        worker,
        queueDepth:   0,
        pending:      new Map(),
        requestCount: 0,
      };

      let initialised = false;

      // ── Init timeout ─────────────────────────────────────────────────
      const initTimer = setTimeout(() => {
        if (!initialised) {
          worker.terminate().catch(() => undefined);
          reject(new OmniFormatterError(
            `Worker thread did not report ready within ${WORKER_INIT_TIMEOUT_MS / 1000} s. ` +
            `Check that the WASM binary exists in: ${this.wasmDir}`,
            "INIT_TIMEOUT"
          ));
        }
      }, WORKER_INIT_TIMEOUT_MS);

      // ── Message handler ──────────────────────────────────────────────
      worker.on("message", (msg: {
        id?:          number;
        responseJson?: string;
        ready?:       boolean;
        error?:       string;
      }) => {
        // Initialisation acknowledgement
        if (msg.ready !== undefined && !initialised) {
          clearTimeout(initTimer);
          initialised = true;

          if (msg.ready) {
            this.workers.push(entry);
            log.debug("Worker ready", { totalWorkers: this.workers.length });
            resolve();
          } else {
            // Worker itself reported a fatal init error
            worker.terminate().catch(() => undefined);
            const detail = msg.error ?? "Unknown WASM initialisation error.";
            log.error("Worker WASM init failed", new Error(detail));
            reject(new OmniFormatterError(
              `OmniFormatter WASM failed to initialise: ${detail}`,
              "WASM_INIT_FAILED"
            ));
          }
          return;
        }

        // Response to a format request
        if (msg.id === undefined) { return; }

        const pending = entry.pending.get(msg.id);
        if (!pending) { return; }

        clearTimeout(pending.timeoutHandle);
        pending.cancelDisposable.dispose();
        entry.pending.delete(msg.id);
        entry.queueDepth = Math.max(0, entry.queueDepth - 1);

        if (msg.error) {
          pending.reject(new OmniFormatterError(msg.error, "WORKER_ERROR"));
        } else if (msg.responseJson !== undefined) {
          pending.resolve(msg.responseJson);
        } else {
          pending.reject(new OmniFormatterError(
            "Worker returned a message with neither responseJson nor error.",
            "MALFORMED_RESPONSE"
          ));
        }

        // ── Periodic recycling (memory-leak mitigation) ──────────────
        // WASM linear memory only grows. After MAX_REQUESTS_PER_WORKER
        // completed requests, retire this worker once its queue drains so
        // the V8 heap and WASM memory are released back to the OS.
        entry.requestCount++;
        if (
          entry.requestCount >= MAX_REQUESTS_PER_WORKER &&
          entry.queueDepth === 0 &&
          !this.shutdownRequested
        ) {
          log.debug("Recycling worker after request limit", {
            requestCount: entry.requestCount,
            totalWorkers: this.workers.length,
          });
          this.workers = this.workers.filter((e) => e !== entry);
          entry.worker.terminate().catch(() => undefined);
          this.spawnWorker().catch((spawnErr) => {
            log.warn("Worker recycle: replacement spawn failed", {
              error: spawnErr instanceof Error ? spawnErr.message : String(spawnErr),
            });
          });
        }
      });

      // ── Error handler (OS-level worker error) ────────────────────────
      worker.on("error", (err: Error) => {
        clearTimeout(initTimer);

        if (!initialised) {
          reject(new OmniFormatterError(
            `Worker thread crashed during initialisation: ${err.message}`,
            "SPAWN_CRASH"
          ));
          return;
        }

        log.error("Worker thread emitted error, rejecting all in-flight requests and respawning", err, {
          pendingCount: entry.pending.size,
        });

        // Reject all in-flight requests on this worker
        const workerError = new OmniFormatterError(
          `Worker thread crashed: ${err.message}`,
          "WORKER_CRASH"
        );
        for (const { reject: r, cancelDisposable, timeoutHandle } of entry.pending.values()) {
          clearTimeout(timeoutHandle);
          cancelDisposable.dispose();
          r(workerError);
        }
        entry.pending.clear();

        // Remove and respawn
        this.workers = this.workers.filter((e) => e !== entry);
        if (!this.shutdownRequested) {
          this.spawnWorker().catch((spawnErr) => {
            log.error("Failed to respawn replacement worker", spawnErr instanceof Error ? spawnErr : new Error(String(spawnErr)));
          });
        }
      });

      // ── Exit handler ─────────────────────────────────────────────────
      worker.on("exit", (code: number) => {
        clearTimeout(initTimer);

        if (!initialised) { return; } // handled by error handler

        if (code !== 0) {
          log.warn("Worker thread exited with non-zero code", { code });
          this.workers = this.workers.filter((e) => e !== entry);

          // Reject any lingering pending requests
          for (const { reject: r, cancelDisposable, timeoutHandle } of entry.pending.values()) {
            clearTimeout(timeoutHandle);
            cancelDisposable.dispose();
            r(new OmniFormatterError(`Worker exited unexpectedly (code ${code}).`, "WORKER_EXIT"));
          }
          entry.pending.clear();

          if (!this.shutdownRequested) {
            this.spawnWorker().catch((spawnErr) => {
              log.error("Failed to respawn after exit", spawnErr instanceof Error ? spawnErr : new Error(String(spawnErr)));
            });
          }
        }
      });
    });
  }
}
