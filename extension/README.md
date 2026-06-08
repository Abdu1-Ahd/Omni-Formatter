<div align="center">
# OmniFormatter

**One extension. Every language. Zero configuration.**

[![VS Code Marketplace](https://img.shields.io/visual-studio-marketplace/v/Abdu1-Ahd.omni-formatter?style=for-the-badge&label=VS%20Code%20Marketplace&color=0066B8&logo=visualstudiocode)](https://marketplace.visualstudio.com/items?itemName=Abdu1-Ahd.omni-formatter)
[![Open VSX](https://img.shields.io/open-vsx/v/Abdu1-Ahd/omni-formatter?style=for-the-badge&label=Open%20VSX&color=952ca0&logo=eclipse)](https://open-vsx.org/extension/Abdu1-Ahd/omni-formatter)
[![Build](https://img.shields.io/github/actions/workflow/status/Abdu1-Ahd/Omni-Formatter/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white)](https://github.com/Abdu1-Ahd/Omni-Formatter/actions)
[![Rust](https://img.shields.io/badge/Core-Rust-CE422B?style=flat-square&logo=rust)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/Runtime-WebAssembly-654ff0?style=flat-square&logo=webassembly)](https://webassembly.org/)
[![Cloudflare Workers](https://img.shields.io/badge/Registry-Cloudflare%20Workers-F38020?style=flat-square&logo=cloudflare)](https://workers.cloudflare.com/)

[Install for VS Code](https://marketplace.visualstudio.com/items?itemName=Abdu1-Ahd.omni-formatter) ‚Ä¢ [Install for Open VSX](https://open-vsx.org/extension/Abdu1-Ahd/omni-formatter) ‚Ä¢ [Documentation](docs/) ‚Ä¢ [Add a Language](docs/ADD_LANGUAGE_TEMPLATE.md)

</div>

---

## üöÄ Why OmniFormatter?

Stop installing 10 different formatters (Prettier, ESLint, Black, rustfmt, clang-format, gofmt...) that constantly conflict with each other. 

**OmniFormatter eliminates the chaos.** It provides a single, blazing-fast WASM core that formats *everything* seamlessly.

<table>
  <tr>
    <td align="center">‚ö°<br/><b>Blazing Fast</b></td>
    <td align="center">üì¶<br/><b>Zero Config</b></td>
    <td align="center">üõ°Ô∏è<br/><b>Secure</b></td>
    <td align="center">üåç<br/><b>Universal</b></td>
  </tr>
  <tr>
    <td>WASM core activates in under 5ms with zero-copy infinite file size scaling.</td>
    <td>Automatically detects and reads native configurations (<code>.prettierrc</code>, <code>pyproject.toml</code>).</td>
    <td>Runs in a strict WASM Sandbox. All modules are cryptographically signed.</td>
    <td>Supports 70+ languages out-of-the-box via dynamic edge registry.</td>
  </tr>
</table>

---

## üõ†Ô∏è Supported Languages

OmniFormatter downloads the tiny language modules you need **on-the-fly** and caches them forever. 

* üåê **Frontend**: JavaScript, TypeScript, JSX, TSX, Vue, Svelte, Astro, HTML, CSS, SCSS, Sass, Less
* ‚öôÔ∏è **Systems**: Rust, C, C++, Objective-C, Go, Zig, Nim, D
* ‚òï **JVM & .NET**: Java, Kotlin, Scala, Groovy, C#, F#
* üêç **Scripting**: Python, Ruby, PHP, Perl, R, Julia, Lua
* üì± **Mobile**: Swift, Dart
* üìù **Markup & Data**: JSON, YAML, TOML, XML, INI, Markdown, LaTeX
* üìä **DevOps & DB**: Dockerfile, Terraform, Nix, Makefile, SQL, GraphQL
* üß© **And More**: Haskell, Elixir, Erlang, OCaml, Clojure, Lisp, Scheme, Solidity, GDScript, AutoHotkey, Cobol, Fortran, Assembly, Jinja, Liquid, EJS, Handlebars, Twig...

---

## üíª Quick Start

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

## üèóÔ∏è Architecture

```text
     +---------------------------------------+
     ¶         ?? VS Code Extension          ¶
     ¶            (TypeScript)               ¶
     +---------------------------------------+
                        ¶
                [ Zero-Copy IPC ]
                        ¶
                        ?
     +---------------------------------------+
     ¶           ? Worker Pool              ¶
     ¶              (Node.js)                ¶
     +---------------------------------------+
                        ¶
               [ Fast WASM Call ]
                        ¶
                        ?
     +---------------------------------------+
     ¶            ?? WASM Core               ¶
     ¶               (Rust)                  ¶
     +---------------------------------------+
               ¶                   ¶
      [ Loads on Demand ]  [ Reads Workspace ]
               ¶                   ¶
               ?                   ?
     +-------------------+ +-----------------+
     ¶ ?? Lang Modules   ¶ ¶??? Config Adapter¶
     ¶  (.wasm binary)   ¶ ¶ (Native Format) ¶
     +-------------------+ +-----------------+
               ¶
       [ Fetched & Cached ]
               ¶
               ?
     +-------------------+
     ¶  ?? Edge Registry ¶
     ¶(Cloudflare D1/R2) ¶
     +-------------------+
```

## ü§ù Contributing

Contributions are welcome! Adding a language does not require touching the core extension. See our [Language Blueprint](docs/ADD_LANGUAGE_TEMPLATE.md) for how to add a language module in 10 minutes.
