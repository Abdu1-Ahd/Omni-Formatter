/**
 * FormattingState — per-document last-format result store.
 *
 * Singleton shared between the extension host, CodeLens provider,
 * and Hover provider.
 *
 * Invariants:
 * - The map is capped at MAX_ENTRIES; oldest entries are evicted (FIFO) to
 *   prevent unbounded growth across a long editing session.
 * - The EventEmitter is disposed when `dispose()` is called (called in
 *   `deactivate()` so there are no leaks across extension reloads).
 * - The singleton is reset by `dispose()` so a subsequent `getInstance()`
 *   call (e.g. in tests) gets a fresh instance.
 */

import * as vscode from "vscode";

// ── Types ─────────────────────────────────────────────────────────────────

export interface FormattingResult {
  formatterChain: string;
  elapsedMs:      number;
  timestamp:      number;
}

// ── Constants ─────────────────────────────────────────────────────────────

/** Maximum number of per-document results kept in memory. */
const MAX_ENTRIES = 200;

// ── Singleton ─────────────────────────────────────────────────────────────

export class FormattingState implements vscode.Disposable {
  private static _instance: FormattingState | undefined;

  /** Ordered insertion-key list for FIFO eviction. */
  private readonly _insertionOrder: string[] = [];
  private readonly _state:          Map<string, FormattingResult> = new Map();
  private readonly _emitter:        vscode.EventEmitter<vscode.Uri> = new vscode.EventEmitter<vscode.Uri>();

  public readonly onDidChange: vscode.Event<vscode.Uri> = this._emitter.event;

  private constructor() {}

  public static getInstance(): FormattingState {
    if (!FormattingState._instance) {
      FormattingState._instance = new FormattingState();
    }
    return FormattingState._instance;
  }

  // ── Mutations ─────────────────────────────────────────────────────────

  public updateState(uri: vscode.Uri, result: FormattingResult): void {
    const key = uri.toString();

    if (!this._state.has(key)) {
      // New entry — track insertion order
      this._insertionOrder.push(key);

      // Evict oldest entry if over cap
      while (this._insertionOrder.length > MAX_ENTRIES) {
        const oldest = this._insertionOrder.shift();
        if (oldest) {
          this._state.delete(oldest);
        }
      }
    }

    this._state.set(key, result);
    this._emitter.fire(uri);
  }

  /** Remove state for a document (e.g. when it is closed). */
  public deleteState(uri: vscode.Uri): void {
    const key = uri.toString();
    const idx = this._insertionOrder.indexOf(key);
    if (idx !== -1) {
      this._insertionOrder.splice(idx, 1);
    }
    this._state.delete(key);
  }

  // ── Queries ───────────────────────────────────────────────────────────

  public getState(uri: vscode.Uri): FormattingResult | undefined {
    return this._state.get(uri.toString());
  }

  // ── Lifecycle ─────────────────────────────────────────────────────────

  public dispose(): void {
    this._emitter.dispose();
    this._state.clear();
    this._insertionOrder.length = 0;
    // Reset the singleton so tests / reloads get a fresh instance.
    FormattingState._instance = undefined;
  }
}
