/**
 * Format-on-Type Request Handler (L-13 mitigation)
 *
 * Handles VS Code's `onDidChangeTextDocument` event for format-on-type.
 * Target: under 16ms total round-trip for a 2000-line file with a 1-char edit.
 *
 * Protocol:
 * 1. Receives edit delta from VS Code.
 * 2. Converts VS Code UTF-16 positions to UTF-8 byte offsets (L-14).
 * 3. Sends FormatOnTypeRequest to the worker pool.
 * 4. Applies the returned TextEdits to the document.
 *
 * The WASM core's incremental protocol (crates/core/src/incremental.rs) ensures
 * only the dirty region is re-parsed and re-formatted, not the full file.
 */

import * as vscode from "vscode";
import { WorkerPool } from "./workerPool";
import { StatusBar } from "./statusBar";
import { toUtf8ByteOffset } from "./offsets";

export class OnTypeHandler implements vscode.Disposable {
  private readonly pool: WorkerPool;
  private readonly statusBar: StatusBar;
  private readonly disposables: vscode.Disposable[] = [];

  /** The serialised previous Tree-sitter tree, keyed by document URI. */
  private readonly previousTrees = new Map<string, Uint8Array>();

  constructor(pool: WorkerPool, statusBar: StatusBar) {
    this.pool = pool;
    this.statusBar = statusBar;
  }

  /**
   * Register the format-on-type handler for all supported language IDs.
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
          provideOnTypeFormattingEdits: async (
            document,
            position,
            _ch,
            _options,
            token
          ) => this.handleOnType(document, position, token),
        },
        // Trigger characters: space, semicolon, closing brace
        " ", ";", "}"
      );
      this.disposables.push(disposable);
    }

    context.subscriptions.push(...this.disposables);
  }

  private async handleOnType(
    document: vscode.TextDocument,
    position: vscode.Position,
    token: vscode.CancellationToken
  ): Promise<vscode.TextEdit[]> {
    const sourceText = document.getText();
    const sourceBytes = Buffer.from(sourceText, "utf8");

    // Convert the position to a UTF-8 byte offset (L-14 mitigation)
    const editOffset = toUtf8ByteOffset(sourceText, document.offsetAt(position));

    const previousTree = this.previousTrees.get(document.uri.toString());

    const request = {
      source: Array.from(sourceBytes),
      language_id: document.languageId,
      config: {},
      range: null,
      previous_tree: previousTree ? Array.from(previousTree) : null,
      edit: {
        start: editOffset,
        deleted: 0,
        inserted: [],
      },
    };

    try {
      const startMs = performance.now();
      const responseJson = await this.pool.dispatch(JSON.stringify(request), token);
      const elapsedMs = performance.now() - startMs;

      // Log if we exceed the 16ms target
      if (elapsedMs > 16) {
        console.warn(
          `[OmniFormatter] format-on-type exceeded 16ms target: ${elapsedMs.toFixed(1)}ms`
        );
      }

      const response = JSON.parse(responseJson);
      if (response.error || response.is_noop) return [];

      // Update the cached tree for the next incremental request
      if (response.next_tree) {
        this.previousTrees.set(
          document.uri.toString(),
          new Uint8Array(response.next_tree)
        );
      }

      this.statusBar.update(document.languageId, response.formatter_chain ?? "", Math.round(elapsedMs));
      return []; // Phase 3 stub: edits returned in Phase 4
    } catch {
      return [];
    }
  }

  dispose(): void {
    this.disposables.forEach((d) => d.dispose());
    this.previousTrees.clear();
  }
}
