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
import { ConflictDetector } from "./conflictDetector";
import { toUtf8ByteOffset } from "./offsets";

/** VS Code language IDs that OmniFormatter handles. */
const SUPPORTED_LANGUAGE_IDS = [
  "javascript",
  "typescript",
  "javascriptreact",
  "typescriptreact",
  "python",
  "rust",
  "go",
  "css",
  "scss",
  "less",
  "html",
  "svelte",
  "vue",
  "astro",
] as const;

/** Map from languageId to the module name that handles it. */
const LANGUAGE_MODULE_MAP: Record<string, string> = {
  javascript: "lang-js",
  typescript: "lang-js",
  javascriptreact: "lang-js",
  typescriptreact: "lang-js",
  python: "lang-python",
  rust: "lang-rust",
  go: "lang-go",
  css: "lang-css",
  scss: "lang-css",
  less: "lang-css",
  html: "lang-css",
  svelte: "lang-js", // Svelte uses the JS module + zone detection
  vue: "lang-js",    // Vue SFC uses the JS module + zone detection
  astro: "lang-js",  // Astro uses the JS module + zone detection
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

  // Register commands
  context.subscriptions.push(
    vscode.commands.registerCommand("omniFormatter.formatDocument", () => {
      vscode.commands.executeCommand("editor.action.formatDocument");
    }),
    vscode.commands.registerCommand("omniFormatter.showStatus", () => {
      outputChannel?.show();
    }),
    vscode.commands.registerCommand("omnifmt.formatWorkspace", async () => {
      // Only format known source file types; exclude large non-source directories
      const INCLUDE_GLOB = '**/*.{js,ts,tsx,jsx,py,rs,go,css,scss,less,html,svelte,vue}';
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
  moduleLoader: ModuleLoader,
  context: vscode.ExtensionContext
): Promise<vscode.TextEdit[]> {
  const langId = document.languageId;
  const moduleName = LANGUAGE_MODULE_MAP[langId];

  if (!moduleName) {
    log(`No module for language: ${langId}`);
    return [];
  }

  // Enforce 10MB file size limit at the extension host (L-01 mitigation)
  const sourceText = document.getText();
  const sourceBytes = Buffer.from(sourceText, "utf8");
  const MAX_FILE_BYTES = 10 * 1024 * 1024;

  if (sourceBytes.length > MAX_FILE_BYTES) {
    vscode.window.showWarningMessage(
      `OmniFormatter: File exceeds 10MB limit (${Math.round(sourceBytes.length / 1024 / 1024)}MB). Formatting skipped.`
    );
    return [];
  }

  const request = {
    source: Array.from(sourceBytes),
    language_id: langId,
    config: {},       // Config adapter reads from disk in Phase 3+
    range: null,
    previous_tree: null,
    edit: null,
  };

  try {
    const startMs = Date.now();
    const responseJson = await workerPool!.dispatch(JSON.stringify(request), token);
    const elapsedMs = Date.now() - startMs;

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
