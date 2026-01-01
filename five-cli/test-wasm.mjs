import wasmModule from './dist/assets/vm/five_vm_wasm.js';

console.log('WASM module imported successfully');
console.log('Available classes:', Object.keys(wasmModule).filter(k => k.startsWith('Wasm') || k === 'FiveVMWasm'));

try {
  console.log('Creating WasmFiveCompiler...');
  const compiler = new wasmModule.WasmFiveCompiler();
  console.log('WasmFiveCompiler created successfully:', !!compiler);
} catch (error) {
  console.error('Error creating WasmFiveCompiler:', error.message);
}

console.log('Test completed');