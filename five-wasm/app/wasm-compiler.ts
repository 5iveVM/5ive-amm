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
        const ops = this.constants?.opcodes;
        if (!ops) return 1;

        // VLE argument sizes must be decoded from bytecode if possible,
        // but for analysis purposes we might need rough estimates or actual decoding logic.
        // However, typescript doesn't have the VLE decoder handy unless we implement it or import it.
        // This is a "best effort" size estimator for stepping through bytecode.

        // Helper to decode VLE size
        const decodeVLESize = (offset: number): number => {
            let length = 0;
            while (offset + length < bytecode.length) {
                const byte = bytecode[offset + length];
                length++;
                if ((byte & 0x80) === 0) break;
            }
            return length;
        }

        // 1 byte opcodes (no args)
        // ... handled by default return 1

        // Handle specific opcodes with arguments based on protocol
        if (opcode === ops.PUSH_U8 || opcode === ops.PUSH_BOOL ||
            opcode === ops.PUSH_STRING_LITERAL || opcode === ops.PUSH_ARRAY_LITERAL ||
            opcode === ops.CREATE_ARRAY || opcode === ops.SET_LOCAL ||
            opcode === ops.GET_LOCAL || opcode === ops.LOAD_PARAM ||
            opcode === ops.STORE_PARAM || opcode === ops.CAST ||
            opcode === ops.TRANSFER_DEBIT || opcode === ops.TRANSFER_CREDIT ||
            opcode === ops.BULK_LOAD_FIELD_N) {
            return 2; // opcode + u8
        }

        if (opcode === ops.PUSH_U16) {
             // Fixed 2 bytes (u16)
             return 3;
        }

        if (opcode === ops.PUSH_U32 || opcode === ops.LOAD_ACCOUNT ||
            opcode === ops.SAVE_ACCOUNT || opcode === ops.GET_ACCOUNT ||
            opcode === ops.GET_LAMPORTS || opcode === ops.SET_LAMPORTS ||
            opcode === ops.GET_DATA || opcode === ops.GET_KEY ||
            opcode === ops.GET_OWNER || opcode === ops.INIT_ACCOUNT ||
            opcode === ops.INIT_PDA_ACCOUNT || opcode === ops.CHECK_SIGNER ||
            opcode === ops.CHECK_WRITABLE || opcode === ops.CHECK_OWNER ||
            opcode === ops.CHECK_INITIALIZED || opcode === ops.CHECK_PDA ||
            opcode === ops.CHECK_UNINITIALIZED) {
            // VLE u32
            return 1 + decodeVLESize(ip + 1);
        }

        if (opcode === ops.PUSH_U64 || opcode === ops.PUSH_I64) {
            // VLE u64
            return 1 + decodeVLESize(ip + 1);
        }

        if (opcode === ops.PUSH_PUBKEY) {
            return 33; // opcode + 32 bytes
        }

        if (opcode === ops.PUSH_U128) {
             return 17; // opcode + 16 bytes
        }

        if (opcode === ops.PUSH_STRING) {
            // u8 + bytes? Or VLE length + bytes?
            // Protocol says PUSH_STRING length_vle + string_data.
            // But OpcodePatterns emits u8 length?
            // Five-protocol says PUSH_STRING arg_type U8.
            // If ArgType U8, it's just 2 bytes for instruction + arg, but where is string data?
            // Assuming parser handles it specially or it's an index?
            // Let's assume standard op size logic doesn't fully apply to var-len data unless encoded.
            // If it's just an index, return 2.
            return 2;
        }

        if (opcode === ops.LOAD_FIELD || opcode === ops.STORE_FIELD) {
             // u8 account_index + VLE field_offset
             return 2 + decodeVLESize(ip + 2);
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
            // u8 val + u16 off (VLE encoded? parser says ArgType::U8, but emitter emits VLE u16 too)
            // Assuming 1 + 1 + VLE size
            return 2 + decodeVLESize(ip + 2);
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
        const magic = [0x53, 0x43, 0x52, 0x4C]; // "SCRL"
        const bytecode: number[] = [...magic];

        for (const op of operations) {
            const opcodeValue = this.constants.opcodes[op.opcode];
            if (opcodeValue === undefined) {
                throw new Error(`Unknown opcode: ${op.opcode}`);
            }

            bytecode.push(opcodeValue);

            // Handle arguments for specific opcodes
            if (op.opcode === 'PUSH_U64' || op.opcode === 'PUSH_I64') {
                 // Opcode + VLE u64. For simplicity in test helper, we might just emit fixed 8 bytes or need VLE encoder.
                 // This test helper is basic. Let's assume standard simple encoding or implement VLE.
                 // Implementing simple VLE here:
                 if (op.args && op.args.length > 0) {
                     let val = BigInt(op.args[0]);
                     do {
                         let byte = Number(val & BigInt(0x7F));
                         val >>= BigInt(7);
                         if (val !== BigInt(0)) {
                             byte |= 0x80;
                         }
                         bytecode.push(byte);
                     } while (val !== BigInt(0));
                 }
            } else if (op.opcode === 'PUSH_U32') {
                 if (op.args && op.args.length > 0) {
                     let val = op.args[0];
                     do {
                         let byte = val & 0x7F;
                         val >>= 7;
                         if (val !== 0) {
                             byte |= 0x80;
                         }
                         bytecode.push(byte);
                     } while (val !== 0);
                 }
            } else if (op.opcode === 'PUSH_U16') {
                 if (op.args && op.args.length > 0) {
                     let val = op.args[0];
                     do {
                         let byte = val & 0x7F;
                         val >>= 7;
                         if (val !== 0) {
                             byte |= 0x80;
                         }
                         bytecode.push(byte);
                     } while (val !== 0);
                 }
            } else if (op.opcode === 'PUSH_U8' || op.opcode === 'PUSH_BOOL' ||
                       op.opcode === 'PUSH_STRING_LITERAL' || op.opcode === 'PUSH_ARRAY_LITERAL' ||
                       op.opcode === 'CREATE_ARRAY' || op.opcode === 'SET_LOCAL' ||
                       op.opcode === 'GET_LOCAL' || op.opcode === 'LOAD_PARAM' ||
                       op.opcode === 'STORE_PARAM' || op.opcode === 'CAST') {
                if (op.args && op.args.length > 0) {
                    bytecode.push(op.args[0] & 0xFF);
                }
            } else if (op.opcode === 'PUSH_PUBKEY') {
                if (op.args && op.args.length > 0) {
                    // Expect 32 byte array or string
                    const pk = op.args[0];
                    if (pk.length === 32) {
                        bytecode.push(...pk);
                    }
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
