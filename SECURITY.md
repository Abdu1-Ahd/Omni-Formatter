# Security Policy

## Supported Versions

| Version | Supported |
|---|---|
| 0.1.x (latest) | ✅ Active |
| < 0.1.0 | ❌ No |

Only the latest release receives security patches. Users on older versions are expected to upgrade.

---

## Reporting a Vulnerability

**Do not file a public GitHub issue for security vulnerabilities.**

Public disclosure of an unpatched vulnerability puts all users at risk. Use GitHub's private vulnerability reporting mechanism instead:

1. Go to the [Security tab](https://github.com/Abdu1-Ahd/Omni-Formatter/security) of this repository.
2. Click **"Report a vulnerability"**.
3. Provide a clear description of the issue, the affected component, and a proof of concept if available.

You will receive an acknowledgment within **72 hours** and a status update within **7 days**.

---

## Disclosure Policy

- We will work with you to understand and reproduce the issue.
- We will keep you informed of the remediation timeline.
- We will credit you in the release notes unless you prefer to remain anonymous.
- We request a **90-day embargo** before public disclosure, giving us time to release a patch and notify users.

---

## Security Architecture

OmniFormatter is designed with the following security constraints:

### Plugin Sandboxing

All language modules execute inside a strict WebAssembly sandbox. A formatting plugin:
- **Cannot** read files from the file system
- **Cannot** make network requests
- **Cannot** spawn child processes
- **Cannot** access environment variables

The host (VS Code extension or CLI) manually reads the source file, copies it into WASM linear memory, invokes the formatting function, and reads the result. The WASM binary never touches disk or network.

### Cryptographic Integrity

Every module in the OmniFormatter Registry is:
1. **Signed with Ed25519** by the publisher at publish time
2. **Verified by the registry** before the WASM binary is stored
3. **Hash-verified (SHA-256)** by the extension before the WASM binary is instantiated

A tampered or unsigned binary will never execute.

### Yank Protocol

Compromised versions are never deleted. They are marked with `status = 'yanked'` in the registry database, and the registry returns HTTP 410 Gone for any request resolving that version. This preserves a complete audit trail of all published and yanked modules.

### Memory Safety

The Rust core is written in safe Rust where possible. All `unsafe` blocks are annotated with `// SAFETY:` comments explaining the invariants being upheld. The custom WASM allocator (`talc`) eliminates the memory fragmentation and double-free bugs present in the default `dlmalloc` allocator under heavy Tree-sitter AST traversal.
