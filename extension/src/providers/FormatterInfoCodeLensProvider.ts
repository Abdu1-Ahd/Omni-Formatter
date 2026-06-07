import * as vscode from "vscode";
import { FormattingState } from "../formattingState";

export class FormatterInfoCodeLensProvider implements vscode.CodeLensProvider {
  private _onDidChangeCodeLenses = new vscode.EventEmitter<void>();
  public readonly onDidChangeCodeLenses = this._onDidChangeCodeLenses.event;

  constructor() {
    FormattingState.getInstance().onDidChange(() => {
      this._onDidChangeCodeLenses.fire();
    });
  }

  public provideCodeLenses(document: vscode.TextDocument): vscode.CodeLens[] {
    const state = FormattingState.getInstance().getState(document.uri);
    const range = new vscode.Range(0, 0, 0, 0);
    const title = state
      ? `⬡ OmniFormatter: Formatted (${state.elapsedMs}ms)`
      : `⬡ OmniFormatter: Ready`;

    return [
      new vscode.CodeLens(range, {
        title,
        command: "omniFormatter.showStatus",
        arguments: [],
      }),
    ];
  }
}
