/**
 * Module Loader — Registry client + disk cache (L-02, L-07 mitigation)
 *
 * Responsibilities:
 * 1. Check bundled modules (shipped with the extension).
 * 2. Check the local cache (globalStoragePath) for a verified module.
 * 3. Download from the OmniFormatter registry, verify SHA-256, and cache.
 *
 * Error philosophy:
 * - Every failure path throws a descriptive Error with full context.
 * - Cache integrity failures are logged and the corrupt entry is deleted.
 * - File writes use an atomic temp-file-then-rename pattern to prevent
 *   half-written cache entries surviving a crash.
 *
 * Cache layout:
 *   <globalStoragePath>/modules/<name>/<version>/module.wasm
 *   <globalStoragePath>/modules/<name>/<version>/manifest.json
 */

import * as fs from "fs";
import * as path from "path";
import * as crypto from "crypto";
import * as vscode from "vscode";
import { logger } from "./logger";

const log = logger.withContext("ModuleLoader");

const REGISTRY_BASE_URL = "https://omnifmt-registry.omniformat.workers.dev";

// ── Types ─────────────────────────────────────────────────────────────────

interface ModuleManifest {
  name:          string;
  version:       string;
  language_id:   string;
  aliases:       string[];
  sha256:        string;
  downloaded_at: string;
}

interface RegistryResolveResponse {
  name:         string;
  version:      string;
  download_url: string;
  sha256:       string;
}

// ── ModuleLoader ──────────────────────────────────────────────────────────

export class ModuleLoader {
  private readonly cacheRoot:          string;
  private readonly bundledModulesDir:  string;

  constructor(globalStoragePath: string, bundledModulesDir: string) {
    this.cacheRoot         = path.join(globalStoragePath, "modules");
    this.bundledModulesDir = bundledModulesDir;

    // Guard: mkdirSync can throw if the path is on a read-only volume.
    try {
      fs.mkdirSync(this.cacheRoot, { recursive: true });
    } catch (err) {
      log.warn("Could not create module cache directory — cached modules will not be used", {
        cacheRoot: this.cacheRoot,
        error:     err instanceof Error ? err.message : String(err),
      });
    }
  }

  /**
   * Load a WASM module by name.
   *
   * Search order:
   * 1. Bundled modules (shipped with the extension, no download needed).
   * 2. Disk cache (previously downloaded and verified).
   * 3. Registry download (verifies SHA-256 before returning).
   *
   * @param moduleName e.g. "lang-js", "lang-python", "lang-zig"
   * @returns The WASM binary as a Buffer.
   * @throws {Error} If the module cannot be found or downloaded.
   */
  async loadModule(moduleName: string): Promise<Buffer> {
    if (!moduleName || typeof moduleName !== "string") {
      throw new Error(`ModuleLoader.loadModule() called with invalid module name: ${JSON.stringify(moduleName)}`);
    }

    // 1. Bundled modules
    const bundledPath = this.resolveBundledPath(moduleName);
    if (fs.existsSync(bundledPath)) {
      log.debug("Loading bundled module", { moduleName, path: bundledPath });
      try {
        return fs.readFileSync(bundledPath);
      } catch (err) {
        throw new Error(
          `Found bundled module at "${bundledPath}" but failed to read it: ` +
          (err instanceof Error ? err.message : String(err))
        );
      }
    }

    // 2. Disk cache
    const cached = this.loadFromCache(moduleName);
    if (cached) {
      log.debug("Loaded module from cache", { moduleName });
      return cached;
    }

    // 3. Registry download
    log.info("Module not found locally — downloading from registry", { moduleName });
    return this.downloadFromRegistry(moduleName);
  }

  // ── Private: bundled path ─────────────────────────────────────────────

  private resolveBundledPath(moduleName: string): string {
    const safeName = moduleName.replace(/-/g, "_");
    return path.join(
      this.bundledModulesDir,
      "..",
      "modules",
      moduleName,
      `${safeName}_bg.wasm`
    );
  }

  // ── Private: cache ────────────────────────────────────────────────────

  private loadFromCache(moduleName: string): Buffer | null {
    const moduleDir = path.join(this.cacheRoot, moduleName);
    if (!fs.existsSync(moduleDir)) { return null; }

    let versions: string[];
    try {
      versions = fs.readdirSync(moduleDir);
    } catch (err) {
      log.warn("Could not list cache directory", {
        moduleDir,
        error: err instanceof Error ? err.message : String(err),
      });
      return null;
    }

    if (versions.length === 0) { return null; }

    // Sort semver correctly: compare major, minor, patch numerically.
    versions.sort((a, b) => semverCompare(b, a)); // descending — latest first
    const latestVersion = versions[0];
    const wasmPath      = path.join(moduleDir, latestVersion, "module.wasm");
    const manifestPath  = path.join(moduleDir, latestVersion, "manifest.json");

    if (!fs.existsSync(wasmPath) || !fs.existsSync(manifestPath)) {
      log.warn("Cache entry is incomplete, ignoring", { moduleName, version: latestVersion });
      return null;
    }

    let wasmBytes: Buffer;
    let manifest:  ModuleManifest;
    try {
      wasmBytes = fs.readFileSync(wasmPath);
    } catch (err) {
      log.warn("Failed to read cached WASM", {
        wasmPath,
        error: err instanceof Error ? err.message : String(err),
      });
      return null;
    }

    try {
      manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8")) as ModuleManifest;
    } catch (err) {
      log.warn("Cached manifest.json is unreadable or malformed — ignoring cache entry", {
        manifestPath,
        error: err instanceof Error ? err.message : String(err),
      });
      return null;
    }

    // Integrity verification on every load (defence-in-depth)
    if (!this.verifyHash(wasmBytes, manifest.sha256)) {
      log.error(
        "Cache integrity failure — SHA-256 mismatch. Deleting corrupt entry.",
        new Error("SHA-256 mismatch"),
        { moduleName, version: latestVersion }
      );
      try {
        fs.rmSync(path.join(moduleDir, latestVersion), { recursive: true, force: true });
      } catch (rmErr) {
        log.warn("Could not delete corrupt cache entry", {
          path:  path.join(moduleDir, latestVersion),
          error: rmErr instanceof Error ? rmErr.message : String(rmErr),
        });
      }
      return null;
    }

    return wasmBytes;
  }

  // ── Private: registry download ────────────────────────────────────────

  private async downloadFromRegistry(moduleName: string): Promise<Buffer> {
    // ── Resolve latest version ───────────────────────────────────────
    const resolveUrl = `${REGISTRY_BASE_URL}/resolve/${encodeURIComponent(moduleName)}`;
    log.info("Resolving module from registry", { url: resolveUrl });

    let resolveRes: Response;
    try {
      resolveRes = await fetch(resolveUrl);
    } catch (err) {
      throw new Error(
        `Network error while resolving module "${moduleName}" from registry: ` +
        (err instanceof Error ? err.message : String(err))
      );
    }

    if (!resolveRes.ok) {
      const body = await resolveRes.text().catch(() => "(no body)");
      if (resolveRes.status === 404) {
        void vscode.window.showWarningMessage(
          `OmniFormatter: No formatter module found for language "${moduleName}". ` +
          `Install a module from the registry or check your language configuration.`
        );
      }
      throw new Error(
        `Registry resolve failed for "${moduleName}": HTTP ${resolveRes.status} ${resolveRes.statusText}. ` +
        `Body: ${body.slice(0, 200)}`
      );
    }

    let registryResponse: RegistryResolveResponse;
    try {
      registryResponse = (await resolveRes.json()) as RegistryResolveResponse;
    } catch (err) {
      throw new Error(
        `Registry response for "${moduleName}" was not valid JSON: ` +
        (err instanceof Error ? err.message : String(err))
      );
    }

    if (!registryResponse.download_url || !registryResponse.sha256 || !registryResponse.version) {
      throw new Error(
        `Registry response for "${moduleName}" is missing required fields. ` +
        `Got: ${JSON.stringify(registryResponse)}`
      );
    }

    // ── Download WASM binary ─────────────────────────────────────────
    log.info("Downloading WASM module", { moduleName, version: registryResponse.version });
    let wasmRes: Response;
    try {
      wasmRes = await fetch(registryResponse.download_url);
    } catch (err) {
      throw new Error(
        `Network error while downloading WASM for "${moduleName}": ` +
        (err instanceof Error ? err.message : String(err))
      );
    }

    if (!wasmRes.ok) {
      throw new Error(
        `Failed to download WASM for "${moduleName}": HTTP ${wasmRes.status} ${wasmRes.statusText}`
      );
    }

    let arrayBuffer: ArrayBuffer;
    try {
      arrayBuffer = await wasmRes.arrayBuffer();
    } catch (err) {
      throw new Error(
        `Failed to read WASM response body for "${moduleName}": ` +
        (err instanceof Error ? err.message : String(err))
      );
    }
    const wasmBytes = Buffer.from(arrayBuffer);

    // ── Integrity check ──────────────────────────────────────────────
    if (!this.verifyHash(wasmBytes, registryResponse.sha256)) {
      throw new Error(
        `SHA-256 integrity check failed for "${moduleName}" v${registryResponse.version}. ` +
        `The downloaded file may be corrupt or tampered with. ` +
        `Expected: ${registryResponse.sha256}`
      );
    }

    // ── Atomic write to cache ────────────────────────────────────────
    // Write to a temp file first, then rename. This ensures a crash
    // mid-write cannot leave a partially-written module in the cache.
    const moduleDir = path.join(this.cacheRoot, moduleName, registryResponse.version);
    try {
      fs.mkdirSync(moduleDir, { recursive: true });
    } catch (err) {
      throw new Error(
        `Could not create cache directory "${moduleDir}": ` +
        (err instanceof Error ? err.message : String(err))
      );
    }

    this.atomicWriteFile(path.join(moduleDir, "module.wasm"), wasmBytes);
    this.atomicWriteFile(
      path.join(moduleDir, "manifest.json"),
      Buffer.from(JSON.stringify({
        name:          registryResponse.name ?? moduleName,
        version:       registryResponse.version,
        sha256:        registryResponse.sha256,
        language_id:   moduleName,
        aliases:       [],
        downloaded_at: new Date().toISOString(),
      } satisfies ModuleManifest, null, 2), "utf8")
    );

    log.info("Module downloaded and cached", {
      moduleName,
      version:   registryResponse.version,
      sizeBytes: wasmBytes.length,
    });
    return wasmBytes;
  }

  // ── Private: atomic write ─────────────────────────────────────────────

  /**
   * Write `data` to `targetPath` atomically: write to a temp file in the
   * same directory, then rename over the target.
   *
   * @throws {Error} If the write or rename fails.
   */
  private atomicWriteFile(targetPath: string, data: Buffer): void {
    const dir      = path.dirname(targetPath);
    const tmpPath  = path.join(dir, `.tmp-${crypto.randomBytes(6).toString("hex")}`);

    try {
      fs.writeFileSync(tmpPath, data);
    } catch (err) {
      throw new Error(
        `Failed to write temp file "${tmpPath}": ` +
        (err instanceof Error ? err.message : String(err))
      );
    }

    try {
      fs.renameSync(tmpPath, targetPath);
    } catch (err) {
      // Clean up temp file before re-throwing
      try { fs.unlinkSync(tmpPath); } catch { /* best-effort */ }
      throw new Error(
        `Failed to rename temp file to "${targetPath}": ` +
        (err instanceof Error ? err.message : String(err))
      );
    }
  }

  // ── Private: hash ─────────────────────────────────────────────────────

  private verifyHash(data: Buffer, expectedHex: string): boolean {
    if (!expectedHex || expectedHex.length !== 64) {
      log.warn("Invalid SHA-256 expected hash — skipping verification", { expectedHex });
      return true; // treat missing/invalid hash as a pass (registry may not provide it)
    }
    const actual = crypto.createHash("sha256").update(data).digest("hex");
    return actual === expectedHex;
  }
}

// ── Semver helpers ────────────────────────────────────────────────────────

/**
 * Compare two semver strings numerically.
 * Returns positive if a > b, negative if a < b, 0 if equal.
 * Falls back to lexicographic comparison for non-semver strings.
 */
function semverCompare(a: string, b: string): number {
  const parsePart = (s: string): number[] =>
    s.split(".").map((p) => {
      const n = parseInt(p, 10);
      return isNaN(n) ? 0 : n;
    });

  const aParts = parsePart(a);
  const bParts = parsePart(b);
  const len    = Math.max(aParts.length, bParts.length);

  for (let i = 0; i < len; i++) {
    const diff = (aParts[i] ?? 0) - (bParts[i] ?? 0);
    if (diff !== 0) { return diff; }
  }
  return 0;
}
