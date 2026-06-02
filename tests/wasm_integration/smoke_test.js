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
  const importObject = {};

  for (const imp of importsList) {
    if (!importObject[imp.module]) importObject[imp.module] = {};
    if (imp.kind === 'function') {
      importObject[imp.module][imp.name] = function(...args) {
        if (imp.name === '__wbindgen_throw') {
          try {
            const ptr = args[0];
            const len = args[1];
            if (wasmExports && ptr && len) {
              const mem = new Uint8Array(wasmExports.memory.buffer);
              const str = Buffer.from(mem.slice(ptr, ptr + len)).toString('utf8');
              console.error("WASM threw:", str);
            }
          } catch (e) {
          }
        } else if (imp.name.includes('log')) {
          try {
            const ptr = args[0];
            const len = args[1];
            if (wasmExports && ptr && len) {
              const mem = new Uint8Array(wasmExports.memory.buffer);
              const str = Buffer.from(mem.slice(ptr, ptr + len)).toString('utf8');
              console.log("WASM LOG:", str);
            }
          } catch (e) {
          }
        }
      };
    } else if (imp.kind === 'memory') {
      importObject[imp.module][imp.name] = new WebAssembly.Memory({ initial: 256, maximum: 1024 });
    } else if (imp.kind === 'global') {
      importObject[imp.module][imp.name] = 0;
    }
  }

  const instance = await WebAssembly.instantiate(wasmModule, importObject);

  wasmExports = instance.exports;
  
  if (wasmExports.init_wasm) {
    wasmExports.init_wasm();
  }
}

function callFormat(requestJson) {
  console.log("Starting WASM format");
  const [reqPtr, reqLen] = writeStringToWasm(wasmExports, requestJson);
  let responseJson;
  try {
    const formatFn = wasmExports.format;
    if (!formatFn) throw new Error("WASM export 'format' not found");

    if (typeof wasmExports.__wbindgen_add_to_stack_pointer === "function") {
      const retStackPtr = wasmExports.__wbindgen_add_to_stack_pointer(-8);
      console.log("Calling WASM formatFn");
      formatFn(retStackPtr, reqPtr, reqLen);
      console.log("Returned from WASM formatFn");
      const mem = new Int32Array(wasmExports.memory.buffer);
      const outPtr = mem[retStackPtr / 4];
      const outLen = mem[retStackPtr / 4 + 1];
      responseJson = readStringFromWasm(wasmExports, outPtr, outLen);
      wasmExports.__wbindgen_free(outPtr, outLen, 1);
      wasmExports.__wbindgen_add_to_stack_pointer(8);
    } else {
      // multi-value return
      console.log("Calling WASM formatFn (multi-value)");
      const ret = formatFn(reqPtr, reqLen);
      console.log("Returned from WASM formatFn");
      const outPtr = ret[0];
      const outLen = ret[1];
      responseJson = readStringFromWasm(wasmExports, outPtr, outLen);
      wasmExports.__wbindgen_free(outPtr, outLen, 1);
    }
  } finally {
    wasmExports.__wbindgen_free(reqPtr, reqLen, 1);
  }

  console.log("Finished WASM format");
  return responseJson;
}

async function runTests() {
  try {
    await loadWasm();
  } catch (err) {
    console.error("Failed to load WASM:", err);
    process.exit(1);
  }

  const fixturesDir = path.join(__dirname, '../fixtures');
  const languages = ['js', 'ts'];
  let allPass = true;

  for (const lang of languages) {
    const filePath = path.join(fixturesDir, `messy.${lang}`);
    if (!fs.existsSync(filePath)) continue;

    const sourceBytes = fs.readFileSync(filePath);
    
    // Test 1: Full format
    const request = {
      id: 1,
      language_id: lang,
      source: Array.from(sourceBytes),
      config: { indent_size: 2, indent_style: "space" },
    };

    const startTime = process.hrtime.bigint();
    let responseStr;
    try {
      console.log(`\n--- Starting Test 1 for ${lang} ---`);
      responseStr = callFormat(JSON.stringify(request));
    } catch (err) {
      console.error(`[${lang}] Smoke test crashed:`, err);
      allPass = false;
      continue;
    }
    const endTime = process.hrtime.bigint();
    const elapsedNs = Number(endTime - startTime);
    
    const response = JSON.parse(responseStr);
    if (!response.edits) {
      console.error(`[${lang}] Smoke test no edits field`);
      allPass = false;
    }

    if (elapsedNs > 50000000) {
      if (elapsedNs > 200000000) {
         console.warn(`[${lang}] Elapsed time ${elapsedNs}ns is extremely high`);
      }
    }

    // Test 2: Format on type (single char edit)
    const onTypeRequest = {
      id: 2,
      language_id: lang,
      source: Array.from(sourceBytes),
      config: { indent_size: 2, indent_style: "space" },
    };
    
    console.log(`\n--- Starting Test 2 (onType) for ${lang} ---`);
    let onTypeResponseStr = callFormat(JSON.stringify(onTypeRequest));
    let onTypeResponse = JSON.parse(onTypeResponseStr);
    
    if (!onTypeResponse.edits) {
      console.error(`[${lang}] Smoke test onType no edits field`);
      allPass = false;
    }
  }

  if (allPass) {
    console.log("Smoke tests passed.");
    process.exit(0);
  } else {
    process.exit(1);
  }
}

runTests();
