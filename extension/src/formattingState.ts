import * as vscode from "vscode";

export interface FormattingResult {
  formatterChain: string;
  elapsedMs: number;
  timestamp: number;
}

export class FormattingState {
  private static instance: FormattingState;
  private state: Map<string, FormattingResult> = new Map();
  private _onDidChange = new vscode.EventEmitter<vscode.Uri>();
  public readonly onDidChange = this._onDidChange.event;

  private constructor() {}

  public static getInstance(): FormattingState {
    if (!FormattingState.instance) {
      FormattingState.instance = new FormattingState();
    }
    return FormattingState.instance;
  }

  public updateState(uri: vscode.Uri, result: FormattingResult): void {
    this.state.set(uri.toString(), result);
    this._onDidChange.fire(uri);
  }

  public getState(uri: vscode.Uri): FormattingResult | undefined {
    return this.state.get(uri.toString());
  }
}
