/**
 * WASM-based execution and testing service.
 * Not a compiler; uses the WASM module to run bytecode and report partial execution.
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
     * Executes bytecode and reports status, including interruptions at system calls.
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
     * Interpret TestResult.
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

        const headerInfo = this.getHeaderInfo(bytecode);
        let ip = headerInfo?.startOffset ?? 0;
        
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
     * Parse minimal optimized header info for analysis.
     * Returns start offset and features when available.
     */
    private getHeaderInfo(bytecode: Uint8Array): { startOffset: number; features: number } | null {
        if (bytecode.length < 4) {
            return null;
        }

        const magic = this.constants?.FIVE_MAGIC
            ? Array.from(this.constants.FIVE_MAGIC)
            : [0x35, 0x49, 0x56, 0x45]; // "5IVE"

        const isMagic =
            bytecode[0] === magic[0] &&
            bytecode[1] === magic[1] &&
            bytecode[2] === magic[2] &&
            bytecode[3] === magic[3];

        if (!isMagic) {
            return { startOffset: 0, features: 0 };
        }

        if (bytecode.length < 10) {
            return { startOffset: 4, features: 0 };
        }

        const features =
            (bytecode[4]) |
            (bytecode[5] << 8) |
            (bytecode[6] << 16) |
            (bytecode[7] << 24);

        let offset = 10;

        const FEATURE_FUNCTION_NAMES = 1 << 8;
        const FEATURE_CONSTANT_POOL = 1 << 10;

        if ((features & FEATURE_FUNCTION_NAMES) !== 0) {
            if (offset + 2 > bytecode.length) {
                return { startOffset: offset, features };
            }
            const sectionSize = bytecode[offset] | (bytecode[offset + 1] << 8);
            offset += 2 + sectionSize;
        }

        if ((features & FEATURE_CONSTANT_POOL) !== 0) {
            const descSize = 16;
            if (offset + descSize > bytecode.length) {
                return { startOffset: offset, features };
            }
            const poolOffset =
                (bytecode[offset]) |
                (bytecode[offset + 1] << 8) |
                (bytecode[offset + 2] << 16) |
                (bytecode[offset + 3] << 24);
            const poolSlots = bytecode[offset + 12] | (bytecode[offset + 13] << 8);
            const poolSize = poolSlots * 8;
            const codeOffset = poolOffset + poolSize;
            if (codeOffset > 0 && codeOffset <= bytecode.length) {
                offset = codeOffset;
            }
        }

        if (offset > bytecode.length) {
            offset = bytecode.length;
        }

        return { startOffset: offset, features };
    }

    /**
     * Get instruction size for given opcode
     */
    private getInstructionSize(bytecode: Uint8Array, ip: number): number {
        if (ip >= bytecode.length) {
            return 1;
        }

        const opcode = bytecode[ip];
        const ops = this.constants?.opcodes;
        if (!ops) return 1;

        const headerInfo = this.getHeaderInfo(bytecode);
        const features = headerInfo?.features ?? 0;
        const FEATURE_CONSTANT_POOL = 1 << 10;
        const constantPoolEnabled = (features & FEATURE_CONSTANT_POOL) !== 0;

        // 1 byte opcodes (no args)
        // ... handled by default return 1

        // Handle specific opcodes with arguments based on protocol
        if (opcode === ops.PUSH_U8 || opcode === ops.PUSH_BOOL ||
            opcode === ops.PUSH_STRING_LITERAL || opcode === ops.PUSH_ARRAY_LITERAL ||
            opcode === ops.CREATE_ARRAY || opcode === ops.SET_LOCAL ||
            opcode === ops.GET_LOCAL || opcode === ops.LOAD_PARAM ||
            opcode === ops.STORE_PARAM || opcode === ops.CAST ||
            opcode === ops.ALLOC_LOCALS || opcode === ops.CREATE_TUPLE ||
            opcode === ops.CALL_NATIVE || opcode === ops.LOAD_INPUT ||
            opcode === ops.TRANSFER_DEBIT || opcode === ops.TRANSFER_CREDIT ||
            opcode === ops.BULK_LOAD_FIELD_N) {
            return 2; // opcode + u8
        }

        if (opcode === ops.PUSH_U16) {
            if (constantPoolEnabled) {
                return 2; // opcode + u8 index
            }
            return 3; // opcode + u16
        }

        if (opcode === ops.PUSH_U32) {
            if (constantPoolEnabled) {
                return 2; // opcode + u8 index
            }
            return 5; // opcode + u32
        }

        if (opcode === ops.PUSH_U64 || opcode === ops.PUSH_I64) {
            if (constantPoolEnabled) {
                return 2; // opcode + u8 index
            }
            return 9; // opcode + u64
        }

        if (opcode === ops.PUSH_PUBKEY) {
            if (constantPoolEnabled) {
                return 2; // opcode + u8 index
            }
            return 33; // opcode + 32 bytes
        }

        if (opcode === ops.PUSH_U128) {
            if (constantPoolEnabled) {
                return 2; // opcode + u8 index
            }
            return 17; // opcode + 16 bytes
        }

        if (opcode === ops.PUSH_STRING) {
            if (constantPoolEnabled) {
                return 2; // opcode + u8 index
            }
            // u32 length + string bytes
            if (ip + 5 <= bytecode.length) {
                const len =
                    bytecode[ip + 1] |
                    (bytecode[ip + 2] << 8) |
                    (bytecode[ip + 3] << 16) |
                    (bytecode[ip + 4] << 24);
                return 5 + len;
            }
            return 5; // opcode + u32 length
        }

        if (opcode === ops.PUSH_U8_W || opcode === ops.PUSH_U16_W || opcode === ops.PUSH_U32_W ||
            opcode === ops.PUSH_U64_W || opcode === ops.PUSH_I64_W || opcode === ops.PUSH_BOOL_W ||
            opcode === ops.PUSH_U128_W || opcode === ops.PUSH_PUBKEY_W || opcode === ops.PUSH_STRING_W) {
            return 3; // opcode + u16 index/operand
        }

        if (opcode === ops.PUSH_STRING_LITERAL || opcode === ops.PUSH_ARRAY_LITERAL) {
            if (ip + 2 <= bytecode.length) {
                const len = bytecode[ip + 1];
                return 2 + len;
            }
            return 2;
        }

        if (opcode === ops.STORE || opcode === ops.LOAD_FIELD || opcode === ops.STORE_FIELD ||
            opcode === ops.LOAD_EXTERNAL_FIELD || opcode === ops.LOAD_FIELD_PUBKEY) {
            return 6; // opcode + u8 + u32
        }

        if (opcode === ops.LOAD) {
            return 1; // no immediate operand
        }

        if (opcode === ops.LOAD_GLOBAL || opcode === ops.STORE_GLOBAL) {
            return 3; // opcode + u16 id
        }

        if (opcode === ops.LOAD_ACCOUNT ||
            opcode === ops.SAVE_ACCOUNT || opcode === ops.GET_ACCOUNT ||
            opcode === ops.GET_LAMPORTS || opcode === ops.SET_LAMPORTS ||
            opcode === ops.GET_DATA || opcode === ops.GET_KEY ||
            opcode === ops.GET_OWNER || opcode === ops.INIT_ACCOUNT ||
            opcode === ops.INIT_PDA_ACCOUNT || opcode === ops.CHECK_SIGNER ||
            opcode === ops.CHECK_WRITABLE || opcode === ops.CHECK_OWNER ||
            opcode === ops.CHECK_INITIALIZED || opcode === ops.CHECK_PDA ||
            opcode === ops.CHECK_UNINITIALIZED) {
            return 2; // opcode + u8
        }

        if (opcode === ops.JUMP || opcode === ops.JUMP_IF || opcode === ops.JUMP_IF_NOT ||
            opcode === ops.EQ_ZERO_JUMP || opcode === ops.GT_ZERO_JUMP || opcode === ops.LT_ZERO_JUMP) {
            return 3; // opcode + u16 (fixed)
        }

        if (opcode === ops.CALL) {
             // u8 param_count + u16 func_addr
             return 4;
        }

        if (opcode === ops.CALL_EXTERNAL) {
            // u8 acc + u16 off + u8 param
            return 5;
        }

        if (opcode === ops.BR_EQ_U8) {
            // u8 val + u16 off
            return 4;
        }

        // 1 byte opcodes
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
        const magic: number[] = this.constants?.FIVE_MAGIC
            ? Array.from(this.constants.FIVE_MAGIC as Iterable<number>)
            : [0x35, 0x49, 0x56, 0x45]; // "5IVE"
        const bytecode: number[] = [
            ...magic,
            0x00, 0x00, 0x00, 0x00, // features (u32 LE)
            0x00, // public_function_count
            0x00  // total_function_count
        ];

        for (const op of operations) {
            const isPushAlias = op.opcode === 'PUSH';
            const resolvedOpcode = isPushAlias && op.args && op.args.length > 0
                ? `PUSH_${op.args[0]}`
                : op.opcode;

            const opcodeValue = this.constants.opcodes[resolvedOpcode];
            if (opcodeValue === undefined) {
                if (isPushAlias) {
                    throw new Error(`Unknown type: ${op.args?.[0]}`);
                }
                throw new Error(`Unknown opcode: ${resolvedOpcode}`);
            }

            bytecode.push(opcodeValue);

            const opArgs = isPushAlias ? op.args?.slice(1) : op.args;

            const pushFixedLE = (value: number | bigint, bytes: number) => {
                let v = BigInt(value);
                for (let i = 0; i < bytes; i++) {
                    bytecode.push(Number(v & BigInt(0xFF)));
                    v >>= BigInt(8);
                }
            };

            // Handle arguments for specific opcodes
            if (resolvedOpcode === 'PUSH_U64' || resolvedOpcode === 'PUSH_I64') {
                if (opArgs && opArgs.length > 0) {
                    pushFixedLE(opArgs[0], 8);
                }
            } else if (resolvedOpcode === 'PUSH_U32') {
                if (opArgs && opArgs.length > 0) {
                    pushFixedLE(opArgs[0], 4);
                }
            } else if (resolvedOpcode === 'PUSH_U16') {
                if (opArgs && opArgs.length > 0) {
                    pushFixedLE(opArgs[0], 2);
                }
            } else if (resolvedOpcode === 'PUSH_U8' || resolvedOpcode === 'PUSH_BOOL' ||
                       resolvedOpcode === 'PUSH_STRING_LITERAL' || resolvedOpcode === 'PUSH_ARRAY_LITERAL' ||
                       resolvedOpcode === 'CREATE_ARRAY' || resolvedOpcode === 'SET_LOCAL' ||
                       resolvedOpcode === 'GET_LOCAL' || resolvedOpcode === 'LOAD_PARAM' ||
                       resolvedOpcode === 'STORE_PARAM' || resolvedOpcode === 'CAST') {
                if (opArgs && opArgs.length > 0) {
                    bytecode.push(opArgs[0] & 0xFF);
                }
            } else if (resolvedOpcode === 'PUSH_PUBKEY') {
                if (opArgs && opArgs.length > 0) {
                    // Expect 32 byte array
                    const pk = opArgs[0];
                    if (pk.length === 32) {
                        bytecode.push(...pk);
                    }
                }
            } else if (resolvedOpcode === 'PUSH_U128') {
                if (opArgs && opArgs.length > 0) {
                    pushFixedLE(opArgs[0], 16);
                }
            } else if (resolvedOpcode === 'PUSH_STRING') {
                if (opArgs && opArgs.length > 0) {
                    const encoder = new TextEncoder();
                    const data = typeof opArgs[0] === 'string' ? encoder.encode(opArgs[0]) : opArgs[0];
                    pushFixedLE(data.length, 4);
                    bytecode.push(...data);
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
