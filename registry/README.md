# OmniFormatter Registry

The OmniFormatter module registry serves community-built language modules
securely to the VS Code extension and CLI.

## Registry API (Phase 5)

```
GET /resolve/{name}
    Returns the latest manifest for a module by name.
    Response: { name, version, sha256, download_url, language_ids, aliases }

GET /resolve/{name}/{version}
    Returns the manifest for a specific version.

GET /download/{name}/{version}/module.wasm
    Streams the verified WASM binary.

GET /modules
    Lists all available modules with metadata.

POST /publish
    Publishes a new module version. Requires a signed token.
    Body: { name, version, language_ids, sha256, wasm_base64 }
```

## Security Model (L-02 mitigation)

1. Every module is verified by SHA-256 hash before it is published to the registry.
2. The registry signs each module manifest with an Ed25519 key.
3. The extension host and CLI verify the signature before saving any module to disk.
4. Community modules never run in the extension host process — they run inside an
   isolated WASM sandbox in a Worker thread.
5. The registry maintains an immutable audit log of all module versions.

## Architecture

```
┌─────────────────────────────────────┐
│  OmniFormatter Registry Server       │
│  (Node.js + Hono.js, Cloudflare)    │
│                                     │
│  /resolve/{name}  ──► manifest.json │
│  /download/{name} ──► module.wasm   │
│  /modules         ──► index.json    │
│  /publish         ──► verify + save │
│                                     │
│  Storage: Cloudflare R2 (WASM files)│
│  Database: Cloudflare D1 (manifests)│
│  CDN: Cloudflare Workers (edge)     │
└─────────────────────────────────────┘
```

## Publishing a Module

Community module authors use the CLI to publish:

```bash
# Build the WASM module
wasm-pack build --target no-modules --release

# Publish to the registry (requires API token)
omnifmt modules publish --name lang-toml --wasm pkg/lang_toml_bg.wasm
```

The registry:
1. Validates the WASM binary against the module interface contract.
2. Computes the SHA-256 hash.
3. Signs the manifest with the registry Ed25519 key.
4. Stores the module in R2.
5. Makes it available via `omnifmt modules install lang-toml`.

## Implementation Status

Phase 5 scaffold. The registry server is not yet deployed.
The spec in this file is the authoritative design document for Phase 5.
