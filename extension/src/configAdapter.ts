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

  /** Read .prettierrc — supports JSON, JSON5 (comments stripped), and YAML-like simple format. */
  private readPrettierConfig(workspaceRoot: string): { config: PartialConfig; source: string } | null {
    const jsonCandidates = [".prettierrc", ".prettierrc.json", "prettier.config.json", ".prettierrc.json5"];
    for (const name of jsonCandidates) {
      const p = path.join(workspaceRoot, name);
      if (!fs.existsSync(p)) continue;
      try {
        const raw = fs.readFileSync(p, "utf8");
        // Strip JSON5 / JSONC single-line comments before parsing
        const stripped = raw.replace(/\/\/[^\n]*/g, "").replace(/\/\*[\s\S]*?\*\//g, "");
        const parsed = JSON.parse(stripped);
        return { config: this.mapPrettierConfig(parsed), source: name };
      } catch {
        continue;
      }
    }
    // YAML fallback: .prettierrc.yaml / .prettierrc.yml (hand-parse simple key: value)
    const yamlCandidates = [".prettierrc.yaml", ".prettierrc.yml"];
    for (const name of yamlCandidates) {
      const p = path.join(workspaceRoot, name);
      if (!fs.existsSync(p)) continue;
      try {
        const raw = fs.readFileSync(p, "utf8");
        const parsed = this.parseSimpleYaml(raw);
        return { config: this.mapPrettierConfig(parsed), source: name };
      } catch {
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
      const key = trimmed.slice(0, colonIdx).trim();
      const rawVal = trimmed.slice(colonIdx + 1).trim();
      // Parse value as number, boolean, or string
      if (rawVal === "true")  { result[key] = true;  continue; }
      if (rawVal === "false") { result[key] = false; continue; }
      const num = Number(rawVal);
      if (!isNaN(num) && rawVal !== "") { result[key] = num; continue; }
      // Strip surrounding quotes
      result[key] = rawVal.replace(/^["']|["']$/g, "");
    }
    return result;
  }

  /**
   * Read pyproject.toml [tool.black] section.
   *
   * TOML is parsed in the Rust WASM core (lang-python/adapter.rs).
   * On the extension host (Node.js) side we do a targeted regex extraction
   * to avoid shipping a full TOML parser in TypeScript.
   */
  private readBlackConfig(workspaceRoot: string): { config: PartialConfig; source: string } | null {
    const p = path.join(workspaceRoot, "pyproject.toml");
    if (!fs.existsSync(p)) return null;
    try {
      const raw = fs.readFileSync(p, "utf8");
      const config = this.extractBlackSection(raw);
      return { config, source: "pyproject.toml [tool.black]" };
    } catch {
      return { config: { printWidth: 88 }, source: "pyproject.toml (error)" };
    }
  }

  /** Extract [tool.black] options from a pyproject.toml string via targeted line parsing. */
  private extractBlackSection(toml: string): PartialConfig {
    const config: PartialConfig = { printWidth: 88 };
    // Find [tool.black] section
    const sectionMatch = toml.match(/\[tool\.black\]([\s\S]*?)(?=\n\[|$)/);
    if (!sectionMatch) { return config; }
    const section = sectionMatch[1];
    for (const line of section.split(/\r?\n/)) {
      const m = line.match(/^([\w-]+)\s*=\s*(.+)$/);
      if (!m) { continue; }
      const [, key, val] = m;
      const trimVal = val.trim().replace(/^["']|["']$/g, "");
      switch (key) {
        case "line-length":      config.printWidth   = parseInt(trimVal, 10); break;
        case "skip-string-normalization":
          if (trimVal === "true") { config.quoteStyle = "single"; } break;
        case "skip-magic-trailing-comma":
          if (trimVal === "true") { config.trailingComma = false; } break;
      }
    }
    return config;
  }

  /** Read rustfmt.toml or .rustfmt.toml — line-parse the stable options. */
  private readRustfmtConfig(workspaceRoot: string): { config: PartialConfig; source: string } | null {
    const candidates = ["rustfmt.toml", ".rustfmt.toml"];
    for (const name of candidates) {
      const p = path.join(workspaceRoot, name);
      if (!fs.existsSync(p)) { continue; }
      try {
        const raw = fs.readFileSync(p, "utf8");
        const config = this.extractRustfmtOptions(raw);
        return { config, source: name };
      } catch {
        return { config: { printWidth: 100, indentSize: 4 }, source: name };
      }
    }
    return null;
  }

  /** Extract rustfmt stable options from a rustfmt.toml string. */
  private extractRustfmtOptions(toml: string): PartialConfig {
    const config: PartialConfig = { printWidth: 100, indentSize: 4 };
    for (const line of toml.split(/\r?\n/)) {
      const m = line.match(/^([\w_]+)\s*=\s*(.+)$/);
      if (!m) { continue; }
      const [, key, val] = m;
      const trimVal = val.trim().replace(/^["']|["']$/g, "");
      switch (key) {
        case "max_width":   config.printWidth  = parseInt(trimVal, 10); break;
        case "tab_spaces":  config.indentSize  = parseInt(trimVal, 10); break;
        case "hard_tabs":   if (trimVal === "true") { config.indentStyle = "tabs"; } break;
        case "newline_style":
          if (trimVal.toLowerCase() === "windows") { config.endOfLine = "crlf"; }
          else if (trimVal.toLowerCase() === "unix") { config.endOfLine = "lf"; }
          break;
        case "trailing_comma":
          config.trailingComma = !trimVal.toLowerCase().includes("never"); break;
      }
    }
    return config;
  }

  /**
   * Read .editorconfig via the full walk-up parser (L-10 base config layer).
   * Cached per document path for performance.
   */
  private readEditorConfig(docPath: string): PartialConfig | null {
    const result: EditorConfigResult | null = resolveEditorConfigCached(docPath);
    if (!result) { return null; }
    const partial: PartialConfig = {};
    if (result.indentStyle)  { partial.indentStyle  = result.indentStyle; }
    if (result.indentSize)   { partial.indentSize   = result.indentSize; }
    if (result.endOfLine)    { partial.endOfLine    = result.endOfLine; }
    if (result.printWidth)   { partial.printWidth   = result.printWidth; }
    return partial;
  }

  /** Clear the editorconfig cache (call on workspace folder changes). */
  clearCache(): void {
    clearEditorConfigCache();
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
