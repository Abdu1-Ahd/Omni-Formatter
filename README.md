<div align="center">
  <img src="extension/media/Omni-Formatter-Logo.svg" alt="OmniFormatter Logo" width="300" />
  <br/>

# OmniFormatter

**One extension. Every language. Zero configuration.**

[![VS Code Marketplace Downloads](https://img.shields.io/visual-studio-marketplace/d/Abdu1-Ahd.omni-formatter?style=for-the-badge&label=VS%20Code%20Marketplace&color=0066B8&logo=visualstudiocode)](https://marketplace.visualstudio.com/items?itemName=Abdu1-Ahd.omni-formatter)
[![Open VSX Downloads](https://img.shields.io/open-vsx/d/Abdu1-Ahd/omni-formatter?style=for-the-badge&label=Open%20VSX&color=952ca0&logo=eclipse)](https://open-vsx.org/extension/Abdu1-Ahd/omni-formatter)
[![Build](https://img.shields.io/github/actions/workflow/status/Abdu1-Ahd/Omni-Formatter/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white)](https://github.com/Abdu1-Ahd/Omni-Formatter/actions)
<br/>
[![Rust](https://img.shields.io/badge/Core-Rust-CE422B?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/Runtime-WebAssembly-654ff0?style=flat-square&logo=webassembly)](https://webassembly.org/)
[![Cloudflare Workers](https://img.shields.io/badge/Registry-Cloudflare%20Workers-F38020?style=flat-square&logo=cloudflare)](https://workers.cloudflare.com/)

[Install for VS Code](https://marketplace.visualstudio.com/items?itemName=Abdu1-Ahd.omni-formatter) • [Install for Open VSX](https://open-vsx.org/extension/Abdu1-Ahd/omni-formatter) • [Documentation](docs/) • [Add a Language](docs/ADD_LANGUAGE_TEMPLATE.md)

</div>

---

## 🚀 Why OmniFormatter?

Stop installing 10 different formatters (Prettier, ESLint, Black, rustfmt, clang-format, gofmt...) that constantly conflict with each other. 

**OmniFormatter eliminates the chaos.** It provides a single, blazing-fast WASM core that formats *everything* seamlessly.

<table>
  <tr>
    <td align="center">⚡<br/><b>Blazing Fast</b></td>
    <td align="center">📦<br/><b>Zero Config</b></td>
    <td align="center">🛡️<br/><b>Secure</b></td>
    <td align="center">🌍<br/><b>Universal</b></td>
  </tr>
  <tr>
    <td>WASM core activates in under 5ms with zero-copy infinite file size scaling.</td>
    <td>Automatically detects and reads native configurations (<code>.prettierrc</code>, <code>pyproject.toml</code>).</td>
    <td>Runs in a strict WASM Sandbox. All modules are cryptographically signed.</td>
    <td>Supports 70+ languages out-of-the-box via dynamic edge registry.</td>
  </tr>
</table>

---

## 🛠️ Supported Languages

OmniFormatter downloads the tiny language modules you need **on-the-fly** and caches them forever. 

* 🌐 **Frontend**: JavaScript, TypeScript, JSX, TSX, Vue, Svelte, Astro, HTML, CSS, SCSS, Sass, Less
* ⚙️ **Systems**: Rust, C, C++, Objective-C, Go, Zig, Nim, D
* ☕ **JVM & .NET**: Java, Kotlin, Scala, Groovy, C#, F#
* 🐍 **Scripting**: Python, Ruby, PHP, Perl, R, Julia, Lua
* 📱 **Mobile**: Swift, Dart
* 📝 **Markup & Data**: JSON, YAML, TOML, XML, INI, Markdown, LaTeX
* 📊 **DevOps & DB**: Dockerfile, Terraform, Nix, Makefile, SQL, GraphQL
* 🧩 **And More**: Haskell, Elixir, Erlang, OCaml, Clojure, Lisp, Scheme, Solidity, GDScript, AutoHotkey, Cobol, Fortran, Assembly, Jinja, Liquid, EJS, Handlebars, Twig...

---

## 💻 Quick Start

Set OmniFormatter as your default formatter and enable format-on-save in your `settings.json`:

```json
{
  "editor.defaultFormatter": "Abdu1-Ahd.omni-formatter",
  "editor.formatOnSave": true,
  "editor.formatOnType": true
}
```

That's it. Keep using your existing configuration files (e.g., `.prettierrc`, `rustfmt.toml`), and OmniFormatter will adapt automatically.

---

## 🏗️ Architecture

```mermaid
graph TD
    %% Custom Styles %%
    classDef vscode fill:#0066b8,stroke:#005599,stroke-width:2px,color:#fff,rx:8,ry:8
    classDef nodejs fill:#339933,stroke:#2a802a,stroke-width:2px,color:#fff,rx:8,ry:8
    classDef rust fill:#ce422b,stroke:#a63522,stroke-width:3px,color:#fff,rx:8,ry:8
    classDef module fill:#654ff0,stroke:#513fc0,stroke-width:2px,color:#fff,rx:15,ry:15
    classDef edge fill:#f38020,stroke:#d46a16,stroke-width:2px,color:#fff,rx:20,ry:20
    classDef config fill:#6c757d,stroke:#495057,stroke-width:2px,color:#fff,rx:5,ry:5

    %% Nodes %%
    A["🔌 VS Code Extension<br/><small>(TypeScript)</small>"]:::vscode
    B["⚡ Worker Pool<br/><small>(Node.js)</small>"]:::nodejs
    C{"⚙️ WASM Core<br/><small>(Rust)</small>"}:::rust
    D(["📦 Language Modules<br/><small>(.wasm binary)</small>"]):::module
    E[/"🛠️ Config Adapter<br/><small>(Native Format)</small>"/]:::config
    F(("☁️ Edge Registry<br/><small>(Cloudflare D1+R2)</small>")):::edge

    %% Connections %%
    A ==>|Zero-Copy IPC| B
    B ==>|Fast WASM Call| C
    C -->|Loads on demand| D
    C -->|Reads workspace rules| E
    D -.->|Fetched once & cached| F
```

## 🤝 Contributing

Contributions are welcome! Adding a language does not require touching the core extension. See our [Language Blueprint](docs/ADD_LANGUAGE_TEMPLATE.md) for how to add a language module in 10 minutes.