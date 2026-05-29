/**
 * Status Bar Item — shows the active formatter chain per file (L-11 mitigation).
 *
 * Always displays what ran and in what order, e.g.:
 *   OmniFmt: JS → ESLint fix
 *   OmniFmt: Python (Black 24.x)
 *   OmniFmt: TS [16ms]
 */

import * as vscode from "vscode";

export class StatusBar implements vscode.Disposable {
  private readonly item: vscode.StatusBarItem;

  constructor() {
    this.item = vscode.window.createStatusBarItem(
      "omniFormatter.status",
      vscode.StatusBarAlignment.Right,
      // Priority: right side, above most other items
      100
    );
    this.item.name = "OmniFormatter";
    this.item.command = "omniFormatter.showStatus";
    this.item.tooltip = "Click to open OmniFormatter output channel";
    this.item.text = "$(zap) OmniFmt";
    this.item.show();
  }

  /**
   * Update the status bar after a successful format operation.
   *
   * @param languageId The VS Code language ID of the formatted document.
   * @param formatterChain Human-readable description from FormatResponse.formatter_chain.
   * @param elapsedMs Time taken for the full round-trip in milliseconds.
   */
  update(languageId: string, formatterChain: string, elapsedMs: number): void {
    const langLabel = this.shortLanguageLabel(languageId);
    const timing = elapsedMs < 50 ? "" : ` [${elapsedMs}ms]`;
    this.item.text = `$(zap) OmniFmt: ${langLabel}${timing}`;
    this.item.tooltip = `Last format: ${formatterChain} (${elapsedMs}ms)\nClick to open output channel`;
    this.item.backgroundColor = undefined; // Clear any error state
  }

  /**
   * Show an error state in the status bar.
   *
   * @param message Short error description.
   */
  showError(message: string): void {
    this.item.text = `$(error) OmniFmt: Error`;
    this.item.tooltip = `OmniFormatter error: ${message}\nClick to open output channel`;
    this.item.backgroundColor = new vscode.ThemeColor("statusBarItem.errorBackground");
  }

  /** Reset to idle state. */
  reset(): void {
    this.item.text = "$(zap) OmniFmt";
    this.item.tooltip = "OmniFormatter — Click to open output channel";
    this.item.backgroundColor = undefined;
  }

  dispose(): void {
    this.item.dispose();
  }

  /** Map a verbose languageId to a compact display label. */
  private shortLanguageLabel(languageId: string): string {
    const labels: Record<string, string> = {
      javascript: "JS",
      typescript: "TS",
      javascriptreact: "JSX",
      typescriptreact: "TSX",
      python: "Python",
      rust: "Rust",
      go: "Go",
      css: "CSS",
      scss: "SCSS",
      less: "Less",
      html: "HTML",
      svelte: "Svelte",
      vue: "Vue",
      astro: "Astro",
    };
    return labels[languageId] ?? languageId;
  }
}
