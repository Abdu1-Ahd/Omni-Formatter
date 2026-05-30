/**
 * OmniFormatter Registry Server (Phase 5 scaffold)
 *
 * Built on Cloudflare Workers + Hono.js.
 * Serves module manifests and verified WASM binaries.
 *
 * Security (L-02 mitigation):
 * - Every module is SHA-256 verified before storage.
 * - Manifests are signed with Ed25519 (implementation in Phase 5).
 * - No module is ever served without a valid signature.
 */

import { Hono } from "hono";

type Bindings = {
  MODULES_BUCKET: R2Bucket;      // Cloudflare R2 — WASM binary storage
  REGISTRY_DB: D1Database;       // Cloudflare D1 — module manifests
  REGISTRY_SIGNING_KEY: string;  // Ed25519 private key (secret)
};

const app = new Hono<{ Bindings: Bindings }>();

// ─── Health check ────────────────────────────────────────────────────────────

app.get("/health", (c) => c.json({ status: "ok", version: "0.1.0" }));

// ─── GET /modules — list all available modules ────────────────────────────────

app.get("/modules", async (c) => {
  // Phase 5 stub: return bundled modules only.
  const modules = [
    { name: "lang-js",     version: "0.1.0", languageIds: ["javascript","typescript","jsx","tsx"] },
    { name: "lang-python", version: "0.1.0", languageIds: ["python"] },
    { name: "lang-rust",   version: "0.1.0", languageIds: ["rust"] },
    { name: "lang-go",     version: "0.1.0", languageIds: ["go"] },
    { name: "lang-css",    version: "0.1.0", languageIds: ["css","scss","less","html"] },
  ];
  return c.json({ modules });
});

// ─── GET /resolve/:name — resolve the latest module manifest ─────────────────

app.get("/resolve/:name", async (c) => {
  const name = c.req.param("name");

  // Phase 5 stub: bundled modules only.
  const manifest = BUNDLED_MANIFESTS[name];
  if (!manifest) {
    return c.json({ error: `Module not found: ${name}` }, 404);
  }
  return c.json(manifest);
});

// ─── GET /resolve/:name/:version — resolve a specific version ─────────────────

app.get("/resolve/:name/:version", async (c) => {
  const name = c.req.param("name");
  const version = c.req.param("version");
  const manifest = BUNDLED_MANIFESTS[name];
  if (!manifest || manifest.version !== version) {
    return c.json({ error: `Module ${name}@${version} not found` }, 404);
  }
  return c.json(manifest);
});

// ─── GET /download/:name/:version/module.wasm — stream verified WASM ─────────

app.get("/download/:name/:version/module.wasm", async (c) => {
  const name = c.req.param("name");
  const version = c.req.param("version");

  // Phase 5: stream from R2.
  // Stub: 404 until R2 bucket is populated.
  return c.json(
    { error: `Module binary not yet available: ${name}@${version}. Phase 5 in progress.` },
    503
  );
});

// ─── POST /publish — publish a new module version ────────────────────────────

app.post("/publish", async (c) => {
  // Phase 5: verify token, validate WASM binary, store in R2, update D1.
  return c.json({ error: "Publishing not yet available. Phase 5 in progress." }, 503);
});

// ─── Bundled module manifests (Phase 5 stub) ─────────────────────────────────

const BUNDLED_MANIFESTS: Record<string, object> = {
  "lang-js": {
    name: "lang-js",
    version: "0.1.0",
    description: "JavaScript/TypeScript/JSX/TSX — Prettier 3.x parity",
    language_ids: ["javascript", "typescript", "javascriptreact", "typescriptreact"],
    aliases: [".js", ".mjs", ".cjs", ".ts", ".mts", ".cts", ".jsx", ".tsx"],
    sha256: "PLACEHOLDER_COMPUTED_ON_BUILD",
    download_url: "https://registry.omnifmt.dev/download/lang-js/0.1.0/module.wasm",
  },
  "lang-python": {
    name: "lang-python",
    version: "0.1.0",
    description: "Python — Black 24.x parity",
    language_ids: ["python"],
    aliases: [".py", ".pyi"],
    sha256: "PLACEHOLDER_COMPUTED_ON_BUILD",
    download_url: "https://registry.omnifmt.dev/download/lang-python/0.1.0/module.wasm",
  },
  "lang-rust": {
    name: "lang-rust",
    version: "0.1.0",
    description: "Rust — rustfmt stable parity",
    language_ids: ["rust"],
    aliases: [".rs"],
    sha256: "PLACEHOLDER_COMPUTED_ON_BUILD",
    download_url: "https://registry.omnifmt.dev/download/lang-rust/0.1.0/module.wasm",
  },
  "lang-go": {
    name: "lang-go",
    version: "0.1.0",
    description: "Go — gofmt parity",
    language_ids: ["go"],
    aliases: [".go"],
    sha256: "PLACEHOLDER_COMPUTED_ON_BUILD",
    download_url: "https://registry.omnifmt.dev/download/lang-go/0.1.0/module.wasm",
  },
  "lang-css": {
    name: "lang-css",
    version: "0.1.0",
    description: "CSS/SCSS/Less/HTML — Prettier 3.x parity",
    language_ids: ["css", "scss", "less", "html"],
    aliases: [".css", ".scss", ".less", ".html", ".htm"],
    sha256: "PLACEHOLDER_COMPUTED_ON_BUILD",
    download_url: "https://registry.omnifmt.dev/download/lang-css/0.1.0/module.wasm",
  },
};

export default app;
