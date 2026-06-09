/**
 * Format-on-Type Request Handler (L-13 mitigation)
 *
 * Handles VS Code's `onDidChangeTextDocument` event for format-on-type.
 * Target: under 16 ms total round-trip for a 2000-line file with a 1-char edit.
 *
 * Protocol:
 * 1. Receives edit delta from VS Code.
 * 2. Converts VS Code UTF-16 positions to UTF-8 byte offsets (L-14).
 * 3. Sends FormatOnTypeRequest to the worker pool.
 * 4. Applies the returned TextEdits to the document.
 *
 * Error handling:
 * - On-type errors are logged at warn level (not silently swallowed) so
 *   they appear in the output channel for diagnostics.
 * - `previousTrees` is evicted when a document is closed to prevent
 *   unbounded memory growth.
 * - WASM response shapes are validated at runtime to catch unexpected
 *   changes in the WASM API.
 */

import * as vscode from "vscode";
import { WorkerPool } from "./workerPool";
import { StatusBar } from "./statusBar";
import { toUtf8ByteOffset, toUtf16CodeUnitOffset } from "./offsets";
import { logger } from "./logger";

const log = logger.withContext("OnTypeHandler");

// ── Runtime types for WASM response ──────────────────────────────────────

interface WasmEdit {
  range:    { start: number; end: number };
  new_text: string;
}

interface WasmFormatResponse {
  edits:           WasmEdit[];
  formatter_chain: string;
  is_noop:         boolean;
  next_tree?:      number[];
  error?:          string | { message: string };
}

function isWasmFormatResponse(value: unknown): value is WasmFormatResponse {
  if (typeof value !== "object" || value === null) { return false; }
  const v = value as Record<string, unknown>;
  return Array.isArray(v["edits"]) && typeof v["is_noop"] === "boolean";
}

function isWasmEdit(value: unknown): value is WasmEdit {
  if (typeof value !== "object" || value === null) { return false; }
  const v = value as Record<string, unknown>;
  return (
    typeof v["new_text"] === "string" &&
    typeof v["range"]    === "object" && v["range"] !== null &&
    typeof (v["range"] as Record<string, unknown>)["start"] === "number" &&
    typeof (v["range"] as Record<string, unknown>)["end"]   === "number"
  );
}

// ── OnTypeHandler ─────────────────────────────────────────────────────────

export class OnTypeHandler implements vscode.Disposable {
  private readonly pool:        WorkerPool;
  private readonly statusBar:   StatusBar;
  private readonly disposables: vscode.Disposable[] = [];

  /** The serialised previous Tree-sitter tree, keyed by document URI string. */
  private readonly previousTrees = new Map<string, Uint8Array>();

  constructor(pool: WorkerPool, statusBar: StatusBar) {
    this.pool      = pool;
    this.statusBar = statusBar;
  }

  // ── Registration ─────────────────────────────────────────────────────────

  /**
   * Register the format-on-type handler for all supported language IDs.
   *
   * Also registers a `onDidCloseTextDocument` listener to evict stale
   * tree caches.
   *
   * @param supportedLanguageIds Language IDs to enable format-on-type for.
   * @param context Extension context for subscription management.
   */
  register(
    supportedLanguageIds: string[],
    context: vscode.ExtensionContext
  ): void {
    for (const langId of supportedLanguageIds) {
      const disposable = vscode.languages.registerOnTypeFormattingEditProvider(
        { language: langId },
        {
          provideOnTypeFormattingEdits: (
            document: vscode.TextDocument,
            position: vscode.Position,
            _ch:      string,
            _options: vscode.FormattingOptions,
            token:    vscode.CancellationToken
          ) => this.handleOnType(document, position, token),
        },
        // Trigger characters: space, semicolon, closing brace
        " ", ";", "}"
      );
      this.disposables.push(disposable);
    }

    // Evict tree caches when documents are closed to prevent memory leak.
    this.disposables.push(
      vscode.workspace.onDidCloseTextDocument((doc) => {
        const key = doc.uri.toString();
        if (this.previousTrees.delete(key)) {
          log.debug("Evicted tree cache for closed document", { uri: key });
        }
      })
    );

    context.subscriptions.push(...this.disposables);
    log.info("On-type handler registered", { languages: supportedLanguageIds.length });
  }

  // ── Core handler ─────────────────────────────────────────────────────────

  private async handleOnType(
    document: vscode.TextDocument,
    position: vscode.Position,
    token:    vscode.CancellationToken
  ): Promise<vscode.TextEdit[]> {
    const sourceText  = document.getText();
    const sourceBytes = Buffer.from(sourceText, "utf8");

    // Convert the cursor position to a UTF-8 byte offset (L-14 mitigation)
    const editOffset    = toUtf8ByteOffset(sourceText, document.offsetAt(position));
    const previousTree  = this.previousTrees.get(document.uri.toString());

    const request = {
      source:       Array.from(sourceBytes),
      language_id:  document.languageId,
      config:       {},
      range:        null,
      previous_tree: previousTree ? Array.from(previousTree) : null,
      edit: {
        start:    editOffset,
        deleted:  0,
        inserted: [],
      },
    };

    let requestJson: string;
    try {
      requestJson = JSON.stringify(request);
    } catch (err) {
      log.error("Failed to serialise on-type request", err instanceof Error ? err : new Error(String(err)), {
        languageId: document.languageId,
      });
      return [];
    }

    const startMs = performance.now();
    let responseJson: string;
    try {
      responseJson = await this.pool.dispatch(requestJson, token);
    } catch (err) {
      // Cancellation is normal and expected — don't log it as an error.
      const isCancelled =
        err instanceof Error && (err.message.includes("cancelled") || err.message.includes("CANCELLED"));

      if (!isCancelled) {
        log.warn("On-type dispatch failed", {
          languageId: document.languageId,
          error:      err instanceof Error ? err.message : String(err),
        });
      }
      return [];
    }

    const elapsedMs = performance.now() - startMs;

    // Log if we exceed the 16 ms target
    if (elapsedMs > 16) {
      log.warn("On-type format exceeded 16 ms target", {
        elapsedMs:  elapsedMs.toFixed(1),
        languageId: document.languageId,
        lines:      document.lineCount,
      });
    }

    // ── Parse and validate response ────────────────────────────────────
    let response: unknown;
    try {
      response = JSON.parse(responseJson);
    } catch (err) {
      log.error("On-type: WASM returned non-JSON response", err instanceof Error ? err : new Error(String(err)), {
        responsePreview: responseJson.slice(0, 200),
      });
      return [];
    }

    if (!isWasmFormatResponse(response)) {
      log.error(
        "On-type: WASM response did not match expected shape",
        new Error("Unexpected WASM response shape"),
        { responsePreview: JSON.stringify(response).slice(0, 200) }
      );
      return [];
    }

    if (response.error) {
      const errorMsg = typeof response.error === "string"
        ? response.error
        : response.error.message;
      log.warn("On-type: WASM returned an error", { error: errorMsg, languageId: document.languageId });
      return [];
    }

    if (response.is_noop || response.edits.length === 0) {
      return [];
    }

    // Update the cached tree for the next incremental request
    if (Array.isArray(response.next_tree)) {
      this.previousTrees.set(
        document.uri.toString(),
        new Uint8Array(response.next_tree)
      );
    }

    this.statusBar.update(
      document.languageId,
      response.formatter_chain ?? "",
      Math.round(elapsedMs)
    );

    // ── Convert edits, validating each one ───────────────────────────
    const textEdits: vscode.TextEdit[] = [];
    for (const edit of response.edits) {
      if (!isWasmEdit(edit)) {
        log.warn("On-type: skipping malformed edit in WASM response", {
          edit: JSON.stringify(edit).slice(0, 200),
        });
        continue;
      }
      const startUtf16 = toUtf16CodeUnitOffset(sourceText, edit.range.start);
      const endUtf16   = toUtf16CodeUnitOffset(sourceText, edit.range.end);
      const startPos   = document.positionAt(startUtf16);
      const endPos     = document.positionAt(endUtf16);
      textEdits.push(vscode.TextEdit.replace(new vscode.Range(startPos, endPos), edit.new_text));
    }

    return textEdits;
  }

  // ── Disposal ──────────────────────────────────────────────────────────────

  dispose(): void {
    for (const d of this.disposables) {
      d.dispose();
    }
    this.disposables.length = 0;
    this.previousTrees.clear();
  }
}
