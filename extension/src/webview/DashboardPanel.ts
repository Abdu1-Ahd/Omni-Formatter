import * as vscode from "vscode";

/**
 * Manages the OmniFormatter Interactive Config Dashboard Webview.
 * Singleton pattern — only one panel open at a time.
 */
export class DashboardPanel {
  public static readonly viewType = "omniFormatterDashboard";

  private static _instance: DashboardPanel | undefined;
  private readonly _panel: vscode.WebviewPanel;
  private readonly _extensionUri: vscode.Uri;
  private _disposables: vscode.Disposable[] = [];

  public static createOrShow(extensionContext: vscode.ExtensionContext): void {
    const column = vscode.window.activeTextEditor
      ? vscode.window.activeTextEditor.viewColumn
      : undefined;

    // If panel already exists, reveal it.
    if (DashboardPanel._instance) {
      DashboardPanel._instance._panel.reveal(column);
      return;
    }

    // Create a new panel.
    const panel = vscode.window.createWebviewPanel(
      DashboardPanel.viewType,
      "OmniFormatter Dashboard",
      column || vscode.ViewColumn.One,
      {
        enableScripts: true,
        localResourceRoots: [
          vscode.Uri.joinPath(extensionContext.extensionUri, "media"),
        ],
        retainContextWhenHidden: true,
      }
    );

    DashboardPanel._instance = new DashboardPanel(panel, extensionContext.extensionUri);
  }

  private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
    this._panel = panel;
    this._extensionUri = extensionUri;

    // Load initial content
    this._update();

    // Listen for panel disposal
    this._panel.onDidDispose(() => this.dispose(), null, this._disposables);

    // Handle messages from the webview
    this._panel.webview.onDidReceiveMessage(
      async (message) => {
        switch (message.command) {
          case "getConfig": {
            const config = vscode.workspace.getConfiguration("omniFormatter");
            const settings = {
              enable: config.get("enable", true),
              logLevel: config.get("logLevel", "warn"),
            };
            this._panel.webview.postMessage({ command: "configLoaded", settings });
            break;
          }
          case "updateConfig": {
            const config = vscode.workspace.getConfiguration("omniFormatter");
            const { key, value } = message;
            await config.update(key, value, vscode.ConfigurationTarget.Workspace);
            vscode.window.showInformationMessage(`OmniFormatter: Setting "${key}" updated.`);
            break;
          }
        }
      },
      null,
      this._disposables
    );
  }

  private _update(): void {
    const version = vscode.extensions.getExtension("Abdu1-Ahd.omni-formatter")?.packageJSON.version || "0.1.3";
    this._panel.webview.html = this._getHtmlContent(version);
  }

  private _getHtmlContent(version: string): string {
    // Nonce for Content Security Policy
    const nonce = getNonce();
    return getDashboardHtml(nonce, version);
  }

  public dispose(): void {
    DashboardPanel._instance = undefined;
    this._panel.dispose();
    while (this._disposables.length) {
      const x = this._disposables.pop();
      if (x) {
        x.dispose();
      }
    }
  }
}

function getNonce(): string {
  let text = "";
  const possible = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  for (let i = 0; i < 32; i++) {
    text += possible.charAt(Math.floor(Math.random() * possible.length));
  }
  return text;
}

function getDashboardHtml(nonce: string, version: string): string {
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1.0" />
  <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src 'nonce-${nonce}'; script-src 'nonce-${nonce}';" />
  <title>OmniFormatter Dashboard</title>
  <style nonce="${nonce}">
    :root {
      --bg: var(--vscode-editor-background);
      --fg: var(--vscode-editor-foreground);
      --sidebar-bg: var(--vscode-sideBar-background, #1e1e2e);
      --border: var(--vscode-panel-border, #3c3c4e);
      --accent: var(--vscode-button-background, #7c3aed);
      --accent-fg: var(--vscode-button-foreground, #ffffff);
      --input-bg: var(--vscode-input-background, #2d2d3f);
      --input-border: var(--vscode-input-border, #555);
      --hover-bg: var(--vscode-list-hoverBackground, #2a2a3e);
      --font: var(--vscode-font-family, 'Segoe UI', sans-serif);
      --font-size: var(--vscode-font-size, 13px);
      --radius: 8px;
      --transition: 0.18s ease;
    }

    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    body {
      background: var(--bg);
      color: var(--fg);
      font-family: var(--font);
      font-size: var(--font-size);
      min-height: 100vh;
      display: flex;
      flex-direction: column;
    }

    /* ── Header ── */
    header {
      display: flex;
      align-items: center;
      gap: 12px;
      padding: 18px 24px;
      border-bottom: 1px solid var(--border);
      background: var(--sidebar-bg);
    }

    .logo-mark {
      width: 32px; height: 32px;
      background: linear-gradient(135deg, #7c3aed, #06b6d4);
      border-radius: 8px;
      display: flex; align-items: center; justify-content: center;
      font-size: 18px;
      flex-shrink: 0;
    }

    header h1 {
      font-size: 15px;
      font-weight: 600;
      letter-spacing: -0.3px;
    }

    header p {
      font-size: 11px;
      opacity: 0.55;
      margin-top: 1px;
    }

    .status-pill {
      margin-left: auto;
      padding: 3px 10px;
      border-radius: 20px;
      font-size: 11px;
      font-weight: 600;
      background: rgba(16, 185, 129, 0.15);
      color: #10b981;
      border: 1px solid rgba(16, 185, 129, 0.3);
    }

    /* ── Main layout ── */
    main {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 0;
      flex: 1;
    }

    /* ── Settings panel ── */
    .settings-panel {
      padding: 24px;
      border-right: 1px solid var(--border);
      display: flex;
      flex-direction: column;
      gap: 20px;
    }

    .panel-label {
      font-size: 10px;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.8px;
      opacity: 0.4;
      margin-bottom: 2px;
    }

    .setting-group {
      display: flex;
      flex-direction: column;
      gap: 12px;
    }

    .setting-item {
      background: var(--sidebar-bg);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 14px 16px;
      display: flex;
      align-items: center;
      gap: 14px;
      transition: border-color var(--transition), background var(--transition);
      cursor: pointer;
    }

    .setting-item:hover { border-color: var(--accent); }

    .setting-info { flex: 1; }
    .setting-name { font-weight: 600; font-size: 13px; }
    .setting-desc { font-size: 11px; opacity: 0.5; margin-top: 2px; }

    /* Toggle switch */
    .toggle {
      position: relative;
      width: 36px; height: 20px;
      flex-shrink: 0;
    }

    .toggle input { opacity: 0; width: 0; height: 0; }

    .toggle-track {
      position: absolute;
      inset: 0;
      border-radius: 20px;
      background: var(--input-bg);
      border: 1px solid var(--input-border);
      cursor: pointer;
      transition: background var(--transition), border-color var(--transition);
    }

    .toggle-track::after {
      content: '';
      position: absolute;
      top: 2px; left: 2px;
      width: 14px; height: 14px;
      border-radius: 50%;
      background: #888;
      transition: transform var(--transition), background var(--transition);
    }

    .toggle input:checked + .toggle-track { background: var(--accent); border-color: var(--accent); }
    .toggle input:checked + .toggle-track::after { transform: translateX(16px); background: #fff; }

    /* Select */
    select {
      background: var(--input-bg);
      color: var(--fg);
      border: 1px solid var(--input-border);
      border-radius: 6px;
      padding: 5px 10px;
      font-family: var(--font);
      font-size: 12px;
      cursor: pointer;
      transition: border-color var(--transition);
      outline: none;
    }

    select:focus, select:hover { border-color: var(--accent); }

    /* ── Sandbox panel ── */
    .sandbox-panel {
      padding: 24px;
      display: flex;
      flex-direction: column;
      gap: 16px;
    }

    .sandbox-editor-wrap {
      flex: 1;
      display: flex;
      flex-direction: column;
      gap: 10px;
    }

    .sandbox-toolbar {
      display: flex;
      align-items: center;
      gap: 8px;
    }

    .lang-selector {
      flex: 1;
    }

    .btn {
      padding: 6px 16px;
      border-radius: 6px;
      border: none;
      background: var(--accent);
      color: var(--accent-fg);
      font-family: var(--font);
      font-size: 12px;
      font-weight: 600;
      cursor: pointer;
      display: flex; align-items: center; gap: 6px;
      transition: opacity var(--transition), transform var(--transition);
    }

    .btn:hover { opacity: 0.88; transform: translateY(-1px); }
    .btn:active { transform: translateY(0); }

    .btn.secondary {
      background: var(--input-bg);
      color: var(--fg);
      border: 1px solid var(--border);
    }

    .editor-panes {
      display: grid;
      grid-template-columns: 1fr 1fr;
      gap: 10px;
      flex: 1;
    }

    .editor-pane {
      display: flex;
      flex-direction: column;
      gap: 6px;
    }

    .pane-label {
      font-size: 10px;
      font-weight: 700;
      text-transform: uppercase;
      letter-spacing: 0.8px;
      opacity: 0.4;
    }

    textarea, .output-box {
      flex: 1;
      min-height: 200px;
      background: var(--sidebar-bg);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      color: var(--fg);
      font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
      font-size: 12px;
      padding: 12px;
      resize: vertical;
      outline: none;
      transition: border-color var(--transition);
      line-height: 1.6;
    }

    textarea:focus { border-color: var(--accent); }

    .output-box {
      white-space: pre-wrap;
      opacity: 0.85;
      font-family: 'Cascadia Code', 'Fira Code', 'Consolas', monospace;
      overflow-y: auto;
    }

    .output-box.placeholder { opacity: 0.3; font-style: italic; }

    /* ── Footer ── */
    footer {
      padding: 10px 24px;
      border-top: 1px solid var(--border);
      display: flex;
      align-items: center;
      gap: 8px;
      font-size: 11px;
      opacity: 0.45;
    }

    footer span { margin-left: auto; }

    /* Animations */
    @keyframes fadeIn {
      from { opacity: 0; transform: translateY(6px); }
      to { opacity: 1; transform: translateY(0); }
    }

    .settings-panel, .sandbox-panel {
      animation: fadeIn 0.25s ease both;
    }

    .sandbox-panel { animation-delay: 0.05s; }
  </style>
</head>
<body>

<header>
  <div class="logo-mark">⬡</div>
  <div>
    <h1>OmniFormatter Dashboard</h1>
    <p>Configure settings and preview formatting in real-time</p>
  </div>
  <div class="status-pill" id="status-pill">● Active</div>
</header>

<main>
  <!-- Settings Panel -->
  <div class="settings-panel">
    <div class="panel-label">Configuration</div>

    <div class="setting-group">
      <label class="setting-item" for="toggle-enable">
        <div class="setting-info">
          <div class="setting-name">Enable OmniFormatter</div>
          <div class="setting-desc">Format documents using OmniFormatter on save or on demand</div>
        </div>
        <label class="toggle">
          <input type="checkbox" id="toggle-enable" checked />
          <span class="toggle-track"></span>
        </label>
      </label>

      <div class="setting-item">
        <div class="setting-info">
          <div class="setting-name">Log Level</div>
          <div class="setting-desc">Output channel verbosity</div>
        </div>
        <select id="select-log-level">
          <option value="error">Error</option>
          <option value="warn" selected>Warn</option>
          <option value="info">Info</option>
          <option value="debug">Debug</option>
        </select>
      </div>
    </div>

    <div style="margin-top: auto; display: flex; flex-direction: column; gap: 10px;">
      <div class="panel-label">Status</div>
      <div class="setting-item" style="cursor: default;">
        <div class="setting-info">
          <div class="setting-name">Last Format Result</div>
          <div class="setting-desc" id="last-format-info">No formatting run yet this session</div>
        </div>
      </div>
    </div>
  </div>

  <!-- Sandbox Panel -->
  <div class="sandbox-panel">
    <div class="panel-label">Live Formatting Sandbox</div>

    <div class="sandbox-editor-wrap">
      <div class="sandbox-toolbar">
        <select class="lang-selector" id="sandbox-lang">
          <option value="js">JavaScript</option>
          <option value="ts">TypeScript</option>
          <option value="py">Python</option>
          <option value="rs">Rust</option>
          <option value="css">CSS</option>
        </select>
        <button class="btn secondary" onclick="clearSandbox()">✕ Clear</button>
        <button class="btn" onclick="previewFormat()">▶ Preview</button>
      </div>

      <div class="editor-panes">
        <div class="editor-pane">
          <div class="pane-label">Input</div>
          <textarea id="sandbox-input" spellcheck="false" placeholder="Paste your code here…">const x=1;const y=2;if(x>y){console.log('hello')} </textarea>
        </div>
        <div class="editor-pane">
          <div class="pane-label">Output (preview)</div>
          <div class="output-box placeholder" id="sandbox-output">Hit "▶ Preview" to see result…</div>
        </div>
      </div>
    </div>
  </div>
</main>

<footer>
  OmniFormatter v${version} · Universal WASM Formatter
  <span id="footer-clock"></span>
</footer>

<script nonce="${nonce}">
  const vscode = acquireVsCodeApi();

  // Request current config on load
  vscode.postMessage({ command: 'getConfig' });

  // Receive config from extension
  window.addEventListener('message', (event) => {
    const message = event.data;
    if (message.command === 'configLoaded') {
      const { settings } = message;
      document.getElementById('toggle-enable').checked = settings.enable;
      document.getElementById('select-log-level').value = settings.logLevel;
    }
  });

  // Setting change handlers
  document.getElementById('toggle-enable').addEventListener('change', (e) => {
    vscode.postMessage({ command: 'updateConfig', key: 'enable', value: e.target.checked });
    document.getElementById('status-pill').textContent = e.target.checked ? '● Active' : '○ Disabled';
    document.getElementById('status-pill').style.background = e.target.checked ? 'rgba(16,185,129,0.15)' : 'rgba(239,68,68,0.15)';
    document.getElementById('status-pill').style.color = e.target.checked ? '#10b981' : '#ef4444';
    document.getElementById('status-pill').style.borderColor = e.target.checked ? 'rgba(16,185,129,0.3)' : 'rgba(239,68,68,0.3)';
  });

  document.getElementById('select-log-level').addEventListener('change', (e) => {
    vscode.postMessage({ command: 'updateConfig', key: 'logLevel', value: e.target.value });
  });

  // Sandbox preview (client-side beautifier simulation)
  function previewFormat() {
    const input = document.getElementById('sandbox-input').value.trim();
    const output = document.getElementById('sandbox-output');
    if (!input) {
      output.textContent = '// Nothing to format';
      output.className = 'output-box placeholder';
      return;
    }
    // Basic cosmetic preview — real formatting uses the extension host
    const formatted = input
      .replace(/;(\\S)/g, ';\\n$1')
      .replace(/\\{(\\S)/g, '{\\n  $1')
      .replace(/(\\S)\\}/g, '\\n$1\\n}')
      .split('\n').map(l => l.trimEnd()).join('\n');
    output.textContent = formatted;
    output.className = 'output-box';
  }

  function clearSandbox() {
    document.getElementById('sandbox-input').value = '';
    document.getElementById('sandbox-output').textContent = 'Hit "▶ Preview" to see result…';
    document.getElementById('sandbox-output').className = 'output-box placeholder';
  }

  // Live clock in footer
  function updateClock() {
    document.getElementById('footer-clock').textContent = new Date().toLocaleTimeString();
  }
  setInterval(updateClock, 1000);
  updateClock();
</script>
</body>
</html>`;
}
