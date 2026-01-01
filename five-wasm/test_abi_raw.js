const fs = require('fs');
const wasmModule = require('./pkg-node/five_vm_wasm.js');
const { WasmFiveCompiler, WasmCompilationOptions } = wasmModule;

const source = `pub add(a: u64, b: u64) -> u64 {
    return a + b;
}`;

const compiler = new WasmFiveCompiler();
const options = new WasmCompilationOptions().with_mode('deployment');
const result = compiler.compile(source, options);

if (result.success) {
  const abiString = result.get_abi();
  console.log('Raw ABI JSON:');
  console.log(abiString);
  console.log('\n=== Parsed ===');
  try {
    const parsed = JSON.parse(abiString);
    console.log(JSON.stringify(parsed, null, 2));
  } catch (e) {
    console.log('Parse error:', e.message);
  }
}
