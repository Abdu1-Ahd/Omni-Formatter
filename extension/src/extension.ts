/**
 * OmniFormatter VS Code Extension Entry Point
 *
 * Activation: onStartupFinished — runs after VS Code has fully started,
 * preventing any startup latency impact (L-03 mitigation).
 *
 * Responsibilities:
 * - Registers the single DocumentFormattingEditProvider for all language IDs.
 * - Initialises the WorkerPool.
 * - Initiates background WASM compilation for top-5 languages (L-03).
 * - Detects conflicting formatter extensions (L-11).
 * - Shows the status bar item (L-11 status communication).
 */

import * as vscode from "vscode";
import * as path from "path";
import * as os from "os";

import { WorkerPool } from "./workerPool";
import { StatusBar } from "./statusBar";
import { ModuleLoader } from "./moduleLoader";
import { WasmCompiler } from "./wasmCompiler";
import { DashboardPanel } from "./webview/DashboardPanel";
import { ConflictDetector } from "./conflictDetector";
import { FormattingState } from "./formattingState";
import { FormatterInfoCodeLensProvider } from "./providers/FormatterInfoCodeLensProvider";
import { FormatterHoverProvider } from "./providers/FormatterHoverProvider";


/** VS Code language IDs that OmniFormatter handles. */
const SUPPORTED_LANGUAGE_IDS = [
  // ── Frontend & Web ────────────────────────────────────────────────────────
  "javascript",
  "typescript",
  "javascriptreact",
  "typescriptreact",
  "css",
  "scss",
  "sass",
  "less",
  "html",
  "svelte",
  "vue",
  "astro",
  // ── Systems & Compiled ───────────────────────────────────────────────────
  "c",
  "cpp",
  "objective-c",
  "objective-cpp",
  "cuda-cpp",
  "rust",
  "go",
  "zig",
  "nim",
  "d",
  // ── JVM & .NET ───────────────────────────────────────────────────────────
  "java",
  "kotlin",
  "scala",
  "groovy",
  "csharp",
  "fsharp",
  // ── Scripting & General Purpose ──────────────────────────────────────────
  "python",
  "ruby",
  "php",
  "perl",
  "r",
  "julia",
  "lua",
  // ── Shell & Automation ───────────────────────────────────────────────────
  "shellscript",
  "powershell",
  "zsh",
  // ── Mobile Development ───────────────────────────────────────────────────
  "swift",
  "objective-c",     // handled in lang-swift
  "dart",
  // ── Data Serialization & Config ──────────────────────────────────────────
  "json",
  "json5",
  "yaml",
  "toml",
  "xml",
  "ini",
  // ── Query Languages ───────────────────────────────────────────────────────
  "sql",
  "graphql",
  // ── DevOps & Systems Config ──────────────────────────────────────────────
  "terraform",
  "dockerfile",
  "makefile",
  "nix",
  // ── Functional Languages ─────────────────────────────────────────────────
  "haskell",
  "elixir",
  "erlang",
  "ocaml",
  "clojure",
  "lisp",
  "scheme",
  // ── Documentation ────────────────────────────────────────────────────────
  "markdown",
  "latex",
  // ── Blockchain ───────────────────────────────────────────────────────────
  "solidity",
  // ── Game Scripting ───────────────────────────────────────────────────────
  "gdscript",
  // ── Embedded & Automation ────────────────────────────────────────────────
  "ahk",
  // ── Stubs (no grammar — identity pass-through) ───────────────────────────
  "cobol",
  "fortran",
  "asm",
  // ── Template Languages ───────────────────────────────────────────────────
  "jinja",
  "liquid",
  "ejs",
  "handlebars",
  "twig",
] as const;

/** Map from languageId to the module name that handles it. */
const LANGUAGE_MODULE_MAP: Record<string, string> = {
  // lang-js
  javascript:       "lang-js",
  typescript:       "lang-js",
  javascriptreact:  "lang-js",
  typescriptreact:  "lang-js",
  svelte:           "lang-js",
  vue:              "lang-js",
  astro:            "lang-js",
  // lang-css
  css:              "lang-css",
  scss:             "lang-css",
  less:             "lang-css",
  html:             "lang-css",
  // lang-sass
  sass:             "lang-sass",
  // lang-python
  python:           "lang-python",
  // lang-rust
  rust:             "lang-rust",
  // lang-go
  go:               "lang-go",
  // lang-c
  c:                "lang-c",
  cpp:              "lang-c",
  "objective-c":    "lang-c",
  "objective-cpp":  "lang-c",
  "cuda-cpp":       "lang-c",
  // lang-java
  java:             "lang-java",
  kotlin:           "lang-java",
  scala:            "lang-java",
  groovy:           "lang-java",
  // lang-csharp
  csharp:           "lang-csharp",
  fsharp:           "lang-csharp",
  // lang-ruby
  ruby:             "lang-ruby",
  php:              "lang-ruby",
  perl:             "lang-ruby",
  lua:              "lang-ruby",
  // lang-shell
  shellscript:      "lang-shell",
  powershell:       "lang-shell",
  zsh:              "lang-shell",
  // lang-swift
  swift:            "lang-swift",
  // lang-mobile
  dart:             "lang-mobile",
  // lang-data
  json:             "lang-data",
  json5:            "lang-data",
  yaml:             "lang-data",
  toml:             "lang-data",
  xml:              "lang-data",
  ini:              "lang-data",
  // lang-sql
  sql:              "lang-sql",
  graphql:          "lang-sql",
  // lang-devops
  terraform:        "lang-devops",
  dockerfile:       "lang-devops",
  makefile:         "lang-devops",
  nix:              "lang-devops",
  // lang-functional
  haskell:          "lang-functional",
  elixir:           "lang-functional",
  erlang:           "lang-functional",
  ocaml:            "lang-functional",
  clojure:          "lang-functional",
  lisp:             "lang-functional",
  scheme:           "lang-functional",
  r:                "lang-functional",
  julia:            "lang-functional",
  // lang-markdown
  markdown:         "lang-markdown",
  latex:            "lang-markdown",
  // lang-modern
  zig:              "lang-modern",
  nim:              "lang-modern",
  d:                "lang-modern",
  // lang-other
  solidity:         "lang-other",
  gdscript:         "lang-other",
  ahk:              "lang-other",
  cobol:            "lang-other",
  fortran:          "lang-other",
  asm:              "lang-other",
  // lang-template
  jinja:            "lang-template",
  liquid:           "lang-template",
  ejs:              "lang-template",
  handlebars:       "lang-template",
  twig:             "lang-template",
};

/** Extension-level globals — initialised in activate(), torn down in deactivate(). */
let workerPool: WorkerPool | undefined;
let statusBar: StatusBar | undefined;
let outputChannel: vscode.OutputChannel | undefined;

/**
 * Extension activation function.
 *
 * Called once when the extension activates (onStartupFinished).
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
  outputChannel = vscode.window.createOutputChannel("OmniFormatter");
  context.subscriptions.push(outputChannel);

  log("OmniFormatter activating…");

  // Initialise status bar
  statusBar = new StatusBar();
  context.subscriptions.push(statusBar);

  // Initialise worker pool
  const workerScript = path.join(context.extensionPath, "dist", "workers", "formatWorker.js");
  const wasmDir = path.join(context.extensionPath, "dist", "wasm");
  const numWorkers = Math.max(2, os.cpus().length - 1);

  workerPool = new WorkerPool(workerScript, wasmDir, numWorkers);
  await workerPool.initialise();
  log(`Worker pool started with ${numWorkers} workers.`);

  // Initialise module loader
  const moduleLoader = new ModuleLoader(
    context.globalStorageUri.fsPath,
    wasmDir
  );

  // Background WASM compilation for top languages (L-03 mitigation)
  const wasmCompiler = new WasmCompiler(wasmDir, context.globalStorageUri.fsPath);
  wasmCompiler.precompileTopLanguages(["lang-js", "lang-python", "lang-rust", "lang-css"]);

  // Detect conflicting formatters (L-11 mitigation)
  const conflictDetector = new ConflictDetector();
  conflictDetector.detectAndNotify(SUPPORTED_LANGUAGE_IDS as unknown as string[]);

  // Register DocumentFormattingEditProvider for all supported languages
  for (const langId of SUPPORTED_LANGUAGE_IDS) {
    const provider = vscode.languages.registerDocumentFormattingEditProvider(
      { language: langId },
      {
        provideDocumentFormattingEdits: async (
          document: vscode.TextDocument,
          options: vscode.FormattingOptions,
          token: vscode.CancellationToken
        ): Promise<vscode.TextEdit[]> => {
          return handleFormatRequest(
            document,
            options,
            token,
            moduleLoader,
            context
          );
        },
      }
    );
    context.subscriptions.push(provider);
  }

  // Register CodeLens and Hover providers for premium UX
  const codeLensProvider = new FormatterInfoCodeLensProvider();
  const hoverProvider = new FormatterHoverProvider();
  for (const langId of SUPPORTED_LANGUAGE_IDS) {
    context.subscriptions.push(
      vscode.languages.registerCodeLensProvider({ language: langId }, codeLensProvider),
      vscode.languages.registerHoverProvider({ language: langId }, hoverProvider)
    );
  }

  // Format on save — triggered whenever a supported file is about to be saved.
  // This fires independently of editor.formatOnSave so users don't need to
  // configure that setting manually.
  context.subscriptions.push(
    vscode.workspace.onWillSaveTextDocument((event) => {
      const { document } = event;
      if (!SUPPORTED_LANGUAGE_IDS.includes(document.languageId as any)) return;
      const config = vscode.workspace.getConfiguration("omniFormatter", document.uri);
      if (!config.get<boolean>("enable", true)) return;

      event.waitUntil(
        handleFormatRequest(document, { tabSize: 2, insertSpaces: true }, new vscode.CancellationTokenSource().token, moduleLoader, context)
      );
    })
  );

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand("omniFormatter.formatDocument", () => {
      vscode.commands.executeCommand("editor.action.formatDocument");
    }),
    vscode.commands.registerCommand("omniFormatter.showStatus", () => {
      outputChannel?.show();
    }),
    vscode.commands.registerCommand("omniFormatter.openDashboard", () => {
      DashboardPanel.createOrShow(context);
    }),
    vscode.commands.registerCommand("omnifmt.formatWorkspace", async () => {
      // Only format known source file types; exclude large non-source directories
      const INCLUDE_GLOB = '**/*.{js,mjs,cjs,ts,mts,cts,tsx,jsx,py,pyw,rs,go,css,scss,sass,less,html,htm,svelte,vue,astro,c,h,cpp,hpp,cc,cxx,hh,mm,m,java,kt,kts,scala,sc,groovy,cs,fs,fsi,fsx,rb,php,pl,pm,lua,sh,bash,zsh,ps1,psm1,swift,dart,json,json5,yaml,yml,toml,xml,ini,sql,graphql,gql,tf,hcl,Dockerfile,nix,hs,lhs,ex,exs,erl,hrl,ml,mli,clj,cljs,r,R,jl,md,markdown,tex,zig,nim,sol,gd,ahk,lisp,lsp,scm,ss,jinja,jinja2,liquid,ejs,hbs,handlebars,twig}';

      const EXCLUDE_GLOB = '**/{node_modules,.vscode-test,.vscode-test-user-data,.git,dist,out,target}/**';
      const uris = await vscode.workspace.findFiles(INCLUDE_GLOB, EXCLUDE_GLOB);
      log(`Formatting ${uris.length} files in workspace...`);
      for (const uri of uris) {
        try {
          const doc = await vscode.workspace.openTextDocument(uri);
          await vscode.window.showTextDocument(doc, { preview: false });
          await vscode.commands.executeCommand("editor.action.formatDocument");
          await doc.save();
        } catch (e) {
          log(`Failed to format ${uri.fsPath}: ${e}`);
        }
      }
    })
  );

  log("OmniFormatter activated.");
}

/**
 * Handle a format request for a single document.
 *
 * Converts VS Code UTF-16 positions to UTF-8 byte offsets,
 * dispatches to the worker pool, and converts the response
 * back to VS Code TextEdits.
 */
async function handleFormatRequest(
  document: vscode.TextDocument,
  _options: vscode.FormattingOptions,
  token: vscode.CancellationToken,
  _moduleLoader: ModuleLoader,
  _context: vscode.ExtensionContext
): Promise<vscode.TextEdit[]> {
  const langId = document.languageId;
  const moduleName = LANGUAGE_MODULE_MAP[langId];

  if (!moduleName) {
    log(`No module for language: ${langId}`);
    return [];
  }

  // ── Build format request (zero-copy path) ──────────────────────────
  // `source_text` sends the raw UTF-8 string directly into the JSON payload.
  // No Buffer conversion. No Array.from(). No base64. No size cap.
  // WASM deserialises it as a Rust String — one JSON parse, done.
  // File size is unlimited: the only bound is available memory.
  const byteEstimate = sourceText.length * 3; // worst-case UTF-8 bytes

  // Progress notification for files over 1 MB (purely cosmetic — the actual
  // IPC is fast; the WASM formatter is what takes time on huge files).
  let progressDispose: vscode.Disposable | undefined;
  if (byteEstimate > 1_048_576) {
    vscode.window.withProgress(
      {
        location: vscode.ProgressLocation.Window,
        title: `OmniFormatter: formatting ${langId} (≈${Math.round(byteEstimate / 1024)} KB)…`,
        cancellable: false,
      },
      () => new Promise<void>((res) => { progressDispose = { dispose: res }; })
    );
  }

  const request = {
    source_text:       sourceText,   // ← raw string, WASM reads as Rust String
    source_byte_length: byteEstimate, // ← worker uses this to size its timeout
    language_id: langId,
    config: {},
    range: null,
    previous_tree: null,
    edit: null,
  };

  try {
    const startMs = Date.now();
    const responseJson = await workerPool!.dispatch(JSON.stringify(request), token);
    const elapsedMs = Date.now() - startMs;

    // Dismiss progress notification if shown
    progressDispose?.dispose();

    const response = JSON.parse(responseJson) as {
      edits: Array<{ range: { start: number; end: number }; new_text: string }>;
      formatter_chain: string;
      is_noop: boolean;
      error?: unknown;
    };

    if (response.error) {
      log(`Format error: ${JSON.stringify(response.error)}`);
      return [];
    }

    statusBar?.update(langId, response.formatter_chain, elapsedMs);
    log(`Formatted ${langId} in ${elapsedMs}ms via ${response.formatter_chain}`);

    FormattingState.getInstance().updateState(document.uri, {
      formatterChain: response.formatter_chain,
      elapsedMs,
      timestamp: Date.now(),
    });

    if (response.is_noop || response.edits.length === 0) {
      return [];
    }

    // Convert UTF-8 byte offset TextEdits back to VS Code UTF-16 positions
    return response.edits.map((edit) => {
      const startPos = document.positionAt(
        utf8ByteOffsetToUtf16CodeUnit(sourceText, edit.range.start)
      );
      const endPos = document.positionAt(
        utf8ByteOffsetToUtf16CodeUnit(sourceText, edit.range.end)
      );
      return vscode.TextEdit.replace(
        new vscode.Range(startPos, endPos),
        edit.new_text
      );
    });
  } catch (err) {
    log(`Unexpected error: ${err}`);
    return [];
  }
}

/**
 * Convert a UTF-8 byte offset to a VS Code UTF-16 code unit offset.
 *
 * VS Code uses UTF-16 code unit offsets in all its position APIs.
 * WASM operates on UTF-8 bytes. This conversion runs on the extension
 * host boundary (L-14 mitigation).
 */
function utf8ByteOffsetToUtf16CodeUnit(text: string, byteOffset: number): number {
  const utf8 = Buffer.from(text, "utf8");
  const slice = utf8.slice(0, byteOffset);
  return slice.toString("utf8").length;
}

/** Write to the OmniFormatter output channel. */
function log(message: string): void {
  console.log(`[OmniFormatter] ${message}`);
  outputChannel?.appendLine(`[${new Date().toISOString()}] ${message}`);
}

/**
 * Extension deactivation function.
 * Tears down the worker pool cleanly.
 */
export async function deactivate(): Promise<void> {
  await workerPool?.shutdown();
  workerPool = undefined;
  log("OmniFormatter deactivated.");
}
