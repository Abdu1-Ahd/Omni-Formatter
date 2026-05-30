/**
 * EditorConfig Walk-Up Parser (L-10 mitigation — base config layer)
 *
 * Implements the full EditorConfig specification:
 *   - Walks up the directory tree from the document path until it finds a
 *     file with `root = true` or reaches the filesystem root.
 *   - Collects all `.editorconfig` files in the chain (child → root order).
 *   - Merges sections that glob-match the document path (child wins on conflict).
 *   - Translates EditorConfig keys to OmniFormatter's `PartialConfig`.
 *
 * # Spec Reference
 *   https://editorconfig.org/
 *
 * # Supported Keys
 *
 * | EditorConfig key         | PartialConfig field |
 * |--------------------------|---------------------|
 * | indent_style             | indentStyle         |
 * | indent_size / tab_width  | indentSize          |
 * | end_of_line              | endOfLine           |
 * | max_line_length          | printWidth          |
 * | insert_final_newline     | (validated)         |
 * | trim_trailing_whitespace | (validated)         |
 *
 * # Non-Destructive
 *
 * This module NEVER writes to `.editorconfig` files.
 * It only reads and translates. All reads are cached per document path.
 */

import * as fs from "fs";
import * as path from "path";

/** A PartialConfig that maps EditorConfig options to OmniFormatter config. */
export interface EditorConfigResult {
  indentStyle?: "spaces" | "tabs";
  indentSize?: number;
  endOfLine?: "lf" | "crlf" | "cr" | "auto";
  printWidth?: number;
  insertFinalNewline?: boolean;
  trimTrailingWhitespace?: boolean;
}

// ── Glob matching ─────────────────────────────────────────────────────────

/**
 * Convert an EditorConfig glob pattern to a RegExp.
 *
 * Supported EditorConfig glob syntax:
 *  - `*`      — any string not containing `/`
 *  - `**`     — any string including `/`
 *  - `?`      — any single character not containing `/`
 *  - `[seq]`  — character class
 *  - `{a,b}`  — alternation
 */
function globToRegex(pattern: string): RegExp {
  let regex = "";
  let i = 0;

  while (i < pattern.length) {
    const ch = pattern[i];

    if (ch === "*") {
      if (pattern[i + 1] === "*") {
        regex += ".*";
        i += 2;
        // Skip optional trailing slash
        if (pattern[i] === "/") { regex += "\\/?"; i++; }
      } else {
        regex += "[^/]*";
        i++;
      }
    } else if (ch === "?") {
      regex += "[^/]";
      i++;
    } else if (ch === "{") {
      // Alternation group: {a,b,c} → (a|b|c)
      const end = pattern.indexOf("}", i);
      if (end === -1) {
        regex += "\\{";
        i++;
      } else {
        const alternatives = pattern
          .slice(i + 1, end)
          .split(",")
          .map(escapeRegex)
          .join("|");
        regex += `(?:${alternatives})`;
        i = end + 1;
      }
    } else if (ch === "[") {
      // Character class — pass through verbatim
      const end = pattern.indexOf("]", i);
      if (end === -1) {
        regex += "\\[";
        i++;
      } else {
        regex += pattern.slice(i, end + 1);
        i = end + 1;
      }
    } else {
      regex += escapeRegex(ch);
      i++;
    }
  }

  return new RegExp(`^${regex}$`, "i");
}

function escapeRegex(s: string): string {
  return s.replace(/[.+^${}()|[\]\\]/g, "\\$&");
}

// ── EditorConfig file parser ──────────────────────────────────────────────

interface Section {
  /** The raw glob pattern from the section header (e.g. `*.{js,ts}`). */
  pattern: string;
  /** Properties in this section. */
  properties: Record<string, string>;
}

interface EditorConfigFile {
  /** True if this file has `root = true`. */
  isRoot: boolean;
  /** Parsed sections in order. */
  sections: Section[];
}

/** Parse a single `.editorconfig` file from its raw text content. */
function parseEditorConfigFile(content: string): EditorConfigFile {
  const lines = content.split(/\r?\n/);
  const sections: Section[] = [];
  let isRoot = false;
  let currentSection: Section | null = null;

  for (const rawLine of lines) {
    const line = rawLine.trim();

    // Skip blank lines and comments
    if (!line || line.startsWith("#") || line.startsWith(";")) {
      continue;
    }

    // Section header [pattern]
    if (line.startsWith("[") && line.endsWith("]")) {
      currentSection = { pattern: line.slice(1, -1).trim(), properties: {} };
      sections.push(currentSection);
      continue;
    }

    // Key = value pair
    const eqIdx = line.indexOf("=");
    if (eqIdx === -1) { continue; }

    const key   = line.slice(0, eqIdx).trim().toLowerCase();
    const value = line.slice(eqIdx + 1).trim().toLowerCase();

    if (currentSection === null) {
      // Top-level (preamble) — only `root` is valid here
      if (key === "root" && value === "true") {
        isRoot = true;
      }
    } else {
      currentSection.properties[key] = value;
    }
  }

  return { isRoot, sections };
}

// ── Walk-up and resolution ────────────────────────────────────────────────

/** Absolute paths to `.editorconfig` files in ancestor order (child first). */
function collectEditorConfigPaths(docPath: string): string[] {
  const collected: string[] = [];
  let dir = path.dirname(docPath);
  const MAX_DEPTH = 32; // Safety limit
  let depth = 0;

  while (depth < MAX_DEPTH) {
    const ecPath = path.join(dir, ".editorconfig");
    if (fs.existsSync(ecPath)) {
      collected.push(ecPath);
      try {
        const content = fs.readFileSync(ecPath, "utf8");
        const parsed = parseEditorConfigFile(content);
        if (parsed.isRoot) { break; }
      } catch {
        break;
      }
    }
    const parent = path.dirname(dir);
    if (parent === dir) { break; } // Filesystem root
    dir = parent;
    depth++;
  }

  return collected;
}

/**
 * Resolve the effective EditorConfig properties for `docPath`.
 *
 * Files closer to the document (lower in the tree) take precedence.
 * Within a file, later matching sections override earlier ones.
 */
export function resolveEditorConfig(docPath: string): EditorConfigResult | null {
  const ecPaths = collectEditorConfigPaths(docPath);
  if (ecPaths.length === 0) { return null; }

  // Normalise the doc path to forward slashes for glob matching
  const normalised = docPath.replace(/\\/g, "/");

  // Merge from root outward (root files first, then child wins)
  const reversedPaths = [...ecPaths].reverse();
  const merged: Record<string, string> = {};

  for (const ecPath of reversedPaths) {
    let content: string;
    try {
      content = fs.readFileSync(ecPath, "utf8");
    } catch {
      continue;
    }

    const parsed = parseEditorConfigFile(content);
    const ecDir  = path.dirname(ecPath).replace(/\\/g, "/");

    for (const section of parsed.sections) {
      // Build the absolute glob pattern for matching
      const absPattern = section.pattern.includes("/")
        ? `${ecDir}/${section.pattern.replace(/^\//, "")}`
        : `**/${section.pattern}`;

      const regex = globToRegex(absPattern);
      if (regex.test(normalised)) {
        Object.assign(merged, section.properties);
      }
    }
  }

  if (Object.keys(merged).length === 0) { return null; }

  return translateProperties(merged);
}

// ── Property translation ──────────────────────────────────────────────────

/** Translate raw EditorConfig key-value pairs to EditorConfigResult. */
function translateProperties(props: Record<string, string>): EditorConfigResult {
  const result: EditorConfigResult = {};

  // indent_style
  if (props["indent_style"] === "tab") {
    result.indentStyle = "tabs";
  } else if (props["indent_style"] === "space") {
    result.indentStyle = "spaces";
  }

  // indent_size / tab_width
  const indentSizeRaw = props["indent_size"] ?? props["tab_width"];
  if (indentSizeRaw && indentSizeRaw !== "tab") {
    const n = parseInt(indentSizeRaw, 10);
    if (!isNaN(n) && n > 0) { result.indentSize = n; }
  }

  // end_of_line
  if (props["end_of_line"]) {
    const eol = props["end_of_line"].toLowerCase();
    if (eol === "lf")   { result.endOfLine = "lf"; }
    if (eol === "crlf") { result.endOfLine = "crlf"; }
    if (eol === "cr")   { result.endOfLine = "cr"; }
  }

  // max_line_length (maps to printWidth)
  if (props["max_line_length"] && props["max_line_length"] !== "off") {
    const n = parseInt(props["max_line_length"], 10);
    if (!isNaN(n) && n > 0) { result.printWidth = n; }
  }

  // insert_final_newline
  if (props["insert_final_newline"] === "true")  { result.insertFinalNewline = true; }
  if (props["insert_final_newline"] === "false") { result.insertFinalNewline = false; }

  // trim_trailing_whitespace
  if (props["trim_trailing_whitespace"] === "true")  { result.trimTrailingWhitespace = true; }
  if (props["trim_trailing_whitespace"] === "false") { result.trimTrailingWhitespace = false; }

  return result;
}

// ── Cache ─────────────────────────────────────────────────────────────────

const cache = new Map<string, EditorConfigResult | null>();

/**
 * Cached version of `resolveEditorConfig`.
 *
 * Cache is keyed by document path. Call `clearCache()` if the workspace
 * changes (e.g. workspace folder added/removed).
 */
export function resolveEditorConfigCached(docPath: string): EditorConfigResult | null {
  if (cache.has(docPath)) {
    return cache.get(docPath) ?? null;
  }
  const result = resolveEditorConfig(docPath);
  cache.set(docPath, result);
  return result;
}

/** Clear the entire EditorConfig resolution cache. */
export function clearEditorConfigCache(): void {
  cache.clear();
}
