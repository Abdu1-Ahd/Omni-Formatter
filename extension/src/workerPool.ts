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
 * - A crashed worker is silently replaced within 100ms.
 * - Workers communicate via postMessage structured-clone. No shared memory.
 */

import { Worker } from "worker_threads";
import * as os from "os";
import * as vscode from "vscode";

/** Internal state for a single worker in the pool. */
interface WorkerEntry {
  worker: Worker;
  /** Number of requests currently queued or in-flight on this worker. */
  queueDepth: number;
  /** Resolve/reject callbacks for each in-flight message ID. */
  pending: Map<number, { resolve: (v: string) => void; reject: (e: Error) => void }>;
}

export class WorkerPool {
  private readonly workerScript: string;
  private readonly wasmDir: string;
  private readonly maxWorkers: number;
  private workers: WorkerEntry[] = [];
  private nextMessageId = 1;

  constructor(workerScript: string, wasmDir: string, maxWorkers?: number) {
    this.workerScript = workerScript;
    this.wasmDir = wasmDir;
    this.maxWorkers = maxWorkers ?? Math.max(2, os.cpus().length - 1);
  }

  /** Spawn all workers and wait for them to report ready. */
  async initialise(): Promise<void> {
    const promises: Promise<void>[] = [];
    for (let i = 0; i < this.maxWorkers; i++) {
      promises.push(this.spawnWorker());
    }
    await Promise.all(promises);
  }

  /**
   * Dispatch a format request to the least-loaded worker.
   *
   * Returns the JSON-serialised FormatResponse from the WASM core.
   * Rejects if the cancellation token fires before a response arrives.
   *
   * Timeout: mirrors the worker-side timeout (30s + 1s per 100 KB) with a
   * small buffer so the worker always replies first with a clean error.
   */
  dispatch(requestJson: string, token: vscode.CancellationToken): Promise<string> {
    return new Promise<string>((resolve, reject) => {
      const entry = this.leastLoadedWorker();

      const messageId = this.nextMessageId++;
      entry.queueDepth++;
      entry.pending.set(messageId, { resolve, reject });

      // ── Cancellation support ──────────────────────────────────────────
      const cancelListener = token.onCancellationRequested(() => {
        entry.pending.delete(messageId);
        entry.queueDepth = Math.max(0, entry.queueDepth - 1);
        reject(new Error("Format request cancelled by user"));
        cancelListener.dispose();
        extensionTimer && clearTimeout(extensionTimer);
      });

      // ── Extension-side safety timeout ─────────────────────────────────
      // Slightly longer than the worker timeout so the worker always wins
      // the race and can return a clean error message.
      let byteLength = 0;
      try {
        const parsed = JSON.parse(requestJson) as { source_byte_length?: number };
        byteLength = parsed.source_byte_length ?? 0;
      } catch { /* ignore */ }

      const workerTimeoutMs   = 30_000 + Math.ceil(byteLength / 102_400) * 1_000;
      const extensionTimeoutMs = workerTimeoutMs + 5_000; // 5s buffer

      const extensionTimer = setTimeout(() => {
        if (entry.pending.has(messageId)) {
          entry.pending.delete(messageId);
          entry.queueDepth = Math.max(0, entry.queueDepth - 1);
          cancelListener.dispose();
          reject(new Error(
            `OmniFormatter: format timed out after ${Math.round(extensionTimeoutMs / 1000)}s ` +
            `(${Math.round(byteLength / 1024)} KB file)`
          ));
        }
      }, extensionTimeoutMs);

      entry.worker.postMessage({ id: messageId, requestJson });
    });
  }


  /** Gracefully shut down all workers. */
  async shutdown(): Promise<void> {
    await Promise.all(
      this.workers.map((entry) => {
        entry.pending.forEach(({ reject }) => reject(new Error("Worker pool shutting down")));
        return entry.worker.terminate();
      })
    );
    this.workers = [];
  }

  /** Returns the worker entry with the fewest queued requests. */
  private leastLoadedWorker(): WorkerEntry {
    return this.workers.reduce((min, w) =>
      w.queueDepth < min.queueDepth ? w : min
    );
  }

  /** Spawn a single worker thread and add it to the pool. */
  private spawnWorker(): Promise<void> {
    return new Promise((resolve, reject) => {
      const worker = new Worker(this.workerScript, {
        workerData: { wasmDir: this.wasmDir },
      });

      const entry: WorkerEntry = {
        worker,
        queueDepth: 0,
        pending: new Map(),
      };

      let initialised = false;

      worker.on("message", (msg: { id?: number; responseJson?: string; ready?: boolean; error?: string }) => {
        // Worker signals ready after loading WASM
        if (msg.ready && !initialised) {
          initialised = true;
          this.workers.push(entry);
          resolve();
          return;
        }

        if (msg.id === undefined) return;

        const pending = entry.pending.get(msg.id);
        if (!pending) return;

        entry.pending.delete(msg.id);
        entry.queueDepth = Math.max(0, entry.queueDepth - 1);

        if (msg.error) {
          pending.reject(new Error(msg.error));
        } else if (msg.responseJson !== undefined) {
          pending.resolve(msg.responseJson);
        }
      });

      worker.on("error", (err) => {
        if (!initialised) {
          reject(err);
          return;
        }
        // Reject all pending requests on this worker
        entry.pending.forEach(({ reject: r }) => r(err));
        entry.pending.clear();
        // Remove the crashed worker and spawn a replacement
        this.workers = this.workers.filter((e) => e !== entry);
        this.spawnWorker().catch(console.error);
      });

      worker.on("exit", (code) => {
        if (code !== 0 && initialised) {
          // Worker exited unexpectedly — spawn a replacement
          this.workers = this.workers.filter((e) => e !== entry);
          this.spawnWorker().catch(console.error);
        }
      });

      // Timeout: if worker doesn't signal ready within 5 seconds, reject
      setTimeout(() => {
        if (!initialised) {
          worker.terminate();
          reject(new Error("Worker failed to initialise within 5 seconds"));
        }
      }, 5000);
    });
  }
}
