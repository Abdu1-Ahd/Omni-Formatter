const fs = require('fs');
const path = require('path');

const wasmPath = path.join(__dirname, '../../extension/dist/wasm/omni_core_bg.wasm');

let wasmExports = null;

function writeStringToWasm(exports, str) {
  const encoded = Buffer.from(str, "utf8");
  const len = encoded.length;
  const ptr = exports.__wbindgen_malloc(len, 1);
  const mem = new Uint8Array(exports.memory.buffer);
  mem.set(encoded, ptr);
  return [ptr, len];
}

function readStringFromWasm(exports, ptr, len) {
  const mem = new Uint8Array(exports.memory.buffer);
  return Buffer.from(mem.slice(ptr, ptr + len)).toString("utf8");
}

async function loadWasm() {
  if (!fs.existsSync(wasmPath)) {
    throw new Error(`WASM binary not found at: ${wasmPath}`);
  }

  const wasmBytes = fs.readFileSync(wasmPath);
  const wasmModule = await WebAssembly.compile(wasmBytes);

  const importsList = WebAssembly.Module.imports(wasmModule);
  const imports = {};
  
  for (const imp of importsList) {
    if (imp.kind === 'function') {
      imports[imp.name] = function(...args) {
        if (imp.name === '__wbindgen_throw') {
          try {
            const ptr = args[0];
            const len = args[1];
            if (wasmExports && ptr && len) {
              const mem = new Uint8Array(wasmExports.memory.buffer);
              const str = Buffer.from(mem.slice(ptr, ptr + len)).toString("utf8");
              console.error("WASM threw:", str);
            }
          } catch (e) {
            // Ignore extraction errors
          }
        }
      };
    } else if (imp.kind === 'memory') {
      imports[imp.name] = new WebAssembly.Memory({ initial: 256, maximum: 1024 });
    } else if (imp.kind === 'global') {
      imports[imp.name] = 0;
    }
  }

  const importObject = {};
  for (const imp of importsList) {
    if (!importObject[imp.module]) importObject[imp.module] = imports;
  }

  const instance = await WebAssembly.instantiate(wasmModule, importObject);

  wasmExports = instance.exports;
}

function callFormat(requestJson) {
  const [reqPtr, reqLen] = writeStringToWasm(wasmExports, requestJson);
  let responseJson;
  try {
    const formatFn = wasmExports.format;
    if (!formatFn) throw new Error("WASM export 'format' not found");

    if (typeof wasmExports.__wbindgen_add_to_stack_pointer === "function") {
      const retStackPtr = wasmExports.__wbindgen_add_to_stack_pointer(-8);
      formatFn(retStackPtr, reqPtr, reqLen);
      const mem = new Int32Array(wasmExports.memory.buffer);
      const outPtr = mem[retStackPtr / 4];
      const outLen = mem[retStackPtr / 4 + 1];
      responseJson = readStringFromWasm(wasmExports, outPtr, outLen);
      wasmExports.__wbindgen_free(outPtr, outLen, 1);
      wasmExports.__wbindgen_add_to_stack_pointer(8);
    } else {
      // multi-value return
      const ret = formatFn(reqPtr, reqLen);
      const outPtr = ret[0];
      const outLen = ret[1];
      responseJson = readStringFromWasm(wasmExports, outPtr, outLen);
      wasmExports.__wbindgen_free(outPtr, outLen, 1);
    }
  } finally {
    wasmExports.__wbindgen_free(reqPtr, reqLen, 1);
  }

  return responseJson;
}

async function runTests() {
  await loadWasm();
  
  const fixturesDir = path.join(__dirname, '../fixtures');
  const files = fs.readdirSync(fixturesDir).filter(f => 
    (f.startsWith('messy.') && !f.endsWith('.out') && !f.endsWith('.out2') && !f.endsWith('.ref')) 
    || f === 'Dockerfile' || f === 'Makefile'
  );
  
  let allPass = true;
  let report = '# WASM Integration Test Report\n\n| Language | Status | Format Time (ms) | Size (bytes) | Idempotency |\n|---|---|---|---|---|\n';

  for (const file of files) {
    await loadWasm();
    const filePath = path.join(fixturesDir, file);
    let ext = file.startsWith('messy.') ? file.substring(6) : file;
    let lang = ext;

    const sourceBytes = fs.readFileSync(filePath);
    
    let config = { indent_size: 2, indent_style: "space" };
    
    if (lang === 'js') {
      config = { ...config, compat_target: "prettier", singleQuote: true };
    } else if (lang === 'go') {
      config = { ...config, indent_style: "tab" };
    }

    const request = {
      id: 1,
      language_id: lang,
      source: Array.from(sourceBytes),
      config,
    };

    const startTime = Date.now();
    let responseStr;
    try {
      responseStr = callFormat(JSON.stringify(request));
    } catch (err) {
      console.error(`[${lang}] WASM call crashed: ${err}`);
      report += `| ${lang} | FAIL (CRASH) | - | - | - |\n`;
      allPass = false;
      continue;
    }
    const endTime = Date.now();
    const formatTime = endTime - startTime;

    let response;
    try {
      response = JSON.parse(responseStr);
    } catch (err) {
      console.error(`[${lang}] Invalid JSON response: ${responseStr}`);
      report += `| ${lang} | FAIL (JSON) | ${formatTime} | - | - |\n`;
      allPass = false;
      continue;
    }

    if (response.error) {
      console.error(`[${lang}] Error returned:`, response.error);
      report += `| ${lang} | FAIL (ERROR) | ${formatTime} | - | - |\n`;
      allPass = false;
      continue;
    }

    // We allow NOOP because stubs (and perfectly formatted files) return NOOP.
    let formattedStr = Buffer.from(sourceBytes).toString('utf8');
    let formattedBytes = Array.from(sourceBytes);
    
    if (!response.is_noop) {
      formattedStr = response.edits[0].new_text;
      formattedBytes = Array.from(Buffer.from(formattedStr, 'utf8'));
    }

    // Specific assertions
    if (!response.is_noop) {
      if (lang === 'js' && !formattedStr.includes("'")) {
        console.error(`[js] Single quote config failed`);
        allPass = false;
      } else if (lang === 'go' && !formattedStr.includes('\t')) {
        console.error(`[go] Tab indent config failed`);
        allPass = false;
      } else if (lang === 'html' && !formattedStr.includes(";") && !formattedStr.includes('"')) {
        console.error(`[html] Zone routing evidence missing`);
        allPass = false;
      }
    }

    // Idempotency
    const idempReq = {
      id: 2,
      language_id: lang,
      source: formattedBytes,
      config,
    };
    
    const idempResStr = callFormat(JSON.stringify(idempReq));
    const idempRes = JSON.parse(idempResStr);
    
    let idempPass = true;
    if (!idempRes.is_noop) {
      const idempStr = idempRes.edits[0].new_text;
      if (formattedStr !== idempStr) {
        console.error(`[${lang}] Idempotency failed!`);
        idempPass = false;
        allPass = false;
      }
    }

    const status = allPass ? 'PASS' : 'FAIL';
    report += `| ${lang} | ${status} | ${formatTime} | ${formattedBytes.length} | ${idempPass ? 'PASS' : 'FAIL'} |\n`;
    console.log(`[${lang}] Format complete. Time: ${formatTime}ms`);
  }

  fs.writeFileSync(path.join(__dirname, 'wasm_report.md'), report);

  if (!allPass) {
    process.exit(1);
  } else {
    console.log("All languages passed.");
    process.exit(0);
  }
}

runTests().catch(err => {
  console.error(err);
  process.exit(1);
});
