const fs = require('fs');
const path = require('path');

// Load the WASM module from pkg-node
const wasmModule = require('./pkg-node/five_vm_wasm.js');

const { WasmFiveCompiler, WasmCompilationOptions } = wasmModule;

const source = `pub add(a: u64, b: u64) -> u64 {
    return a + b;
}

pub mint_to(amount: u64) {
    return;
}

pub transfer(from: u64, to: u64, amount: u64) {
    return;
}`;

console.log('=== Testing ABI Generation with Parameters ===\n');

try {
  console.log('Creating compiler...');
  const compiler = new WasmFiveCompiler();
  
  console.log('Creating compilation options...');
  const options = new WasmCompilationOptions()
    .with_mode('deployment')
    .with_optimization_level('production')
    .with_v2_preview(true);
  
  console.log('Compiling...\n');
  const result = compiler.compile(source, options);
  
  console.log('\n=== Compilation Result ===');
  console.log('Success:', result.success);
  console.log('Bytecode size:', result.bytecode_size);
  
  if (result.success) {
    const abiString = result.get_abi();
    console.log('\n=== Raw ABI from get_abi() ===');
    console.log('ABI type:', typeof abiString);
    console.log('ABI string length:', abiString ? abiString.length : 'undefined');
    
    if (abiString) {
      try {
        const abi = JSON.parse(abiString);
        console.log('\n=== Parsed ABI Structure ===');
        console.log('Functions:', abi.functions?.length);
        console.log('Fields:', abi.fields?.length);
        console.log('Version:', abi.version);
        
        if (abi.functions && abi.functions.length > 0) {
          console.log('\n=== First Function (add) ===');
          console.log(JSON.stringify(abi.functions[0], null, 2));
          
          if (abi.functions.length > 1) {
            console.log('\n=== Second Function (mint_to) ===');
            console.log(JSON.stringify(abi.functions[1], null, 2));
          }
          
          if (abi.functions.length > 2) {
            console.log('\n=== Third Function (transfer) ===');
            console.log(JSON.stringify(abi.functions[2], null, 2));
          }
        }
      } catch (e) {
        console.log('Failed to parse ABI:', e.message);
        console.log('Raw ABI string:', abiString.substring(0, 500));
      }
    }
  } else {
    console.log('Compilation failed!');
    console.log('Errors:', result.get_formatted_errors_terminal());
  }
} catch (error) {
  console.error('Error during test:', error);
  console.error('Stack:', error.stack);
}
