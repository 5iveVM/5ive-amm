#!/usr/bin/env node

/**
 * CLI tool for testing Stacks VM bytecode using WASM module
 * 
 * This tool provides a command-line interface for testing bytecode execution
 * with honest partial execution reporting and detailed analysis.
 */

import { readFileSync, existsSync } from 'fs';
import { resolve } from 'path';
import { WasmCompilerService, TestResultHelper } from './wasm-compiler';

interface CLIOptions {
    bytecodeFile?: string;
    inputData?: string;
    accounts?: number;
    verbose?: boolean;
    help?: boolean;
    version?: boolean;
    analyze?: boolean;
    benchmark?: boolean;
    iterations?: number;
}

const VERSION = '1.0.0';

const HELP_TEXT = `
Stacks VM WASM Test CLI v${VERSION}

USAGE:
    wasm-test-cli [OPTIONS] [BYTECODE_FILE]

OPTIONS:
    -i, --input-data <hex>     Input data as hex string (default: empty)
    -a, --accounts <num>       Number of test accounts to create (default: 0)
    -v, --verbose              Enable verbose output
    -h, --help                 Show this help message
    --version                  Show version information
    --analyze                  Analyze bytecode structure without execution
    --benchmark                Run performance benchmark
    --iterations <num>         Number of benchmark iterations (default: 100)

EXAMPLES:
    # Test bytecode file
    wasm-test-cli my-script.sbin

    # Test with input data
    wasm-test-cli --input-data "01020304" my-script.sbin

    # Test with accounts
    wasm-test-cli --accounts 2 my-script.sbin

    # Analyze bytecode structure
    wasm-test-cli --analyze my-script.sbin

    # Performance benchmark
    wasm-test-cli --benchmark --iterations 50 my-script.sbin

    # Verbose execution details
    wasm-test-cli --verbose my-script.sbin
`;

function parseArgs(args: string[]): CLIOptions {
    const options: CLIOptions = {};
    
    for (let i = 0; i < args.length; i++) {
        const arg = args[i];
        
        switch (arg) {
            case '-h':
            case '--help':
                options.help = true;
                break;
            case '--version':
                options.version = true;
                break;
            case '-v':
            case '--verbose':
                options.verbose = true;
                break;
            case '--analyze':
                options.analyze = true;
                break;
            case '--benchmark':
                options.benchmark = true;
                break;
            case '-i':
            case '--input-data':
                options.inputData = args[++i];
                break;
            case '-a':
            case '--accounts':
                options.accounts = parseInt(args[++i], 10);
                break;
            case '--iterations':
                options.iterations = parseInt(args[++i], 10);
                break;
            default:
                if (!arg.startsWith('-') && !options.bytecodeFile) {
                    options.bytecodeFile = arg;
                }
                break;
        }
    }
    
    return options;
}

function hexToUint8Array(hex: string): Uint8Array {
    // Remove '0x' prefix if present
    const cleanHex = hex.replace(/^0x/, '');
    
    // Ensure even length
    const paddedHex = cleanHex.length % 2 === 0 ? cleanHex : '0' + cleanHex;
    
    const bytes = new Uint8Array(paddedHex.length / 2);
    for (let i = 0; i < paddedHex.length; i += 2) {
        bytes[i / 2] = parseInt(paddedHex.substr(i, 2), 16);
    }
    
    return bytes;
}

async function loadBytecode(filePath: string): Promise<Uint8Array> {
    const resolvedPath = resolve(filePath);
    
    if (!existsSync(resolvedPath)) {
        throw new Error(`Bytecode file not found: ${resolvedPath}`);
    }
    
    try {
        const buffer = readFileSync(resolvedPath);
        return new Uint8Array(buffer);
    } catch (error) {
        throw new Error(`Failed to read bytecode file: ${error instanceof Error ? error.message : error}`);
    }
}

async function analyzeBytecode(wasmService: WasmCompilerService, bytecode: Uint8Array) {
    console.log('🔍 Bytecode Analysis');
    console.log('==================');
    
    // Validate bytecode
    const isValid = wasmService.validateBytecode(bytecode);
    console.log(`✅ Valid format: ${isValid ? 'Yes' : 'No'}`);
    
    if (!isValid) {
        console.log('❌ Invalid bytecode format or magic bytes');
        return;
    }
    
    console.log(`📏 Size: ${bytecode.length} bytes`);
    
    // Show magic bytes
    if (bytecode.length >= 4) {
        const magic = Array.from(bytecode.slice(0, 4))
            .map(b => String.fromCharCode(b))
            .join('');
        console.log(`🔮 Magic: "${magic}"`);
    }
    
    // Show first few instructions
    console.log('\n📋 Instructions:');
    const constants = wasmService.getConstants();
    let ip = 4; // Skip magic bytes
    let instructionCount = 0;
    
    while (ip < bytecode.length && instructionCount < 10) {
        const opcode = bytecode[ip];
        let opcodeName = 'UNKNOWN';
        
        // Find opcode name
        for (const [name, value] of Object.entries(constants.opcodes)) {
            if (value === opcode) {
                opcodeName = name;
                break;
            }
        }
        
        console.log(`  ${ip.toString().padStart(3, '0')}: ${opcodeName} (0x${opcode.toString(16).padStart(2, '0')})`);
        
        // Simple instruction size calculation (could be more sophisticated)
        let instructionSize = 1;
        if (opcode === constants.opcodes.PUSH && ip + 1 < bytecode.length) {
            const valueType = bytecode[ip + 1];
            const typeSizes: { [key: number]: number } = {
                1: 8, // U64
                2: 1, // BOOL
                3: 32, // PUBKEY
                4: 8, // I64
                5: 1, // U8
            };
            instructionSize = 2 + (typeSizes[valueType] || 0);
        }
        
        ip += instructionSize;
        instructionCount++;
    }
    
    if (ip < bytecode.length) {
        console.log(`  ... and ${bytecode.length - ip} more bytes`);
    }
}

async function executeBytecode(
    wasmService: WasmCompilerService,
    bytecode: Uint8Array,
    inputData: Uint8Array,
    accountCount: number,
    verbose: boolean
) {
    console.log('🚀 Executing Bytecode');
    console.log('=====================');
    
    // Create test accounts if requested
    const accounts = [];
    for (let i = 0; i < accountCount; i++) {
        const account = wasmService.createTestAccount(
            `account${i}`.padEnd(64, '0'),
            new Uint8Array(256), // 256 bytes data
            BigInt(1000000 + i * 100000), // Varying lamports
            true, // writable
            i === 0, // first account is signer
            'system'.padEnd(64, '0')
        );
        accounts.push(account);
    }
    
    if (verbose && accounts.length > 0) {
        console.log(`\n👥 Created ${accounts.length} test account(s):`);
        accounts.forEach((account, i) => {
            console.log(`  Account ${i}: ${account.lamports} lamports, ${account.data.length} bytes data`);
        });
    }
    
    // Execute
    const startTime = performance.now();
    const result = await wasmService.testBytecodeExecution(bytecode, inputData, accounts);
    const endTime = performance.now();
    const executionTime = endTime - startTime;
    
    // Display results
    console.log('\n📊 Execution Results');
    console.log('====================');
    console.log(`⏱️  Execution time: ${executionTime.toFixed(2)}ms`);
    console.log(`🎯 Status: ${result.outcome.toUpperCase()}`);
    console.log(`✅ Test success: ${result.test_success ? 'Yes' : 'No'}`);
    console.log(`📝 Description: ${result.description}`);
    
    if (result.operations_tested.length > 0) {
        console.log(`\n🔧 Operations tested (${result.operations_tested.length}):`);
        console.log(`   ${result.operations_tested.join(' → ')}`);
    }
    
    console.log(`\n📈 Final state:`);
    console.log(`   Compute units: ${result.final_state.compute_units_used}`);
    console.log(`   Instruction pointer: ${result.final_state.instruction_pointer}`);
    console.log(`   Stack size: ${result.final_state.stack_size}`);
    console.log(`   Has result: ${result.final_state.has_result ? 'Yes' : 'No'}`);
    
    if (result.stopped_at_operation) {
        console.log(`\n🛑 Stopped at: ${result.stopped_at_operation}`);
    }
    
    if (result.error_details) {
        console.log(`\n🚨 Error details: ${result.error_details}`);
    }
    
    if (verbose) {
        console.log('\n📋 Detailed Summary:');
        console.log(TestResultHelper.formatSummary(result));
    }
    
    return result;
}

async function runBenchmark(
    wasmService: WasmCompilerService,
    bytecode: Uint8Array,
    inputData: Uint8Array,
    iterations: number
) {
    console.log('⚡ Performance Benchmark');
    console.log('========================');
    console.log(`Running ${iterations} iterations...\n`);
    
    const times: number[] = [];
    const results: any[] = [];
    
    for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();
        const result = await wasmService.testBytecodeExecution(bytecode, inputData, []);
        const endTime = performance.now();
        
        times.push(endTime - startTime);
        results.push(result);
        
        if ((i + 1) % 10 === 0) {
            process.stdout.write(`\r⏳ Progress: ${i + 1}/${iterations}`);
        }
    }
    
    console.log('\n');
    
    // Calculate statistics
    const mean = times.reduce((a, b) => a + b, 0) / times.length;
    const min = Math.min(...times);
    const max = Math.max(...times);
    const sorted = times.sort((a, b) => a - b);
    const median = sorted[Math.floor(sorted.length / 2)];
    const p95 = sorted[Math.floor(sorted.length * 0.95)];
    
    const variance = times.reduce((acc, time) => acc + Math.pow(time - mean, 2), 0) / times.length;
    const stdDev = Math.sqrt(variance);
    
    console.log('📊 Performance Statistics:');
    console.log(`   Mean:     ${mean.toFixed(2)}ms`);
    console.log(`   Median:   ${median.toFixed(2)}ms`);
    console.log(`   Min:      ${min.toFixed(2)}ms`);
    console.log(`   Max:      ${max.toFixed(2)}ms`);
    console.log(`   95th %:   ${p95.toFixed(2)}ms`);
    console.log(`   Std Dev:  ${stdDev.toFixed(2)}ms`);
    
    // Success rate
    const successes = results.filter(r => r.test_success).length;
    const successRate = (successes / results.length) * 100;
    console.log(`   Success:  ${successRate.toFixed(1)}% (${successes}/${results.length})`);
    
    // Throughput
    const throughput = 1000 / mean; // operations per second
    console.log(`   Throughput: ${throughput.toFixed(0)} ops/sec`);
    
    return { mean, median, min, max, stdDev, successRate, throughput };
}

async function main() {
    const args = process.argv.slice(2);
    const options = parseArgs(args);
    
    if (options.help) {
        console.log(HELP_TEXT);
        return;
    }
    
    if (options.version) {
        console.log(`Stacks VM WASM Test CLI v${VERSION}`);
        return;
    }
    
    if (!options.bytecodeFile) {
        console.error('❌ Error: Bytecode file is required');
        console.log('\nUse --help for usage information');
        process.exit(1);
    }
    
    try {
        // Initialize WASM service
        console.log('🚀 Initializing WASM service...\n');
        const wasmService = new WasmCompilerService();
        await wasmService.initialize();
        
        // Load bytecode
        const bytecode = await loadBytecode(options.bytecodeFile);
        console.log(`📁 Loaded bytecode: ${options.bytecodeFile} (${bytecode.length} bytes)\n`);
        
        // Parse input data
        const inputData = options.inputData ? hexToUint8Array(options.inputData) : new Uint8Array();
        if (options.verbose && inputData.length > 0) {
            console.log(`📥 Input data: ${Array.from(inputData).map(b => b.toString(16).padStart(2, '0')).join(' ')}\n`);
        }
        
        if (options.analyze) {
            await analyzeBytecode(wasmService, bytecode);
        } else if (options.benchmark) {
            const iterations = options.iterations || 100;
            await runBenchmark(wasmService, bytecode, inputData, iterations);
        } else {
            const accountCount = options.accounts || 0;
            const verbose = options.verbose || false;
            await executeBytecode(wasmService, bytecode, inputData, accountCount, verbose);
        }
        
    } catch (error) {
        console.error('❌ Error:', error instanceof Error ? error.message : error);
        process.exit(1);
    }
}

// Export for testing
export { parseArgs, hexToUint8Array, analyzeBytecode, executeBytecode, runBenchmark };

// Run if called directly
if (require.main === module) {
    main().catch(console.error);
}