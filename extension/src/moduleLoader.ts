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
  private readonly registryUrl:        string;

  constructor(globalStoragePath: string, bundledModulesDir: string, registryUrl: string) {
    this.cacheRoot         = path.join(globalStoragePath, "modules");
    this.bundledModulesDir = bundledModulesDir;
    this.registryUrl       = registryUrl;

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

  // ── Public: resolve path only (preferred for worker pool wiring) ────────

  /**
   * Resolve the on-disk path of a WASM module without loading its bytes.
   *
   * Workers call `fs.readFileSync` on the path themselves — there is no need
   * to load the full binary (3 MB+) into the extension-host process just to
   * hand a file path to a Node.js Worker thread.
   *
   * Resolution order:
   *   1. Bundled path (ships inside the .vsix) — exists check only, zero I/O.
   *   2. Disk cache  (previously downloaded + verified).
   *   3. Registry download → written to cache → returns cache path.
   *
   * @param moduleName  e.g. "core", "lang-js", "lang-python"
   * @returns Absolute path to the verified .wasm binary.
   * @throws  {Error} If the module cannot be found locally or downloaded.
   */
  async resolveModulePath(moduleName: string): Promise<string> {
    if (!moduleName || typeof moduleName !== "string") {
      throw new Error(`ModuleLoader.resolveModulePath() called with invalid module name: ${JSON.stringify(moduleName)}`);
    }

    // 1. Bundled path
    const bundledPath = this.resolveBundledPath(moduleName);
    if (fs.existsSync(bundledPath)) {
      log.debug("Resolved bundled module path", { moduleName, path: bundledPath });
      return bundledPath;
    }

    // 2. Disk cache — find the latest cached version
    const cachedPath = this.resolveCachePath(moduleName);
    if (cachedPath) {
      log.debug("Resolved cached module path", { moduleName, path: cachedPath });
      return cachedPath;
    }

    // 3. Download from registry and return the cache path it was written to
    log.info("Module not found locally — downloading from registry", { moduleName });
    return this.downloadFromRegistryAndReturnPath(moduleName);
  }

  // ── Private: bundled path ─────────────────────────────────────────────

  private resolveBundledPath(moduleName: string): string {
    // The core WASM binary lives directly in the wasm/ directory, not modules/.
    if (moduleName === "core") {
      return path.join(this.bundledModulesDir, "omni_core_bg.wasm");
    }
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

  /**
   * Resolve the on-disk path for the latest cached version of a module.
   *
   * Performs the same integrity check as `loadFromCache` but returns the
   * path rather than the Buffer, avoiding the full read for path-only callers.
   */
  private resolveCachePath(moduleName: string): string | null {
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

    // Descending semver order — latest first.
    versions.sort((a, b) => semverCompare(b, a));
    const latestVersion = versions[0];
    const wasmPath      = path.join(moduleDir, latestVersion, "module.wasm");
    const manifestPath  = path.join(moduleDir, latestVersion, "manifest.json");

    if (!fs.existsSync(wasmPath) || !fs.existsSync(manifestPath)) {
      log.warn("Cache entry is incomplete, ignoring", { moduleName, version: latestVersion });
      return null;
    }

    // Integrity check: read manifest only (cheap), verify hash of wasm bytes.
    let manifest: ModuleManifest;
    try {
      manifest = JSON.parse(fs.readFileSync(manifestPath, "utf8")) as ModuleManifest;
    } catch (err) {
      log.warn("Cached manifest.json is unreadable or malformed — ignoring cache entry", {
        manifestPath,
        error: err instanceof Error ? err.message : String(err),
      });
      return null;
    }

    let wasmBytes: Buffer;
    try {
      wasmBytes = fs.readFileSync(wasmPath);
    } catch (err) {
      log.warn("Failed to read cached WASM for integrity check", {
        wasmPath,
        error: err instanceof Error ? err.message : String(err),
      });
      return null;
    }

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

    return wasmPath;
  }

  private loadFromCache(moduleName: string): Buffer | null {
    const wasmPath = this.resolveCachePath(moduleName);
    if (!wasmPath) { return null; }
    try {
      return fs.readFileSync(wasmPath);
    } catch (err) {
      log.warn("Failed to read cached WASM", {
        wasmPath,
        error: err instanceof Error ? err.message : String(err),
      });
      return null;
    }
  }

  // ── Private: registry download ────────────────────────────────────────

  private async downloadFromRegistry(moduleName: string): Promise<Buffer> {
    // ── Resolve latest version ───────────────────────────────────────
    // We append .json so this works transparently with static hosting (like GitHub Pages)
    // where the server cannot perform dynamic URL rewrites.
    const resolveUrl = `${this.registryUrl}/resolve/${encodeURIComponent(moduleName)}.json`;
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

    // Evict all older versions so disk usage stays bounded.
    // ponytail: run best-effort after the happy-path write; errors are warned, not thrown.
    this.pruneOldCacheVersions(moduleName, registryResponse.version);

    return wasmBytes;
  }

  // ── Private: registry download (path-returning variant) ───────────────

  /**
   * Download a module from the registry, write it to the cache atomically,
   * and return the on-disk path of the cached file.
   *
   * Called by `resolveModulePath()` so callers that only need a path never
   * have to load the full binary into the extension-host process.
   */
  private async downloadFromRegistryAndReturnPath(moduleName: string): Promise<string> {
    const wasmBytes = await this.downloadFromRegistry(moduleName);

    // downloadFromRegistry already wrote the file atomically.
    // Reconstruct the cache path it used, then verify it exists.
    // We cannot re-use the internal version string outside that method, so
    // we delegate back to resolveCachePath() which reads the cache directory.
    const cachedPath = this.resolveCachePath(moduleName);
    if (!cachedPath) {
      // Should never happen — downloadFromRegistry just wrote it.
      // As a last resort, write the bytes to a temp location and return that.
      const fallbackDir  = path.join(this.cacheRoot, moduleName, "fallback");
      fs.mkdirSync(fallbackDir, { recursive: true });
      const fallbackPath = path.join(fallbackDir, "module.wasm");
      fs.writeFileSync(fallbackPath, wasmBytes);
      log.warn("resolveCachePath returned null immediately after download — using fallback path", {
        moduleName,
        fallbackPath,
      });
      return fallbackPath;
    }

    return cachedPath;
  }


  // ── Private: stale cache pruning ─────────────────────────────────────

  /**
   * Delete all cached versions of `moduleName` except `keepVersion`.
   *
   * ponytail: called immediately after a successful download so old binaries
   * never accumulate on the user's disk. Best-effort — errors are warned only.
   */
  private pruneOldCacheVersions(moduleName: string, keepVersion: string): void {
    const moduleDir = path.join(this.cacheRoot, moduleName);
    let versions: string[];
    try {
      versions = fs.readdirSync(moduleDir);
    } catch {
      return; // nothing to prune
    }
    for (const v of versions) {
      if (v === keepVersion) { continue; }
      const vDir = path.join(moduleDir, v);
      try {
        fs.rmSync(vDir, { recursive: true, force: true });
        log.debug("Pruned old cache version", { moduleName, version: v });
      } catch (err) {
        log.warn("Could not prune old cache version", {
          moduleName,
          version: v,
          error:   err instanceof Error ? err.message : String(err),
        });
      }
    }
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
