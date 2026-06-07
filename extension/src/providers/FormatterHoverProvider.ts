import * as vscode from "vscode";
import { FormattingState } from "../formattingState";

export class FormatterHoverProvider implements vscode.HoverProvider {
  public provideHover(
    document: vscode.TextDocument,
    position: vscode.Position
  ): vscode.ProviderResult<vscode.Hover> {
    if (position.line !== 0) {
      return null;
    }

    const state = FormattingState.getInstance().getState(document.uri);
    if (!state) {
      return null;
    }

    const md = new vscode.MarkdownString(undefined, true);
    md.isTrusted = true;
    md.appendMarkdown(`**⬡ OmniFormatter** — last format details\n\n`);
    md.appendMarkdown(`| | |\n|---|---|\n`);
    md.appendMarkdown(`| **Parser Chain** | \`${state.formatterChain}\` |\n`);
    md.appendMarkdown(`| **Time** | \`${state.elapsedMs}ms\` |\n`);
    md.appendMarkdown(`| **At** | ${new Date(state.timestamp).toLocaleTimeString()} |\n`);
    md.appendMarkdown(`\n---\n`);
    md.appendMarkdown(`[⚙️ Open Dashboard](command:omniFormatter.openDashboard "Open OmniFormatter Dashboard")`);

    return new vscode.Hover(md);
  }
}
