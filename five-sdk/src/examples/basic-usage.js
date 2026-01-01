/**
 * Five SDK - Basic Usage Examples
 *
 * This file demonstrates the core Five SDK functionality with correct terminology:
 * - Five scripts (not contracts)
 * - Script accounts (not program IDs)
 * - Bytecode compilation and deployment
 * - Script function execution
 */
import { FiveSDK, FiveScriptSource, compileAndExecuteLocally } from '../index.js';
// No Solana client imports needed in client-agnostic SDK!
// ==================== Example 1: Basic SDK Setup ====================
async function example1_BasicSetup() {
    console.log('=== Example 1: Basic SDK Setup ===');
    // Method 1: Use convenience factory methods
    const devnetSDK = FiveSDK.devnet();
    const mainnetSDK = FiveSDK.mainnet();
    const localSDK = FiveSDK.localnet();
    // Method 2: Manual configuration
    const customSDK = new FiveSDK({
        fiveVMProgramId: '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo',
        debug: true,
        network: 'devnet'
    });
    console.log('SDK Configuration:', customSDK.getConfig());
}
// ==================== Example 2: Script Compilation ====================
async function example2_ScriptCompilation() {
    console.log('=== Example 2: Script Compilation ===');
    const sdk = FiveSDK.devnet();
    // Five script source code
    const scriptSource = `
    script SimpleCalculator {
      // Add two numbers
      add(a: u64, b: u64) -> u64 {
        return a + b;
      }
      
      // Main entry point (0 parameters)
      test() -> u64 {
        return add(5, 3);
      }
    }
  `;
    try {
        // Compile the script using static method
        const compilation = await FiveSDK.compile(scriptSource, {
            optimize: true,
            debug: false
        });
        if (compilation.success) {
            console.log('✓ Compilation successful!');
            console.log(`  Bytecode size: ${compilation.bytecode?.length} bytes`);
            console.log(`  Functions found: ${compilation.metadata?.functions.length}`);
            console.log(`  Compilation time: ${compilation.metadata?.compilationTime}ms`);
            // Show function information
            compilation.metadata?.functions.forEach(func => {
                console.log(`    Function: ${func.name}(${func.parameters.map(p => `${p.name}: ${p.type}`).join(', ')})`);
            });
        }
        else {
            console.log('✗ Compilation failed:');
            compilation.errors?.forEach(error => {
                console.log(`  ${error.severity}: ${error.message} (line ${error.line})`);
            });
        }
    }
    catch (error) {
        console.error('Compilation error:', error);
    }
}
// ==================== Example 3: Deployment Instruction Generation ====================
async function example3_DeploymentInstructionGeneration() {
    console.log('=== Example 3: Deployment Instruction Generation ===');
    const scriptSource = `
    script HelloWorld {
      test() -> u64 {
        return 42;
      }
    }
  `;
    try {
        console.log('Compiling script...');
        // First compile the script
        const compilation = await FiveSDK.compile(scriptSource, { optimize: true });
        if (compilation.success && compilation.bytecode) {
            console.log('✓ Script compiled successfully!');
            console.log(`  Bytecode size: ${compilation.bytecode.length} bytes`);
            // Generate deployment instruction (client-agnostic)
            const deployerAddress = 'DeployerPublicKey1111111111111111111111111';
            const deploymentData = await FiveSDK.generateDeployInstruction(compilation.bytecode, deployerAddress, { debug: true });
            console.log('✓ Deployment instruction generated!');
            console.log(`  Script account (PDA): ${deploymentData.scriptAccount}`);
            console.log(`  Required signers: ${deploymentData.requiredSigners.join(', ')}`);
            console.log(`  Estimated cost: ${(deploymentData.estimatedCost / 1e9).toFixed(6)} SOL`);
            console.log(`  Instruction data size: ${deploymentData.instruction.data.length} chars (base64)`);
            console.log(`  Account count: ${deploymentData.instruction.accounts.length}`);
            return deploymentData.scriptAccount;
        }
        else {
            console.log('✗ Compilation failed');
            compilation.errors?.forEach(error => {
                console.log(`  ${error.severity}: ${error.message} (line ${error.line})`);
            });
        }
    }
    catch (error) {
        console.error('Deployment instruction generation error:', error);
    }
    return null;
}
// ==================== Example 4: Execution Instruction Generation ====================
async function example4_ExecutionInstructionGeneration() {
    console.log('=== Example 4: Execution Instruction Generation ===');
    // First, get a script account from deployment example
    const scriptAccount = await example3_DeploymentInstructionGeneration();
    if (!scriptAccount) {
        console.log('Skipping execution example - no script account');
        return;
    }
    try {
        console.log(`Generating execution instruction for script: ${scriptAccount}`);
        // Generate execution instruction (client-agnostic)
        const executionData = await FiveSDK.generateExecuteInstruction(scriptAccount, 'test', // Function name
        [], // No parameters
        [], // No additional accounts
        undefined, { debug: true, computeUnitLimit: 50000 });
        console.log('✓ Execution instruction generated!');
        console.log(`  Script account: ${executionData.scriptAccount}`);
        console.log(`  Function: ${executionData.parameters.function}`);
        console.log(`  Parameter count: ${executionData.parameters.count}`);
        console.log(`  Estimated compute units: ${executionData.estimatedComputeUnits}`);
        console.log(`  Instruction data size: ${executionData.instruction.data.length} chars (base64)`);
        console.log(`  Account count: ${executionData.instruction.accounts.length}`);
        // Show how to use the instruction with any Solana client
        console.log('\n📝 Usage with any Solana client:');
        console.log('  1. Decode base64 instruction data');
        console.log('  2. Create transaction with accounts and program ID');
        console.log('  3. Submit transaction to Solana network');
        console.log('  4. Parse transaction logs for execution results');
    }
    catch (error) {
        console.error('Execution instruction generation error:', error);
    }
}
// ==================== Example 5: Parameter Handling ====================
async function example5_ParameterHandling() {
    console.log('=== Example 5: Parameter Handling ===');
    const sdk = FiveSDK.devnet();
    // Script with parameters
    const scriptSource = `
    script MathOperations {
      // Function that requires parameters
      multiply(a: u64, b: u64) -> u64 {
        return a * b;
      }
      
      // Function with mixed parameter types
      process(number: u32, flag: bool, name: string) -> u64 {
        if (flag) {
          return number * 2;
        } else {
          return number;
        }
      }
    }
  `;
    try {
        // Compile the script using static method
        const compilation = await FiveSDK.compile(scriptSource);
        if (compilation.success && compilation.bytecode) {
            console.log('✓ Script compiled successfully!');
            // Direct bytecode execution with parameters (for testing)
            console.log('Testing parameter handling...');
            // Test multiply function with parameters
            const multiplyResult = await FiveSDK.executeLocally(compilation.bytecode, 'multiply', // Function name
            [10, 5], // Parameters
            { debug: true, trace: true });
            if (multiplyResult.success) {
                console.log('✓ Multiply function executed successfully!');
                console.log(`  Result: ${JSON.stringify(multiplyResult.result)}`);
            }
            // Test process function with mixed types
            const processResult = await FiveSDK.executeLocally(compilation.bytecode, 'process', // Function name
            [42, true, "test"], // u32, bool, string parameters
            { debug: true, trace: true });
            if (processResult.success) {
                console.log('✓ Process function executed successfully!');
                console.log(`  Result: ${JSON.stringify(processResult.result)}`);
            }
        }
        else {
            console.log('✗ Script compilation failed');
        }
    }
    catch (error) {
        console.error('Parameter handling error:', error);
    }
}
// ==================== Example 6: Error Handling ====================
async function example6_ErrorHandling() {
    console.log('=== Example 6: Error Handling ===');
    const sdk = FiveSDK.devnet();
    // Intentionally broken script
    const brokenScript = `
    script BrokenScript {
      test() -> u64 {
        return unknown_function(); // This will cause a compilation error
      }
    }
  `;
    try {
        const compilation = await FiveSDK.compile(brokenScript);
        if (!compilation.success) {
            console.log('✓ Compilation errors caught correctly:');
            compilation.errors?.forEach(error => {
                console.log(`  ${error.severity}: ${error.message}`);
                if (error.line) {
                    console.log(`    at line ${error.line}${error.column ? `:${error.column}` : ''}`);
                }
            });
        }
    }
    catch (error) {
        console.log('SDK Error handling:');
        if (error instanceof Error) {
            console.log(`  Type: ${error.constructor.name}`);
            console.log(`  Message: ${error.message}`);
        }
    }
    // Test execution error handling
    try {
        const result = await FiveSDK.executeLocally(new Uint8Array([1, 2, 3]), // Invalid bytecode
        0, // Function index
        [] // No parameters
        );
        console.log('Execution result:', result.success ? 'success' : 'failed');
    }
    catch (error) {
        console.log('✓ Execution error caught:', error instanceof Error ? error.message : error);
    }
}
// ==================== Example 7: Local WASM VM Execution ====================
async function example7_LocalWASMExecution() {
    console.log('=== Example 7: Local WASM VM Execution ===');
    const scriptSource = `
    script LocalTesting {
      // Test basic arithmetic
      add(a: u64, b: u64) -> u64 {
        return a + b;
      }
      
      // Test with no parameters
      getAnswer() -> u64 {
        return 42;
      }
      
      // Test complex logic
      fibonacci(n: u64) -> u64 {
        if (n <= 1) {
          return n;
        } else {
          return fibonacci(n - 1) + fibonacci(n - 2);
        }
      }
    }
  `;
    try {
        console.log('🚀 Testing local WASM VM execution (no blockchain needed!)...');
        // Method 1: Compile and execute in one step (easiest for testing)
        console.log('\n📝 Method 1: Compile and execute in one step');
        const quickResult = await compileAndExecuteLocally(scriptSource, 'getAnswer', // Function name
        [], // No parameters
        { debug: true, trace: true });
        if (quickResult.success) {
            console.log('✅ Quick execution successful!');
            if ('result' in quickResult) {
                console.log(`  Result: ${quickResult.result}`);
            }
            if ('executionTime' in quickResult) {
                console.log(`  Execution time: ${quickResult.executionTime}ms`);
            }
            if ('computeUnitsUsed' in quickResult) {
                console.log(`  Compute units: ${quickResult.computeUnitsUsed}`);
            }
            if ('bytecodeSize' in quickResult) {
                console.log(`  Bytecode size: ${quickResult.bytecodeSize} bytes`);
            }
        }
        else {
            console.log('❌ Quick execution failed:', quickResult.error);
        }
        // Method 2: Two-step process (more control)
        console.log('\n🔧 Method 2: Separate compilation and execution');
        const compilation = await FiveSDK.compile(scriptSource, { debug: true });
        if (compilation.success && compilation.bytecode) {
            console.log('✅ Compilation successful!');
            // Execute different functions with different parameters
            const functions = [
                { name: 'add', params: [10, 5] },
                { name: 'getAnswer', params: [] },
                { name: 'fibonacci', params: [8] }
            ];
            for (const func of functions) {
                console.log(`\n🎯 Testing function: ${func.name}(${func.params.join(', ')})`);
                const result = await FiveSDK.executeLocally(compilation.bytecode, func.name, func.params, { debug: true, trace: false, computeUnitLimit: 100000 });
                if (result.success) {
                    console.log(`  ✅ Result: ${result.result}`);
                    console.log(`  ⏱️  Time: ${result.executionTime}ms`);
                    console.log(`  ⚡ CU: ${result.computeUnitsUsed}`);
                }
                else {
                    console.log(`  ❌ Failed: ${result.error}`);
                }
            }
            // Method 3: Validate bytecode
            console.log('\n🔍 Method 3: Bytecode validation');
            const validation = await FiveSDK.validateBytecode(compilation.bytecode, { debug: true });
            if (validation.valid) {
                console.log('✅ Bytecode validation passed!');
                console.log(`  Functions found: ${validation.functions?.length || 0}`);
            }
            else {
                console.log('❌ Bytecode validation failed:', validation.errors);
            }
        }
        else {
            console.log('❌ Compilation failed:', compilation.errors);
        }
        console.log('\n🎉 Local WASM execution benefits:');
        console.log('• No blockchain connection needed');
        console.log('• Instant feedback for development');
        console.log('• Perfect for unit testing');
        console.log('• Debug bytecode before deployment');
        console.log('• Trace execution with detailed logs');
    }
    catch (error) {
        console.error('Local execution error:', error);
    }
}
// ==================== Run All Examples ====================
async function runAllExamples() {
    console.log('Five SDK - Usage Examples');
    console.log('='.repeat(50));
    try {
        await example1_BasicSetup();
        console.log();
        await example2_ScriptCompilation();
        console.log();
        await example3_DeploymentInstructionGeneration();
        console.log();
        await example4_ExecutionInstructionGeneration();
        console.log();
        await example5_ParameterHandling();
        console.log();
        await example6_ErrorHandling();
        console.log();
        await example7_LocalWASMExecution();
        console.log();
        console.log('✅ All client-agnostic examples completed successfully!');
        console.log('');
        console.log('Key Features Demonstrated:');
        console.log('• Client-agnostic SDK design (no Solana client dependencies)');
        console.log('• Static methods for compilation and instruction generation');
        console.log('• ABI-driven automatic parameter type coercion');
        console.log('• Serialized instruction data for any Solana client library');
        console.log('• Direct WASM VM execution for local testing');
        console.log('• Works in Node.js, browser, and mobile environments');
    }
    catch (error) {
        console.error('Example failed:', error);
    }
}
// Export examples for individual testing
export { example1_BasicSetup, example2_ScriptCompilation, example3_DeploymentInstructionGeneration, example4_ExecutionInstructionGeneration, example5_ParameterHandling, example6_ErrorHandling, example7_LocalWASMExecution, runAllExamples };
// Run examples if this file is executed directly
if (import.meta.url === `file://${process.argv[1]}`) {
    runAllExamples().catch(console.error);
}
//# sourceMappingURL=basic-usage.js.map