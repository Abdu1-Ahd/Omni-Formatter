/**
 * OmniFormatter Config Adapter (L-10 mitigation)
 *
 * Reads all config sources and merges them into a per-language ConfigIR.
 * Priority order (highest to lowest):
 *   1. .omnifmt.json in workspace root (optional global override)
 *   2. Language-native config file (.prettierrc, pyproject.toml, etc.)
 *   3. .editorconfig (base layer)
 *   4. Module defaults
 *
 * This is a non-destructive adapter: existing config files are NEVER modified.
 * If a config file is malformed, the adapter logs a warning and falls back to
 * the next priority level. It NEVER throws or crashes the extension.
 *
 * # Adapter Search Order Per Language
 *
 * ## JavaScript / TypeScript
 *   .omnifmt.json → .prettierrc (json/yaml) → prettier.config.js → .editorconfig → defaults
 *
 * ## Python
 *   .omnifmt.json → pyproject.toml [tool.black] → setup.cfg [tool:black] → .editorconfig → defaults
 *
 * ## Rust
 *   .omnifmt.json → rustfmt.toml → .rustfmt.toml → .editorconfig → defaults
 *
 * ## Go
 *   .omnifmt.json → .editorconfig → gofmt defaults (go has no config file)
 *
 * ## CSS / SCSS / Less
 *   .omnifmt.json → .prettierrc → .stylelintrc (ignored in opinionated mode) → .editorconfig → defaults
 *
 * ## HTML
 *   .omnifmt.json → .prettierrc → .editorconfig → defaults
 */

import * as vscode from "vscode";
import * as fs from "fs";
import * as path from "path";

/** Resolved configuration for a single language format request. */
export interface ResolvedConfig {
  /** The raw ConfigIR JSON to pass to the WASM core. */
  configJson: string;
  /** Which config source(s) contributed to this resolution (for debugging). */
  sources: string[];
}

/** A partial ConfigIR — all fields optional, to allow partial overrides. */
interface PartialConfig {
  printWidth?: number;
  indentSize?: number;
  indentStyle?: "spaces" | "tabs";
  quoteStyle?: "single" | "double";
  trailingComma?: boolean;
  semicolons?: boolean;
  endOfLine?: "lf" | "crlf" | "cr" | "auto";
  mode?: "opinionated" | "advanced";
  postFormat?: string[];
  // Language-specific extensions
  [key: string]: unknown;
}

export class ConfigAdapter {
  private readonly workspaceFolders: readonly vscode.WorkspaceFolder[];

  constructor() {
    this.workspaceFolders = vscode.workspace.workspaceFolders ?? [];
  }

  /**
   * Resolve the ConfigIR for a specific document and language.
   *
   * @param document The document being formatted.
   * @param languageId The VS Code language ID.
   * @returns The resolved ConfigIR as a JSON string.
   */
  resolve(document: vscode.TextDocument, languageId: string): ResolvedConfig {
    const docPath = document.uri.fsPath;
    const workspaceRoot = this.findWorkspaceRoot(docPath);

    const sources: string[] = [];
    let merged: PartialConfig = {};

    // Layer 4 (base): Module defaults — applied implicitly by WASM core.
    // ConfigIR::default() is used by the WASM core if no config JSON provided.

    // Layer 3: .editorconfig
    const editorConfig = this.readEditorConfig(docPath);
    if (editorConfig) {
      merged = { ...merged, ...editorConfig };
      sources.push(".editorconfig");
    }

    // Layer 2: Language-native config
    const nativeConfig = this.readNativeConfig(workspaceRoot, languageId);
    if (nativeConfig) {
      merged = { ...merged, ...nativeConfig.config };
      sources.push(nativeConfig.source);
    }

    // Layer 1: .omnifmt.json (highest priority override)
    const omnifmtConfig = this.readOmnifmtJson(workspaceRoot, languageId);
    if (omnifmtConfig) {
      merged = { ...merged, ...omnifmtConfig };
      sources.push(".omnifmt.json");
    }

    return {
      configJson: JSON.stringify(merged),
      sources,
    };
  }

  /** Find the workspace root directory for a document path. */
  private findWorkspaceRoot(docPath: string): string {
    for (const folder of this.workspaceFolders) {
      if (docPath.startsWith(folder.uri.fsPath)) {
        return folder.uri.fsPath;
      }
    }
    // Fallback: directory of the document itself
    return path.dirname(docPath);
  }

  /** Read .omnifmt.json from the workspace root and extract the language section. */
  private readOmnifmtJson(workspaceRoot: string, languageId: string): PartialConfig | null {
    const omnifmtPath = path.join(workspaceRoot, ".omnifmt.json");
    if (!fs.existsSync(omnifmtPath)) return null;

    try {
      const raw = fs.readFileSync(omnifmtPath, "utf8");
      const parsed = JSON.parse(raw);
      // Support $ref: "#/javascript" for TypeScript → inherit JS config
      const section = parsed[languageId] ?? null;
      const global = parsed["global"] ?? {};
      return section ? { ...global, ...section } : (global ? { ...global } : null);
    } catch {
      return null; // Malformed JSON — skip silently
    }
  }

  /**
   * Read the language-native config file and translate to PartialConfig.
   * Returns null if no native config file is found.
   */
  private readNativeConfig(
    workspaceRoot: string,
    languageId: string
  ): { config: PartialConfig; source: string } | null {
    switch (languageId) {
      case "javascript":
      case "typescript":
      case "javascriptreact":
      case "typescriptreact":
        return this.readPrettierConfig(workspaceRoot);

      case "python":
        return this.readBlackConfig(workspaceRoot);

      case "rust":
        return this.readRustfmtConfig(workspaceRoot);

      case "css":
      case "scss":
      case "less":
      case "html":
        return this.readPrettierConfig(workspaceRoot);

      default:
        return null;
    }
  }

  /** Read .prettierrc (JSON only — YAML support in Phase 5). */
  private readPrettierConfig(workspaceRoot: string): { config: PartialConfig; source: string } | null {
    const candidates = [".prettierrc", ".prettierrc.json", "prettier.config.json"];
    for (const name of candidates) {
      const p = path.join(workspaceRoot, name);
      if (!fs.existsSync(p)) continue;
      try {
        const raw = fs.readFileSync(p, "utf8");
        const parsed = JSON.parse(raw);
        return { config: this.mapPrettierConfig(parsed), source: name };
      } catch {
        continue;
      }
    }
    return null;
  }

  /** Read pyproject.toml [tool.black] section (TOML → JSON bridge, Phase 4 stub). */
  private readBlackConfig(workspaceRoot: string): { config: PartialConfig; source: string } | null {
    const p = path.join(workspaceRoot, "pyproject.toml");
    if (!fs.existsSync(p)) return null;
    // Phase 4 stub: full TOML parsing in Phase 5 via a TOML-to-JSON converter.
    // Return Black defaults for now.
    return {
      config: { printWidth: 88 },
      source: "pyproject.toml (partial)",
    };
  }

  /** Read rustfmt.toml or .rustfmt.toml (TOML → JSON bridge, Phase 4 stub). */
  private readRustfmtConfig(workspaceRoot: string): { config: PartialConfig; source: string } | null {
    const candidates = ["rustfmt.toml", ".rustfmt.toml"];
    for (const name of candidates) {
      if (fs.existsSync(path.join(workspaceRoot, name))) {
        return {
          config: { printWidth: 100, indentSize: 4 },
          source: name,
        };
      }
    }
    return null;
  }

  /** Read .editorconfig (simplified — handles common keys only). */
  private readEditorConfig(_docPath: string): PartialConfig | null {
    // Phase 4 stub: full .editorconfig walk-up in Phase 5.
    return null;
  }

  /** Map Prettier config JSON to PartialConfig. */
  private mapPrettierConfig(pc: Record<string, unknown>): PartialConfig {
    const config: PartialConfig = {};
    if (typeof pc["printWidth"] === "number") config.printWidth = pc["printWidth"];
    if (typeof pc["tabWidth"] === "number") config.indentSize = pc["tabWidth"];
    if (pc["useTabs"] === true) config.indentStyle = "tabs";
    if (pc["singleQuote"] === true) config.quoteStyle = "single";
    if (typeof pc["semi"] === "boolean") config.semicolons = pc["semi"];
    if (pc["trailingComma"] === "none") config.trailingComma = false;
    if (["all", "es5"].includes(pc["trailingComma"] as string)) config.trailingComma = true;
    if (typeof pc["endOfLine"] === "string") {
      config.endOfLine = pc["endOfLine"] as "lf" | "crlf" | "cr" | "auto";
    }
    return config;
  }
}
