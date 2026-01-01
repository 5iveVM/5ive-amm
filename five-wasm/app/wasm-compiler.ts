/**
 * WASM-based compiler and testing service for Stacks VM
 * 
 * This service provides WASM-based execution and testing capabilities,
 * properly handling partial execution results and system call stops.
 * 
 * CRITICAL: This is NOT for compilation - it's for testing bytecode execution
 * using the WASM module with honest partial execution results.
 */

import init, { 
    FiveVMWasm, 
    WasmAccount, 
    TestResult,
    ExecutionStatus,
    validate_bytecode,
    get_constants 
} from '../pkg/five_vm_wasm.js';

/**
 * TypeScript interface for TestResult that matches Rust implementation
 */
export interface TestResultInterface {
    status: ExecutionStatus | string;
    compute_units_used: number;
    instruction_pointer: number;
    stopped_at_opcode?: number;
    stopped_at_opcode_name?: string;
    has_result_value: boolean;
    get_result_value?: any;
    final_stack: any[];
    error_message?: string;
}

/**
 * Execution summary for partial execution
 */
export interface PartialExecutionSummary {
    /** What actually happened during execution */
    outcome: 'completed' | 'stopped_at_system_call' | 'stopped_at_init_pda' | 'stopped_at_invoke' | 'stopped_at_invoke_signed' | 'compute_limit_exceeded' | 'failed';
    
    /** Human-readable description of what was tested */
    description: string;
    
    /** Operations that were successfully executed */
    operations_tested: string[];
    
    /** The operation that caused execution to stop (if any) */
    stopped_at_operation?: string;
    
    /** Final state of execution */
    final_state: {
        compute_units_used: number;
        instruction_pointer: number;
        stack_size: number;
        has_result: boolean;
    };
    
    /** Error details if execution failed */
    error_details?: string;
    
    /** Whether this represents a successful test (even if stopped) */
    test_success: boolean;
}

/**
 * Account interface compatible with Solana and WASM
 */
export interface WasmAccountInterface {
    key: Uint8Array;
    data: Uint8Array;
    lamports: bigint;
    isWritable: boolean;
    isSigner: boolean;
    owner: Uint8Array;
}

/**
 * WASM-based compiler service for testing bytecode execution
 */
export class WasmCompilerService {
    private initialized = false;
    private constants: any = null;

    /**
     * Initialize WASM module
     */
    async initialize(): Promise<void> {
        if (!this.initialized) {
            await init();
            this.constants = JSON.parse(get_constants());
            this.initialized = true;
        }
    }

    /**
     * Validate bytecode format
     */
    validateBytecode(bytecode: Uint8Array): boolean {
        this.ensureInitialized();
        return validate_bytecode(bytecode);
    }

    /**
     * Test bytecode execution with partial execution support
     * 
     * This method provides honest reporting about what was actually tested.
     * It never pretends execution completed when it stopped at system calls.
     */
    async testBytecodeExecution(
        bytecode: Uint8Array,
        inputData: Uint8Array = new Uint8Array(),
        accounts: WasmAccountInterface[] = []
    ): Promise<PartialExecutionSummary> {
        this.ensureInitialized();

        // Validate bytecode first
        if (!this.validateBytecode(bytecode)) {
            return {
                outcome: 'failed',
                description: 'Bytecode validation failed - invalid format or magic bytes',
                operations_tested: [],
                final_state: {
                    compute_units_used: 0,
                    instruction_pointer: 0,
                    stack_size: 0,
                    has_result: false,
                },
                error_details: 'Invalid bytecode format',
                test_success: false,
            };
        }

        let vm: FiveVMWasm;
        let result: TestResult;

        try {
            // Create VM instance
            vm = new FiveVMWasm(bytecode);
            
            // Convert accounts to WASM format
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
            result = vm.execute_partial(inputData, wasmAccounts);

        } catch (error) {
            return {
                outcome: 'failed',
                description: 'VM execution failed due to initialization or runtime error',
                operations_tested: [],
                final_state: {
                    compute_units_used: 0,
                    instruction_pointer: 0,
                    stack_size: 0,
                    has_result: false,
                },
                error_details: error instanceof Error ? error.message : String(error),
                test_success: false,
            };
        }

        // Interpret the test result honestly
        return this.interpretTestResult(result, bytecode);
    }

    /**
     * Interpret TestResult and provide honest summary
     */
    private interpretTestResult(result: TestResult, bytecode: Uint8Array): PartialExecutionSummary {
        const status = result.status();
        const computeUnits = Number(result.compute_units_used);
        const instructionPointer = result.instruction_pointer;
        const finalStack = result.final_stack();
        const errorMessage = result.error_message();
        const stoppedAtOpcode = result.stopped_at_opcode();
        const stoppedAtOpcodeName = result.stopped_at_opcode_name();

        // Analyze what operations were tested
        const operationsTested = this.analyzeExecutedOperations(bytecode, instructionPointer);

        let outcome: PartialExecutionSummary['outcome'];
        let description: string;
        let testSuccess: boolean;

        switch (status) {
            case 'Completed':
                outcome = 'completed';
                description = `Full execution completed successfully. All ${operationsTested.length} operations executed.`;
                testSuccess = true;
                break;

            case 'StoppedAtSystemCall':
                outcome = 'stopped_at_system_call';
                description = `Execution stopped at system call (${stoppedAtOpcodeName || 'unknown'}). ` +
                             `Successfully tested ${operationsTested.length} operations before system call.`;
                testSuccess = true;
                break;

            case 'StoppedAtInitPDA':
                outcome = 'stopped_at_init_pda';
                description = `Execution stopped at INIT_PDA operation. ` +
                             `Successfully tested ${operationsTested.length} operations before PDA initialization.`;
                testSuccess = true;
                break;

            case 'StoppedAtInvoke':
                outcome = 'stopped_at_invoke';
                description = `Execution stopped at INVOKE operation. ` +
                             `Successfully tested ${operationsTested.length} operations before program invocation.`;
                testSuccess = true;
                break;

            case 'StoppedAtInvokeSigned':
                outcome = 'stopped_at_invoke_signed';
                description = `Execution stopped at INVOKE_SIGNED operation. ` +
                             `Successfully tested ${operationsTested.length} operations before signed program invocation.`;
                testSuccess = true;
                break;

            case 'ComputeLimitExceeded':
                outcome = 'compute_limit_exceeded';
                description = `Execution stopped due to compute limit (${computeUnits} CU). ` +
                             `Successfully tested ${operationsTested.length} operations before limit.`;
                testSuccess = true;
                break;

            case 'Failed':
                outcome = 'failed';
                description = `Execution failed: ${errorMessage || 'Unknown error'}. ` +
                             `Tested ${operationsTested.length} operations before failure.`;
                testSuccess = false;
                break;

            default:
                outcome = 'failed';
                description = `Unknown execution status: ${status}`;
                testSuccess = false;
        }

        return {
            outcome,
            description,
            operations_tested: operationsTested,
            stopped_at_operation: stoppedAtOpcodeName || undefined,
            final_state: {
                compute_units_used: computeUnits,
                instruction_pointer: instructionPointer,
                stack_size: Array.isArray(finalStack) ? finalStack.length : 0,
                has_result: result.has_result_value(),
            },
            error_details: errorMessage || undefined,
            test_success: testSuccess,
        };
    }

    /**
     * Analyze which operations were executed based on instruction pointer
     */
    private analyzeExecutedOperations(bytecode: Uint8Array, finalIP: number): string[] {
        const operations: string[] = [];
        
        if (bytecode.length < 4) {
            return operations;
        }

        // Skip magic bytes (first 4 bytes)
        let ip = 4;
        
        while (ip < finalIP && ip < bytecode.length) {
            const opcode = bytecode[ip];
            const opcodeName = this.getOpcodeName(opcode);
            operations.push(opcodeName);
            
            // Advance IP based on instruction size
            const instructionSize = this.getInstructionSize(bytecode, ip);
            ip += instructionSize;
        }

        return operations;
    }

    /**
     * Get opcode name from opcode number
     */
    private getOpcodeName(opcode: number): string {
        if (!this.constants) {
            return `OPCODE_${opcode}`;
        }

        // Find opcode name by value
        for (const [name, value] of Object.entries(this.constants.opcodes)) {
            if (value === opcode) {
                return name;
            }
        }

        return `UNKNOWN_${opcode}`;
    }

    /**
     * Get instruction size for given opcode
     */
    private getInstructionSize(bytecode: Uint8Array, ip: number): number {
        if (ip >= bytecode.length) {
            return 1;
        }

        const opcode = bytecode[ip];
        
        // Handle PUSH instruction which has variable size
        if (opcode === this.constants?.opcodes?.PUSH) {
            if (ip + 1 < bytecode.length) {
                const valueType = bytecode[ip + 1];
                return 2 + this.getValueTypeSize(valueType);
            }
        }

        // Most instructions are single byte
        return 1;
    }

    /**
     * Get size of value type for PUSH instructions
     */
    private getValueTypeSize(valueType: number): number {
        const typeSizes: { [key: number]: number } = {
            1: 8, // U64
            2: 1, // BOOL  
            3: 32, // PUBKEY
            4: 8, // I64
            5: 1, // U8
            6: 0, // STRING (variable length - simplified)
            7: 0, // ACCOUNT (variable length - simplified)
        };

        return typeSizes[valueType] || 0;
    }

    /**
     * Create a test account for WASM execution
     */
    createTestAccount(
        key: string | Uint8Array,
        data: Uint8Array = new Uint8Array(64),
        lamports: bigint = BigInt(1000000),
        isWritable: boolean = true,
        isSigner: boolean = false,
        owner: string | Uint8Array = new Uint8Array(32)
    ): WasmAccountInterface {
        const keyBytes = typeof key === 'string' ? 
            this.hexStringToUint8Array(key, 32) : key;
        const ownerBytes = typeof owner === 'string' ? 
            this.hexStringToUint8Array(owner, 32) : owner;

        if (keyBytes.length !== 32) {
            throw new Error('Account key must be 32 bytes');
        }
        if (ownerBytes.length !== 32) {
            throw new Error('Account owner must be 32 bytes');
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
     * Helper to convert hex string to Uint8Array
     */
    private hexStringToUint8Array(hex: string, expectedLength: number): Uint8Array {
        const bytes = new Uint8Array(expectedLength);
        
        // Remove '0x' prefix if present
        const cleanHex = hex.startsWith('0x') ? hex.slice(2) : hex;
        
        // Pad with zeros if needed
        const paddedHex = cleanHex.padStart(expectedLength * 2, '0');
        
        for (let i = 0; i < expectedLength; i++) {
            const byte = paddedHex.substr(i * 2, 2);
            bytes[i] = parseInt(byte, 16);
        }

        return bytes;
    }

    /**
     * Get VM constants
     */
    getConstants(): any {
        this.ensureInitialized();
        return this.constants;
    }

    /**
     * Create helper bytecode for testing
     */
    createTestBytecode(operations: Array<{ opcode: string, args?: any[] }>): Uint8Array {
        const magic = [0x53, 0x43, 0x52, 0x4C]; // "SCRL"
        const bytecode: number[] = [...magic];

        for (const op of operations) {
            const opcodeValue = this.constants.opcodes[op.opcode];
            if (opcodeValue === undefined) {
                throw new Error(`Unknown opcode: ${op.opcode}`);
            }

            bytecode.push(opcodeValue);

            // Handle arguments for specific opcodes
            if (op.opcode === 'PUSH' && op.args && op.args.length >= 2) {
                const [valueType, value] = op.args;
                const typeValue = this.constants.types[valueType];
                if (typeValue === undefined) {
                    throw new Error(`Unknown type: ${valueType}`);
                }
                
                bytecode.push(typeValue);
                
                // Encode value based on type
                if (valueType === 'U64' || valueType === 'I64') {
                    const value64 = BigInt(value);
                    const bytes = new Array(8);
                    for (let i = 0; i < 8; i++) {
                        bytes[i] = Number((value64 >> BigInt(i * 8)) & BigInt(0xFF));
                    }
                    bytecode.push(...bytes);
                } else if (valueType === 'U8') {
                    bytecode.push(value & 0xFF);
                } else if (valueType === 'BOOL') {
                    bytecode.push(value ? 1 : 0);
                }
            }
        }

        return new Uint8Array(bytecode);
    }

    /**
     * Ensure service is initialized
     */
    private ensureInitialized(): void {
        if (!this.initialized) {
            throw new Error('WasmCompilerService not initialized. Call initialize() first.');
        }
    }
}

/**
 * Helper functions for test result interpretation
 */
export class TestResultHelper {
    /**
     * Check if a test result represents successful testing
     * (even if execution stopped at system calls)
     */
    static isSuccessfulTest(result: PartialExecutionSummary): boolean {
        return result.test_success;
    }

    /**
     * Get human-readable status message
     */
    static getStatusMessage(result: PartialExecutionSummary): string {
        return result.description;
    }

    /**
     * Check if execution was stopped by a system call
     */
    static wasStoppedAtSystemCall(result: PartialExecutionSummary): boolean {
        return ['stopped_at_system_call', 'stopped_at_init_pda', 'stopped_at_invoke', 'stopped_at_invoke_signed']
            .includes(result.outcome);
    }

    /**
     * Get operations that were successfully tested
     */
    static getTestedOperations(result: PartialExecutionSummary): string[] {
        return result.operations_tested;
    }

    /**
     * Format execution summary for display
     */
    static formatSummary(result: PartialExecutionSummary): string {
        const lines = [
            `Status: ${result.outcome.toUpperCase()}`,
            `Description: ${result.description}`,
            `Operations Tested: ${result.operations_tested.join(', ') || 'None'}`,
            `Compute Units Used: ${result.final_state.compute_units_used}`,
            `Final Instruction Pointer: ${result.final_state.instruction_pointer}`,
            `Stack Size: ${result.final_state.stack_size}`,
        ];

        if (result.stopped_at_operation) {
            lines.push(`Stopped At: ${result.stopped_at_operation}`);
        }

        if (result.error_details) {
            lines.push(`Error: ${result.error_details}`);
        }

        return lines.join('\n');
    }
}

// Default export
export default WasmCompilerService;
