# OmniFormatter Summary

This document explains what this project is and why we built it. It is meant to help new people understand everything quickly.

## The Goal
Most code formatters are slow and require you to install heavy programs like Python or Node.js. 

**OmniFormatter** fixes this by turning all formatters into tiny, safe web plugins (WASM). It runs directly inside VS Code. You do not need to install Python, Go, or anything else on your computer.

## Four Main Ideas
1. **Zero Setup**: It copies the style of popular tools (like Prettier and Gofmt) perfectly, so you don't have to change any settings.
2. **Extremely Fast**: It finishes in less than a millisecond because it is built with Rust.
3. **Safe**: The plugins run in a locked box. They cannot read your personal files or access the internet.
4. **Reliable**: Formatting happens in the background, so it never freezes your editor.

## The Parts of the Project

### 1. `crates/` (The Brain)
Written in Rust. It reads the code, understands the structure, and rearranges it nicely. This folder contains all the rules for different languages.

### 2. `extension/` (The VS Code Link)
Written in TypeScript. It connects to VS Code. It downloads the formatting plugins from the internet and asks them to format your text in the background.

### 3. `registry/` (The Cloud Server)
Hosted on the internet (Cloudflare). This is the store where all the plugins are kept. It safely sends the plugins to the extension when needed.

### 4. `cli/` (The Command Line Tool)
A tool to run the formatter from your terminal, without opening VS Code. This is useful for automated robots.

## Important Details
- **Memory Safety**: We are very careful to ensure our fast engine does not leak memory or crash.
- **Talking to the Plugin**: VS Code sends the text to the plugin using shared computer memory to make it super fast.
- **Mixed Files**: If you have a file that mixes HTML, CSS, and JavaScript, the tool is smart enough to separate them, format them individually, and put them back together perfectly.
