const path = require("path");
const fs = require("fs");
const { runTests } = require("@vscode/test-electron");

function main() {
  const extensionDevelopmentPath = path.resolve(__dirname, '../../extension');
  const extensionTestsPath = path.resolve(__dirname, 'index.js');
  const workspacePath = path.resolve(__dirname);
  
  const vsixFiles = fs.readdirSync(extensionDevelopmentPath).filter(f => f.endsWith('.vsix'));
  if (vsixFiles.length === 0) {
    console.error("No .vsix found");
    process.exit(1);
  }
  const vsixPath = path.join(extensionDevelopmentPath, vsixFiles[0]);

  runTests({
    extensionDevelopmentPath,
    extensionTestsPath,
    launchArgs: [
      workspacePath,
      '--install-extension', vsixPath,
      '--disable-extensions',
      '--user-data-dir', path.join(require('os').tmpdir(), 'vscode-test-user-data-' + Date.now())
    ]
  }).catch(err => {
    console.error('Failed to run tests', err);
    process.exit(1);
  });
}
main();
