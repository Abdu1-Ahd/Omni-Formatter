# Security Policy

## Supported Versions

| Version | Supported |
|---|---|
| `main` (latest) | ✅ Active |
| All prior releases | ❌ End of life |

Security fixes are backported only to the latest release. Upgrade to the latest version before filing a vulnerability report.

---

## Reporting a Vulnerability

**Do not open a public GitHub issue for security vulnerabilities.**

Report vulnerabilities privately using GitHub's Security Advisory feature:

1. Navigate to the [Security tab](https://github.com/Abdu1-Ahd/Omni-Formatter/security/advisories/new).
2. Click **"Report a vulnerability"**.
3. Complete the advisory form with:
   - Affected component (e.g., `crates/core`, `registry/`, `extension/`)
   - Reproduction steps
   - Potential impact and CVSS score estimate (if known)
   - Any proposed mitigation

---

## Disclosure Timeline

| Stage | Target Time |
|---|---|
| Acknowledgement | Within 48 hours of receipt |
| Triage and severity assessment | Within 5 business days |
| Fix development | Within 30 days for critical/high severity |
| Coordinated public disclosure | After fix is released and users have had time to upgrade |

---

## Scope

In scope for vulnerability reports:

- WASM core memory safety issues (buffer overruns, out-of-bounds access)
- Registry server authentication bypass or arbitrary WASM upload
- Extension host remote code execution via malicious config files
- SHA-256 verification bypass in module loader
- Denial of service in format-on-type path affecting VS Code stability

Out of scope:

- Issues in third-party dependencies (report upstream)
- Language formatter output differences (not a security issue)
- Performance regressions without a security impact

---

## Credits

Responsible disclosure contributors are credited in the release notes for the version that includes the fix, unless the reporter requests anonymity.
