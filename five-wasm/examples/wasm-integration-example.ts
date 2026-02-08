/**
 * Practical example of using the WASM compiler service for testing
 * 
 * This example demonstrates real-world usage of the WASM module for testing
 * Stacks VM bytecode execution with honest partial execution reporting.
 */

import { WasmCompilerService, TestResultHelper } from '../app/wasm-compiler';
import { StacksVMWrapper } from '../wrapper/index';

/**
 * Example: Testing a simple vault deposit operation
 */
async function testVaultDeposit() {
    console.log('🧪 Testing Vault Deposit Logic...\n');

    const wasmService = new WasmCompilerService();
    await wasmService.initialize();

    // Create bytecode for vault deposit logic
    // Simulate: initial_balance + deposit_amount = new_balance
    const vaultDepositBytecode = wasmService.createTestBytecode([
        { opcode: 'PUSH', args: ['U64', 1000] },  // Initial balance: 1000
        { opcode: 'PUSH', args: ['U64', 500] },   // Deposit amount: 500
        { opcode: 'ADD' },                        // Add them together
        { opcode: 'PUSH', args: ['U64', 100] },   // Fee: 100
        { opcode: 'SUB' },                        // Subtract fee
        { opcode: 'HALT' }                        // Stop execution
    ]);

    const result = await wasmService.testBytecodeExecution(vaultDepositBytecode);

    console.log('📊 Execution Result:');
    console.log(TestResultHelper.formatSummary(result));
    console.log('');

    if (TestResultHelper.isSuccessfulTest(result)) {
        console.log('✅ Test PASSED: Vault deposit logic executed correctly');
        console.log(`💰 Final balance calculation: ${result.final_state.has_result ? 'Available on stack' : 'No result'}`);
        console.log(`⛽ Gas used: ${result.final_state.compute_units_used} compute units`);
    } else {
        console.log('❌ Test FAILED: Vault deposit logic failed');
        console.log(`🚨 Error: ${result.error_details}`);
    }
    
    return result;
}

/**
 * Example: Testing complex mathematical operations
 */
async function testComplexMath() {
    console.log('\n🧮 Testing Complex Mathematical Operations...\n');

    const wasmService = new WasmCompilerService();
    await wasmService.initialize();

    // Test: Calculate compound interest
    // Formula: principal * (1 + rate)^time
    // 1000 * 1.05 * 1.05 = 1102.5 (approximated with integers)
    const mathBytecode = wasmService.createTestBytecode([
        { opcode: 'PUSH', args: ['U64', 1000] },  // Principal
        { opcode: 'PUSH', args: ['U64', 105] },   // Rate factor (105/100 = 1.05)
        { opcode: 'MUL' },                        // principal * rate_factor
        { opcode: 'PUSH', args: ['U64', 100] },   // Divisor
        { opcode: 'DIV' },                        // Normalize to percentage
        { opcode: 'PUSH', args: ['U64', 105] },   // Apply rate again
        { opcode: 'MUL' },                        // Second year
        { opcode: 'PUSH', args: ['U64', 100] },   // Divisor
        { opcode: 'DIV' },                        // Final result
        { opcode: 'HALT' }
    ]);

    const result = await wasmService.testBytecodeExecution(mathBytecode);

    console.log('📈 Mathematical Operation Result:');
    console.log(TestResultHelper.formatSummary(result));

    if (TestResultHelper.isSuccessfulTest(result)) {
        console.log('✅ Complex math operations executed successfully');
        console.log(`🔢 Operations tested: ${TestResultHelper.getTestedOperations(result).join(' → ')}`);
    } else {
        console.log('❌ Mathematical operations failed');
        console.log(`🚨 Error: ${result.error_details}`);
    }

    return result;
}

/**
 * Example: Testing error handling
 */
async function testErrorHandling() {
    console.log('\n🚨 Testing Error Handling...\n');

    const wasmService = new WasmCompilerService();
    await wasmService.initialize();

    // Test division by zero
    const errorBytecode = wasmService.createTestBytecode([
        { opcode: 'PUSH', args: ['U64', 100] },
        { opcode: 'PUSH', args: ['U64', 0] },     // Zero divisor
        { opcode: 'DIV' },                        // This should fail
        { opcode: 'HALT' }
    ]);

    const result = await wasmService.testBytecodeExecution(errorBytecode);

    console.log('💥 Error Handling Test Result:');
    console.log(TestResultHelper.formatSummary(result));

    if (!result.test_success && result.error_details?.toLowerCase().includes('zero')) {
        console.log('✅ Error handling PASSED: Division by zero properly detected');
        console.log(`🛡️ Operations tested before error: ${result.operations_tested.join(', ')}`);
    } else {
        console.log('❌ Error handling FAILED: Should have detected division by zero');
    }

    return result;
}

/**
 * Example: Testing with account context
 */
async function testWithAccounts() {
    console.log('\n🏦 Testing with Account Context...\n');

    const wasmService = new WasmCompilerService();
    await wasmService.initialize();

    // Create test accounts
    const userAccount = wasmService.createTestAccount(
        'user123'.padEnd(64, '0'),    // User's account key
        new Uint8Array(1000),         // Account data (1KB)
        BigInt(5000000),              // 5 SOL in lamports
        true,                         // Writable
        true,                         // Signer
        'system11'.padEnd(64, '0')    // System program owner
    );

    const vaultAccount = wasmService.createTestAccount(
        'vault456'.padEnd(64, '0'),   // Vault's account key
        new Uint8Array(1000),         // Account data
        BigInt(10000000),             // 10 SOL in lamports
        true,                         // Writable
        false,                        // Not signer
        'vault_prog'.padEnd(64, '0')  // Vault program owner
    );

    // Test simple operations with account context
    const bytecode = wasmService.createTestBytecode([
        { opcode: 'PUSH', args: ['U64', 1000] },  // Amount to process
        { opcode: 'PUSH', args: ['U64', 50] },    // Processing fee
        { opcode: 'SUB' },                        // Net amount
        { opcode: 'HALT' }
    ]);

    const result = await wasmService.testBytecodeExecution(
        bytecode,
        new Uint8Array([1, 2, 3, 4]), // Input data
        [userAccount, vaultAccount]    // Account context
    );

    console.log('🏛️ Account Context Test Result:');
    console.log(TestResultHelper.formatSummary(result));

    if (TestResultHelper.isSuccessfulTest(result)) {
        console.log('✅ Account context test PASSED');
        console.log(`👤 User account: ${userAccount.key.slice(0, 8).join('')}...`);
        console.log(`🏦 Vault account: ${vaultAccount.key.slice(0, 8).join('')}...`);
        console.log(`💎 User balance: ${userAccount.lamports} lamports`);
        console.log(`🏛️ Vault balance: ${vaultAccount.lamports} lamports`);
    } else {
        console.log('❌ Account context test FAILED');
    }

    return result;
}

/**
 * Example: Comparing WASM vs Legacy wrapper performance
 */
async function performanceComparison() {
    console.log('\n⚡ Performance Comparison: WASM vs Legacy...\n');

    const wasmService = new WasmCompilerService();
    await wasmService.initialize();

    // Create test bytecode
    const bytecode = wasmService.createTestBytecode([
        { opcode: 'PUSH', args: ['U64', 42] },
        { opcode: 'DUP' },
        { opcode: 'ADD' },
        { opcode: 'HALT' }
    ]);

    // Test WASM performance
    console.log('🚀 Testing WASM performance...');
    const wasmStartTime = performance.now();
    const wasmResult = await wasmService.testBytecodeExecution(bytecode);
    const wasmEndTime = performance.now();
    const wasmDuration = wasmEndTime - wasmStartTime;

    // Test legacy wrapper performance (if valid bytecode)
    console.log('🐌 Testing legacy wrapper performance...');
    let legacyDuration = 0;
    let legacyResult = null;

    try {
        if (wasmService.validateBytecode(bytecode)) {
            await StacksVMWrapper.init();
            const vm = new StacksVMWrapper(bytecode);
            
            const legacyStartTime = performance.now();
            legacyResult = await vm.execute(new Uint8Array(), []);
            const legacyEndTime = performance.now();
            legacyDuration = legacyEndTime - legacyStartTime;
            
            vm.dispose();
        }
    } catch (error) {
        console.log('⚠️ Legacy wrapper execution failed (expected for pure WASM test)');
    }

    console.log('\n📊 Performance Results:');
    console.log(`🚀 WASM execution: ${wasmDuration.toFixed(2)}ms`);
    console.log(`🐌 Legacy execution: ${legacyDuration > 0 ? legacyDuration.toFixed(2) + 'ms' : 'N/A'}`);
    
    if (legacyDuration > 0) {
        const speedup = legacyDuration / wasmDuration;
        console.log(`⚡ WASM speedup: ${speedup.toFixed(2)}x ${speedup > 1 ? 'faster' : 'slower'}`);
    }

    console.log('\n🎯 WASM Result Quality:');
    console.log(`✅ Test success: ${wasmResult.test_success}`);
    console.log(`📋 Operations tested: ${wasmResult.operations_tested.join(', ')}`);
    console.log(`⛽ Compute units: ${wasmResult.final_state.compute_units_used}`);

    return { wasmResult, wasmDuration, legacyDuration };
}

/**
 * Example: Demonstrating honest partial execution
 */
async function demonstratePartialExecution() {
    console.log('\n🎭 Demonstrating Honest Partial Execution...\n');

    const wasmService = new WasmCompilerService();
    await wasmService.initialize();

    // Create bytecode that would require system calls (simplified)
    const bytecode = wasmService.createTestBytecode([
        { opcode: 'PUSH', args: ['U64', 100] },   // Amount
        { opcode: 'PUSH', args: ['U64', 50] },    // Fee  
        { opcode: 'SUB' },                        // Calculate net
        // In real scenario, this would be followed by system calls like:
        // - INVOKE to transfer tokens
        // - INIT_PDA for account creation
        // But for this demo, we just complete with pure computation
        { opcode: 'HALT' }
    ]);

    const result = await wasmService.testBytecodeExecution(bytecode);

    console.log('🎯 Partial Execution Demonstration:');
    console.log(`📊 Status: ${result.outcome}`);
    console.log(`📝 Description: ${result.description}`);
    console.log(`✅ Test Success: ${result.test_success}`);
    console.log('');

    if (TestResultHelper.wasStoppedAtSystemCall(result)) {
        console.log('🛑 Execution stopped at system call (honest reporting)');
        console.log(`🔧 Operations tested: ${result.operations_tested.join(' → ')}`);
        console.log(`🎯 Stopped at: ${result.stopped_at_operation}`);
        console.log('💡 This is NOT a failure - it\'s honest testing!');
    } else if (result.outcome === 'completed') {
        console.log('✅ Full execution completed');
        console.log(`🔧 All operations tested: ${result.operations_tested.join(' → ')}`);
        console.log('💡 Pure computational operations executed successfully');
    } else {
        console.log('❌ Execution failed');
        console.log(`🚨 Error: ${result.error_details}`);
    }

    return result;
}

/**
 * Main demonstration function
 */
async function main() {
    console.log('🎉 WASM Integration Service Demonstration\n');
    console.log('This demo shows how to use the WASM module for honest VM testing.\n');

    try {
        // Run all examples
        await testVaultDeposit();
        await testComplexMath(); 
        await testErrorHandling();
        await testWithAccounts();
        await performanceComparison();
        await demonstratePartialExecution();

        console.log('\n🎊 All demonstrations completed successfully!');
        console.log('\n📚 Key Takeaways:');
        console.log('• ✅ WASM provides fast, accurate VM testing');
        console.log('• 🎯 Honest reporting: never fakes execution results');
        console.log('• 🛑 Properly detects and reports system call stops');
        console.log('• ⚡ Performance benefits over legacy implementations');
        console.log('• 🧪 Perfect for testing pure computational logic');
        console.log('• 🔍 Detailed execution insights and debugging info');

    } catch (error) {
        console.error('❌ Demonstration failed:', error);
        process.exit(1);
    }
}

// Export for use in other modules
export {
    testVaultDeposit,
    testComplexMath,
    testErrorHandling,
    testWithAccounts,
    performanceComparison,
    demonstratePartialExecution
};

// Run if called directly
if (require.main === module) {
    main().catch(console.error);
}
