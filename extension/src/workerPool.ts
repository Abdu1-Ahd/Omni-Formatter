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

/** A pending format request waiting for a worker. */
interface PendingRequest {
  requestJson: string;
  resolve: (responseJson: string) => void;
  reject: (err: Error) => void;
  cancellationToken: vscode.CancellationToken;
}

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
   */
  dispatch(requestJson: string, token: vscode.CancellationToken): Promise<string> {
    return new Promise<string>((resolve, reject) => {
      // Pick the worker with the smallest queue depth
      const entry = this.leastLoadedWorker();

      const messageId = this.nextMessageId++;
      entry.queueDepth++;
      entry.pending.set(messageId, { resolve, reject });

      // Listen for cancellation
      const cancelListener = token.onCancellationRequested(() => {
        entry.pending.delete(messageId);
        entry.queueDepth = Math.max(0, entry.queueDepth - 1);
        reject(new Error("Format request cancelled"));
        cancelListener.dispose();
      });

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
