# omnifmt-cli

Command-line interface for OmniFormatter. Format files from the terminal with
the same WASM core used by the VS Code extension.

## Usage

```bash
# Format a file in-place
omnifmt format src/main.ts

# Format stdin → stdout
cat src/main.rs | omnifmt format --language rust -

# Check if files are already formatted (exit 1 if not)
omnifmt check src/**/*.ts

# Print the resolved config for a file
omnifmt config src/main.py

# List installed language modules
omnifmt modules list

# Install a community module from the registry
omnifmt modules install lang-toml

# Verify a module's SHA-256 integrity
omnifmt modules verify lang-toml
```

## Installation

```bash
# Via cargo
cargo install omnifmt-cli

# Via npm (wraps the binary)
npm install -g @omnifmt/cli

# Pre-built binary (GitHub Releases)
curl -fsSL https://get.omnifmt.dev/install.sh | sh
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (or no formatting needed for `check`) |
| 1 | Files were not formatted (only for `check`) |
| 2 | Fatal error (module not found, WASM error, etc.) |
| 3 | Config parse error |

## Notes

- omnifmt-cli uses the same WASM modules as the VS Code extension.
- Module cache is shared at `~/.omnifmt/modules/`.
- Config resolution follows the same priority as the extension.
