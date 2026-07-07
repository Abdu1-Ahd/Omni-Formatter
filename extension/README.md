<div align="center">

**One extension. Every language. Zero configuration.**


[![Build](https://img.shields.io/github/actions/workflow/status/Abdu1-Ahd/Omni-Formatter/ci.yml?style=for-the-badge&logo=githubactions&logoColor=white)](https://github.com/Abdu1-Ahd/Omni-Formatter/actions)
<br/>
[![Rust](https://img.shields.io/badge/Core-Rust-CE422B?style=for-the-badge&logo=rust)](https://www.rust-lang.org/)
[![WebAssembly](https://img.shields.io/badge/Runtime-WebAssembly-654ff0?style=for-the-badge&logo=webassembly)](https://webassembly.org/)
[![Registry](https://img.shields.io/badge/Registry-GitHub%20Pages-121013?style=for-the-badge&logo=github)](https://abdu1-ahd.github.io/Omni-Formatter/)

[Github](https://github.com/Abdu1-Ahd/Omni-Formatter)

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
<td>Supports 70+ languages out-of-the-box via the GitHub Pages registry.</td>
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

Set OmniFormatter as your default formatter in your `settings.json`:

```json
{
  "editor.defaultFormatter": "Abdu1-Ahd.omni-formatter"
}
```

That's it. Keep using your existing configuration files (e.g., `.prettierrc`, `rustfmt.toml`), and OmniFormatter will adapt automatically. 

**Pro Tip:** You do NOT need to enable `"editor.formatOnSave"`. OmniFormatter natively formats your code whenever you manually save (`Ctrl+S` / `Cmd+S`) and intelligently ignores VS Code's Auto Save (e.g., `onFocusChange`) to prevent disrupting your incomplete code!

---

## 🏗️ Architecture

```text
┌───────────────────────────────────┐
│       🔌 VS Code Extension        │
│           (TypeScript)            │
└─────────────────┬─────────────────┘
                  │
          [ Zero-Copy IPC ]
                  │
                  ▼
┌───────────────────────────────────┐
│          ⚡ Worker Pool           │
│             (Node.js)             │
└─────────────────┬─────────────────┘
                  │
         [ Fast WASM Call ]
                  │
                  ▼
┌───────────────────────────────────┐
│           ⚙️ WASM Core            │
│              (Rust)               │
└────────┬─────────────────┬────────┘
         │                 │
[ Loads on Demand ] [ Reads Configs ]
         │                 │
         ▼                 ▼
 ┌───────────────┐ ┌───────────────┐
 │📦 Lang Modules│ │🛠️ Config Adpt│
 │ (.wasm binary)│ │(Native Format)│
 └───────┬───────┘ └───────────────┘
         │
[ Fetched & Cached]
         │
         ▼
 ┌───────────────┐
 │ ☁️ Registry   │
 │(GitHub Pages) │
 └───────────────┘
```

## 🤝 Contributing

Contributions are welcome! If you encounter any issues or formatting errors with the extension, please report them so we can fix them. We also deeply appreciate any [feedback](https://abdu1-ahd.github.io/Omni-Formatter/#feedback) or feature requests.
