/**
 * OmniFormatter VS Code Extension Entry Point
 *
 * Activation: onStartupFinished — runs after VS Code has fully started,
 * preventing any startup latency impact (L-03 mitigation).
 *
 * Responsibilities:
 * - Initialises the structured logger (attached to the OutputChannel).
 * - Registers the single DocumentFormattingEditProvider for all language IDs.
 * - Initialises the WorkerPool (WASM-backed formatting engine).
 * - Initiates background WASM compilation for top-5 languages (L-03).
 * - Detects conflicting formatter extensions (L-11).
 * - Shows the status bar item (L-11 status communication).
 *
 * Error handling philosophy:
 * - Commands are registered BEFORE any potentially-failing initialisation so
 *   the status bar item remains clickable even if the worker pool fails.
 * - All format errors are surfaced to the status bar and the output channel.
 * - No swallowed exceptions anywhere in the activation path.
 */

import * as vscode from "vscode";
import * as path from "path";
import * as os from "os";

import { logger } from "./logger";
import { WorkerPool, OmniFormatterError } from "./workerPool";
import { StatusBar } from "./statusBar";
import { ModuleLoader } from "./moduleLoader";
import { WasmCompiler } from "./wasmCompiler";
import { DashboardPanel } from "./webview/DashboardPanel";
import { ConflictDetector } from "./conflictDetector";
import { FormattingState } from "./formattingState";
import { FormatterInfoCodeLensProvider } from "./providers/FormatterInfoCodeLensProvider";
import { FormatterHoverProvider } from "./providers/FormatterHoverProvider";
import { toUtf16CodeUnitOffset } from "./offsets";

// ── Logger (module-level so it is accessible in handleFormatRequest) ──────

const log = logger.withContext("Extension");

// ── Language configuration ────────────────────────────────────────────────

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
/** Map from languageId to the WASM module name that handles it. */
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

// ── WASM response types ───────────────────────────────────────────────────

interface WasmEdit {
  range:    { start: number; end: number };
  new_text: string;
}

interface WasmFormatResponse {
  edits:           WasmEdit[];
  formatter_chain: string;
  is_noop:         boolean;
  error?:          unknown;
}

/**
 * Accept either a successful response  { edits, formatter_chain, is_noop }
 * OR an error envelope                 { error: {...} }
 *
 * The WASM core always returns one of these two shapes. Accepting both here
 * prevents the error path from being misclassified as a shape validation
 * failure and double-logging the same problem.
 */
function isWasmFormatResponse(v: unknown): v is WasmFormatResponse {
  if (typeof v !== "object" || v === null) { return false; }
  const r = v as Record<string, unknown>;
  // Happy path — full response.
  if (Array.isArray(r["edits"]) && typeof r["is_noop"] === "boolean") {
    return true;
  }
  // Error envelope — { error: <FormatError> } with no edits/is_noop.
  // We treat this as a valid (but failed) response so the error branch
  // below can surface a clean message instead of a shape-validation error.
  if ("error" in r && r["error"] !== null && r["error"] !== undefined) {
    // Synthesise the missing fields so callers don't need to null-check.
    r["edits"]           = r["edits"] ?? [];
    r["is_noop"]         = r["is_noop"] ?? false;
    r["formatter_chain"] = r["formatter_chain"] ?? "";
    return true;
  }
  return false;
}

function isWasmEdit(v: unknown): v is WasmEdit {
  if (typeof v !== "object" || v === null) { return false; }
  const e = v as Record<string, unknown>;
  return (
    typeof e["new_text"] === "string" &&
    typeof e["range"]    === "object" && e["range"] !== null &&
    typeof (e["range"] as Record<string, unknown>)["start"] === "number" &&
    typeof (e["range"] as Record<string, unknown>)["end"]   === "number"
  );
}

// ── Extension-level globals ───────────────────────────────────────────────

let workerPool:     WorkerPool | undefined;
let statusBar:      StatusBar  | undefined;
let wasmCompiler:   WasmCompiler | undefined;

// ── activate() ───────────────────────────────────────────────────────────

/**
 * Extension activation function.
 *
 * Called once when the extension activates (onStartupFinished).
 */
export async function activate(context: vscode.ExtensionContext): Promise<void> {
  // ── Step 1: OutputChannel + Logger (must be first) ───────────────────
  const channel = vscode.window.createOutputChannel("OmniFormatter");
  context.subscriptions.push(channel);
  logger.attach(channel);

  log.info("OmniFormatter activating…", {
    extensionPath: context.extensionPath,
    cpus:          os.cpus().length,
  });

  // ── Step 2: Status bar (must be early — shows error state on failures) ─
  statusBar = new StatusBar();
  context.subscriptions.push(statusBar);

  // ── Step 3: Register all commands BEFORE any potentially-failing init ──
  //    This ensures the status bar button and palette commands always work,
  //    even if the WASM worker pool fails to start.
  registerCommands(context);

  // ── Step 4: FormattingState (singleton with Disposable) ──────────────
  const formattingState = FormattingState.getInstance();
  context.subscriptions.push(formattingState);

  // ── Step 5: Worker pool + everything that depends on it ───────────────
  const workerScript = path.join(context.extensionPath, "dist", "workers", "formatWorker.js");
  const wasmDir      = path.join(context.extensionPath, "dist", "wasm");
  const numWorkers   = Math.max(2, os.cpus().length - 1);

  workerPool = new WorkerPool(workerScript, wasmDir, numWorkers);

  try {
    await workerPool.initialise();
    log.info("Worker pool started", { numWorkers });
  } catch (err) {
    const errMsg = err instanceof Error ? err.message : String(err);
    log.error("Worker pool failed to initialise", err instanceof Error ? err : new Error(errMsg));
    statusBar.showError(
      `WASM worker pool failed to start. Click to see details in the output channel.`
    );
    // Do NOT return — the extension can still register providers (they will
    // produce empty results until the pool is available, or on next restart).
  }

  // ── Step 6: ModuleLoader ──────────────────────────────────────────────
  const moduleLoader = new ModuleLoader(
    context.globalStorageUri.fsPath,
    wasmDir
  );

  // ── Step 7: Background WASM pre-compilation ───────────────────────────
  wasmCompiler = new WasmCompiler(wasmDir, context.globalStorageUri.fsPath);
  // The extension ships a single unified WASM binary (omni_core_bg.wasm)
  // that handles ALL languages. There are no per-language WASM modules.
  // Precompile only the one real binary that is always present.
  wasmCompiler.precompileTopLanguages(["core"]);

  // ── Step 8: Conflict detection ────────────────────────────────────────
  const conflictDetector = new ConflictDetector();
  conflictDetector.detectAndNotify(SUPPORTED_LANGUAGE_IDS as unknown as string[]);

  // ── Step 9: Document formatting providers ────────────────────────────
  for (const langId of SUPPORTED_LANGUAGE_IDS) {
    const provider = vscode.languages.registerDocumentFormattingEditProvider(
      { language: langId },
      {
        provideDocumentFormattingEdits: (
          document: vscode.TextDocument,
          options:  vscode.FormattingOptions,
          token:    vscode.CancellationToken
        ) => handleFormatRequest(document, options, token, moduleLoader, context),
      }
    );
    context.subscriptions.push(provider);
  }

  // ── Step 10: CodeLens + Hover providers ──────────────────────────────
  const codeLensProvider = new FormatterInfoCodeLensProvider();
  const hoverProvider    = new FormatterHoverProvider();
  for (const langId of SUPPORTED_LANGUAGE_IDS) {
    context.subscriptions.push(
      vscode.languages.registerCodeLensProvider({ language: langId }, codeLensProvider),
      vscode.languages.registerHoverProvider({ language: langId }, hoverProvider)
    );
  }

  // ── Step 11: Format-on-save ───────────────────────────────────────────
  context.subscriptions.push(
    vscode.workspace.onWillSaveTextDocument((event) => {
      const { document } = event;
      if (!(SUPPORTED_LANGUAGE_IDS as readonly string[]).includes(document.languageId)) {
        return;
      }

      const config = vscode.workspace.getConfiguration("omniFormatter", document.uri);
      if (!config.get<boolean>("enable", true)) { return; }

      // Each save gets its own CancellationTokenSource that is always disposed.
      const cts = new vscode.CancellationTokenSource();
      context.subscriptions.push(cts); // disposes on extension deactivation as a safety net

      event.waitUntil(
        handleFormatRequest(
          document,
          { tabSize: 2, insertSpaces: true },
          cts.token,
          moduleLoader,
          context
        ).finally(() => {
          cts.dispose();
        })
      );
    })
  );

  // ── Step 12: Evict FormattingState on document close ──────────────────
  context.subscriptions.push(
    vscode.workspace.onDidCloseTextDocument((doc) => {
      formattingState.deleteState(doc.uri);
    })
  );

  log.info("OmniFormatter activated successfully.");
}

// ── deactivate() ─────────────────────────────────────────────────────────

export async function deactivate(): Promise<void> {
  log.info("OmniFormatter deactivating…");

  // Shut down the worker pool (rejects all in-flight requests gracefully).
  if (workerPool) {
    try {
      await workerPool.shutdown();
    } catch (err) {
      log.warn("Worker pool shutdown encountered an error", {
        error: err instanceof Error ? err.message : String(err),
      });
    }
    workerPool = undefined;
  }

  // Release the compiled WASM module cache.
  wasmCompiler?.clearCache();
  wasmCompiler = undefined;

  // Detach the logger from the output channel (channel will be disposed by VS Code).
  logger.detach();

  log.info("OmniFormatter deactivated.");
}

// ── handleFormatRequest() ─────────────────────────────────────────────────

/**
 * Handle a format request for a single document.
 *
 * Converts VS Code UTF-16 positions to UTF-8 byte offsets, dispatches to the
 * worker pool, validates the response shape, and converts back to VS Code
 * TextEdits. Every error path is fully logged and surfaces to the status bar.
 */
async function handleFormatRequest(
  document:    vscode.TextDocument,
  _options:    vscode.FormattingOptions,
  token:       vscode.CancellationToken,
  _moduleLoader: ModuleLoader,
  _context:    vscode.ExtensionContext
): Promise<vscode.TextEdit[]> {
  // ── Guard: pool must exist ────────────────────────────────────────────
  if (!workerPool) {
    log.warn("Format request ignored — worker pool is not running", {
      uri: document.uri.toString(),
    });
    return [];
  }

  // ── Guard: document must still be open ───────────────────────────────
  if (document.isClosed) {
    log.debug("Format request ignored — document was closed before formatting started", {
      uri: document.uri.toString(),
    });
    return [];
  }

  const langId     = document.languageId;
  const moduleName = LANGUAGE_MODULE_MAP[langId];

  if (!moduleName) {
    log.debug("No module registered for language", { langId });
    return [];
  }

  // ── Read source text ─────────────────────────────────────────────────
  const sourceText    = document.getText();
  // Estimate byte size for timeout scaling and progress notification.
  // UTF-8 worst case: 4 bytes per character (supplementary codepoints).
  const byteEstimate  = Buffer.byteLength(sourceText, "utf8");

  // ── Build format request ──────────────────────────────────────────────
  //
  // FormatRequest protocol (see crates/protocol/src/lib.rs):
  //
  //   source_text  — UTF-8 source as a JSON string (preferred, zero-copy).
  //   source       — UTF-8 bytes as number[] (legacy, required by older WASM
  //                  builds compiled before source_text became Optional).
  //
  // We send BOTH fields so the extension is forward- AND backward-compatible
  // with any compiled WASM binary regardless of which protocol version it was
  // built against. The WASM core uses source_text when present and falls back
  // to source, so there is zero overhead for up-to-date builds.
  //
  // ⚠ source_byte_length is NOT a FormatRequest field — it is metadata that
  //   the worker reads from the message to scale its timeout. Strip it out
  //   of the JSON that goes to the WASM core by keeping it separate.
  const sourceBytes = Array.from(Buffer.from(sourceText, "utf8"));
  const request = {
    source_text:   sourceText,
    source:        sourceBytes,
    language_id:   langId,
    config:        {},
    range:         null,
    previous_tree: null,
    edit:          null,
  };

  let requestJson: string;
  try {
    requestJson = JSON.stringify(request);
  } catch (err) {
    log.error("Failed to serialise format request", err instanceof Error ? err : new Error(String(err)), {
      langId,
      uri: document.uri.toString(),
    });
    return [];
  }

  // ── Progress notification for large files (> 1 MB) ───────────────────
  let progressResolve: (() => void) | undefined;
  if (byteEstimate > 1_048_576) {
    void vscode.window.withProgress(
      {
        location:    vscode.ProgressLocation.Window,
        title:       `OmniFormatter: formatting ${langId} (≈${Math.round(byteEstimate / 1024)} KB)…`,
        cancellable: false,
      },
      () => new Promise<void>((resolve) => { progressResolve = resolve; })
    );
  }

  // ── Dispatch to worker pool ───────────────────────────────────────────
  const startMs = Date.now();
  let responseJson: string;
  try {
    responseJson = await workerPool.dispatch(requestJson, byteEstimate, token);
  } catch (err) {
    progressResolve?.();

    // Cancellation is a normal user action — no error surfacing needed.
    const isCancelled = err instanceof OmniFormatterError && err.code === "CANCELLED";
    if (isCancelled) {
      log.debug("Format request cancelled", { langId, uri: document.uri.toString() });
      return [];
    }

    const errorMsg = err instanceof Error ? err.message : String(err);
    log.error("Worker pool dispatch failed", err instanceof Error ? err : new Error(errorMsg), {
      langId,
      uri:       document.uri.toString(),
      byteEstimate,
    });
    statusBar?.showError(`Failed to format ${langId} — see output channel for details.`);
    return [];
  }

  const elapsedMs = Date.now() - startMs;
  progressResolve?.();

  // ── Parse and validate response ───────────────────────────────────────
  let response: unknown;
  try {
    response = JSON.parse(responseJson);
  } catch (err) {
    log.error("WASM returned non-JSON response", err instanceof Error ? err : new Error(String(err)), {
      langId,
      responsePreview: responseJson.slice(0, 300),
    });
    statusBar?.showError(`Internal error formatting ${langId} — bad response from WASM.`);
    return [];
  }

  if (!isWasmFormatResponse(response)) {
    log.error(
      "WASM response did not match expected shape",
      new Error("Unexpected WASM response shape"),
      {
        langId,
        responsePreview: JSON.stringify(response).slice(0, 300),
      }
    );
    statusBar?.showError(`Internal error formatting ${langId} — unexpected WASM response.`);
    return [];
  }

  if (response.error) {
    const errorMsg = typeof response.error === "string"
      ? response.error
      : JSON.stringify(response.error);
    log.error("WASM formatter returned an error", new Error(errorMsg), {
      langId,
      elapsedMs,
      uri: document.uri.toString(),
    });
    statusBar?.showError(`Format error in ${langId} — see output channel for details.`);
    return [];
  }

  // ── Guard: document may have been closed while we were waiting ────────
  if (document.isClosed) {
    log.debug("Document was closed while formatting — discarding edits", {
      langId,
      uri: document.uri.toString(),
    });
    return [];
  }

  // ── Update status bar and state ───────────────────────────────────────
  statusBar?.update(langId, response.formatter_chain, elapsedMs);
  log.debug("Format complete", {
    langId,
    formatterChain: response.formatter_chain,
    elapsedMs,
    editCount:      response.edits.length,
    isNoop:         response.is_noop,
  });

  FormattingState.getInstance().updateState(document.uri, {
    formatterChain: response.formatter_chain,
    elapsedMs,
    timestamp:      Date.now(),
  });

  if (response.is_noop || response.edits.length === 0) {
    return [];
  }

  // ── Convert UTF-8 byte offset TextEdits to VS Code UTF-16 positions ───
  const textEdits: vscode.TextEdit[] = [];

  for (const edit of response.edits) {
    if (!isWasmEdit(edit)) {
      log.warn("Skipping malformed edit in WASM response", {
        langId,
        edit: JSON.stringify(edit).slice(0, 200),
      });
      continue;
    }

    const startUtf16 = toUtf16CodeUnitOffset(sourceText, edit.range.start);
    const endUtf16   = toUtf16CodeUnitOffset(sourceText, edit.range.end);
    const startPos   = document.positionAt(startUtf16);
    const endPos     = document.positionAt(endUtf16);
    textEdits.push(
      vscode.TextEdit.replace(new vscode.Range(startPos, endPos), edit.new_text)
    );
  }

  return textEdits;
}

// ── registerCommands() ────────────────────────────────────────────────────

/**
 * Register all extension commands.
 *
 * Called BEFORE workerPool.initialise() so the commands are always available.
 */
function registerCommands(context: vscode.ExtensionContext): void {
  context.subscriptions.push(
    // Format the active document
    vscode.commands.registerCommand("omniFormatter.formatDocument", () => {
      void vscode.commands.executeCommand("editor.action.formatDocument");
    }),

    // Show the output channel (status bar click target)
    vscode.commands.registerCommand("omniFormatter.showStatus", () => {
      const channel = context.subscriptions.find(
        (s): s is vscode.OutputChannel =>
          typeof (s as vscode.OutputChannel).appendLine === "function"
      );
      if (channel) {
        (channel as vscode.OutputChannel).show();
      } else {
        // Fallback: try to show by name
        void vscode.commands.executeCommand("workbench.action.output.show", "OmniFormatter");
      }
    }),

    // Open the interactive dashboard webview
    vscode.commands.registerCommand("omniFormatter.openDashboard", () => {
      DashboardPanel.createOrShow(context);
    }),

    // Format every source file in the workspace
    vscode.commands.registerCommand("omnifmt.formatWorkspace", async () => {
      const INCLUDE_GLOB =
        "**/*.{js,mjs,cjs,ts,mts,cts,tsx,jsx,py,pyw,rs,go,css,scss,sass,less," +
        "html,htm,svelte,vue,astro,c,h,cpp,hpp,cc,cxx,hh,mm,m,java,kt,kts," +
        "scala,sc,groovy,cs,fs,fsi,fsx,rb,php,pl,pm,lua,sh,bash,zsh,ps1,psm1," +
        "swift,dart,json,json5,yaml,yml,toml,xml,ini,sql,graphql,gql,tf,hcl," +
        "Dockerfile,nix,hs,lhs,ex,exs,erl,hrl,ml,mli,clj,cljs,r,R,jl," +
        "md,markdown,tex,zig,nim,sol,gd,ahk,lisp,lsp,scm,ss,jinja,jinja2," +
        "liquid,ejs,hbs,handlebars,twig}";

      const EXCLUDE_GLOB =
        "**/{node_modules,.vscode-test,.vscode-test-user-data,.git,dist,out,target}/**";

      let uris: vscode.Uri[];
      try {
        uris = await vscode.workspace.findFiles(INCLUDE_GLOB, EXCLUDE_GLOB);
      } catch (err) {
        log.error("formatWorkspace: findFiles failed", err instanceof Error ? err : new Error(String(err)));
        void vscode.window.showErrorMessage("OmniFormatter: Failed to enumerate workspace files.");
        return;
      }

      log.info(`Formatting ${uris.length} files in workspace…`);

      let successCount = 0;
      let failCount    = 0;

      for (const uri of uris) {
        try {
          const doc = await vscode.workspace.openTextDocument(uri);
          await vscode.window.showTextDocument(doc, { preview: false });
          await vscode.commands.executeCommand("editor.action.formatDocument");
          await doc.save();
          successCount++;
        } catch (err) {
          failCount++;
          log.warn("formatWorkspace: failed to format file", {
            uri:   uri.fsPath,
            error: err instanceof Error ? err.message : String(err),
          });
        }
      }

      log.info("formatWorkspace complete", { successCount, failCount, total: uris.length });
      if (failCount > 0) {
        void vscode.window.showWarningMessage(
          `OmniFormatter: Workspace format complete. ${successCount} succeeded, ${failCount} failed. ` +
          `Check the output channel for details.`
        );
      } else {
        void vscode.window.showInformationMessage(
          `OmniFormatter: Formatted ${successCount} files successfully.`
        );
      }
    })
  );

  log.debug("Commands registered.", {
    commands: [
      "omniFormatter.formatDocument",
      "omniFormatter.showStatus",
      "omniFormatter.openDashboard",
      "omnifmt.formatWorkspace",
    ],
  });
}
