const fs = require('fs');
const path = require('path');

const extensions = [
  "js", "mjs", "cjs", "ts", "mts", "cts", "jsx", "tsx", "html", "htm", "css", "scss", "sass", "less", "vue", "svelte", "astro", "c", "h", "cpp", "hpp", "cc", "cxx", "rs", "go", "zig", "nim", "d", "f90", "f95", "cob", "cbl", "asm", "s", "java", "class", "jar", "kt", "kts", "scala", "sc", "groovy", "cs", "fs", "fsi", "fsx", "py", "pyw", "ipynb", "rb", "php", "pl", "pm", "r", "R", "jl", "lua", "sh", "bash", "zsh", "ps1", "psm1", "fish", "awk", "sed", "swift", "m", "mm", "dart", "sql", "graphql", "gql", "tf", "hcl", "nix", "Dockerfile", "Makefile", "json", "json5", "yaml", "yml", "toml", "xml", "ini", "csv", "hs", "lhs", "erl", "hrl", "ex", "exs", "clj", "cljs", "ml", "mli", "lisp", "lsp", "scm", "ss", "sol", "vy", "ahk", "au3", "gd", "jinja", "jinja2", "liquid", "ejs", "hbs", "handlebars", "twig", "md", "markdown", "tex", "adoc", "asciidoc"
];

const fixturesDir = path.join(__dirname, '../tests/fixtures');
if (!fs.existsSync(fixturesDir)) {
  fs.mkdirSync(fixturesDir, { recursive: true });
}

for (const ext of extensions) {
  const file = path.join(fixturesDir, ext === "Dockerfile" || ext === "Makefile" ? ext : `messy.${ext}`);
  if (!fs.existsSync(file)) {
    fs.writeFileSync(file, `/* default messy content for ${ext} */\nconsole.log("hello");\n`);
  }
}
console.log("Generated fixtures for", extensions.length, "extensions.");
