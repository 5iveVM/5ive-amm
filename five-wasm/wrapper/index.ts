import init, { 
    FiveVMWasm, 
    WasmAccount, 
    BytecodeAnalyzer,
    FiveVMState,
    TestResult,
    ExecutionStatus,
    validate_bytecode,
    get_constants,
    js_value_to_vm_value,
    InitOutput 
} from '../pkg/five_vm_wasm';

// Re-export core types for convenience
export { 
    FiveVMWasm, 
    WasmAccount, 
    BytecodeAnalyzer,
    FiveVMState,
    TestResult,
    ExecutionStatus,
    validate_bytecode,
    get_constants,
    js_value_to_vm_value 
};

// Backward-compatibility aliases for legacy Stacks naming in tests/scripts
export { FiveVMWasm as StacksVMWasm };

/**
 * TypeScript-friendly account interface
 */
export interface FiveAccount {
    key: Uint8Array;
    data: Uint8Array;
    lamports: bigint;
    isWritable: boolean;
    isSigner: boolean;
    owner: Uint8Array;
}

// Legacy alias for compatibility with older tests/scripts
export type StacksAccount = FiveAccount;

/**
 * VM execution result (legacy interface)
 */
export interface VMExecutionResult {
    success: boolean;
    result?: any;
    error?: string;
    computeUnits: bigint;
    updatedAccounts: FiveAccount[];
}

/**
 * Partial execution result
 */
export interface PartialExecutionResult {
    status: 'Completed' | 'StoppedAtSystemCall' | 'StoppedAtInitPDA' | 'StoppedAtInvoke' | 'StoppedAtInvokeSigned' | 'ComputeLimitExceeded' | 'Failed';
    computeUnits: number;
    instructionPointer: number;
    finalStack: any[];
    hasResultValue: boolean;
    resultValue?: any;
    errorMessage?: string;
    stoppedAtOpcode?: number;
    stoppedAtOpcodeName?: string;
    updatedAccounts: FiveAccount[];
}

/**
 * Bytecode analysis result
 */
export interface BytecodeAnalysis {
    totalSize: number;
    instructionCount: number;
    instructions: Array<{
        offset: number;
        opcode: number;
        name: string;
        size: number;
    }>;
}

/**
 * VM constants with TypeScript types
 */
export interface FiveVMConstants {
    MAX_SCRIPT_SIZE: number;
    MAX_COMPUTE_UNITS: number;
    FIVE_MAGIC: Uint8Array;
    opcodes: {
        [key: string]: number;
    };
    types: {
        [key: string]: number;
    };
}

/**
 * High-level TypeScript wrapper for Five VM WASM
 */
export class FiveVMWrapper {
    private vm: FiveVMWasm | null = null;
    private constants: FiveVMConstants | null = null;
    private initialized = false;

    /**
     * Initialize the WASM module
     */
    static async init(wasmPath?: string): Promise<InitOutput> {
        return await init(wasmPath);
    }

    /**
     * Create a new VM instance
     */
    constructor(private bytecode: Uint8Array) {
        this.validateAndInitialize();
    }

    /**
     * Validate bytecode and initialize VM
     */
    private validateAndInitialize(): void {
        if (!validate_bytecode(this.bytecode)) {
            throw new Error('Invalid bytecode format');
        }

        try {
            this.vm = new FiveVMWasm(this.bytecode);
            this.initialized = true;
        } catch (error) {
            throw new Error(`Failed to initialize VM: ${error}`);
        }
    }

    /**
     * Get VM constants (cached)
     */
    getConstants(): FiveVMConstants {
        if (!this.constants) {
            const rawConstants = JSON.parse(get_constants());
            this.constants = {
                MAX_SCRIPT_SIZE: rawConstants.MAX_SCRIPT_SIZE,
                MAX_COMPUTE_UNITS: rawConstants.MAX_COMPUTE_UNITS,
                FIVE_MAGIC: new Uint8Array(rawConstants.FIVE_MAGIC),
                opcodes: rawConstants.opcodes,
                types: rawConstants.types,
            };
        }
        return this.constants;
    }

    /**
     * Execute VM with improved error handling and TypeScript types (legacy method)
     */
    async execute(
        inputData: Uint8Array,
        accounts: FiveAccount[]
    ): Promise<VMExecutionResult> {
        if (!this.initialized || !this.vm) {
            throw new Error('VM not initialized');
        }

        try {
            // Convert TypeScript accounts to WASM accounts
            const wasmAccounts = accounts.map(acc => 
                new WasmAccount(
                    acc.key,
                    acc.data,
                    acc.lamports,
                    acc.isWritable,
                    acc.isSigner,
                    acc.owner
                )
            );

            // Execute VM
            const result = this.vm.execute(inputData, wasmAccounts);

            // Get updated account state
            const updatedAccounts: FiveAccount[] = wasmAccounts.map(wasmAcc => ({
                key: wasmAcc.key,
                data: wasmAcc.data,
                lamports: wasmAcc.lamports,
                isWritable: wasmAcc.is_writable,
                isSigner: wasmAcc.is_signer,
                owner: wasmAcc.owner,
            }));

            // Get current state for compute units
            const state = JSON.parse(this.vm.get_state());

            return {
                success: true,
                result,
                computeUnits: BigInt(state.compute_units || 0),
                updatedAccounts,
            };

        } catch (error) {
            return {
                success: false,
                error: error instanceof Error ? error.message : String(error),
                computeUnits: BigInt(0),
                updatedAccounts: accounts,
            };
        }
    }

    /**
     * Execute VM with partial execution support.
     * 
     * Executes bytecode and reports status, including interruptions at system calls.
     */
    async executePartial(
        inputData: Uint8Array,
        accounts: FiveAccount[]
    ): Promise<PartialExecutionResult> {
        if (!this.initialized || !this.vm) {
            throw new Error('VM not initialized');
        }

        try {
            // Convert TypeScript accounts to WASM accounts
            const wasmAccounts = accounts.map(acc => 
                new WasmAccount(
                    acc.key,
                    acc.data,
                    acc.lamports,
                    acc.isWritable,
                    acc.isSigner,
                    acc.owner
                )
            );

            // Execute with partial execution support
            const testResult = this.vm.execute_partial(inputData, wasmAccounts);

            // Get updated account state
            const updatedAccounts: FiveAccount[] = wasmAccounts.map(wasmAcc => ({
                key: wasmAcc.key,
                data: wasmAcc.data,
                lamports: wasmAcc.lamports,
                isWritable: wasmAcc.is_writable,
                isSigner: wasmAcc.is_signer,
                owner: wasmAcc.owner,
            }));

            // Convert TestResult to PartialExecutionResult
            return {
                status: testResult.status() as any,
                computeUnits: Number(testResult.compute_units_used),
                instructionPointer: testResult.instruction_pointer,
                finalStack: testResult.final_stack(),
                hasResultValue: testResult.has_result_value(),
                resultValue: testResult.has_result_value() ? testResult.get_result_value() : undefined,
                errorMessage: testResult.error_message() || undefined,
                stoppedAtOpcode: testResult.stopped_at_opcode || undefined,
                stoppedAtOpcodeName: testResult.stopped_at_opcode_name() || undefined,
                updatedAccounts,
            };

        } catch (error) {
            return {
                status: 'Failed',
                computeUnits: 0,
                instructionPointer: 0,
                finalStack: [],
                hasResultValue: false,
                errorMessage: error instanceof Error ? error.message : String(error),
                updatedAccounts: accounts,
            };
        }
    }

    /**
     * Get current VM state
     */
    getState(): any {
        if (!this.initialized || !this.vm) {
            throw new Error('VM not initialized');
        }
        return JSON.parse(this.vm.get_state());
    }

    /**
     * Analyze bytecode with typed results
     */
    static analyzeBytecode(bytecode: Uint8Array): BytecodeAnalysis {
        const analysis = JSON.parse(BytecodeAnalyzer.analyze(bytecode));
        return {
            totalSize: analysis.total_size,
            instructionCount: analysis.instruction_count,
            instructions: analysis.instructions,
        };
    }

    /**
     * Create a value for the VM with proper type encoding
     */
    static createVMValue(value: any, type: keyof FiveVMConstants['types']): any {
        const constants = JSON.parse(get_constants());
        const typeCode = constants.types[type];
        if (typeCode === undefined) {
            throw new Error(`Unknown type: ${type}`);
        }
        return js_value_to_vm_value(value, typeCode);
    }

    /**
     * Utility: Create account helper
     */
    static createAccount(
        key: string | Uint8Array,
        data: Uint8Array = new Uint8Array(),
        lamports: bigint = BigInt(0),
        isWritable: boolean = false,
        isSigner: boolean = false,
        owner: string | Uint8Array = new Uint8Array(32)
    ): FiveAccount {
        const keyBytes = typeof key === 'string' ? 
            Uint8Array.from(Buffer.from(key, 'hex')) : key;
        const ownerBytes = typeof owner === 'string' ? 
            Uint8Array.from(Buffer.from(owner, 'hex')) : owner;

        if (keyBytes.length !== 32) {
            throw new Error('Key must be 32 bytes');
        }
        if (ownerBytes.length !== 32) {
            throw new Error('Owner must be 32 bytes');
        }

        return {
            key: keyBytes,
            data,
            lamports,
            isWritable,
            isSigner,
            owner: ownerBytes,
        };
    }

    /**
     * Cleanup resources
     */
    dispose(): void {
        if (this.vm) {
            this.vm.free();
            this.vm = null;
        }
        this.initialized = false;
    }
}

// Legacy alias for compatibility with older tests/scripts
export const StacksVMWrapper = FiveVMWrapper;

/**
 * Performance benchmark utilities
 */
export class VMPerformanceBenchmark {
    private static measurements: Array<{
        operation: string;
        duration: number;
        timestamp: number;
    }> = [];

    /**
     * Benchmark VM execution performance
     */
    static async benchmarkExecution(
        bytecode: Uint8Array,
        inputData: Uint8Array,
        accounts: FiveAccount[],
        iterations: number = 100
    ): Promise<{
        averageTime: number;
        minTime: number;
        maxTime: number;
        totalTime: number;
        iterations: number;
        successRate: number;
    }> {
        const times: number[] = [];
        let successes = 0;

        for (let i = 0; i < iterations; i++) {
            const vm = new FiveVMWrapper(bytecode);
            const startTime = performance.now();
            
            try {
                const result = await vm.execute(inputData, accounts);
                if (result.success) {
                    successes++;
                }
            } catch (error) {
                // Count as failure
            }
            
            const endTime = performance.now();
            times.push(endTime - startTime);
            vm.dispose();
        }

        return {
            averageTime: times.reduce((a, b) => a + b, 0) / times.length,
            minTime: Math.min(...times),
            maxTime: Math.max(...times),
            totalTime: times.reduce((a, b) => a + b, 0),
            iterations,
            successRate: successes / iterations,
        };
    }

    /**
     * Compare WASM performance vs current implementation
     */
    static async comparePerformance(
        wasmFn: () => Promise<any>,
        nativeFn: () => Promise<any>,
        iterations: number = 50
    ): Promise<{
        wasmPerformance: number;
        nativePerformance: number;
        performanceRatio: number;
        wasmFaster: boolean;
    }> {
        // Benchmark WASM
        const wasmTimes: number[] = [];
        for (let i = 0; i < iterations; i++) {
            const start = performance.now();
            await wasmFn();
            wasmTimes.push(performance.now() - start);
        }

        // Benchmark native
        const nativeTimes: number[] = [];
        for (let i = 0; i < iterations; i++) {
            const start = performance.now();
            await nativeFn();
            nativeTimes.push(performance.now() - start);
        }

        const wasmAvg = wasmTimes.reduce((a, b) => a + b, 0) / wasmTimes.length;
        const nativeAvg = nativeTimes.reduce((a, b) => a + b, 0) / nativeTimes.length;

        return {
            wasmPerformance: wasmAvg,
            nativePerformance: nativeAvg,
            performanceRatio: nativeAvg / wasmAvg,
            wasmFaster: wasmAvg < nativeAvg,
        };
    }
}

/**
 * Bundle size analyzer for WASM module
 */
export class BundleAnalyzer {
    /**
     * Get WASM bundle information
     */
    static async analyzeBundleSize(): Promise<{
        wasmSize: number;
        jsSize: number;
        totalSize: number;
        gzippedEstimate: number;
    }> {
        // This would need to be implemented with actual bundle analysis
        // Return placeholder values
        return {
            wasmSize: 0,
            jsSize: 0,
            totalSize: 0,
            gzippedEstimate: 0,
        };
    }
}

// Default export for convenience
export default FiveVMWrapper;
