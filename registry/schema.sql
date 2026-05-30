-- OmniFormatter Registry — Cloudflare D1 Schema
--
-- Apply with:
--   wrangler d1 execute omnifmt-registry --file=schema.sql
--   wrangler d1 execute omnifmt-registry-staging --file=schema.sql --env staging
--
-- Tables:
--   modules       — one row per module name (canonical metadata)
--   versions      — one row per (name, version) tuple
--   publishers    — registry accounts with Ed25519 public keys
--   audit_log     — immutable append-only record of all publish/yank events

-- ── Extensions / Pragmas ──────────────────────────────────────────────────
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

-- ── Publishers ────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS publishers (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    username      TEXT    NOT NULL UNIQUE,
    -- Ed25519 public key, base64url-encoded (RFC 4648 §5, no padding)
    public_key    TEXT    NOT NULL,
    -- Account status: 'active' | 'suspended' | 'deleted'
    status        TEXT    NOT NULL DEFAULT 'active',
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

-- ── Modules ───────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS modules (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    name          TEXT    NOT NULL UNIQUE,        -- e.g. "lang-toml"
    description   TEXT    NOT NULL DEFAULT '',
    repository    TEXT,                           -- GitHub/GitLab URL
    homepage      TEXT,
    -- The publisher who first registered this module name
    owner_id      INTEGER NOT NULL REFERENCES publishers(id),
    -- Comma-separated VS Code languageIds this module handles
    language_ids  TEXT    NOT NULL DEFAULT '',
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    updated_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_modules_name ON modules(name);

-- ── Versions ──────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS versions (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    module_id     INTEGER NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    -- semver string (validated before insert)
    version       TEXT    NOT NULL,
    -- SHA-256 hex digest of the WASM binary (verified before serving)
    sha256        TEXT    NOT NULL,
    -- Ed25519 signature over (name || "@" || version || ":" || sha256), base64url
    signature     TEXT    NOT NULL,
    -- R2 object key where the WASM binary is stored
    r2_key        TEXT    NOT NULL,
    -- Size of the WASM binary in bytes
    wasm_size     INTEGER NOT NULL DEFAULT 0,
    -- "active" | "yanked" — yanked versions are not served (but not deleted)
    status        TEXT    NOT NULL DEFAULT 'active',
    -- Timestamp of the publish event
    published_at  TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now')),
    -- Publisher who uploaded this version
    publisher_id  INTEGER NOT NULL REFERENCES publishers(id),
    UNIQUE(module_id, version)
);

CREATE INDEX IF NOT EXISTS idx_versions_module  ON versions(module_id, status);
CREATE INDEX IF NOT EXISTS idx_versions_sha256  ON versions(sha256);

-- ── Audit log ─────────────────────────────────────────────────────────────
-- Append-only. Never UPDATE or DELETE rows in this table.
CREATE TABLE IF NOT EXISTS audit_log (
    id            INTEGER PRIMARY KEY AUTOINCREMENT,
    -- Event type: 'publish' | 'yank' | 'transfer' | 'account_created' | 'account_suspended'
    event_type    TEXT    NOT NULL,
    -- JSON payload with event-specific fields
    payload       TEXT    NOT NULL DEFAULT '{}',
    -- Who triggered the event (publisher ID or NULL for system events)
    actor_id      INTEGER REFERENCES publishers(id),
    -- Cloudflare Ray ID for traceability
    ray_id        TEXT,
    created_at    TEXT    NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%SZ', 'now'))
);

CREATE INDEX IF NOT EXISTS idx_audit_log_event  ON audit_log(event_type, created_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_actor  ON audit_log(actor_id, created_at);

-- ── Seed: bundled modules (installed with every registry deployment) ───────
-- These are the five built-in modules. They are owned by the system publisher (id=1).
INSERT OR IGNORE INTO publishers (id, username, public_key, status)
VALUES (1, 'omnifmt-system', 'SYSTEM_KEY_PLACEHOLDER', 'active');

INSERT OR IGNORE INTO modules (id, name, description, owner_id, language_ids)
VALUES
    (1, 'lang-js',     'JavaScript/TypeScript/JSX/TSX — Prettier 3.x parity', 1, 'javascript,typescript,javascriptreact,typescriptreact'),
    (2, 'lang-python', 'Python — Black 24.x parity',                          1, 'python'),
    (3, 'lang-rust',   'Rust — rustfmt stable parity',                        1, 'rust'),
    (4, 'lang-go',     'Go — gofmt/goimports parity',                         1, 'go'),
    (5, 'lang-css',    'CSS/SCSS/Less/HTML — Prettier 3.x parity',            1, 'css,scss,less,html');
