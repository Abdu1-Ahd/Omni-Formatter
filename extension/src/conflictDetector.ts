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
import { logger } from "./logger";

const log = logger.withContext("ConflictDetector");

// ── Known conflicts ───────────────────────────────────────────────────────

interface ConflictingExtension {
  id:        string;
  name:      string;
  languages: string[];
}

const KNOWN_CONFLICTING_EXTENSIONS: ConflictingExtension[] = [
  {
    id:        "esbenp.prettier-vscode",
    name:      "Prettier",
    languages: ["javascript", "typescript", "css", "html"],
  },
  {
    id:        "ms-python.black-formatter",
    name:      "Black Formatter",
    languages: ["python"],
  },
  {
    id:        "rust-lang.rust-analyzer",
    name:      "rust-analyzer (formatter)",
    languages: ["rust"],
  },
  {
    id:        "golang.go",
    name:      "Go (gofmt)",
    languages: ["go"],
  },
];

// ── ConflictDetector ──────────────────────────────────────────────────────

export class ConflictDetector {
  /**
   * Detect conflicting formatter extensions and show a one-time notification
   * if any are found.
   *
   * @param supportedLanguageIds The language IDs OmniFormatter handles.
   */
  detectAndNotify(supportedLanguageIds: string[]): void {
    const conflicts = this.findConflicts(supportedLanguageIds);

    if (conflicts.length === 0) {
      log.debug("No conflicting formatters detected.");
      return;
    }

    log.info("Conflicting formatters detected", {
      conflicts: conflicts.map((c) => c.id),
    });

    // Only show the notification once per VS Code installation.
    let alreadyShown = false;
    try {
      alreadyShown = vscode.workspace
        .getConfiguration("omniFormatter")
        .get<boolean>("_conflictNotificationShown", false);
    } catch (err) {
      log.warn("Could not read _conflictNotificationShown setting", {
        error: err instanceof Error ? err.message : String(err),
      });
    }

    if (alreadyShown) { return; }

    const names   = conflicts.map((c) => c.name).join(", ");
    const message =
      `OmniFormatter: Conflicting formatters detected — ${names}. ` +
      `Disable them to avoid unpredictable formatting order.`;

    vscode.window
      .showWarningMessage(message, "Disable Conflicting Formatters", "Ignore")
      .then(
        (choice) => {
          if (choice === "Disable Conflicting Formatters") {
            this.disableConflicts(conflicts);
          }
        },
        (err) => {
          // The notification was dismissed programmatically or VS Code shut down.
          log.debug("Conflict notification dismissed", {
            error: err instanceof Error ? err.message : String(err),
          });
        }
      );

    // Mark as shown regardless of the user's choice.
    vscode.workspace
      .getConfiguration("omniFormatter")
      .update("_conflictNotificationShown", true, vscode.ConfigurationTarget.Global)
      .then(
        () => { /* success — no action needed */ },
        (err) => {
          log.warn("Could not persist _conflictNotificationShown setting", {
            error: err instanceof Error ? err.message : String(err),
          });
        }
      );
  }

  // ── Private ───────────────────────────────────────────────────────────────

  /** Find which known conflicting extensions are currently installed and enabled. */
  private findConflicts(supportedLanguageIds: string[]): ConflictingExtension[] {
    return KNOWN_CONFLICTING_EXTENSIONS.filter((conflict) => {
      const ext = vscode.extensions.getExtension(conflict.id);
      if (!ext) { return false; }
      // Only flag as a conflict if it handles languages OmniFormatter supports.
      return conflict.languages.some((lang) => supportedLanguageIds.includes(lang));
    });
  }

  /**
   * Open the Extensions panel and show an informational message to guide
   * the user through manually disabling the conflicting extensions.
   */
  private disableConflicts(conflicts: ConflictingExtension[]): void {
    vscode.commands
      .executeCommand("workbench.extensions.action.showInstalledExtensions")
      .then(
        () => { /* panel opened */ },
        (err) => {
          log.warn("Could not open extensions panel", {
            error: err instanceof Error ? err.message : String(err),
          });
        }
      );

    vscode.window.showInformationMessage(
      `Disable the following extensions to prevent formatting conflicts: ` +
      `${conflicts.map((c) => c.name).join(", ")}. ` +
      `Alternatively, set "editor.defaultFormatter": "Abdu1-Ahd.omni-formatter" in settings.json.`
    );
  }
}
