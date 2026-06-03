/**
 * Module Loader — Registry client + disk cache (L-02, L-07 mitigation)
 *
 * Responsibilities:
 * 1. Check the local cache (globalStoragePath) for a module by name + version.
 * 2. If not cached, download from the OmniFormatter registry.
 * 3. Verify the SHA-256 hash of the downloaded WASM binary.
 * 4. Save to cache and return the verified module bytes.
 *
 * Cache layout:
 *   <globalStoragePath>/modules/<name>/<version>/module.wasm
 *   <globalStoragePath>/modules/<name>/<version>/manifest.json
 *
 * Version pinning: cache entries are keyed by semver version.
 * A new module version downloads silently in the background during idle time
 * and replaces the old version at the start of the next VS Code session.
 */

import * as fs from "fs";
import * as path from "path";
import * as crypto from "crypto";
import * as vscode from "vscode";

const REGISTRY_BASE_URL = "https://omnifmt-registry.omniformat.workers.dev";

/** Module manifest stored alongside the cached WASM binary. */
interface ModuleManifest {
  name: string;
  version: string;
  language_id: string;
  aliases: string[];
  sha256: string;
  downloaded_at: string;
}

export class ModuleLoader {
  private readonly cacheRoot: string;
  private readonly bundledModulesDir: string;

  constructor(globalStoragePath: string, bundledModulesDir: string) {
    this.cacheRoot = path.join(globalStoragePath, "modules");
    this.bundledModulesDir = bundledModulesDir;
    fs.mkdirSync(this.cacheRoot, { recursive: true });
  }

  /**
   * Load a WASM module by name.
   *
   * Search order:
   * 1. Bundled modules (shipped with the extension, no download needed).
   * 2. Disk cache (previously downloaded).
   * 3. Registry download (verifies SHA-256 before returning).
   *
   * @param moduleName e.g. "lang-js", "lang-python", "lang-zig"
   * @returns The WASM binary bytes.
   */
  async loadModule(moduleName: string): Promise<Buffer> {
    // 1. Check bundled modules
    const bundledPath = this.resolveBundledPath(moduleName);
    if (fs.existsSync(bundledPath)) {
      return fs.readFileSync(bundledPath);
    }

    // 2. Check disk cache
    const cached = this.loadFromCache(moduleName);
    if (cached) return cached;

    // 3. Download from registry
    return this.downloadFromRegistry(moduleName);
  }

  /** Check if a bundled module exists. */
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

  /** Load a module from the disk cache if present. */
  private loadFromCache(moduleName: string): Buffer | null {
    const moduleDir = path.join(this.cacheRoot, moduleName);
    if (!fs.existsSync(moduleDir)) return null;

    // Find the latest cached version
    const versions = fs.readdirSync(moduleDir).sort().reverse();
    if (versions.length === 0) return null;

    const latestVersion = versions[0];
    const wasmPath = path.join(moduleDir, latestVersion, "module.wasm");
    const manifestPath = path.join(moduleDir, latestVersion, "manifest.json");

    if (!fs.existsSync(wasmPath) || !fs.existsSync(manifestPath)) return null;

    const wasmBytes = fs.readFileSync(wasmPath);
    const manifest: ModuleManifest = JSON.parse(fs.readFileSync(manifestPath, "utf8"));

    // Verify integrity on each load (defence-in-depth)
    if (!this.verifyHash(wasmBytes, manifest.sha256)) {
      console.error(`[OmniFormatter] Cache integrity failure for ${moduleName}. Deleting.`);
      fs.rmSync(path.join(moduleDir, latestVersion), { recursive: true, force: true });
      return null;
    }

    return wasmBytes;
  }

  private async downloadFromRegistry(moduleName: string): Promise<Buffer> {
    const resolveUrl = `${REGISTRY_BASE_URL}/resolve/${moduleName}`;

    const res = await fetch(resolveUrl);
    if (!res.ok) {
      vscode.window.showWarningMessage(
        `OmniFormatter: No formatter module found for language "${moduleName}". Install a module from the registry.`
      );
      throw new Error(`Failed to resolve module ${moduleName} from registry: HTTP ${res.status}`);
    }
    const manifest = await res.json() as any;

    const wasmRes = await fetch(manifest.download_url);
    if (!wasmRes.ok) {
      throw new Error(`Failed to download WASM for ${moduleName}: HTTP ${wasmRes.status}`);
    }
    
    const arrayBuffer = await wasmRes.arrayBuffer();
    const wasmBytes = Buffer.from(arrayBuffer);

    if (!this.verifyHash(wasmBytes, manifest.sha256)) {
      throw new Error(`Integrity check failed for ${moduleName}!`);
    }

    const moduleDir = path.join(this.cacheRoot, moduleName, manifest.version);
    fs.mkdirSync(moduleDir, { recursive: true });
    
    fs.writeFileSync(path.join(moduleDir, "module.wasm"), wasmBytes);
    fs.writeFileSync(path.join(moduleDir, "manifest.json"), JSON.stringify({
      name: manifest.name,
      version: manifest.version,
      sha256: manifest.sha256,
      language_id: moduleName,
      aliases: [],
      downloaded_at: new Date().toISOString(),
    }, null, 2));

    return wasmBytes;
  }

  /** Verify a WASM binary's SHA-256 hash. */
  private verifyHash(data: Buffer, expectedHex: string): boolean {
    const actual = crypto.createHash("sha256").update(data).digest("hex");
    return actual === expectedHex;
  }
}
