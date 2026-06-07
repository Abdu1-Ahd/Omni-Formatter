/**
 * Conflict Detector (L-11 mitigation)
 *
 * Scans installed VS Code extensions for competing DocumentFormattingEditProvider
 * registrations. On first activation, shows a one-time notification listing
 * conflicts with a "Disable conflicting formatters" button.
 *
 * Known conflicting extensions (by extension ID):
 * - esbenp.prettier-vscode
 * - ms-python.black-formatter
 * - rust-lang.rust-analyzer (formatter)
 * - golang.go (formatter)
 * - stylelint.vscode-stylelint
 */

import * as vscode from "vscode";

/** Extension IDs that register DocumentFormattingEditProvider. */
const KNOWN_CONFLICTING_EXTENSIONS: Array<{
  id: string;
  name: string;
  languages: string[];
}> = [
  {
    id: "esbenp.prettier-vscode",
    name: "Prettier",
    languages: ["javascript", "typescript", "css", "html"],
  },
  {
    id: "ms-python.black-formatter",
    name: "Black Formatter",
    languages: ["python"],
  },
  {
    id: "rust-lang.rust-analyzer",
    name: "rust-analyzer (formatter)",
    languages: ["rust"],
  },
  {
    id: "golang.go",
    name: "Go (gofmt)",
    languages: ["go"],
  },
];


export class ConflictDetector {
  /**
   * Detect conflicting formatter extensions and show a notification if any found.
   * Uses a state key so the notification only shows once per installation.
   *
   * @param supportedLanguageIds The language IDs OmniFormatter handles.
   */
  detectAndNotify(supportedLanguageIds: string[]): void {
    const conflicts = this.findConflicts(supportedLanguageIds);

    if (conflicts.length === 0) return;

    // Only show once per installation
    const state = vscode.workspace
      .getConfiguration("omniFormatter")
      .get<boolean>("_conflictNotificationShown", false);

    if (state) return;

    const names = conflicts.map((c) => c.name).join(", ");
    const message = `OmniFormatter: Conflicting formatters detected — ${names}. Disable them to avoid unpredictable formatting order.`;

    vscode.window
      .showWarningMessage(
        message,
        "Disable Conflicting Formatters",
        "Ignore"
      )
      .then((choice) => {
        if (choice === "Disable Conflicting Formatters") {
          this.disableConflicts(conflicts);
        }
      });

    // Mark as shown
    vscode.workspace
      .getConfiguration("omniFormatter")
      .update("_conflictNotificationShown", true, vscode.ConfigurationTarget.Global);
  }

  /** Find which known conflicting extensions are currently installed and enabled. */
  private findConflicts(
    supportedLanguageIds: string[]
  ): typeof KNOWN_CONFLICTING_EXTENSIONS {
    return KNOWN_CONFLICTING_EXTENSIONS.filter((conflict) => {
      const ext = vscode.extensions.getExtension(conflict.id);
      if (!ext) return false;
      // Only flag as a conflict if it handles languages OmniFormatter supports
      return conflict.languages.some((lang) => supportedLanguageIds.includes(lang));
    });
  }

  /** Open VS Code settings to help the user disable conflicting formatters. */
  private disableConflicts(
    conflicts: typeof KNOWN_CONFLICTING_EXTENSIONS
  ): void {
    // Open the Extensions panel so the user can disable manually
    vscode.commands.executeCommand("workbench.extensions.action.showInstalledExtensions");
    vscode.window.showInformationMessage(
      `Disable the following extensions to prevent formatting conflicts: ${conflicts.map((c) => c.name).join(", ")}. ` +
      `Alternatively, set "editor.defaultFormatter": "Abdu1-Ahd.omni-formatter" in settings.json.`
    );
  }
}
