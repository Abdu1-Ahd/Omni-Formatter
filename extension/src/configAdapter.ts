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
import { resolveEditorConfigCached, clearEditorConfigCache, EditorConfigResult } from "./editorConfig";
import { logger } from "./logger";

const log = logger.withContext("ConfigAdapter");

// ── Types ─────────────────────────────────────────────────────────────────

/** Resolved configuration for a single language format request. */
export interface ResolvedConfig {
  /** The raw ConfigIR JSON to pass to the WASM core. */
  configJson: string;
  /** Which config source(s) contributed to this resolution (for debugging). */
  sources: string[];
}

/** A partial ConfigIR — all fields optional, to allow partial overrides. */
interface PartialConfig {
  printWidth?:   number;
  indentSize?:   number;
  indentStyle?:  "spaces" | "tabs";
  quoteStyle?:   "single" | "double";
  trailingComma?: boolean;
  semicolons?:   boolean;
  endOfLine?:    "lf" | "crlf" | "cr" | "auto";
  mode?:         "opinionated" | "advanced";
  postFormat?:   string[];
  [key: string]: unknown;
}

// ── ConfigAdapter ─────────────────────────────────────────────────────────

export class ConfigAdapter {
  private readonly workspaceFolders: readonly vscode.WorkspaceFolder[];

  constructor() {
    this.workspaceFolders = vscode.workspace.workspaceFolders ?? [];
  }

  /**
   * Resolve the ConfigIR for a specific document and language.
   *
   * Never throws — all errors are logged and the next config layer is used.
   *
   * @param document   The document being formatted.
   * @param languageId The VS Code language ID.
   * @returns The resolved ConfigIR as a JSON string plus the list of sources used.
   */
  resolve(document: vscode.TextDocument, languageId: string): ResolvedConfig {
    const docPath      = document.uri.fsPath;
    const workspaceRoot = this.findWorkspaceRoot(docPath);

    const sources: string[] = [];
    let merged: PartialConfig = {};

    // Layer 3: .editorconfig (base)
    const editorConfig = this.readEditorConfig(docPath);
    if (editorConfig) {
      merged = { ...merged, ...editorConfig };
      sources.push(".editorconfig");
    }

    // Layer 2: language-native config
    const nativeConfig = this.readNativeConfig(workspaceRoot, languageId);
    if (nativeConfig) {
      merged = { ...merged, ...nativeConfig.config };
      sources.push(nativeConfig.source);
    }

    // Layer 1: .omnifmt.json (highest priority)
    const omnifmtConfig = this.readOmnifmtJson(workspaceRoot, languageId);
    if (omnifmtConfig) {
      merged = { ...merged, ...omnifmtConfig };
      sources.push(".omnifmt.json");
    }

    let configJson = "{}";
    try {
      configJson = JSON.stringify(merged);
    } catch (err) {
      log.error("Failed to serialise resolved config", err instanceof Error ? err : new Error(String(err)), {
        languageId,
        docPath,
      });
    }

    log.debug("Config resolved", { languageId, sources });
    return { configJson, sources };
  }

  // ── Workspace root ────────────────────────────────────────────────────────

  private findWorkspaceRoot(docPath: string): string {
    for (const folder of this.workspaceFolders) {
      if (docPath.startsWith(folder.uri.fsPath)) {
        return folder.uri.fsPath;
      }
    }
    // Fallback: directory of the document itself
    return path.dirname(docPath);
  }

  // ── .omnifmt.json ─────────────────────────────────────────────────────────

  private readOmnifmtJson(workspaceRoot: string, languageId: string): PartialConfig | null {
    const omnifmtPath = path.join(workspaceRoot, ".omnifmt.json");
    if (!fs.existsSync(omnifmtPath)) { return null; }

    let raw: string;
    try {
      raw = fs.readFileSync(omnifmtPath, "utf8");
    } catch (err) {
      log.warn("Could not read .omnifmt.json", {
        path:  omnifmtPath,
        error: err instanceof Error ? err.message : String(err),
      });
      return null;
    }

    let parsed: Record<string, unknown>;
    try {
      parsed = JSON.parse(raw) as Record<string, unknown>;
    } catch {
      // Malformed JSON — notify the user so they can fix it.
      log.warn(".omnifmt.json is malformed JSON — skipping (fix syntax errors to use it)", {
        path: omnifmtPath,
      });
      void vscode.window.showWarningMessage(
        `OmniFormatter: .omnifmt.json is not valid JSON and will be ignored. ` +
        `Check the file for syntax errors: ${omnifmtPath}`
      );
      return null;
    }

    const section = typeof parsed[languageId] === "object" && parsed[languageId] !== null
      ? (parsed[languageId] as Record<string, unknown>)
      : null;
    const global  = typeof parsed["global"] === "object" && parsed["global"] !== null
      ? (parsed["global"] as Record<string, unknown>)
      : {};

    const merged = section ? { ...global, ...section } : { ...global };
    return Object.keys(merged).length > 0 ? (merged as PartialConfig) : null;
  }

  // ── Language-native config ────────────────────────────────────────────────

  private readNativeConfig(
    workspaceRoot: string,
    languageId:    string
  ): { config: PartialConfig; source: string } | null {
    switch (languageId) {
      case "javascript":
      case "typescript":
      case "javascriptreact":
      case "typescriptreact":
      case "css":
      case "scss":
      case "less":
      case "html":
        return this.readPrettierConfig(workspaceRoot);

      case "python":
        return this.readBlackConfig(workspaceRoot);

      case "rust":
        return this.readRustfmtConfig(workspaceRoot);

      default:
        return null;
    }
  }

  // ── Prettier config ───────────────────────────────────────────────────────

  private readPrettierConfig(workspaceRoot: string): { config: PartialConfig; source: string } | null {
    const jsonCandidates = [".prettierrc", ".prettierrc.json", "prettier.config.json", ".prettierrc.json5"];
    for (const name of jsonCandidates) {
      const p = path.join(workspaceRoot, name);
      if (!fs.existsSync(p)) { continue; }
      try {
        const raw      = fs.readFileSync(p, "utf8");
        // Strip JSON5 / JSONC single-line and block comments before parsing
        const stripped = raw
          .replace(/\/\/[^\n]*/g, "")
          .replace(/\/\*[\s\S]*?\*\//g, "");
        const parsed   = JSON.parse(stripped) as Record<string, unknown>;
        return { config: this.mapPrettierConfig(parsed), source: name };
      } catch (err) {
        log.warn(`Could not parse Prettier config file "${name}"`, {
          path:  p,
          error: err instanceof Error ? err.message : String(err),
        });
        continue;
      }
    }

    // YAML fallback: .prettierrc.yaml / .prettierrc.yml
    const yamlCandidates = [".prettierrc.yaml", ".prettierrc.yml"];
    for (const name of yamlCandidates) {
      const p = path.join(workspaceRoot, name);
      if (!fs.existsSync(p)) { continue; }
      try {
        const raw    = fs.readFileSync(p, "utf8");
        const parsed = this.parseSimpleYaml(raw);
        return { config: this.mapPrettierConfig(parsed), source: name };
      } catch (err) {
        log.warn(`Could not parse YAML Prettier config file "${name}"`, {
          path:  p,
          error: err instanceof Error ? err.message : String(err),
        });
        continue;
      }
    }

    return null;
  }

  /**
   * Minimal YAML parser for flat key: value documents (no nesting, no anchors).
   * Covers the full surface of .prettierrc.yaml in practice.
   */
  private parseSimpleYaml(content: string): Record<string, unknown> {
    const result: Record<string, unknown> = {};
    for (const line of content.split(/\r?\n/)) {
      const trimmed = line.trim();
      if (!trimmed || trimmed.startsWith("#")) { continue; }
      const colonIdx = trimmed.indexOf(":");
      if (colonIdx === -1) { continue; }
      const key    = trimmed.slice(0, colonIdx).trim();
      const rawVal = trimmed.slice(colonIdx + 1).trim();
      if (!key) { continue; }
      if (rawVal === "true")  { result[key] = true;  continue; }
      if (rawVal === "false") { result[key] = false; continue; }
      const num = Number(rawVal);
      if (!isNaN(num) && rawVal !== "") { result[key] = num; continue; }
      // Strip surrounding quotes
      result[key] = rawVal.replace(/^["']|["']$/g, "");
    }
    return result;
  }

  // ── Black (Python) config ─────────────────────────────────────────────────

  private readBlackConfig(workspaceRoot: string): { config: PartialConfig; source: string } | null {
    const p = path.join(workspaceRoot, "pyproject.toml");
    if (!fs.existsSync(p)) { return null; }

    let raw: string;
    try {
      raw = fs.readFileSync(p, "utf8");
    } catch (err) {
      log.warn("Could not read pyproject.toml", {
        path:  p,
        error: err instanceof Error ? err.message : String(err),
      });
      return null;
    }

    try {
      const config = this.extractBlackSection(raw);
      return { config, source: "pyproject.toml [tool.black]" };
    } catch (err) {
      log.warn("Could not parse [tool.black] from pyproject.toml — using Black defaults", {
        path:  p,
        error: err instanceof Error ? err.message : String(err),
      });
      return { config: { printWidth: 88 }, source: "pyproject.toml (error fallback)" };
    }
  }

  private extractBlackSection(toml: string): PartialConfig {
    const config: PartialConfig = { printWidth: 88 };
    const sectionMatch = toml.match(/\[tool\.black\]([\s\S]*?)(?=\n\[|$)/);
    if (!sectionMatch) { return config; }

    const section = sectionMatch[1];
    for (const line of section.split(/\r?\n/)) {
      const m = line.match(/^([\w-]+)\s*=\s*(.+)$/);
      if (!m) { continue; }
      const [, key, val] = m;
      const trimVal = val.trim().replace(/^["']|["']$/g, "");
      switch (key) {
        case "line-length": {
          const n = parseInt(trimVal, 10);
          if (!isNaN(n) && n > 0) { config.printWidth = n; }
          break;
        }
        case "skip-string-normalization":
          if (trimVal === "true") { config.quoteStyle = "single"; }
          break;
        case "skip-magic-trailing-comma":
          if (trimVal === "true") { config.trailingComma = false; }
          break;
      }
    }
    return config;
  }

  // ── Rustfmt config ────────────────────────────────────────────────────────

  private readRustfmtConfig(workspaceRoot: string): { config: PartialConfig; source: string } | null {
    const candidates = ["rustfmt.toml", ".rustfmt.toml"];
    for (const name of candidates) {
      const p = path.join(workspaceRoot, name);
      if (!fs.existsSync(p)) { continue; }

      let raw: string;
      try {
        raw = fs.readFileSync(p, "utf8");
      } catch (err) {
        log.warn(`Could not read rustfmt config "${name}"`, {
          path:  p,
          error: err instanceof Error ? err.message : String(err),
        });
        continue;
      }

      try {
        const config = this.extractRustfmtOptions(raw);
        return { config, source: name };
      } catch (err) {
        log.warn(`Could not parse rustfmt config "${name}" — using defaults`, {
          path:  p,
          error: err instanceof Error ? err.message : String(err),
        });
        return { config: { printWidth: 100, indentSize: 4 }, source: `${name} (error fallback)` };
      }
    }
    return null;
  }

  private extractRustfmtOptions(toml: string): PartialConfig {
    const config: PartialConfig = { printWidth: 100, indentSize: 4 };
    for (const line of toml.split(/\r?\n/)) {
      const m = line.match(/^([\w_]+)\s*=\s*(.+)$/);
      if (!m) { continue; }
      const [, key, val] = m;
      const trimVal = val.trim().replace(/^["']|["']$/g, "");
      switch (key) {
        case "max_width": {
          const n = parseInt(trimVal, 10);
          if (!isNaN(n) && n > 0) { config.printWidth = n; }
          break;
        }
        case "tab_spaces": {
          const n = parseInt(trimVal, 10);
          if (!isNaN(n) && n > 0) { config.indentSize = n; }
          break;
        }
        case "hard_tabs":
          if (trimVal === "true") { config.indentStyle = "tabs"; }
          break;
        case "newline_style":
          if (trimVal.toLowerCase() === "windows") { config.endOfLine = "crlf"; }
          else if (trimVal.toLowerCase() === "unix") { config.endOfLine = "lf"; }
          break;
        case "trailing_comma":
          config.trailingComma = !trimVal.toLowerCase().includes("never");
          break;
      }
    }
    return config;
  }

  // ── EditorConfig ──────────────────────────────────────────────────────────

  private readEditorConfig(docPath: string): PartialConfig | null {
    let result: EditorConfigResult | null;
    try {
      result = resolveEditorConfigCached(docPath);
    } catch (err) {
      log.warn("EditorConfig resolution threw unexpectedly", {
        docPath,
        error: err instanceof Error ? err.message : String(err),
      });
      return null;
    }

    if (!result) { return null; }

    const partial: PartialConfig = {};
    if (result.indentStyle) { partial.indentStyle = result.indentStyle; }
    if (result.indentSize)  { partial.indentSize  = result.indentSize;  }
    if (result.endOfLine)   { partial.endOfLine   = result.endOfLine;   }
    if (result.printWidth)  { partial.printWidth  = result.printWidth;  }
    return partial;
  }

  /** Clear the editorconfig cache (call on workspace folder changes). */
  clearCache(): void {
    clearEditorConfigCache();
    log.debug("EditorConfig cache cleared.");
  }

  // ── Prettier config mapping ───────────────────────────────────────────────

  private mapPrettierConfig(pc: Record<string, unknown>): PartialConfig {
    const config: PartialConfig = {};
    if (typeof pc["printWidth"] === "number") { config.printWidth = pc["printWidth"]; }
    if (typeof pc["tabWidth"]   === "number") { config.indentSize = pc["tabWidth"];   }
    if (pc["useTabs"]    === true)            { config.indentStyle   = "tabs";         }
    if (pc["singleQuote"] === true)           { config.quoteStyle    = "single";       }
    if (typeof pc["semi"] === "boolean")      { config.semicolons    = pc["semi"];     }
    if (pc["trailingComma"] === "none")       { config.trailingComma = false;          }
    if (["all", "es5"].includes(pc["trailingComma"] as string)) {
      config.trailingComma = true;
    }
    if (typeof pc["endOfLine"] === "string") {
      const eol = pc["endOfLine"] as string;
      if (["lf", "crlf", "cr", "auto"].includes(eol)) {
        config.endOfLine = eol as "lf" | "crlf" | "cr" | "auto";
      }
    }
    return config;
  }
}
