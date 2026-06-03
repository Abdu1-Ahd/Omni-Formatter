/**
 * OmniFormatter Registry Server
 *
 * Built on Cloudflare Workers + Hono.js.
 * Serves module manifests and verified WASM binaries.
 *
 * Security model (L-02 mitigation):
 * - Every module version is SHA-256 verified before serving.
 * - Publish requests require an Ed25519 signature over the manifest.
 * - No module binary is served without a valid stored signature.
 * - Yanked versions return 410 Gone (never 404, for auditability).
 *
 * Infrastructure:
 * - MODULES_BUCKET: Cloudflare R2 — WASM binary storage
 * - REGISTRY_DB:    Cloudflare D1 — module manifests (SQLite)
 * - REGISTRY_SIGNING_KEY: Ed25519 private key (Workers secret)
 */

import { Hono } from "hono";
import { cors } from "hono/cors";
import { logger } from "hono/logger";
import { timing } from "hono/timing";

// ── Bindings ───────────────────────────────────────────────────────────────

type Bindings = {
  MODULES_BUCKET?: R2Bucket;
  REGISTRY_DB: D1Database;
  REGISTRY_SIGNING_KEY: string;
};

// ── Data types ─────────────────────────────────────────────────────────────

interface ModuleManifest {
  name: string;
  version: string;
  description: string;
  language_ids: string[];
  sha256: string;
  signature: string;
  wasm_size: number;
  download_url: string;
  published_at: string;
  status: "active" | "yanked";
}

interface PublishRequest {
  /** Module name (e.g. "lang-toml"). */
  name: string;
  /** semver version string. */
  version: string;
  description?: string;
  language_ids: string[];
  /** SHA-256 hex digest of the WASM binary. */
  sha256: string;
  /** Ed25519 signature (base64url) over `name@version:sha256`. */
  signature: string;
  /** Base64-encoded WASM binary. */
  wasm_base64: string;
}

// ── Semver validation ──────────────────────────────────────────────────────

const SEMVER_RE = /^\d+\.\d+\.\d+(?:-[\w.]+)?(?:\+[\w.]+)?$/;

function isValidSemver(v: string): boolean {
  return SEMVER_RE.test(v);
}

// ── Module name validation ─────────────────────────────────────────────────

const MODULE_NAME_RE = /^[a-z][a-z0-9-]{1,62}$/;

function isValidModuleName(name: string): boolean {
  return MODULE_NAME_RE.test(name);
}

// ── Ed25519 signature verification ────────────────────────────────────────

/**
 * Verify an Ed25519 signature using the Web Crypto API (available in Workers).
 *
 * @param publicKeyBase64url - Ed25519 public key, base64url-encoded.
 * @param message            - The signed message bytes.
 * @param signatureBase64url - The signature to verify, base64url-encoded.
 */
async function verifyEd25519(
  publicKeyBase64url: string,
  message: string,
  signatureBase64url: string,
): Promise<boolean> {
  try {
    const keyBytes = base64urlDecode(publicKeyBase64url);
    const sigBytes = base64urlDecode(signatureBase64url);
    const msgBytes = new TextEncoder().encode(message);

    const cryptoKey = await crypto.subtle.importKey(
      "raw",
      keyBytes,
      { name: "Ed25519" },
      false,
      ["verify"],
    );

    return await crypto.subtle.verify("Ed25519", cryptoKey, sigBytes, msgBytes);
  } catch {
    return false;
  }
}

function base64urlDecode(s: string): ArrayBuffer {
  // Pad to multiple of 4
  const padded = s.replace(/-/g, "+").replace(/_/g, "/");
  const padLen  = (4 - (padded.length % 4)) % 4;
  const std     = padded + "=".repeat(padLen);
  const binary  = atob(std);
  const bytes   = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes.buffer;
}

// ── Download URL helper ────────────────────────────────────────────────────

function downloadUrl(req: Request, name: string, version: string): string {
  const origin = new URL(req.url).origin;
  return `${origin}/download/${name}/${version}/module.wasm`;
}

// ── App ────────────────────────────────────────────────────────────────────

const app = new Hono<{ Bindings: Bindings }>();

app.use("*", cors({ origin: "*", allowMethods: ["GET", "POST", "OPTIONS"] }));
app.use("*", logger());
app.use("*", timing());

// ── Health ─────────────────────────────────────────────────────────────────

app.get("/health", (c) =>
  c.json({ status: "ok", version: "0.1.0", ts: new Date().toISOString() })
);

// ── GET /modules — list all active modules ────────────────────────────────

app.get("/modules", async (c) => {
  const { results } = await c.env.REGISTRY_DB.prepare(`
    SELECT m.name, m.description, m.language_ids,
           v.version, v.sha256, v.wasm_size, v.published_at
    FROM modules m
    JOIN versions v ON v.module_id = m.id
      AND v.id = (
        SELECT id FROM versions
        WHERE module_id = m.id AND status = 'active'
        ORDER BY published_at DESC LIMIT 1
      )
    ORDER BY m.name
  `).all();

  const modules = (results as Record<string, unknown>[]).map((row) => ({
    name:         row["name"],
    version:      row["version"],
    description:  row["description"],
    language_ids: String(row["language_ids"]).split(",").filter(Boolean),
    sha256:       row["sha256"],
    wasm_size:    row["wasm_size"],
    published_at: row["published_at"],
    download_url: downloadUrl(c.req.raw, row["name"] as string, row["version"] as string),
  }));

  return c.json({ modules });
});

// ── GET /resolve/:name — latest active version ────────────────────────────

app.get("/resolve/:name", async (c) => {
  const name = c.req.param("name");
  if (!isValidModuleName(name)) {
    return c.json({ error: "Invalid module name" }, 400);
  }

  const row = await c.env.REGISTRY_DB.prepare(`
    SELECT m.name, m.description, m.language_ids,
           v.version, v.sha256, v.signature, v.wasm_size, v.published_at, v.status
    FROM modules m
    JOIN versions v ON v.module_id = m.id
    WHERE m.name = ?
      AND v.status = 'active'
    ORDER BY v.published_at DESC
    LIMIT 1
  `).bind(name).first<Record<string, unknown>>();

  if (!row) {
    return c.json({ error: `Module not found: ${name}` }, 404);
  }

  return c.json({
    name:         row["name"],
    version:      row["version"],
    description:  row["description"],
    language_ids: String(row["language_ids"]).split(",").filter(Boolean),
    sha256:       row["sha256"],
    signature:    row["signature"],
    wasm_size:    row["wasm_size"],
    published_at: row["published_at"],
    download_url: downloadUrl(c.req.raw, name, row["version"] as string),
  } as ModuleManifest);
});

// ── GET /resolve/:name/:version — specific version ────────────────────────

app.get("/resolve/:name/:version", async (c) => {
  const name    = c.req.param("name");
  const version = c.req.param("version");

  if (!isValidModuleName(name) || !isValidSemver(version)) {
    return c.json({ error: "Invalid name or version" }, 400);
  }

  const row = await c.env.REGISTRY_DB.prepare(`
    SELECT m.name, m.description, m.language_ids,
           v.version, v.sha256, v.signature, v.wasm_size, v.published_at, v.status
    FROM modules m
    JOIN versions v ON v.module_id = m.id
    WHERE m.name = ? AND v.version = ?
    LIMIT 1
  `).bind(name, version).first<Record<string, unknown>>();

  if (!row) {
    return c.json({ error: `${name}@${version} not found` }, 404);
  }

  if (row["status"] === "yanked") {
    return c.json({ error: `${name}@${version} has been yanked` }, 410);
  }

  return c.json({
    name:         row["name"],
    version:      row["version"],
    description:  row["description"],
    language_ids: String(row["language_ids"]).split(",").filter(Boolean),
    sha256:       row["sha256"],
    signature:    row["signature"],
    wasm_size:    row["wasm_size"],
    published_at: row["published_at"],
    download_url: downloadUrl(c.req.raw, name, version),
  } as ModuleManifest);
});

// ── GET /download/:name/:version/module.wasm — stream verified WASM ───────

app.get("/download/:name/:version/module.wasm", async (c) => {
  const name    = c.req.param("name");
  const version = c.req.param("version");

  if (!isValidModuleName(name) || !isValidSemver(version)) {
    return c.json({ error: "Invalid name or version" }, 400);
  }

  // Look up the version record
  const row = await c.env.REGISTRY_DB.prepare(`
    SELECT v.r2_key, v.sha256, v.status, v.wasm_size, v.wasm_binary
    FROM modules m
    JOIN versions v ON v.module_id = m.id
    WHERE m.name = ? AND v.version = ?
    LIMIT 1
  `).bind(name, version).first<{ r2_key: string; sha256: string; status: string; wasm_size: number; wasm_binary: ArrayBuffer | null }>();

  if (!row) {
    return c.json({ error: `${name}@${version} not found` }, 404);
  }

  if (row.status === "yanked") {
    return c.json({ error: `${name}@${version} has been yanked` }, 410);
  }

  // Fetch from D1 (free alternative) or R2 (if configured)
  let body: ArrayBuffer;
  if (row.wasm_binary) {
    body = row.wasm_binary;
  } else if (c.env.MODULES_BUCKET && row.r2_key) {
    const object = await c.env.MODULES_BUCKET.get(row.r2_key);
    if (!object) {
      return c.json({ error: "Binary not found in storage" }, 503);
    }
    body = await object.arrayBuffer();
  } else {
    return c.json({ error: "Binary not found in storage" }, 503);
  }

  // Verify SHA-256 before serving (defence-in-depth)
  const hashBuffer = await crypto.subtle.digest("SHA-256", body);
  const hashHex    = Array.from(new Uint8Array(hashBuffer))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");

  if (hashHex !== row.sha256) {
    console.error(`SHA-256 mismatch for ${name}@${version}: stored=${row.sha256} computed=${hashHex}`);
    return c.json({ error: "Binary integrity check failed" }, 500);
  }

  return new Response(body, {
    headers: {
      "Content-Type":        "application/wasm",
      "Content-Length":      body.byteLength.toString(),
      "X-OmniFormatter-SHA": row.sha256,
      "Cache-Control":       "public, max-age=31536000, immutable",
    },
  });
});

// ── POST /publish — publish a new module version ──────────────────────────

app.post("/publish", async (c) => {
  // Authenticate: Bearer token must map to a known publisher
  const authHeader = c.req.header("Authorization") ?? "";
  if (!authHeader.startsWith("Bearer ")) {
    return c.json({ error: "Missing Authorization header" }, 401);
  }
  const token = authHeader.slice(7);

  // Resolve publisher by token (token = base64url(ed25519_public_key))
  const publisher = await c.env.REGISTRY_DB.prepare(
    "SELECT id, username, public_key, status FROM publishers WHERE public_key = ? AND status = 'active' LIMIT 1"
  ).bind(token).first<{ id: number; username: string; public_key: string; status: string }>();

  if (!publisher) {
    return c.json({ error: "Unauthorized: unknown or inactive publisher" }, 401);
  }

  // Parse request body
  let body: PublishRequest;
  try {
    body = await c.req.json<PublishRequest>();
  } catch {
    return c.json({ error: "Invalid JSON body" }, 400);
  }

  // Validate fields
  if (!isValidModuleName(body.name)) {
    return c.json({ error: "Invalid module name (lowercase letters, numbers, hyphens; 2-63 chars)" }, 400);
  }
  if (!isValidSemver(body.version)) {
    return c.json({ error: "Invalid version (must be valid semver)" }, 400);
  }
  if (!body.sha256 || !/^[0-9a-f]{64}$/.test(body.sha256)) {
    return c.json({ error: "Invalid sha256 (must be 64 hex chars)" }, 400);
  }
  if (!body.language_ids?.length) {
    return c.json({ error: "language_ids must be a non-empty array" }, 400);
  }

  // Verify Ed25519 signature: signed message = "name@version:sha256"
  const signedMessage = `${body.name}@${body.version}:${body.sha256}`;
  const signatureValid = await verifyEd25519(publisher.public_key, signedMessage, body.signature);
  if (!signatureValid) {
    return c.json({ error: "Invalid signature" }, 403);
  }

  // Decode and verify WASM binary
  let wasmBytes: ArrayBuffer;
  try {
    const binary = atob(body.wasm_base64);
    wasmBytes = new Uint8Array(binary.length).buffer;
    const arr = new Uint8Array(wasmBytes);
    for (let i = 0; i < binary.length; i++) { arr[i] = binary.charCodeAt(i); }
  } catch {
    return c.json({ error: "Invalid wasm_base64 (must be valid base64)" }, 400);
  }

  // Verify SHA-256 of WASM payload
  const hashBuf = await crypto.subtle.digest("SHA-256", wasmBytes);
  const hashHex = Array.from(new Uint8Array(hashBuf))
    .map((b) => b.toString(16).padStart(2, "0"))
    .join("");
  if (hashHex !== body.sha256) {
    return c.json({ error: "sha256 does not match WASM binary" }, 400);
  }

  // Verify WASM magic bytes (0x00 0x61 0x73 0x6D)
  const magic = new Uint8Array(wasmBytes, 0, 4);
  if (magic[0] !== 0x00 || magic[1] !== 0x61 || magic[2] !== 0x73 || magic[3] !== 0x6D) {
    return c.json({ error: "Binary is not a valid WebAssembly module" }, 400);
  }

  // Ensure module record exists (or create it)
  await c.env.REGISTRY_DB.prepare(`
    INSERT OR IGNORE INTO modules (name, description, owner_id, language_ids)
    VALUES (?, ?, ?, ?)
  `).bind(
    body.name,
    body.description ?? "",
    publisher.id,
    body.language_ids.join(","),
  ).run();

  const moduleRow = await c.env.REGISTRY_DB.prepare(
    "SELECT id FROM modules WHERE name = ? LIMIT 1"
  ).bind(body.name).first<{ id: number }>();

  if (!moduleRow) {
    return c.json({ error: "Failed to create module record" }, 500);
  }

  // Check version uniqueness
  const existing = await c.env.REGISTRY_DB.prepare(
    "SELECT id FROM versions WHERE module_id = ? AND version = ? LIMIT 1"
  ).bind(moduleRow.id, body.version).first();

  if (existing) {
    return c.json({ error: `${body.name}@${body.version} already exists` }, 409);
  }

  // Store WASM in R2 (optional, if configured)
  const r2Key = `${body.name}/${body.version}/module.wasm`;
  if (c.env.MODULES_BUCKET) {
    try {
      await c.env.MODULES_BUCKET.put(r2Key, wasmBytes, {
        httpMetadata: { contentType: "application/wasm" },
        customMetadata: { sha256: body.sha256, publisher: publisher.username },
      });
    } catch (e) {
      console.warn("R2 upload skipped:", e);
    }
  }

  // Insert version record (with D1 blob storage)
  await c.env.REGISTRY_DB.prepare(`
    INSERT INTO versions (module_id, version, sha256, signature, r2_key, wasm_size, publisher_id, wasm_binary)
    VALUES (?, ?, ?, ?, ?, ?, ?, ?)
  `).bind(
    moduleRow.id,
    body.version,
    body.sha256,
    body.signature,
    r2Key,
    wasmBytes.byteLength,
    publisher.id,
    wasmBytes,
  ).run();

  // Audit log
  await c.env.REGISTRY_DB.prepare(`
    INSERT INTO audit_log (event_type, payload, actor_id, ray_id)
    VALUES ('publish', ?, ?, ?)
  `).bind(
    JSON.stringify({ name: body.name, version: body.version, sha256: body.sha256 }),
    publisher.id,
    c.req.header("CF-Ray") ?? null,
  ).run();

  return c.json({
    success: true,
    module:  body.name,
    version: body.version,
    sha256:  body.sha256,
    download_url: downloadUrl(c.req.raw, body.name, body.version),
  }, 201);
});

// ── POST /yank/:name/:version — yank a published version ─────────────────

app.post("/yank/:name/:version", async (c) => {
  const name    = c.req.param("name");
  const version = c.req.param("version");

  const authHeader = c.req.header("Authorization") ?? "";
  if (!authHeader.startsWith("Bearer ")) {
    return c.json({ error: "Missing Authorization header" }, 401);
  }
  const token = authHeader.slice(7);

  const publisher = await c.env.REGISTRY_DB.prepare(
    "SELECT id FROM publishers WHERE public_key = ? AND status = 'active' LIMIT 1"
  ).bind(token).first<{ id: number }>();

  if (!publisher) {
    return c.json({ error: "Unauthorized" }, 401);
  }

  const { meta } = await c.env.REGISTRY_DB.prepare(`
    UPDATE versions SET status = 'yanked'
    WHERE module_id = (SELECT id FROM modules WHERE name = ?)
      AND version = ?
      AND publisher_id = ?
  `).bind(name, version, publisher.id).run();

  if (meta.changes === 0) {
    return c.json({ error: `${name}@${version} not found or not owned by you` }, 404);
  }

  await c.env.REGISTRY_DB.prepare(`
    INSERT INTO audit_log (event_type, payload, actor_id, ray_id)
    VALUES ('yank', ?, ?, ?)
  `).bind(
    JSON.stringify({ name, version }),
    publisher.id,
    c.req.header("CF-Ray") ?? null,
  ).run();

  return c.json({ success: true, yanked: `${name}@${version}` });
});

export default app;
