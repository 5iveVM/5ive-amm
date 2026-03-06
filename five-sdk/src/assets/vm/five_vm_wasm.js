/* @ts-self-types="./five_vm_wasm.d.ts" */

/**
 * Bytecode analyzer for WASM
 */
class BytecodeAnalyzer {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BytecodeAnalyzerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_bytecodeanalyzer_free(ptr, 0);
    }
    /**
     * Analyze bytecode and return instruction breakdown (legacy method for compatibility)
     * @param {Uint8Array} bytecode
     * @returns {any}
     */
    static analyze(bytecode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.bytecodeanalyzer_analyze(retptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get detailed opcode flow analysis - shows execution paths through the bytecode
     * @param {Uint8Array} bytecode
     * @returns {any}
     */
    static analyze_execution_flow(bytecode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.bytecodeanalyzer_analyze_execution_flow(retptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get detailed information about a specific instruction at an offset
     * @param {Uint8Array} bytecode
     * @param {number} offset
     * @returns {any}
     */
    static analyze_instruction_at(bytecode, offset) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.bytecodeanalyzer_analyze_instruction_at(retptr, ptr0, len0, offset);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Advanced semantic analysis with full opcode understanding and instruction flow
     * Performs semantic analysis of bytecode to understand opcode behavior
     * and instruction flow.
     * @param {Uint8Array} bytecode
     * @returns {any}
     */
    static analyze_semantic(bytecode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.bytecodeanalyzer_analyze_semantic(retptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get summary statistics about the bytecode
     * @param {Uint8Array} bytecode
     * @returns {any}
     */
    static get_bytecode_summary(bytecode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.bytecodeanalyzer_get_bytecode_summary(retptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) BytecodeAnalyzer.prototype[Symbol.dispose] = BytecodeAnalyzer.prototype.free;
exports.BytecodeAnalyzer = BytecodeAnalyzer;

/**
 * Bytecode Encoding utilities for JavaScript (Fixed Size)
 */
class BytecodeEncoder {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        BytecodeEncoderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_bytecodeencoder_free(ptr, 0);
    }
    /**
     * Decode a u16 value
     * Returns [value, bytes_consumed] or null if invalid
     * @param {Uint8Array} bytes
     * @returns {Array<any> | undefined}
     */
    static decode_u16(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bytecodeencoder_decode_u16(ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Decode a u32 value
     * Returns [value, bytes_consumed] or null if invalid
     * @param {Uint8Array} bytes
     * @returns {Array<any> | undefined}
     */
    static decode_u32(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bytecodeencoder_decode_u32(ptr0, len0);
        return takeObject(ret);
    }
    /**
     * Encode a u16 value
     * Returns [size, byte1, byte2]
     * @param {number} value
     * @returns {Array<any>}
     */
    static encode_u16(value) {
        const ret = wasm.bytecodeencoder_encode_u16(value);
        return takeObject(ret);
    }
    /**
     * Encode a u32 value
     * Returns [size, byte1, byte2, byte3, byte4]
     * @param {number} value
     * @returns {Array<any>}
     */
    static encode_u32(value) {
        const ret = wasm.bytecodeencoder_encode_u32(value);
        return takeObject(ret);
    }
    /**
     * Calculate encoded size (Always 2 for u16)
     * @param {number} _value
     * @returns {number}
     */
    static encoded_size_u16(_value) {
        const ret = wasm.bytecodeencoder_encoded_size_u16(_value);
        return ret >>> 0;
    }
    /**
     * Calculate encoded size (Always 4 for u32)
     * @param {number} _value
     * @returns {number}
     */
    static encoded_size_u32(_value) {
        const ret = wasm.bytecodeencoder_encoded_size_u32(_value);
        return ret >>> 0;
    }
}
if (Symbol.dispose) BytecodeEncoder.prototype[Symbol.dispose] = BytecodeEncoder.prototype.free;
exports.BytecodeEncoder = BytecodeEncoder;

/**
 * Execution result.
 * @enum {0 | 1 | 2 | 3 | 4 | 5 | 6}
 */
const ExecutionStatus = Object.freeze({
    /**
     * All operations completed successfully.
     */
    Completed: 0, "0": "Completed",
    /**
     * Execution stopped because it hit a system program call that cannot be executed in WASM.
     */
    StoppedAtSystemCall: 1, "1": "StoppedAtSystemCall",
    /**
     * Execution stopped because it hit an INIT_PDA operation that requires real Solana context.
     */
    StoppedAtInitPDA: 2, "2": "StoppedAtInitPDA",
    /**
     * Execution stopped because it hit an INVOKE operation that requires real RPC.
     */
    StoppedAtInvoke: 3, "3": "StoppedAtInvoke",
    /**
     * Execution stopped because it hit an INVOKE_SIGNED operation that requires real RPC.
     */
    StoppedAtInvokeSigned: 4, "4": "StoppedAtInvokeSigned",
    /**
     * Execution stopped because compute limit was reached.
     */
    ComputeLimitExceeded: 5, "5": "ComputeLimitExceeded",
    /**
     * Execution failed due to an error.
     */
    Failed: 6, "6": "Failed",
});
exports.ExecutionStatus = ExecutionStatus;

/**
 * JavaScript-compatible VM state representation.
 */
class FiveVMState {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FiveVMStateFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_fivevmstate_free(ptr, 0);
    }
    /**
     * @returns {bigint}
     */
    get compute_units() {
        const ret = wasm.fivevmstate_compute_units(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @returns {number}
     */
    get instruction_pointer() {
        const ret = wasm.fivevmstate_instruction_pointer(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Array<any>}
     */
    get stack() {
        const ret = wasm.fivevmstate_stack(this.__wbg_ptr);
        return takeObject(ret);
    }
}
if (Symbol.dispose) FiveVMState.prototype[Symbol.dispose] = FiveVMState.prototype.free;
exports.FiveVMState = FiveVMState;

/**
 * Main WASM VM wrapper.
 */
class FiveVMWasm {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FiveVMWasmFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_fivevmwasm_free(ptr, 0);
    }
    /**
     * Execute VM with input data and accounts (legacy method).
     * @param {Uint8Array} input_data
     * @param {Array<any>} accounts
     * @returns {any}
     */
    execute(input_data, accounts) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(input_data, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.fivevmwasm_execute(retptr, this.__wbg_ptr, ptr0, len0, addHeapObject(accounts));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Execute VM with partial execution support - stops at system calls.
     * @param {Uint8Array} input_data
     * @param {Array<any>} accounts
     * @returns {TestResult}
     */
    execute_partial(input_data, accounts) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(input_data, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.fivevmwasm_execute_partial(retptr, this.__wbg_ptr, ptr0, len0, addHeapObject(accounts));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return TestResult.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get VM constants for JavaScript
     * @returns {any}
     */
    static get_constants() {
        const ret = wasm.fivevmwasm_get_constants();
        return takeObject(ret);
    }
    /**
     * Get current VM state
     * @returns {any}
     */
    get_state() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.fivevmwasm_get_state(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Create new VM instance with bytecode.
     * @param {Uint8Array} _bytecode
     */
    constructor(_bytecode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(_bytecode, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.fivevmwasm_new(retptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            this.__wbg_ptr = r0 >>> 0;
            FiveVMWasmFinalization.register(this, this.__wbg_ptr, this);
            return this;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Validate bytecode without execution
     * @param {Uint8Array} bytecode
     * @returns {boolean}
     */
    static validate_bytecode(bytecode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.fivevmwasm_validate_bytecode(retptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return r0 !== 0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) FiveVMWasm.prototype[Symbol.dispose] = FiveVMWasm.prototype.free;
exports.FiveVMWasm = FiveVMWasm;

/**
 * Parameter encoding utilities using fixed-size encoding and protocol types
 */
class ParameterEncoder {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        ParameterEncoderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_parameterencoder_free(ptr, 0);
    }
    /**
     * Encode function parameters using fixed size encoding
     * Returns ONLY parameter data - SDK handles discriminator AND function index
     * @param {number} _function_index
     * @param {Array<any>} params
     * @returns {Uint8Array}
     */
    static encode_execute(_function_index, params) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.parameterencoder_encode_execute(retptr, _function_index, addHeapObject(params));
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) ParameterEncoder.prototype[Symbol.dispose] = ParameterEncoder.prototype.free;
exports.ParameterEncoder = ParameterEncoder;

/**
 * Detailed execution result.
 */
class TestResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(TestResult.prototype);
        obj.__wbg_ptr = ptr;
        TestResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        TestResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_testresult_free(ptr, 0);
    }
    /**
     * Compute units consumed.
     * @returns {bigint}
     */
    get compute_units_used() {
        const ret = wasm.__wbg_get_testresult_compute_units_used(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Final instruction pointer.
     * @returns {number}
     */
    get instruction_pointer() {
        const ret = wasm.__wbg_get_testresult_instruction_pointer(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Which opcode caused the stop (if stopped at system call).
     * @returns {number | undefined}
     */
    get stopped_at_opcode() {
        const ret = wasm.__wbg_get_testresult_stopped_at_opcode(this.__wbg_ptr);
        return ret === 0xFFFFFF ? undefined : ret;
    }
    /**
     * Compute units consumed.
     * @param {bigint} arg0
     */
    set compute_units_used(arg0) {
        wasm.__wbg_set_testresult_compute_units_used(this.__wbg_ptr, arg0);
    }
    /**
     * Final instruction pointer.
     * @param {number} arg0
     */
    set instruction_pointer(arg0) {
        wasm.__wbg_set_testresult_instruction_pointer(this.__wbg_ptr, arg0);
    }
    /**
     * Which opcode caused the stop (if stopped at system call).
     * @param {number | null} [arg0]
     */
    set stopped_at_opcode(arg0) {
        wasm.__wbg_set_testresult_stopped_at_opcode(this.__wbg_ptr, isLikeNone(arg0) ? 0xFFFFFF : arg0);
    }
    /**
     * @returns {string | undefined}
     */
    get error_message() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.testresult_error_message(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {string | undefined}
     */
    get execution_context() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.testresult_execution_context(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {Array<any>}
     */
    get final_accounts() {
        const ret = wasm.testresult_final_accounts(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {Uint8Array}
     */
    get final_memory() {
        const ret = wasm.testresult_final_memory(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {Array<any>}
     */
    get final_stack() {
        const ret = wasm.testresult_final_stack(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {any}
     */
    get get_result_value() {
        const ret = wasm.testresult_get_result_value(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {boolean}
     */
    get has_result_value() {
        const ret = wasm.testresult_has_result_value(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {string}
     */
    get status() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.testresult_status(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string | undefined}
     */
    get stopped_at_opcode_name() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.testresult_stopped_at_opcode_name(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) TestResult.prototype[Symbol.dispose] = TestResult.prototype.free;
exports.TestResult = TestResult;

/**
 * JavaScript-compatible account representation.
 */
class WasmAccount {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmAccountFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmaccount_free(ptr, 0);
    }
    /**
     * @returns {boolean}
     */
    get is_signer() {
        const ret = wasm.__wbg_get_wasmaccount_is_signer(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {boolean}
     */
    get is_writable() {
        const ret = wasm.__wbg_get_wasmaccount_is_writable(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {bigint}
     */
    get lamports() {
        const ret = wasm.__wbg_get_wasmaccount_lamports(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {boolean} arg0
     */
    set is_signer(arg0) {
        wasm.__wbg_set_wasmaccount_is_signer(this.__wbg_ptr, arg0);
    }
    /**
     * @param {boolean} arg0
     */
    set is_writable(arg0) {
        wasm.__wbg_set_wasmaccount_is_writable(this.__wbg_ptr, arg0);
    }
    /**
     * @param {bigint} arg0
     */
    set lamports(arg0) {
        wasm.__wbg_set_wasmaccount_lamports(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {Uint8Array}
     */
    get data() {
        const ret = wasm.wasmaccount_data(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {Uint8Array}
     */
    get key() {
        const ret = wasm.wasmaccount_key(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @param {Uint8Array} key
     * @param {Uint8Array} data
     * @param {bigint} lamports
     * @param {boolean} is_writable
     * @param {boolean} is_signer
     * @param {Uint8Array} owner
     */
    constructor(key, data, lamports, is_writable, is_signer, owner) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(key, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_export);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passArray8ToWasm0(owner, wasm.__wbindgen_export);
            const len2 = WASM_VECTOR_LEN;
            wasm.wasmaccount_new(retptr, ptr0, len0, ptr1, len1, lamports, is_writable, is_signer, ptr2, len2);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            this.__wbg_ptr = r0 >>> 0;
            WasmAccountFinalization.register(this, this.__wbg_ptr, this);
            return this;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {Uint8Array}
     */
    get owner() {
        const ret = wasm.wasmaccount_owner(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @param {Uint8Array} data
     */
    set data(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmaccount_set_data(this.__wbg_ptr, ptr0, len0);
    }
}
if (Symbol.dispose) WasmAccount.prototype[Symbol.dispose] = WasmAccount.prototype.free;
exports.WasmAccount = WasmAccount;

/**
 * WASM source analysis result
 */
class WasmAnalysisResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmAnalysisResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmAnalysisResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmAnalysisResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmanalysisresult_free(ptr, 0);
    }
    /**
     * Analysis time in milliseconds
     * @returns {number}
     */
    get analysis_time() {
        const ret = wasm.__wbg_get_wasmanalysisresult_analysis_time(this.__wbg_ptr);
        return ret;
    }
    /**
     * Whether analysis succeeded
     * @returns {boolean}
     */
    get success() {
        const ret = wasm.__wbg_get_wasmanalysisresult_success(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Analysis time in milliseconds
     * @param {number} arg0
     */
    set analysis_time(arg0) {
        wasm.__wbg_set_wasmanalysisresult_analysis_time(this.__wbg_ptr, arg0);
    }
    /**
     * Whether analysis succeeded
     * @param {boolean} arg0
     */
    set success(arg0) {
        wasm.__wbg_set_wasmanalysisresult_success(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {Array<any>}
     */
    get errors() {
        const ret = wasm.wasmanalysisresult_errors(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get parsed metrics as JavaScript object
     * @returns {any}
     */
    get_metrics_object() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmanalysisresult_get_metrics_object(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {string}
     */
    get metrics() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmanalysisresult_metrics(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get summary() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmanalysisresult_summary(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmAnalysisResult.prototype[Symbol.dispose] = WasmAnalysisResult.prototype.free;
exports.WasmAnalysisResult = WasmAnalysisResult;

/**
 * Compilation options for enhanced error reporting and formatting
 */
class WasmCompilationOptions {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmCompilationOptions.prototype);
        obj.__wbg_ptr = ptr;
        WasmCompilationOptionsFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmCompilationOptionsFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmcompilationoptions_free(ptr, 0);
    }
    /**
     * Include complexity analysis
     * @returns {boolean}
     */
    get complexity_analysis() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_complexity_analysis(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Include comprehensive metrics collection
     * @returns {boolean}
     */
    get comprehensive_metrics() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_comprehensive_metrics(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Enable bytecode compression
     * @returns {boolean}
     */
    get compress_output() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_compress_output(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Disable REQUIRE_BATCH lowering in compiler pipeline.
     * @returns {boolean}
     */
    get disable_require_batch() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_disable_require_batch(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Enable constraint caching optimization
     * @returns {boolean}
     */
    get enable_constraint_cache() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_enable_constraint_cache(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Enable module namespace qualification (module::function)
     * @returns {boolean}
     */
    get enable_module_namespaces() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_enable_module_namespaces(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Enable enhanced error reporting with suggestions
     * @returns {boolean}
     */
    get enhanced_errors() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_enhanced_errors(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Include debug information
     * @returns {boolean}
     */
    get include_debug_info() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_include_debug_info(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Include basic metrics
     * @returns {boolean}
     */
    get include_metrics() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_include_metrics(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Include performance analysis
     * @returns {boolean}
     */
    get performance_analysis() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_performance_analysis(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Suppress non-essential output
     * @returns {boolean}
     */
    get quiet() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_quiet(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Show compilation summary
     * @returns {boolean}
     */
    get summary() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_summary(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Enable v2-preview features (nibble immediates, BR_EQ_U8, etc.)
     * @returns {boolean}
     */
    get v2_preview() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_v2_preview(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Verbose output
     * @returns {boolean}
     */
    get verbose() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_verbose(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Include complexity analysis
     * @param {boolean} arg0
     */
    set complexity_analysis(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_complexity_analysis(this.__wbg_ptr, arg0);
    }
    /**
     * Include comprehensive metrics collection
     * @param {boolean} arg0
     */
    set comprehensive_metrics(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_comprehensive_metrics(this.__wbg_ptr, arg0);
    }
    /**
     * Enable bytecode compression
     * @param {boolean} arg0
     */
    set compress_output(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_compress_output(this.__wbg_ptr, arg0);
    }
    /**
     * Disable REQUIRE_BATCH lowering in compiler pipeline.
     * @param {boolean} arg0
     */
    set disable_require_batch(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_disable_require_batch(this.__wbg_ptr, arg0);
    }
    /**
     * Enable constraint caching optimization
     * @param {boolean} arg0
     */
    set enable_constraint_cache(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_enable_constraint_cache(this.__wbg_ptr, arg0);
    }
    /**
     * Enable module namespace qualification (module::function)
     * @param {boolean} arg0
     */
    set enable_module_namespaces(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_enable_module_namespaces(this.__wbg_ptr, arg0);
    }
    /**
     * Enable enhanced error reporting with suggestions
     * @param {boolean} arg0
     */
    set enhanced_errors(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_enhanced_errors(this.__wbg_ptr, arg0);
    }
    /**
     * Include debug information
     * @param {boolean} arg0
     */
    set include_debug_info(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_include_debug_info(this.__wbg_ptr, arg0);
    }
    /**
     * Include basic metrics
     * @param {boolean} arg0
     */
    set include_metrics(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_include_metrics(this.__wbg_ptr, arg0);
    }
    /**
     * Include performance analysis
     * @param {boolean} arg0
     */
    set performance_analysis(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_performance_analysis(this.__wbg_ptr, arg0);
    }
    /**
     * Suppress non-essential output
     * @param {boolean} arg0
     */
    set quiet(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_quiet(this.__wbg_ptr, arg0);
    }
    /**
     * Show compilation summary
     * @param {boolean} arg0
     */
    set summary(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_summary(this.__wbg_ptr, arg0);
    }
    /**
     * Enable v2-preview features (nibble immediates, BR_EQ_U8, etc.)
     * @param {boolean} arg0
     */
    set v2_preview(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_v2_preview(this.__wbg_ptr, arg0);
    }
    /**
     * Verbose output
     * @param {boolean} arg0
     */
    set verbose(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_verbose(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {string}
     */
    get analysis_depth() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationoptions_analysis_depth(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Create development-debug configuration
     * @returns {WasmCompilationOptions}
     */
    static development_debug() {
        const ret = wasm.wasmcompilationoptions_development_debug();
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * @returns {string}
     */
    get error_format() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationoptions_error_format(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get export_format() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationoptions_export_format(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Create fast iteration configuration
     * @returns {WasmCompilationOptions}
     */
    static fast_iteration() {
        const ret = wasm.wasmcompilationoptions_fast_iteration();
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * @returns {string}
     */
    get metrics_format() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationoptions_metrics_format(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get mode() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationoptions_mode(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Create default compilation options
     */
    constructor() {
        const ret = wasm.wasmcompilationoptions_new();
        this.__wbg_ptr = ret >>> 0;
        WasmCompilationOptionsFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {string}
     */
    get optimization_level() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationoptions_optimization_level(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Create production-optimized configuration
     * @returns {WasmCompilationOptions}
     */
    static production_optimized() {
        const ret = wasm.wasmcompilationoptions_production_optimized();
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * @returns {string | undefined}
     */
    get source_file() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationoptions_source_file(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Set analysis depth level
     * @param {string} depth
     * @returns {WasmCompilationOptions}
     */
    with_analysis_depth(depth) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(depth, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_analysis_depth(ptr, ptr0, len0);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable complexity analysis
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_complexity_analysis(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_complexity_analysis(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable comprehensive metrics collection
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_comprehensive_metrics(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_comprehensive_metrics(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable bytecode compression
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_compression(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_compression(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable constraint caching optimization
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_constraint_cache(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_constraint_cache(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable debug information
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_debug_info(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_debug_info(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable REQUIRE_BATCH lowering.
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_disable_require_batch(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_disable_require_batch(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable enhanced error reporting
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_enhanced_errors(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_enhanced_errors(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Set error output format
     * @param {string} format
     * @returns {WasmCompilationOptions}
     */
    with_error_format(format) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_error_format(ptr, ptr0, len0);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Set export format
     * @param {string} format
     * @returns {WasmCompilationOptions}
     */
    with_export_format(format) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_export_format(ptr, ptr0, len0);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable basic metrics collection
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_metrics(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_metrics(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Set metrics export format
     * @param {string} format
     * @returns {WasmCompilationOptions}
     */
    with_metrics_format(format) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_metrics_format(ptr, ptr0, len0);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Set compilation mode
     * @param {string} mode
     * @returns {WasmCompilationOptions}
     */
    with_mode(mode) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_mode(ptr, ptr0, len0);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable module namespace qualification
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_module_namespaces(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_module_namespaces(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Set optimization level (production)
     * @param {string} level
     * @returns {WasmCompilationOptions}
     */
    with_optimization_level(level) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(level, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_optimization_level(ptr, ptr0, len0);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable performance analysis
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_performance_analysis(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_performance_analysis(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable quiet mode
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_quiet(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_quiet(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Set source file name for better error reporting
     * @param {string} filename
     * @returns {WasmCompilationOptions}
     */
    with_source_file(filename) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(filename, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_source_file(ptr, ptr0, len0);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable compilation summary
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_summary(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_summary(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable v2-preview features
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_v2_preview(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_v2_preview(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Enable or disable verbose output
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_verbose(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_verbose(ptr, enabled);
        return WasmCompilationOptions.__wrap(ret);
    }
}
if (Symbol.dispose) WasmCompilationOptions.prototype[Symbol.dispose] = WasmCompilationOptions.prototype.free;
exports.WasmCompilationOptions = WasmCompilationOptions;

/**
 * WASM compilation result - unified with enhanced error support
 */
class WasmCompilationResult {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmCompilationResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmCompilationResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmCompilationResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmcompilationresult_free(ptr, 0);
    }
    /**
     * Size of generated bytecode
     * @returns {number}
     */
    get bytecode_size() {
        const ret = wasm.__wbg_get_wasmcompilationresult_bytecode_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Compilation time in milliseconds
     * @returns {number}
     */
    get compilation_time() {
        const ret = wasm.__wbg_get_wasmanalysisresult_analysis_time(this.__wbg_ptr);
        return ret;
    }
    /**
     * Total error count
     * @returns {number}
     */
    get error_count() {
        const ret = wasm.__wbg_get_wasmcompilationresult_error_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Whether compilation succeeded
     * @returns {boolean}
     */
    get success() {
        const ret = wasm.__wbg_get_wasmcompilationresult_success(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Total warning count
     * @returns {number}
     */
    get warning_count() {
        const ret = wasm.__wbg_get_wasmcompilationresult_warning_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Size of generated bytecode
     * @param {number} arg0
     */
    set bytecode_size(arg0) {
        wasm.__wbg_set_wasmcompilationresult_bytecode_size(this.__wbg_ptr, arg0);
    }
    /**
     * Compilation time in milliseconds
     * @param {number} arg0
     */
    set compilation_time(arg0) {
        wasm.__wbg_set_wasmanalysisresult_analysis_time(this.__wbg_ptr, arg0);
    }
    /**
     * Total error count
     * @param {number} arg0
     */
    set error_count(arg0) {
        wasm.__wbg_set_wasmcompilationresult_error_count(this.__wbg_ptr, arg0);
    }
    /**
     * Whether compilation succeeded
     * @param {boolean} arg0
     */
    set success(arg0) {
        wasm.__wbg_set_wasmcompilationresult_success(this.__wbg_ptr, arg0);
    }
    /**
     * Total warning count
     * @param {number} arg0
     */
    set warning_count(arg0) {
        wasm.__wbg_set_wasmcompilationresult_warning_count(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {any}
     */
    get abi() {
        const ret = wasm.wasmcompilationresult_abi(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {Uint8Array | undefined}
     */
    get bytecode() {
        const ret = wasm.wasmcompilationresult_bytecode(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {WasmCompilerError[]}
     */
    get compiler_errors() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationresult_compiler_errors(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {Array<any>}
     */
    get disassembly() {
        const ret = wasm.wasmcompilationresult_disassembly(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {Array<any>}
     */
    get errors() {
        const ret = wasm.wasmcompilationresult_errors(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get all errors as JSON array
     * @returns {string}
     */
    format_all_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationresult_format_all_json(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get all errors formatted as terminal output
     * @returns {string}
     */
    format_all_terminal() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationresult_format_all_terminal(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get_formatted_errors_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationresult_get_formatted_errors_json(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get_formatted_errors_terminal() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationresult_get_formatted_errors_terminal(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get fully detailed metrics regardless of export format
     * @returns {any}
     */
    get_metrics_detailed() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationresult_get_metrics_detailed(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get parsed metrics as JavaScript object
     * @returns {any}
     */
    get_metrics_object() {
        const ret = wasm.wasmcompilationresult_get_metrics_object(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {string}
     */
    get metrics() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationresult_metrics(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get metrics_format() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationresult_metrics_format(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Array<any>}
     */
    get warnings() {
        const ret = wasm.wasmcompilationresult_warnings(this.__wbg_ptr);
        return takeObject(ret);
    }
}
if (Symbol.dispose) WasmCompilationResult.prototype[Symbol.dispose] = WasmCompilationResult.prototype.free;
exports.WasmCompilationResult = WasmCompilationResult;

/**
 * WASM compilation result with comprehensive metrics
 */
class WasmCompilationWithMetrics {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmCompilationWithMetricsFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmcompilationwithmetrics_free(ptr, 0);
    }
    /**
     * Size of generated bytecode
     * @returns {number}
     */
    get bytecode_size() {
        const ret = wasm.__wbg_get_wasmcompilationwithmetrics_bytecode_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Compilation time in milliseconds
     * @returns {number}
     */
    get compilation_time() {
        const ret = wasm.__wbg_get_wasmanalysisresult_analysis_time(this.__wbg_ptr);
        return ret;
    }
    /**
     * Whether compilation succeeded
     * @returns {boolean}
     */
    get success() {
        const ret = wasm.__wbg_get_wasmcompilationwithmetrics_success(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Size of generated bytecode
     * @param {number} arg0
     */
    set bytecode_size(arg0) {
        wasm.__wbg_set_wasmcompilationwithmetrics_bytecode_size(this.__wbg_ptr, arg0);
    }
    /**
     * Compilation time in milliseconds
     * @param {number} arg0
     */
    set compilation_time(arg0) {
        wasm.__wbg_set_wasmanalysisresult_analysis_time(this.__wbg_ptr, arg0);
    }
    /**
     * Whether compilation succeeded
     * @param {boolean} arg0
     */
    set success(arg0) {
        wasm.__wbg_set_wasmcompilationwithmetrics_success(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {Uint8Array | undefined}
     */
    get bytecode() {
        const ret = wasm.wasmcompilationwithmetrics_bytecode(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {Array<any>}
     */
    get errors() {
        const ret = wasm.wasmcompilationwithmetrics_errors(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get parsed metrics as JavaScript object
     * @returns {any}
     */
    get_metrics_object() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationwithmetrics_get_metrics_object(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {string}
     */
    get metrics() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilationwithmetrics_metrics(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Array<any>}
     */
    get warnings() {
        const ret = wasm.wasmcompilationwithmetrics_warnings(this.__wbg_ptr);
        return takeObject(ret);
    }
}
if (Symbol.dispose) WasmCompilationWithMetrics.prototype[Symbol.dispose] = WasmCompilationWithMetrics.prototype.free;
exports.WasmCompilationWithMetrics = WasmCompilationWithMetrics;

/**
 * Enhanced compiler error for WASM
 */
class WasmCompilerError {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmCompilerError.prototype);
        obj.__wbg_ptr = ptr;
        WasmCompilerErrorFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmCompilerErrorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmcompilererror_free(ptr, 0);
    }
    /**
     * @returns {string}
     */
    get category() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilererror_category(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get code() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilererror_code(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    get column() {
        const ret = wasm.wasmcompilererror_column(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {string | undefined}
     */
    get description() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilererror_description(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get error as JSON string
     * @returns {string}
     */
    format_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilererror_format_json(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get formatted error message (terminal style)
     * Get formatted error message (terminal style)
     * @returns {string}
     */
    format_terminal() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilererror_format_terminal(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    get line() {
        const ret = wasm.wasmcompilererror_line(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * @returns {WasmSourceLocation | undefined}
     */
    get location() {
        const ret = wasm.wasmcompilererror_location(this.__wbg_ptr);
        return ret === 0 ? undefined : WasmSourceLocation.__wrap(ret);
    }
    /**
     * @returns {string}
     */
    get message() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilererror_message(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get severity() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilererror_severity(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string | undefined}
     */
    get source_line() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilererror_source_line(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {WasmSuggestion[]}
     */
    get suggestions() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmcompilererror_suggestions(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) WasmCompilerError.prototype[Symbol.dispose] = WasmCompilerError.prototype.free;
exports.WasmCompilerError = WasmCompilerError;

/**
 * Enhanced compilation result with rich error information
 */
class WasmEnhancedCompilationResult {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmEnhancedCompilationResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmenhancedcompilationresult_free(ptr, 0);
    }
    /**
     * Size of generated bytecode
     * @returns {number}
     */
    get bytecode_size() {
        const ret = wasm.__wbg_get_wasmenhancedcompilationresult_bytecode_size(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Compilation time in milliseconds
     * @returns {number}
     */
    get compilation_time() {
        const ret = wasm.__wbg_get_wasmanalysisresult_analysis_time(this.__wbg_ptr);
        return ret;
    }
    /**
     * Total error count
     * @returns {number}
     */
    get error_count() {
        const ret = wasm.__wbg_get_wasmenhancedcompilationresult_error_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Whether compilation succeeded
     * @returns {boolean}
     */
    get success() {
        const ret = wasm.__wbg_get_wasmanalysisresult_success(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Total warning count
     * @returns {number}
     */
    get warning_count() {
        const ret = wasm.__wbg_get_wasmenhancedcompilationresult_warning_count(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Size of generated bytecode
     * @param {number} arg0
     */
    set bytecode_size(arg0) {
        wasm.__wbg_set_wasmenhancedcompilationresult_bytecode_size(this.__wbg_ptr, arg0);
    }
    /**
     * Compilation time in milliseconds
     * @param {number} arg0
     */
    set compilation_time(arg0) {
        wasm.__wbg_set_wasmanalysisresult_analysis_time(this.__wbg_ptr, arg0);
    }
    /**
     * Total error count
     * @param {number} arg0
     */
    set error_count(arg0) {
        wasm.__wbg_set_wasmenhancedcompilationresult_error_count(this.__wbg_ptr, arg0);
    }
    /**
     * Whether compilation succeeded
     * @param {boolean} arg0
     */
    set success(arg0) {
        wasm.__wbg_set_wasmanalysisresult_success(this.__wbg_ptr, arg0);
    }
    /**
     * Total warning count
     * @param {number} arg0
     */
    set warning_count(arg0) {
        wasm.__wbg_set_wasmenhancedcompilationresult_warning_count(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {WasmCompilerError[]}
     */
    get compiler_errors() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmenhancedcompilationresult_compiler_errors(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var v1 = getArrayJsValueFromWasm0(r0, r1).slice();
            wasm.__wbindgen_export4(r0, r1 * 4, 4);
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get all errors as JSON array
     * @returns {string}
     */
    format_all_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmenhancedcompilationresult_format_all_json(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get all errors formatted as terminal output
     * @returns {string}
     */
    format_all_terminal() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmenhancedcompilationresult_format_all_terminal(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmEnhancedCompilationResult.prototype[Symbol.dispose] = WasmEnhancedCompilationResult.prototype.free;
exports.WasmEnhancedCompilationResult = WasmEnhancedCompilationResult;

/**
 * WASM DSL Compiler for client-side compilation
 */
class WasmFiveCompiler {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmFiveCompilerFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmfivecompiler_free(ptr, 0);
    }
    /**
     * Get detailed analysis of source code
     * @param {string} source
     * @returns {WasmAnalysisResult}
     */
    analyze_source(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_analyze_source(this.__wbg_ptr, ptr0, len0);
        return WasmAnalysisResult.__wrap(ret);
    }
    /**
     * Get detailed analysis of source code with compilation mode selection
     * @param {string} source
     * @param {string} mode
     * @returns {WasmAnalysisResult}
     */
    analyze_source_mode(source, mode) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(mode, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_analyze_source_mode(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return WasmAnalysisResult.__wrap(ret);
    }
    /**
     * Unified compilation method with enhanced error reporting and metrics
     * @param {string} source
     * @param {WasmCompilationOptions} options
     * @returns {WasmCompilationResult}
     */
    compile(source, options) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(options, WasmCompilationOptions);
        const ret = wasm.wasmfivecompiler_compile(this.__wbg_ptr, ptr0, len0, options.__wbg_ptr);
        return WasmCompilationResult.__wrap(ret);
    }
    /**
     * Compile multi-file project with explicit module list
     * @param {any} module_files
     * @param {string} entry_point
     * @param {WasmCompilationOptions} options
     * @returns {WasmCompilationResult}
     */
    compileModules(module_files, entry_point, options) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(entry_point, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            _assertClass(options, WasmCompilationOptions);
            wasm.wasmfivecompiler_compileModules(retptr, this.__wbg_ptr, addHeapObject(module_files), ptr0, len0, options.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return WasmCompilationResult.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Compile multi-file project with automatic discovery
     * @param {string} entry_point
     * @param {WasmCompilationOptions} options
     * @returns {WasmCompilationResult}
     */
    compileMultiWithDiscovery(entry_point, options) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(entry_point, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            _assertClass(options, WasmCompilationOptions);
            wasm.wasmfivecompiler_compileMultiWithDiscovery(retptr, this.__wbg_ptr, ptr0, len0, options.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return WasmCompilationResult.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Multi-file compilation using module merger (main source + modules)
     * @param {string} main_source
     * @param {any} modules
     * @param {WasmCompilationOptions} options
     * @returns {WasmCompilationResult}
     */
    compile_multi(main_source, modules, options) {
        try {
            const ptr0 = passStringToWasm0(main_source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            _assertClass(options, WasmCompilationOptions);
            const ret = wasm.wasmfivecompiler_compile_multi(this.__wbg_ptr, ptr0, len0, addBorrowedObject(modules), options.__wbg_ptr);
            return WasmCompilationResult.__wrap(ret);
        } finally {
            heap[stack_pointer++] = undefined;
        }
    }
    /**
     * Compile DSL and generate both bytecode and ABI
     * @param {string} source
     * @returns {any}
     */
    compile_with_abi(source) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_compile_with_abi(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Discover modules starting from an entry point
     * @param {string} entry_point
     * @returns {any}
     */
    discoverModules(entry_point) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(entry_point, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_discoverModules(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Extract function name metadata from compiled bytecode
     * Returns a list of discovered functions in the bytecode
     * @param {Uint8Array} bytecode
     * @returns {any}
     */
    extractFunctionMetadata(bytecode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_extractFunctionMetadata(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Extract account definitions from DSL source code
     * @param {string} source
     * @returns {any}
     */
    extract_account_definitions(source) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_extract_account_definitions(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Extract function signatures with account parameters
     * @param {string} source
     * @returns {any}
     */
    extract_function_signatures(source) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_extract_function_signatures(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Format an error message using the native terminal formatter
     * This provides rich Rust-style error output with source context and colors
     * @param {string} message
     * @param {string} code
     * @param {string} severity
     * @param {number} line
     * @param {number} column
     * @param {string} _source
     * @returns {string}
     */
    format_error_terminal(message, code, severity, line, column, _source) {
        let deferred5_0;
        let deferred5_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(message, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(code, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(severity, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len2 = WASM_VECTOR_LEN;
            const ptr3 = passStringToWasm0(_source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len3 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_format_error_terminal(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2, line, column, ptr3, len3);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred5_0 = r0;
            deferred5_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred5_0, deferred5_1, 1);
        }
    }
    /**
     * Generate ABI from DSL source code for function calls
     * @param {string} source
     * @returns {any}
     */
    generate_abi(source) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_generate_abi(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get compiler statistics
     * @returns {any}
     */
    get_compiler_stats() {
        const ret = wasm.wasmfivecompiler_get_compiler_stats(this.__wbg_ptr);
        return takeObject(ret);
    }
    /**
     * Get comprehensive compiler statistics including which opcodes are used vs unused
     * @param {string} source
     * @returns {any}
     */
    get_opcode_analysis(source) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_get_opcode_analysis(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Get opcode usage statistics from compilation
     * @param {string} source
     * @returns {any}
     */
    get_opcode_usage(source) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_get_opcode_usage(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Create a new WASM compiler instance
     */
    constructor() {
        const ret = wasm.wasmfivecompiler_new();
        this.__wbg_ptr = ret >>> 0;
        WasmFiveCompilerFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Optimize bytecode
     * @param {Uint8Array} bytecode
     * @returns {Uint8Array}
     */
    optimize_bytecode(bytecode) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_optimize_bytecode(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Parse DSL source code and return AST information
     * @param {string} source
     * @returns {any}
     */
    parse_dsl(source) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_parse_dsl(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Type-check parsed AST
     * @param {string} _ast_json
     * @returns {any}
     */
    type_check(_ast_json) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(_ast_json, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_type_check(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Validate account constraints against function parameters
     * @param {string} source
     * @param {string} function_name
     * @param {string} accounts_json
     * @returns {any}
     */
    validate_account_constraints(source, function_name, accounts_json) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(function_name, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(accounts_json, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len2 = WASM_VECTOR_LEN;
            wasm.wasmfivecompiler_validate_account_constraints(retptr, this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * Validate DSL syntax without full compilation
     * @param {string} source
     * @returns {any}
     */
    validate_syntax(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_validate_syntax(this.__wbg_ptr, ptr0, len0);
        return takeObject(ret);
    }
}
if (Symbol.dispose) WasmFiveCompiler.prototype[Symbol.dispose] = WasmFiveCompiler.prototype.free;
exports.WasmFiveCompiler = WasmFiveCompiler;

/**
 * WASM-exposed metrics collector wrapper
 */
class WasmMetricsCollector {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmMetricsCollectorFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmmetricscollector_free(ptr, 0);
    }
    /**
     * End the current compilation phase
     */
    end_phase() {
        wasm.wasmmetricscollector_end_phase(this.__wbg_ptr);
    }
    /**
     * Export metrics in the requested format
     * @param {string} format
     * @returns {string}
     */
    export(format) {
        let deferred3_0;
        let deferred3_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passStringToWasm0(format, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len0 = WASM_VECTOR_LEN;
            wasm.wasmmetricscollector_export(retptr, this.__wbg_ptr, ptr0, len0);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            var r3 = getDataViewMemory0().getInt32(retptr + 4 * 3, true);
            var ptr2 = r0;
            var len2 = r1;
            if (r3) {
                ptr2 = 0; len2 = 0;
                throw takeObject(r2);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Finalize metrics collection
     */
    finalize() {
        wasm.wasmmetricscollector_finalize(this.__wbg_ptr);
    }
    /**
     * Get metrics as a JS object for programmatic use
     * @returns {any}
     */
    get_metrics_object() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmmetricscollector_get_metrics_object(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
            if (r2) {
                throw takeObject(r1);
            }
            return takeObject(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    constructor() {
        const ret = wasm.wasmmetricscollector_new();
        this.__wbg_ptr = ret >>> 0;
        WasmMetricsCollectorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Reset the collector for a new compilation
     */
    reset() {
        wasm.wasmmetricscollector_reset(this.__wbg_ptr);
    }
    /**
     * Start timing a compilation phase
     * @param {string} phase_name
     */
    start_phase(phase_name) {
        const ptr0 = passStringToWasm0(phase_name, wasm.__wbindgen_export, wasm.__wbindgen_export2);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmmetricscollector_start_phase(this.__wbg_ptr, ptr0, len0);
    }
}
if (Symbol.dispose) WasmMetricsCollector.prototype[Symbol.dispose] = WasmMetricsCollector.prototype.free;
exports.WasmMetricsCollector = WasmMetricsCollector;

/**
 * Enhanced source location for WASM
 */
class WasmSourceLocation {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmSourceLocation.prototype);
        obj.__wbg_ptr = ptr;
        WasmSourceLocationFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmSourceLocationFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmsourcelocation_free(ptr, 0);
    }
    /**
     * Column number (1-based)
     * @returns {number}
     */
    get column() {
        const ret = wasm.__wbg_get_wasmsourcelocation_column(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Length of the relevant text
     * @returns {number}
     */
    get length() {
        const ret = wasm.__wbg_get_wasmsourcelocation_length(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Line number (1-based)
     * @returns {number}
     */
    get line() {
        const ret = wasm.__wbg_get_wasmsourcelocation_line(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Byte offset in source
     * @returns {number}
     */
    get offset() {
        const ret = wasm.__wbg_get_wasmsourcelocation_offset(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Column number (1-based)
     * @param {number} arg0
     */
    set column(arg0) {
        wasm.__wbg_set_wasmsourcelocation_column(this.__wbg_ptr, arg0);
    }
    /**
     * Length of the relevant text
     * @param {number} arg0
     */
    set length(arg0) {
        wasm.__wbg_set_wasmsourcelocation_length(this.__wbg_ptr, arg0);
    }
    /**
     * Line number (1-based)
     * @param {number} arg0
     */
    set line(arg0) {
        wasm.__wbg_set_wasmsourcelocation_line(this.__wbg_ptr, arg0);
    }
    /**
     * Byte offset in source
     * @param {number} arg0
     */
    set offset(arg0) {
        wasm.__wbg_set_wasmsourcelocation_offset(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {string | undefined}
     */
    get file() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsourcelocation_file(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
if (Symbol.dispose) WasmSourceLocation.prototype[Symbol.dispose] = WasmSourceLocation.prototype.free;
exports.WasmSourceLocation = WasmSourceLocation;

/**
 * Enhanced error suggestion for WASM
 */
class WasmSuggestion {
    static __wrap(ptr) {
        ptr = ptr >>> 0;
        const obj = Object.create(WasmSuggestion.prototype);
        obj.__wbg_ptr = ptr;
        WasmSuggestionFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmSuggestionFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmsuggestion_free(ptr, 0);
    }
    /**
     * Confidence score (0.0 to 1.0)
     * @returns {number}
     */
    get confidence() {
        const ret = wasm.__wbg_get_wasmanalysisresult_analysis_time(this.__wbg_ptr);
        return ret;
    }
    /**
     * Confidence score (0.0 to 1.0)
     * @param {number} arg0
     */
    set confidence(arg0) {
        wasm.__wbg_set_wasmanalysisresult_analysis_time(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {string | undefined}
     */
    get code_suggestion() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsuggestion_code_suggestion(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {string | undefined}
     */
    get explanation() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsuggestion_explanation(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            let v1;
            if (r0 !== 0) {
                v1 = getStringFromWasm0(r0, r1).slice();
                wasm.__wbindgen_export4(r0, r1 * 1, 1);
            }
            return v1;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
     * @returns {string}
     */
    get message() {
        let deferred1_0;
        let deferred1_1;
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.wasmsuggestion_message(retptr, this.__wbg_ptr);
            var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
            var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
            deferred1_0 = r0;
            deferred1_1 = r1;
            return getStringFromWasm0(r0, r1);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
            wasm.__wbindgen_export4(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmSuggestion.prototype[Symbol.dispose] = WasmSuggestion.prototype.free;
exports.WasmSuggestion = WasmSuggestion;

/**
 * Get function names from bytecode as a JS value (array of objects)
 *
 * This function avoids constructing `FunctionNameInfo` JS instances and instead
 * marshals the parsed metadata directly into a serde-friendly structure and
 * returns a `JsValue` via `JsValue::from_serde`.
 * @param {Uint8Array} bytecode
 * @returns {any}
 */
function get_function_names(bytecode) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.get_function_names(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.get_function_names = get_function_names;

/**
 * Get the count of public functions from bytecode header
 * @param {Uint8Array} bytecode
 * @returns {number}
 */
function get_public_function_count(bytecode) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.get_public_function_count(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return r0;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.get_public_function_count = get_public_function_count;

/**
 * Get information about the WASM compiler capabilities
 * @returns {any}
 */
function get_wasm_compiler_info() {
    const ret = wasm.get_wasm_compiler_info();
    return takeObject(ret);
}
exports.get_wasm_compiler_info = get_wasm_compiler_info;

/**
 * Helper function to convert JS value to VM Value
 * @param {any} js_val
 * @param {number} value_type
 * @returns {any}
 */
function js_value_to_vm_value(js_val, value_type) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        wasm.js_value_to_vm_value(retptr, addBorrowedObject(js_val), value_type);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
        heap[stack_pointer++] = undefined;
    }
}
exports.js_value_to_vm_value = js_value_to_vm_value;

/**
 * @param {string} message
 */
function log_to_console(message) {
    const ptr0 = passStringToWasm0(message, wasm.__wbindgen_export, wasm.__wbindgen_export2);
    const len0 = WASM_VECTOR_LEN;
    wasm.log_to_console(ptr0, len0);
}
exports.log_to_console = log_to_console;

/**
 * Parse function names from bytecode metadata
 *
 * Returns a JS value which is a JSON string encoding an array of objects:
 * [ { "name": "...", "function_index": N }, ... ]
 * We serialize via serde_json and return the JSON string as a `JsValue` to
 * avoid complex JS object construction in Rust/WASM glue.
 * @param {Uint8Array} bytecode
 * @returns {any}
 */
function parse_function_names(bytecode) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.parse_function_names(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.parse_function_names = parse_function_names;

/**
 * Utility: Validate optimized headers and mirror bytecode back to JS callers
 * @param {Uint8Array} bytecode
 * @returns {Uint8Array}
 */
function wrap_with_script_header(bytecode) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_export);
        const len0 = WASM_VECTOR_LEN;
        wasm.wrap_with_script_header(retptr, ptr0, len0);
        var r0 = getDataViewMemory0().getInt32(retptr + 4 * 0, true);
        var r1 = getDataViewMemory0().getInt32(retptr + 4 * 1, true);
        var r2 = getDataViewMemory0().getInt32(retptr + 4 * 2, true);
        if (r2) {
            throw takeObject(r1);
        }
        return takeObject(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
}
exports.wrap_with_script_header = wrap_with_script_header;

function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg_Error_8c4e43fe74559d73: function(arg0, arg1) {
            const ret = Error(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_String_fed4d24b68977888: function(arg0, arg1) {
            const ret = String(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_bigint_get_as_i64_8fcf4ce7f1ca72a2: function(arg0, arg1) {
            const v = getObject(arg1);
            const ret = typeof(v) === 'bigint' ? v : undefined;
            getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_boolean_get_bbbb1c18aa2f5e25: function(arg0) {
            const v = getObject(arg0);
            const ret = typeof(v) === 'boolean' ? v : undefined;
            return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
        },
        __wbg___wbindgen_debug_string_0bc8482c6e3508ae: function(arg0, arg1) {
            const ret = debugString(getObject(arg1));
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_in_47fa6863be6f2f25: function(arg0, arg1) {
            const ret = getObject(arg0) in getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_is_bigint_31b12575b56f32fc: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'bigint';
            return ret;
        },
        __wbg___wbindgen_is_function_0095a73b8b156f76: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_object_5ae8e5880f2c1fbd: function(arg0) {
            const val = getObject(arg0);
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_string_cd444516edc5b180: function(arg0) {
            const ret = typeof(getObject(arg0)) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_9e4d92534c42d778: function(arg0) {
            const ret = getObject(arg0) === undefined;
            return ret;
        },
        __wbg___wbindgen_jsval_eq_11888390b0186270: function(arg0, arg1) {
            const ret = getObject(arg0) === getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_jsval_loose_eq_9dd77d8cd6671811: function(arg0, arg1) {
            const ret = getObject(arg0) == getObject(arg1);
            return ret;
        },
        __wbg___wbindgen_number_get_8ff4255516ccad3e: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'number' ? obj : undefined;
            getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
        },
        __wbg___wbindgen_string_get_72fb696202c56729: function(arg0, arg1) {
            const obj = getObject(arg1);
            const ret = typeof(obj) === 'string' ? obj : undefined;
            var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            var len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg___wbindgen_throw_be289d5034ed271b: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_call_389efe28435a9388: function() { return handleError(function (arg0, arg1) {
            const ret = getObject(arg0).call(getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_done_57b39ecd9addfe81: function(arg0) {
            const ret = getObject(arg0).done;
            return ret;
        },
        __wbg_error_7534b8e9a36f1ab4: function(arg0, arg1) {
            let deferred0_0;
            let deferred0_1;
            try {
                deferred0_0 = arg0;
                deferred0_1 = arg1;
                console.error(getStringFromWasm0(arg0, arg1));
            } finally {
                wasm.__wbindgen_export4(deferred0_0, deferred0_1, 1);
            }
        },
        __wbg_error_9a7fe3f932034cde: function(arg0) {
            console.error(getObject(arg0));
        },
        __wbg_getRandomValues_9c5c1b115e142bb8: function() { return handleError(function (arg0, arg1) {
            globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
        }, arguments); },
        __wbg_get_9b94d73e6221f75c: function(arg0, arg1) {
            const ret = getObject(arg0)[arg1 >>> 0];
            return addHeapObject(ret);
        },
        __wbg_get_b3ed3ad4be2bc8ac: function() { return handleError(function (arg0, arg1) {
            const ret = Reflect.get(getObject(arg0), getObject(arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_get_with_ref_key_bb8f74a92cb2e784: function(arg0, arg1) {
            const ret = getObject(arg0)[getObject(arg1)];
            return addHeapObject(ret);
        },
        __wbg_instanceof_ArrayBuffer_c367199e2fa2aa04: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof ArrayBuffer;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_instanceof_Uint8Array_9b9075935c74707c: function(arg0) {
            let result;
            try {
                result = getObject(arg0) instanceof Uint8Array;
            } catch (_) {
                result = false;
            }
            const ret = result;
            return ret;
        },
        __wbg_isArray_d314bb98fcf08331: function(arg0) {
            const ret = Array.isArray(getObject(arg0));
            return ret;
        },
        __wbg_isSafeInteger_bfbc7332a9768d2a: function(arg0) {
            const ret = Number.isSafeInteger(getObject(arg0));
            return ret;
        },
        __wbg_iterator_6ff6560ca1568e55: function() {
            const ret = Symbol.iterator;
            return addHeapObject(ret);
        },
        __wbg_length_32ed9a279acd054c: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_length_35a7bace40f36eac: function(arg0) {
            const ret = getObject(arg0).length;
            return ret;
        },
        __wbg_log_6b5ca2e6124b2808: function(arg0) {
            console.log(getObject(arg0));
        },
        __wbg_new_361308b2356cecd0: function() {
            const ret = new Object();
            return addHeapObject(ret);
        },
        __wbg_new_3eb36ae241fe6f44: function() {
            const ret = new Array();
            return addHeapObject(ret);
        },
        __wbg_new_8a6f238a6ece86ea: function() {
            const ret = new Error();
            return addHeapObject(ret);
        },
        __wbg_new_dca287b076112a51: function() {
            const ret = new Map();
            return addHeapObject(ret);
        },
        __wbg_new_dd2b680c8bf6ae29: function(arg0) {
            const ret = new Uint8Array(getObject(arg0));
            return addHeapObject(ret);
        },
        __wbg_new_from_slice_a3d2629dc1826784: function(arg0, arg1) {
            const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_new_no_args_1c7c842f08d00ebb: function(arg0, arg1) {
            const ret = new Function(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        },
        __wbg_next_3482f54c49e8af19: function() { return handleError(function (arg0) {
            const ret = getObject(arg0).next();
            return addHeapObject(ret);
        }, arguments); },
        __wbg_next_418f80d8f5303233: function(arg0) {
            const ret = getObject(arg0).next;
            return addHeapObject(ret);
        },
        __wbg_now_2c95c9de01293173: function(arg0) {
            const ret = getObject(arg0).now();
            return ret;
        },
        __wbg_now_a3af9a2f4bbaa4d1: function() {
            const ret = Date.now();
            return ret;
        },
        __wbg_parse_708461a1feddfb38: function() { return handleError(function (arg0, arg1) {
            const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
            return addHeapObject(ret);
        }, arguments); },
        __wbg_performance_7a3ffd0b17f663ad: function(arg0) {
            const ret = getObject(arg0).performance;
            return addHeapObject(ret);
        },
        __wbg_prototypesetcall_bdcdcc5842e4d77d: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), getObject(arg2));
        },
        __wbg_push_8ffdcb2063340ba5: function(arg0, arg1) {
            const ret = getObject(arg0).push(getObject(arg1));
            return ret;
        },
        __wbg_set_1eb0999cf5d27fc8: function(arg0, arg1, arg2) {
            const ret = getObject(arg0).set(getObject(arg1), getObject(arg2));
            return addHeapObject(ret);
        },
        __wbg_set_3fda3bac07393de4: function(arg0, arg1, arg2) {
            getObject(arg0)[takeObject(arg1)] = takeObject(arg2);
        },
        __wbg_set_6cb8631f80447a67: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = Reflect.set(getObject(arg0), getObject(arg1), getObject(arg2));
            return ret;
        }, arguments); },
        __wbg_set_f43e577aea94465b: function(arg0, arg1, arg2) {
            getObject(arg0)[arg1 >>> 0] = takeObject(arg2);
        },
        __wbg_stack_0ed75d68575b0f3c: function(arg0, arg1) {
            const ret = getObject(arg1).stack;
            const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_export, wasm.__wbindgen_export2);
            const len1 = WASM_VECTOR_LEN;
            getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
            getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
        },
        __wbg_static_accessor_GLOBAL_12837167ad935116: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_GLOBAL_THIS_e628e89ab3b1c95f: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_SELF_a621d3dfbb60d0ce: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_static_accessor_WINDOW_f8727f0cf888e0bd: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addHeapObject(ret);
        },
        __wbg_value_0546255b415e96c1: function(arg0) {
            const ret = getObject(arg0).value;
            return addHeapObject(ret);
        },
        __wbg_warn_f7ae1b2e66ccb930: function(arg0) {
            console.warn(getObject(arg0));
        },
        __wbg_wasmcompilererror_new: function(arg0) {
            const ret = WasmCompilerError.__wrap(arg0);
            return addHeapObject(ret);
        },
        __wbg_wasmsuggestion_new: function(arg0) {
            const ret = WasmSuggestion.__wrap(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000001: function(arg0) {
            // Cast intrinsic for `F64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000002: function(arg0) {
            // Cast intrinsic for `I64 -> Externref`.
            const ret = arg0;
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000003: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return addHeapObject(ret);
        },
        __wbindgen_cast_0000000000000004: function(arg0) {
            // Cast intrinsic for `U64 -> Externref`.
            const ret = BigInt.asUintN(64, arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_clone_ref: function(arg0) {
            const ret = getObject(arg0);
            return addHeapObject(ret);
        },
        __wbindgen_object_drop_ref: function(arg0) {
            takeObject(arg0);
        },
    };
    return {
        __proto__: null,
        "./five_vm_wasm_bg.js": import0,
    };
}

const BytecodeAnalyzerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_bytecodeanalyzer_free(ptr >>> 0, 1));
const BytecodeEncoderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_bytecodeencoder_free(ptr >>> 0, 1));
const FiveVMStateFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fivevmstate_free(ptr >>> 0, 1));
const FiveVMWasmFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fivevmwasm_free(ptr >>> 0, 1));
const ParameterEncoderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_parameterencoder_free(ptr >>> 0, 1));
const TestResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_testresult_free(ptr >>> 0, 1));
const WasmAccountFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmaccount_free(ptr >>> 0, 1));
const WasmAnalysisResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmanalysisresult_free(ptr >>> 0, 1));
const WasmCompilationOptionsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmcompilationoptions_free(ptr >>> 0, 1));
const WasmCompilationResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmcompilationresult_free(ptr >>> 0, 1));
const WasmCompilationWithMetricsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmcompilationwithmetrics_free(ptr >>> 0, 1));
const WasmCompilerErrorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmcompilererror_free(ptr >>> 0, 1));
const WasmEnhancedCompilationResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmenhancedcompilationresult_free(ptr >>> 0, 1));
const WasmFiveCompilerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmfivecompiler_free(ptr >>> 0, 1));
const WasmMetricsCollectorFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmmetricscollector_free(ptr >>> 0, 1));
const WasmSourceLocationFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmsourcelocation_free(ptr >>> 0, 1));
const WasmSuggestionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmsuggestion_free(ptr >>> 0, 1));

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function addBorrowedObject(obj) {
    if (stack_pointer == 1) throw new Error('out of js stack');
    heap[--stack_pointer] = obj;
    return stack_pointer;
}

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

function dropObject(idx) {
    if (idx < 132) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(takeObject(mem.getUint32(i, true)));
    }
    return result;
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getObject(idx) { return heap[idx]; }

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        wasm.__wbindgen_export3(addHeapObject(e));
    }
}

let heap = new Array(128).fill(undefined);
heap.push(undefined, null, true, false);

let heap_next = heap.length;

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

let stack_pointer = 128;

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
function decodeText(ptr, len) {
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

const wasmPath = `${__dirname}/five_vm_wasm_bg.wasm`;
const wasmBytes = require('fs').readFileSync(wasmPath);
const wasmModule = new WebAssembly.Module(wasmBytes);
const wasm = new WebAssembly.Instance(wasmModule, __wbg_get_imports()).exports;
