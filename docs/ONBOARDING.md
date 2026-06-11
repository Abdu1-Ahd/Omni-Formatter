# OmniFormatter Developer Onboarding

Welcome to OmniFormatter! This tool formats your code to make it look neat and tidy. It works for many languages using a fast, safe engine (WASM). It automatically copies the style of other popular tools so you don't need to change any settings.

This guide will help you start working on the code right away.

## 1. What You Need
- **Node.js** (version 20 or newer): To run the web code.
- **Rust** (version 1.75 or newer): The fast language we use for the engine.
- **wasm-pack**: A tool to pack our Rust code into a web format.
- **VS Code**: The editor where you will test the extension.

## 2. How the Folders are Organized
- **`crates/`**: The core engine written in Rust. It does the actual formatting.
- **`extension/`**: The code that connects our engine to VS Code.
- **`registry/`**: The internet cloud server that sends the engine to users.
- **`cli/`**: A tool to use the formatter directly from your computer's terminal.

## 3. How to Build the Project

First, build the Rust engine into a web format:
```bash
cargo build --release --target wasm32-unknown-unknown -p lang-js
```

Next, build the VS Code extension:
```bash
cd extension
npm install
npm run build:all
```

## 4. How to Test
We test our code to make sure formatting it twice doesn't change it the second time.

Test the Rust engine:
```bash
cargo test
```

Test the VS Code extension:
```bash
cd extension
npm run test
```

## 5. How It Works Under the Hood
1. VS Code asks to format the text.
2. The extension sends a message to our background worker.
3. The worker asks our fast Rust engine to do the work.
4. The Rust engine reads the code and understands its structure.
5. It rearranges the code to look perfect.
6. The perfectly formatted code is sent back to VS Code.

## 6. How to Add a New Language
1. Create a new folder for the language in `crates/`.
2. Connect it to the tool that reads that language.
3. Write the rules for how the code should look.
4. Add it to our list of modules.
5. Upload it to our cloud server.
