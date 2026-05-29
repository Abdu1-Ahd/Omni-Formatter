/**
 * Post-Format Chain Runner (L-11 mitigation)
 *
 * After OmniFormatter runs its primary language module, additional tools
 * (e.g., ESLint fix, import sorting) can be configured to run in sequence.
 *
 * Configuration (in .omnifmt.json):
 * {
 *   "postFormat": ["eslint-fix", "import-sort"]
 * }
 *
 * Each chain step receives the formatted output of the previous step.
 * The status bar displays the full chain that ran, e.g.:
 *   OmniFmt: TS → ESLint fix
 */

import * as vscode from "vscode";

/** A registered post-format chain step. */
interface ChainStep {
  id: string;
  /** Display name shown in the status bar chain. */
  displayName: string;
  /** Function that applies the step to the current document. */
  run: (document: vscode.TextDocument) => Promise<void>;
}

/** Registry of known post-format chain steps. */
const REGISTERED_STEPS: Map<string, ChainStep> = new Map([
  [
    "eslint-fix",
    {
      id: "eslint-fix",
      displayName: "ESLint fix",
      run: async (document) => {
        // Trigger ESLint's fix-all command if the extension is installed
        try {
          await vscode.commands.executeCommand(
            "eslint.executeAutofix",
            document.uri
          );
        } catch {
          // ESLint extension not installed or not applicable — skip silently
        }
      },
    },
  ],
  [
    "import-sort",
    {
      id: "import-sort",
      displayName: "import sort",
      run: async (document) => {
        try {
          await vscode.commands.executeCommand(
            "editor.action.organizeImports",
            document.uri
          );
        } catch {
          // Not supported for this language — skip silently
        }
      },
    },
  ],
]);

export class Chain {
  /**
   * Run the post-format chain for the given document.
   *
   * @param document The document that was just formatted.
   * @param stepIds The list of chain step IDs from ConfigIR.post_format.
   * @returns A human-readable description of what ran, e.g. "ESLint fix → import sort".
   */
  async run(document: vscode.TextDocument, stepIds: string[]): Promise<string> {
    const ran: string[] = [];

    for (const stepId of stepIds) {
      const step = REGISTERED_STEPS.get(stepId);
      if (!step) {
        // Unknown chain step — skip and log
        console.warn(`[OmniFormatter] Unknown post-format chain step: "${stepId}"`);
        continue;
      }

      try {
        await step.run(document);
        ran.push(step.displayName);
      } catch (err) {
        // Chain step failed — log but do not abort the chain
        console.error(`[OmniFormatter] Chain step "${stepId}" failed: ${err}`);
      }
    }

    return ran.length > 0 ? ran.join(" → ") : "";
  }

  /**
   * Build the full formatter chain string for the status bar.
   *
   * @param primaryFormatter e.g. "lang-js 0.1.0 (Prettier 3.x compat)"
   * @param chainDescription e.g. "ESLint fix → import sort"
   * @returns e.g. "JS → ESLint fix → import sort"
   */
  static buildChainLabel(
    languageLabel: string,
    chainDescription: string
  ): string {
    if (!chainDescription) return languageLabel;
    return `${languageLabel} → ${chainDescription}`;
  }
}
