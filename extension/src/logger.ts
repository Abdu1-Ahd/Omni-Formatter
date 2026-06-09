/**
 * OmniFormatter Structured Logger
 *
 * A singleton logger that:
 * - Respects the `omniFormatter.logLevel` VS Code setting at runtime.
 * - Writes structured entries (timestamp, level, context, message, data) to
 *   the shared OutputChannel and to `console`.
 * - On `error()`, appends the full Error stack trace so every failure is
 *   fully debuggable without needing to reproduce it.
 * - Exposes a `withContext(name)` factory so each module stamps its own name
 *   on every log line without repetition.
 *
 * Usage:
 *   import { logger } from "./logger";
 *   const log = logger.withContext("WorkerPool");
 *   log.info("Pool started", { workers: 4 });
 *   log.error("Worker crashed", err, { workerId: 2 });
 */

import * as vscode from "vscode";

// ── Log levels ────────────────────────────────────────────────────────────

export const LOG_LEVELS = ["error", "warn", "info", "debug"] as const;
export type LogLevel = (typeof LOG_LEVELS)[number];

const LEVEL_RANK: Record<LogLevel, number> = {
  error: 0,
  warn:  1,
  info:  2,
  debug: 3,
};

// ── Logger singleton ──────────────────────────────────────────────────────

export class Logger {
  private static _instance: Logger | undefined;
  private _channel: vscode.OutputChannel | undefined;

  private constructor() {}

  public static getInstance(): Logger {
    if (!Logger._instance) {
      Logger._instance = new Logger();
    }
    return Logger._instance;
  }

  /** Attach the VS Code output channel. Must be called once in activate(). */
  public attach(channel: vscode.OutputChannel): void {
    this._channel = channel;
  }

  /** Detach and release the channel reference (called in deactivate). */
  public detach(): void {
    this._channel = undefined;
  }

  // ── Core log methods ────────────────────────────────────────────────────

  public error(context: string, message: string, err?: unknown, data?: Record<string, unknown>): void {
    this._write("error", context, message, err, data);
  }

  public warn(context: string, message: string, data?: Record<string, unknown>): void {
    this._write("warn", context, message, undefined, data);
  }

  public info(context: string, message: string, data?: Record<string, unknown>): void {
    this._write("info", context, message, undefined, data);
  }

  public debug(context: string, message: string, data?: Record<string, unknown>): void {
    this._write("debug", context, message, undefined, data);
  }

  // ── Context factory ─────────────────────────────────────────────────────

  /**
   * Returns a scoped logger that pre-fills `context` on every call.
   *
   * @example
   *   const log = logger.withContext("WorkerPool");
   *   log.info("ready", { workers: 4 });
   */
  public withContext(context: string): ScopedLogger {
    return new ScopedLogger(this, context);
  }

  // ── Internal ────────────────────────────────────────────────────────────

  private _currentLevel(): LogLevel {
    try {
      const cfg = vscode.workspace.getConfiguration("omniFormatter");
      const raw = cfg.get<string>("logLevel", "warn");
      return (LOG_LEVELS as readonly string[]).includes(raw) ? (raw as LogLevel) : "warn";
    } catch {
      return "warn";
    }
  }

  private _write(
    level: LogLevel,
    context: string,
    message: string,
    err?: unknown,
    data?: Record<string, unknown>
  ): void {
    if (LEVEL_RANK[level] > LEVEL_RANK[this._currentLevel()]) {
      return;
    }

    const timestamp = new Date().toISOString();
    const prefix    = `[${timestamp}] [${level.toUpperCase().padEnd(5)}] [${context}]`;
    let line        = `${prefix} ${message}`;

    if (data && Object.keys(data).length > 0) {
      try {
        line += `  ${JSON.stringify(data)}`;
      } catch {
        line += "  [data not serialisable]";
      }
    }

    if (err !== undefined) {
      if (err instanceof Error) {
        line += `\n  Error: ${err.message}`;
        if (err.stack) {
          // Indent stack trace for readability
          const indented = err.stack
            .split("\n")
            .slice(1) // skip the "Error: message" first line (already printed)
            .map((l) => `    ${l.trim()}`)
            .join("\n");
          line += `\n${indented}`;
        }
      } else {
        line += `\n  Thrown: ${String(err)}`;
      }
    }

    // Write to output channel (if attached)
    if (this._channel) {
      this._channel.appendLine(line);
    }

    // Always mirror to console
    switch (level) {
      case "error": console.error(line); break;
      case "warn":  console.warn(line);  break;
      case "info":  console.info(line);  break;
      case "debug": console.debug(line); break;
    }
  }
}

// ── Scoped logger (returned by withContext) ───────────────────────────────

export class ScopedLogger {
  constructor(
    private readonly _logger: Logger,
    private readonly _context: string
  ) {}

  error(message: string, err?: unknown, data?: Record<string, unknown>): void {
    this._logger.error(this._context, message, err, data);
  }

  warn(message: string, data?: Record<string, unknown>): void {
    this._logger.warn(this._context, message, data);
  }

  info(message: string, data?: Record<string, unknown>): void {
    this._logger.info(this._context, message, data);
  }

  debug(message: string, data?: Record<string, unknown>): void {
    this._logger.debug(this._context, message, data);
  }
}

/** Convenience singleton export. */
export const logger = Logger.getInstance();
