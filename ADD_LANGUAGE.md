# How to Add a New Language

You can add a new language without touching the main project. Your code will be a standalone plugin.

## The Quick Summary
- **What you build:** A single Rust folder that compiles to a web format (WASM).
- **How it connects:** The editor finds it automatically or downloads it from the internet.
- **Settings:** You write one file (`schema.json`) to let the editor know your settings.
- **Reading Old Settings:** You write code to read the user's existing settings (like `.prettierrc`), so they don't have to change anything.

## Folder Setup
Create a new folder for your language:
- `Cargo.toml`: Lists your dependencies.
- `schema.json`: Explains your settings.
- `src/lib.rs`: The main bridge to the editor.
- `src/adapter.rs`: Reads the user's existing settings.
- `src/format.rs`: The code that actually formats the text.

## The 5 Required Pieces (`src/lib.rs`)
Your code needs to provide five simple answers to the editor:
1. **Format**: Take the messy text and return the neat text.
2. **Settings Info**: Return your `schema.json` text.
3. **Version**: Return the version number.
4. **Name**: Return the language name (like "python").
5. **File Types**: Return the file extensions it handles (like ".py").

> **Important Rule:** Formatting the text twice must give the exact same result as formatting it once!

## Reading Settings (`src/adapter.rs`)
To make it easy for users, your code must read their existing setting files. If you find a settings file, read it. If not, use standard defaults.

## How to Build and Test
1. **Build:** Use `wasm-pack build` to create the web format.
2. **Test:** Copy the file to the extension folder and try it in VS Code.
3. **Publish:** Use our command-line tool to upload it to the cloud.

## Checklist
- [ ] Create the folder.
- [ ] Write your settings (`schema.json`).
- [ ] Write the code to read user settings (`adapter.rs`).
- [ ] Write the formatting rules (`format.rs`).
- [ ] Run tests to make sure it works perfectly.
- [ ] Build it and upload it.
