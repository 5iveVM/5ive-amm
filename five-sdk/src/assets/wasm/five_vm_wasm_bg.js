let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
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

function getArrayJsValueFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    const mem = getDataViewMemory0();
    const result = [];
    for (let i = ptr; i < ptr + 4 * len; i += 4) {
        result.push(wasm.__wbindgen_externrefs.get(mem.getUint32(i, true)));
    }
    wasm.__externref_drop_slice(ptr, len);
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

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

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

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
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
    }
}

let WASM_VECTOR_LEN = 0;

function findWasmExport(predicate, description) {
    for (const [name, value] of Object.entries(wasm)) {
        if (typeof value === 'function' && predicate(name)) {
            return value;
        }
    }
    throw new Error(`WASM export not found: ${description}`);
}

const BytecodeAnalyzerFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_bytecodeanalyzer_free(ptr >>> 0, 1));

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

const VarintEncoderFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => findWasmExport((name) => name.endsWith('_encoder_free') && !name.includes('parameter'), 'encoder free')(ptr >>> 0, 1));

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

/**
 * Bytecode analyzer for WASM
 */
export class BytecodeAnalyzer {
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
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bytecodeanalyzer_analyze(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Advanced semantic analysis with full opcode understanding and instruction flow
     * This provides the intelligent analysis that understands what each opcode does
     * and what operands follow each instruction.
     * @param {Uint8Array} bytecode
     * @returns {any}
     */
    static analyze_semantic(bytecode) {
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bytecodeanalyzer_analyze_semantic(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get detailed information about a specific instruction at an offset
     * @param {Uint8Array} bytecode
     * @param {number} offset
     * @returns {any}
     */
    static analyze_instruction_at(bytecode, offset) {
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bytecodeanalyzer_analyze_instruction_at(ptr0, len0, offset);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get summary statistics about the bytecode
     * @param {Uint8Array} bytecode
     * @returns {any}
     */
    static get_bytecode_summary(bytecode) {
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bytecodeanalyzer_get_bytecode_summary(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get detailed opcode flow analysis - shows execution paths through the bytecode
     * @param {Uint8Array} bytecode
     * @returns {any}
     */
    static analyze_execution_flow(bytecode) {
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bytecodeanalyzer_analyze_execution_flow(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}
if (Symbol.dispose) BytecodeAnalyzer.prototype[Symbol.dispose] = BytecodeAnalyzer.prototype.free;

/**
 * Execution result that honestly reports what happened
 * @enum {0 | 1 | 2 | 3 | 4 | 5 | 6}
 */
export const ExecutionStatus = Object.freeze({
    /**
     * All operations completed successfully
     */
    Completed: 0, "0": "Completed",
    /**
     * Execution stopped because it hit a system program call that cannot be executed in WASM
     */
    StoppedAtSystemCall: 1, "1": "StoppedAtSystemCall",
    /**
     * Execution stopped because it hit an INIT_PDA operation that requires real Solana context
     */
    StoppedAtInitPDA: 2, "2": "StoppedAtInitPDA",
    /**
     * Execution stopped because it hit an INVOKE operation that requires real RPC
     */
    StoppedAtInvoke: 3, "3": "StoppedAtInvoke",
    /**
     * Execution stopped because it hit an INVOKE_SIGNED operation that requires real RPC
     */
    StoppedAtInvokeSigned: 4, "4": "StoppedAtInvokeSigned",
    /**
     * Execution stopped because compute limit was reached
     */
    ComputeLimitExceeded: 5, "5": "ComputeLimitExceeded",
    /**
     * Execution failed due to an error
     */
    Failed: 6, "6": "Failed",
});

/**
 * JavaScript-compatible VM state representation
 */
export class FiveVMState {
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
     * @returns {Array<any>}
     */
    get stack() {
        const ret = wasm.fivevmstate_stack(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get instruction_pointer() {
        const ret = wasm.fivevmstate_instruction_pointer(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {bigint}
     */
    get compute_units() {
        const ret = wasm.fivevmstate_compute_units(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
}
if (Symbol.dispose) FiveVMState.prototype[Symbol.dispose] = FiveVMState.prototype.free;

/**
 * Main WASM VM wrapper
 */
export class FiveVMWasm {
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
     * Create new VM instance with bytecode
     * @param {Uint8Array} _bytecode
     */
    constructor(_bytecode) {
        const ptr0 = passArray8ToWasm0(_bytecode, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.fivevmwasm_new(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        FiveVMWasmFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Execute VM with input data and accounts (legacy method)
     * @param {Uint8Array} input_data
     * @param {Array<any>} accounts
     * @returns {any}
     */
    execute(input_data, accounts) {
        const ptr0 = passArray8ToWasm0(input_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.fivevmwasm_execute(this.__wbg_ptr, ptr0, len0, accounts);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Execute VM with partial execution support - stops at system calls
     * @param {Uint8Array} input_data
     * @param {Array<any>} accounts
     * @returns {TestResult}
     */
    execute_partial(input_data, accounts) {
        const ptr0 = passArray8ToWasm0(input_data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.fivevmwasm_execute_partial(this.__wbg_ptr, ptr0, len0, accounts);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return TestResult.__wrap(ret[0]);
    }
    /**
     * Get current VM state
     * @returns {any}
     */
    get_state() {
        const ret = wasm.fivevmwasm_get_state(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Validate bytecode without execution
     * @param {Uint8Array} bytecode
     * @returns {boolean}
     */
    static validate_bytecode(bytecode) {
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.fivevmwasm_validate_bytecode(ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0] !== 0;
    }
    /**
     * Get VM constants for JavaScript
     * @returns {any}
     */
    static get_constants() {
        const ret = wasm.fivevmwasm_get_constants();
        return ret;
    }
}
if (Symbol.dispose) FiveVMWasm.prototype[Symbol.dispose] = FiveVMWasm.prototype.free;

/**
 * Parameter encoding utilities using varint and protocol types
 */
export class ParameterEncoder {
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
     * Encode function parameters using varint compression
     * Returns ONLY parameter data - SDK handles discriminator AND function index
     * Each parameter value is varint-encoded regardless of its declared type for maximum compression
     * @param {number} _function_index
     * @param {Array<any>} params
     * @returns {Uint8Array}
     */
    static encode_execute(_function_index, params) {
        const ret = findWasmExport((name) => name.startsWith('parameterencoder_encode_execute_'), 'parameter encode execute')(_function_index, params);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}
if (Symbol.dispose) ParameterEncoder.prototype[Symbol.dispose] = ParameterEncoder.prototype.free;

/**
 * Detailed execution result that provides full context
 */
export class TestResult {
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
     * Compute units consumed
     * @returns {bigint}
     */
    get compute_units_used() {
        const ret = wasm.__wbg_get_testresult_compute_units_used(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * Compute units consumed
     * @param {bigint} arg0
     */
    set compute_units_used(arg0) {
        wasm.__wbg_set_testresult_compute_units_used(this.__wbg_ptr, arg0);
    }
    /**
     * Final instruction pointer
     * @returns {number}
     */
    get instruction_pointer() {
        const ret = wasm.__wbg_get_testresult_instruction_pointer(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Final instruction pointer
     * @param {number} arg0
     */
    set instruction_pointer(arg0) {
        wasm.__wbg_set_testresult_instruction_pointer(this.__wbg_ptr, arg0);
    }
    /**
     * Which opcode caused the stop (if stopped at system call)
     * @returns {number | undefined}
     */
    get stopped_at_opcode() {
        const ret = wasm.__wbg_get_testresult_stopped_at_opcode(this.__wbg_ptr);
        return ret === 0xFFFFFF ? undefined : ret;
    }
    /**
     * Which opcode caused the stop (if stopped at system call)
     * @param {number | null} [arg0]
     */
    set stopped_at_opcode(arg0) {
        wasm.__wbg_set_testresult_stopped_at_opcode(this.__wbg_ptr, isLikeNone(arg0) ? 0xFFFFFF : arg0);
    }
    /**
     * @returns {string}
     */
    get status() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.testresult_status(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {boolean}
     */
    get has_result_value() {
        const ret = wasm.testresult_has_result_value(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {any}
     */
    get get_result_value() {
        const ret = wasm.testresult_get_result_value(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Array<any>}
     */
    get final_stack() {
        const ret = wasm.testresult_final_stack(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint8Array}
     */
    get final_memory() {
        const ret = wasm.testresult_final_memory(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Array<any>}
     */
    get final_accounts() {
        const ret = wasm.testresult_final_accounts(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string | undefined}
     */
    get error_message() {
        const ret = wasm.testresult_error_message(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {string | undefined}
     */
    get execution_context() {
        const ret = wasm.testresult_execution_context(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {string | undefined}
     */
    get stopped_at_opcode_name() {
        const ret = wasm.testresult_stopped_at_opcode_name(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
}
if (Symbol.dispose) TestResult.prototype[Symbol.dispose] = TestResult.prototype.free;

/**
 * Varint encoding utilities for JavaScript
 */
export class VarintEncoder {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        VarintEncoderFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        findWasmExport((name) => name.endsWith('_encoder_free') && !name.includes('parameter'), 'encoder free')(ptr, 0);
    }
    /**
     * Encode a u32 value using Variable-Length Encoding
     * Returns [size, byte1, byte2, byte3] where size is 1-3
     * @param {number} value
     * @returns {Array<any>}
     */
    static encode_u32(value) {
        const ret = findWasmExport((name) => name.endsWith('encoder_encode_u32'), 'encoder encode_u32')(value);
        return ret;
    }
    /**
     * Encode a u16 value using Variable-Length Encoding
     * Returns [size, byte1, byte2] where size is 1-2
     * @param {number} value
     * @returns {Array<any>}
     */
    static encode_u16(value) {
        const ret = findWasmExport((name) => name.endsWith('encoder_encode_u16'), 'encoder encode_u16')(value);
        return ret;
    }
    /**
     * Decode a u32 value from Variable-Length Encoding
     * Returns [value, bytes_consumed] or null if invalid
     * @param {Uint8Array} bytes
     * @returns {Array<any> | undefined}
     */
    static decode_u32(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = findWasmExport((name) => name.endsWith('encoder_decode_u32'), 'encoder decode_u32')(ptr0, len0);
        return ret;
    }
    /**
     * Decode a u16 value from Variable-Length Encoding
     * Returns [value, bytes_consumed] or null if invalid
     * @param {Uint8Array} bytes
     * @returns {Array<any> | undefined}
     */
    static decode_u16(bytes) {
        const ptr0 = passArray8ToWasm0(bytes, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = findWasmExport((name) => name.endsWith('encoder_decode_u16'), 'encoder decode_u16')(ptr0, len0);
        return ret;
    }
    /**
     * Calculate encoded size without encoding
     * @param {number} value
     * @returns {number}
     */
    static encoded_size_u32(value) {
        const ret = findWasmExport((name) => name.endsWith('encoder_encoded_size_u32'), 'encoder encoded_size_u32')(value);
        return ret >>> 0;
    }
    /**
     * Calculate encoded size for u16
     * @param {number} value
     * @returns {number}
     */
    static encoded_size_u16(value) {
        const ret = findWasmExport((name) => name.endsWith('encoder_encoded_size_u16'), 'encoder encoded_size_u16')(value);
        return ret >>> 0;
    }
}
if (Symbol.dispose) VarintEncoder.prototype[Symbol.dispose] = VarintEncoder.prototype.free;

/**
 * JavaScript-compatible account representation
 */
export class WasmAccount {
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
     * @returns {bigint}
     */
    get lamports() {
        const ret = wasm.__wbg_get_wasmaccount_lamports(this.__wbg_ptr);
        return BigInt.asUintN(64, ret);
    }
    /**
     * @param {bigint} arg0
     */
    set lamports(arg0) {
        wasm.__wbg_set_wasmaccount_lamports(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get is_writable() {
        const ret = wasm.__wbg_get_wasmaccount_is_writable(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set is_writable(arg0) {
        wasm.__wbg_set_wasmaccount_is_writable(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {boolean}
     */
    get is_signer() {
        const ret = wasm.__wbg_get_wasmaccount_is_signer(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @param {boolean} arg0
     */
    set is_signer(arg0) {
        wasm.__wbg_set_wasmaccount_is_signer(this.__wbg_ptr, arg0);
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
        const ptr0 = passArray8ToWasm0(key, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passArray8ToWasm0(owner, wasm.__wbindgen_malloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.wasmaccount_new(ptr0, len0, ptr1, len1, lamports, is_writable, is_signer, ptr2, len2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0] >>> 0;
        WasmAccountFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @returns {Uint8Array}
     */
    get key() {
        const ret = wasm.wasmaccount_key(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Uint8Array}
     */
    get data() {
        const ret = wasm.wasmaccount_data(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {Uint8Array} data
     */
    set data(data) {
        const ptr0 = passArray8ToWasm0(data, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmaccount_set_data(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * @returns {Uint8Array}
     */
    get owner() {
        const ret = wasm.wasmaccount_owner(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) WasmAccount.prototype[Symbol.dispose] = WasmAccount.prototype.free;

/**
 * WASM source analysis result
 */
export class WasmAnalysisResult {
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
     * Whether analysis succeeded
     * @returns {boolean}
     */
    get success() {
        const ret = wasm.__wbg_get_wasmanalysisresult_success(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Whether analysis succeeded
     * @param {boolean} arg0
     */
    set success(arg0) {
        wasm.__wbg_set_wasmanalysisresult_success(this.__wbg_ptr, arg0);
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
     * Analysis time in milliseconds
     * @param {number} arg0
     */
    set analysis_time(arg0) {
        wasm.__wbg_set_wasmanalysisresult_analysis_time(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {string}
     */
    get summary() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmanalysisresult_summary(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get metrics() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmanalysisresult_metrics(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {Array<any>}
     */
    get errors() {
        const ret = wasm.wasmanalysisresult_errors(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get parsed metrics as JavaScript object
     * @returns {any}
     */
    get_metrics_object() {
        const ret = wasm.wasmanalysisresult_get_metrics_object(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}
if (Symbol.dispose) WasmAnalysisResult.prototype[Symbol.dispose] = WasmAnalysisResult.prototype.free;

/**
 * Compilation options for enhanced error reporting and formatting
 */
export class WasmCompilationOptions {
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
     * Enable v2-preview features (nibble immediates, BR_EQ_U8, etc.)
     * @returns {boolean}
     */
    get v2_preview() {
        const ret = wasm.__wbg_get_wasmcompilationoptions_v2_preview(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Enable v2-preview features (nibble immediates, BR_EQ_U8, etc.)
     * @param {boolean} arg0
     */
    set v2_preview(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_v2_preview(this.__wbg_ptr, arg0);
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
     * Enable constraint caching optimization
     * @param {boolean} arg0
     */
    set enable_constraint_cache(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_enable_constraint_cache(this.__wbg_ptr, arg0);
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
     * Enable enhanced error reporting with suggestions
     * @param {boolean} arg0
     */
    set enhanced_errors(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_enhanced_errors(this.__wbg_ptr, arg0);
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
     * Include basic metrics
     * @param {boolean} arg0
     */
    set include_metrics(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_include_metrics(this.__wbg_ptr, arg0);
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
     * Include comprehensive metrics collection
     * @param {boolean} arg0
     */
    set comprehensive_metrics(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_comprehensive_metrics(this.__wbg_ptr, arg0);
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
     * Include performance analysis
     * @param {boolean} arg0
     */
    set performance_analysis(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_performance_analysis(this.__wbg_ptr, arg0);
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
     * Include complexity analysis
     * @param {boolean} arg0
     */
    set complexity_analysis(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_complexity_analysis(this.__wbg_ptr, arg0);
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
     * Show compilation summary
     * @param {boolean} arg0
     */
    set summary(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_summary(this.__wbg_ptr, arg0);
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
     * Verbose output
     * @param {boolean} arg0
     */
    set verbose(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_verbose(this.__wbg_ptr, arg0);
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
     * Suppress non-essential output
     * @param {boolean} arg0
     */
    set quiet(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_quiet(this.__wbg_ptr, arg0);
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
     * Include debug information
     * @param {boolean} arg0
     */
    set include_debug_info(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_include_debug_info(this.__wbg_ptr, arg0);
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
     * Enable bytecode compression
     * @param {boolean} arg0
     */
    set compress_output(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_compress_output(this.__wbg_ptr, arg0);
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
     * Enable module namespace qualification (module::function)
     * @param {boolean} arg0
     */
    set enable_module_namespaces(arg0) {
        wasm.__wbg_set_wasmcompilationoptions_enable_module_namespaces(this.__wbg_ptr, arg0);
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
     * Set compilation mode
     * @param {string} mode
     * @returns {WasmCompilationOptions}
     */
    with_mode(mode) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_mode(ptr, ptr0, len0);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Set optimization level (production)
     * @param {string} level
     * @returns {WasmCompilationOptions}
     */
    with_optimization_level(level) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(level, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_optimization_level(ptr, ptr0, len0);
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
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_error_format(ptr, ptr0, len0);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Set source file name for better error reporting
     * @param {string} filename
     * @returns {WasmCompilationOptions}
     */
    with_source_file(filename) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(filename, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_source_file(ptr, ptr0, len0);
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
     * Set metrics export format
     * @param {string} format
     * @returns {WasmCompilationOptions}
     */
    with_metrics_format(format) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_metrics_format(ptr, ptr0, len0);
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
     * Enable or disable verbose output
     * @param {boolean} enabled
     * @returns {WasmCompilationOptions}
     */
    with_verbose(enabled) {
        const ptr = this.__destroy_into_raw();
        const ret = wasm.wasmcompilationoptions_with_verbose(ptr, enabled);
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
     * Set analysis depth level
     * @param {string} depth
     * @returns {WasmCompilationOptions}
     */
    with_analysis_depth(depth) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(depth, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_analysis_depth(ptr, ptr0, len0);
        return WasmCompilationOptions.__wrap(ret);
    }
    /**
     * Set export format
     * @param {string} format
     * @returns {WasmCompilationOptions}
     */
    with_export_format(format) {
        const ptr = this.__destroy_into_raw();
        const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmcompilationoptions_with_export_format(ptr, ptr0, len0);
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
     * Create production-optimized configuration
     * @returns {WasmCompilationOptions}
     */
    static production_optimized() {
        const ret = wasm.wasmcompilationoptions_production_optimized();
        return WasmCompilationOptions.__wrap(ret);
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
    get mode() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationoptions_mode(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get optimization_level() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationoptions_optimization_level(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get error_format() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationoptions_error_format(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string | undefined}
     */
    get source_file() {
        const ret = wasm.wasmcompilationoptions_source_file(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {string}
     */
    get metrics_format() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationoptions_metrics_format(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get analysis_depth() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationoptions_analysis_depth(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get export_format() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationoptions_export_format(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmCompilationOptions.prototype[Symbol.dispose] = WasmCompilationOptions.prototype.free;

/**
 * WASM compilation result - unified with enhanced error support
 */
export class WasmCompilationResult {
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
     * Whether compilation succeeded
     * @returns {boolean}
     */
    get success() {
        const ret = wasm.__wbg_get_wasmcompilationresult_success(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Whether compilation succeeded
     * @param {boolean} arg0
     */
    set success(arg0) {
        wasm.__wbg_set_wasmcompilationresult_success(this.__wbg_ptr, arg0);
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
     * Size of generated bytecode
     * @param {number} arg0
     */
    set bytecode_size(arg0) {
        wasm.__wbg_set_wasmcompilationresult_bytecode_size(this.__wbg_ptr, arg0);
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
     * Compilation time in milliseconds
     * @param {number} arg0
     */
    set compilation_time(arg0) {
        wasm.__wbg_set_wasmanalysisresult_analysis_time(this.__wbg_ptr, arg0);
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
     * Total error count
     * @param {number} arg0
     */
    set error_count(arg0) {
        wasm.__wbg_set_wasmcompilationresult_error_count(this.__wbg_ptr, arg0);
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
     * Total warning count
     * @param {number} arg0
     */
    set warning_count(arg0) {
        wasm.__wbg_set_wasmcompilationresult_warning_count(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {Uint8Array | undefined}
     */
    get bytecode() {
        const ret = wasm.wasmcompilationresult_bytecode(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get abi() {
        const ret = wasm.wasmcompilationresult_abi(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Array<any>}
     */
    get warnings() {
        const ret = wasm.wasmcompilationresult_warnings(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Array<any>}
     */
    get errors() {
        const ret = wasm.wasmcompilationresult_errors(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {WasmCompilerError[]}
     */
    get compiler_errors() {
        const ret = wasm.wasmcompilationresult_compiler_errors(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {Array<any>}
     */
    get disassembly() {
        const ret = wasm.wasmcompilationresult_disassembly(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    get_formatted_errors_terminal() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationresult_get_formatted_errors_terminal(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get_formatted_errors_json() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationresult_get_formatted_errors_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
            const ret = wasm.wasmcompilationresult_format_all_terminal(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
            const ret = wasm.wasmcompilationresult_format_all_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get parsed metrics as JavaScript object
     * @returns {any}
     */
    get_metrics_object() {
        const ret = wasm.wasmcompilationresult_get_metrics_object(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get fully detailed metrics regardless of export format
     * @returns {any}
     */
    get_metrics_detailed() {
        const ret = wasm.wasmcompilationresult_get_metrics_detailed(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * @returns {string}
     */
    get metrics() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationresult_metrics(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get metrics_format() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationresult_metrics_format(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmCompilationResult.prototype[Symbol.dispose] = WasmCompilationResult.prototype.free;

/**
 * WASM compilation result with comprehensive metrics
 */
export class WasmCompilationWithMetrics {
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
     * Whether compilation succeeded
     * @returns {boolean}
     */
    get success() {
        const ret = wasm.__wbg_get_wasmcompilationwithmetrics_success(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Whether compilation succeeded
     * @param {boolean} arg0
     */
    set success(arg0) {
        wasm.__wbg_set_wasmcompilationwithmetrics_success(this.__wbg_ptr, arg0);
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
     * Size of generated bytecode
     * @param {number} arg0
     */
    set bytecode_size(arg0) {
        wasm.__wbg_set_wasmcompilationwithmetrics_bytecode_size(this.__wbg_ptr, arg0);
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
     * Compilation time in milliseconds
     * @param {number} arg0
     */
    set compilation_time(arg0) {
        wasm.__wbg_set_wasmanalysisresult_analysis_time(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {Uint8Array | undefined}
     */
    get bytecode() {
        const ret = wasm.wasmcompilationwithmetrics_bytecode(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Array<any>}
     */
    get warnings() {
        const ret = wasm.wasmcompilationwithmetrics_warnings(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {Array<any>}
     */
    get errors() {
        const ret = wasm.wasmcompilationwithmetrics_errors(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    get metrics() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilationwithmetrics_metrics(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * Get parsed metrics as JavaScript object
     * @returns {any}
     */
    get_metrics_object() {
        const ret = wasm.wasmcompilationwithmetrics_get_metrics_object(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}
if (Symbol.dispose) WasmCompilationWithMetrics.prototype[Symbol.dispose] = WasmCompilationWithMetrics.prototype.free;

/**
 * Enhanced compiler error for WASM
 */
export class WasmCompilerError {
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
    get code() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilererror_code(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {any}
     */
    get line() {
        const ret = wasm.wasmcompilererror_line(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {any}
     */
    get column() {
        const ret = wasm.wasmcompilererror_column(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {string}
     */
    get severity() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilererror_severity(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get category() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilererror_category(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string}
     */
    get message() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmcompilererror_message(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string | undefined}
     */
    get description() {
        const ret = wasm.wasmcompilererror_description(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {WasmSourceLocation | undefined}
     */
    get location() {
        const ret = wasm.wasmcompilererror_location(this.__wbg_ptr);
        return ret === 0 ? undefined : WasmSourceLocation.__wrap(ret);
    }
    /**
     * @returns {WasmSuggestion[]}
     */
    get suggestions() {
        const ret = wasm.wasmcompilererror_suggestions(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {string | undefined}
     */
    get source_line() {
        const ret = wasm.wasmcompilererror_source_line(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
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
            const ret = wasm.wasmcompilererror_format_terminal(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
            const ret = wasm.wasmcompilererror_format_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmCompilerError.prototype[Symbol.dispose] = WasmCompilerError.prototype.free;

/**
 * Enhanced compilation result with rich error information
 */
export class WasmEnhancedCompilationResult {
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
     * Whether compilation succeeded
     * @returns {boolean}
     */
    get success() {
        const ret = wasm.__wbg_get_wasmanalysisresult_success(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * Whether compilation succeeded
     * @param {boolean} arg0
     */
    set success(arg0) {
        wasm.__wbg_set_wasmanalysisresult_success(this.__wbg_ptr, arg0);
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
     * Size of generated bytecode
     * @param {number} arg0
     */
    set bytecode_size(arg0) {
        wasm.__wbg_set_wasmenhancedcompilationresult_bytecode_size(this.__wbg_ptr, arg0);
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
     * Compilation time in milliseconds
     * @param {number} arg0
     */
    set compilation_time(arg0) {
        wasm.__wbg_set_wasmanalysisresult_analysis_time(this.__wbg_ptr, arg0);
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
     * Total error count
     * @param {number} arg0
     */
    set error_count(arg0) {
        wasm.__wbg_set_wasmenhancedcompilationresult_error_count(this.__wbg_ptr, arg0);
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
        const ret = wasm.wasmenhancedcompilationresult_compiler_errors(this.__wbg_ptr);
        var v1 = getArrayJsValueFromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * Get all errors formatted as terminal output
     * @returns {string}
     */
    format_all_terminal() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmenhancedcompilationresult_format_all_terminal(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
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
            const ret = wasm.wasmenhancedcompilationresult_format_all_json(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmEnhancedCompilationResult.prototype[Symbol.dispose] = WasmEnhancedCompilationResult.prototype.free;

/**
 * WASM DSL Compiler for client-side compilation
 */
export class WasmFiveCompiler {
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
     * Create a new WASM compiler instance
     */
    constructor() {
        const ret = wasm.wasmfivecompiler_new();
        this.__wbg_ptr = ret >>> 0;
        WasmFiveCompilerFinalization.register(this, this.__wbg_ptr, this);
        return this;
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
            const ptr0 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(code, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(severity, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ptr3 = passStringToWasm0(_source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len3 = WASM_VECTOR_LEN;
            const ret = wasm.wasmfivecompiler_format_error_terminal(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2, line, column, ptr3, len3);
            deferred5_0 = ret[0];
            deferred5_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
        }
    }
    /**
     * Unified compilation method with enhanced error reporting and metrics
     * @param {string} source
     * @param {WasmCompilationOptions} options
     * @returns {WasmCompilationResult}
     */
    compile(source, options) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(options, WasmCompilationOptions);
        const ret = wasm.wasmfivecompiler_compile(this.__wbg_ptr, ptr0, len0, options.__wbg_ptr);
        return WasmCompilationResult.__wrap(ret);
    }
    /**
     * Compile multi-file project with automatic discovery
     * @param {string} entry_point
     * @param {WasmCompilationOptions} options
     * @returns {WasmCompilationResult}
     */
    compileMultiWithDiscovery(entry_point, options) {
        const ptr0 = passStringToWasm0(entry_point, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(options, WasmCompilationOptions);
        const ret = wasm.wasmfivecompiler_compileMultiWithDiscovery(this.__wbg_ptr, ptr0, len0, options.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmCompilationResult.__wrap(ret[0]);
    }
    /**
     * Discover modules starting from an entry point
     * @param {string} entry_point
     * @returns {any}
     */
    discoverModules(entry_point) {
        const ptr0 = passStringToWasm0(entry_point, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_discoverModules(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Compile multi-file project with explicit module list
     * @param {any} module_files
     * @param {string} entry_point
     * @param {WasmCompilationOptions} options
     * @returns {WasmCompilationResult}
     */
    compileModules(module_files, entry_point, options) {
        const ptr0 = passStringToWasm0(entry_point, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(options, WasmCompilationOptions);
        const ret = wasm.wasmfivecompiler_compileModules(this.__wbg_ptr, module_files, ptr0, len0, options.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return WasmCompilationResult.__wrap(ret[0]);
    }
    /**
     * Extract function name metadata from compiled bytecode
     * Returns a list of discovered functions in the bytecode
     * @param {Uint8Array} bytecode
     * @returns {any}
     */
    extractFunctionMetadata(bytecode) {
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_extractFunctionMetadata(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Multi-file compilation using module merger (main source + modules)
     * @param {string} main_source
     * @param {any} modules
     * @param {WasmCompilationOptions} options
     * @returns {WasmCompilationResult}
     */
    compile_multi(main_source, modules, options) {
        const ptr0 = passStringToWasm0(main_source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(options, WasmCompilationOptions);
        const ret = wasm.wasmfivecompiler_compile_multi(this.__wbg_ptr, ptr0, len0, modules, options.__wbg_ptr);
        return WasmCompilationResult.__wrap(ret);
    }
    /**
     * Get detailed analysis of source code
     * @param {string} source
     * @returns {WasmAnalysisResult}
     */
    analyze_source(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_analyze_source(this.__wbg_ptr, ptr0, len0);
        return WasmAnalysisResult.__wrap(ret);
    }
    /**
     * Get opcode usage statistics from compilation
     * @param {string} source
     * @returns {any}
     */
    get_opcode_usage(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_get_opcode_usage(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get comprehensive compiler statistics including which opcodes are used vs unused
     * @param {string} source
     * @returns {any}
     */
    get_opcode_analysis(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_get_opcode_analysis(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get detailed analysis of source code with compilation mode selection
     * @param {string} source
     * @param {string} mode
     * @returns {WasmAnalysisResult}
     */
    analyze_source_mode(source, mode) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_analyze_source_mode(this.__wbg_ptr, ptr0, len0, ptr1, len1);
        return WasmAnalysisResult.__wrap(ret);
    }
    /**
     * Parse DSL source code and return AST information
     * @param {string} source
     * @returns {any}
     */
    parse_dsl(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_parse_dsl(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Type-check parsed AST
     * @param {string} _ast_json
     * @returns {any}
     */
    type_check(_ast_json) {
        const ptr0 = passStringToWasm0(_ast_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_type_check(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Optimize bytecode
     * @param {Uint8Array} bytecode
     * @returns {Uint8Array}
     */
    optimize_bytecode(bytecode) {
        const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_optimize_bytecode(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Extract account definitions from DSL source code
     * @param {string} source
     * @returns {any}
     */
    extract_account_definitions(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_extract_account_definitions(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Extract function signatures with account parameters
     * @param {string} source
     * @returns {any}
     */
    extract_function_signatures(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_extract_function_signatures(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Validate account constraints against function parameters
     * @param {string} source
     * @param {string} function_name
     * @param {string} accounts_json
     * @returns {any}
     */
    validate_account_constraints(source, function_name, accounts_json) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(function_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(accounts_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_validate_account_constraints(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Get compiler statistics
     * @returns {any}
     */
    get_compiler_stats() {
        const ret = wasm.wasmfivecompiler_get_compiler_stats(this.__wbg_ptr);
        return ret;
    }
    /**
     * Generate ABI from DSL source code for function calls
     * @param {string} source
     * @returns {any}
     */
    generate_abi(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_generate_abi(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Compile DSL and generate both bytecode and ABI
     * @param {string} source
     * @returns {any}
     */
    compile_with_abi(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_compile_with_abi(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Validate DSL syntax without full compilation
     * @param {string} source
     * @returns {any}
     */
    validate_syntax(source) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmfivecompiler_validate_syntax(this.__wbg_ptr, ptr0, len0);
        return ret;
    }
}
if (Symbol.dispose) WasmFiveCompiler.prototype[Symbol.dispose] = WasmFiveCompiler.prototype.free;

/**
 * WASM-exposed metrics collector wrapper
 */
export class WasmMetricsCollector {
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
    constructor() {
        const ret = wasm.wasmmetricscollector_new();
        this.__wbg_ptr = ret >>> 0;
        WasmMetricsCollectorFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Start timing a compilation phase
     * @param {string} phase_name
     */
    start_phase(phase_name) {
        const ptr0 = passStringToWasm0(phase_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.wasmmetricscollector_start_phase(this.__wbg_ptr, ptr0, len0);
    }
    /**
     * End the current compilation phase
     */
    end_phase() {
        wasm.wasmmetricscollector_end_phase(this.__wbg_ptr);
    }
    /**
     * Finalize metrics collection
     */
    finalize() {
        wasm.wasmmetricscollector_finalize(this.__wbg_ptr);
    }
    /**
     * Reset the collector for a new compilation
     */
    reset() {
        wasm.wasmmetricscollector_reset(this.__wbg_ptr);
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
            const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ret = wasm.wasmmetricscollector_export(this.__wbg_ptr, ptr0, len0);
            var ptr2 = ret[0];
            var len2 = ret[1];
            if (ret[3]) {
                ptr2 = 0; len2 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred3_0 = ptr2;
            deferred3_1 = len2;
            return getStringFromWasm0(ptr2, len2);
        } finally {
            wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
        }
    }
    /**
     * Get metrics as a JS object for programmatic use
     * @returns {any}
     */
    get_metrics_object() {
        const ret = wasm.wasmmetricscollector_get_metrics_object(this.__wbg_ptr);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
}
if (Symbol.dispose) WasmMetricsCollector.prototype[Symbol.dispose] = WasmMetricsCollector.prototype.free;

/**
 * Enhanced source location for WASM
 */
export class WasmSourceLocation {
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
     * Line number (1-based)
     * @returns {number}
     */
    get line() {
        const ret = wasm.__wbg_get_wasmsourcelocation_line(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * Line number (1-based)
     * @param {number} arg0
     */
    set line(arg0) {
        wasm.__wbg_set_wasmsourcelocation_line(this.__wbg_ptr, arg0);
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
     * Column number (1-based)
     * @param {number} arg0
     */
    set column(arg0) {
        wasm.__wbg_set_wasmsourcelocation_column(this.__wbg_ptr, arg0);
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
     * Byte offset in source
     * @param {number} arg0
     */
    set offset(arg0) {
        wasm.__wbg_set_wasmsourcelocation_offset(this.__wbg_ptr, arg0);
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
     * Length of the relevant text
     * @param {number} arg0
     */
    set length(arg0) {
        wasm.__wbg_set_wasmsourcelocation_length(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {string | undefined}
     */
    get file() {
        const ret = wasm.wasmsourcelocation_file(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
}
if (Symbol.dispose) WasmSourceLocation.prototype[Symbol.dispose] = WasmSourceLocation.prototype.free;

/**
 * Enhanced error suggestion for WASM
 */
export class WasmSuggestion {
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
     * @returns {string}
     */
    get message() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmsuggestion_message(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
    /**
     * @returns {string | undefined}
     */
    get explanation() {
        const ret = wasm.wasmsuggestion_explanation(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
    /**
     * @returns {string | undefined}
     */
    get code_suggestion() {
        const ret = wasm.wasmsuggestion_code_suggestion(this.__wbg_ptr);
        let v1;
        if (ret[0] !== 0) {
            v1 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v1;
    }
}
if (Symbol.dispose) WasmSuggestion.prototype[Symbol.dispose] = WasmSuggestion.prototype.free;

/**
 * Get function names from bytecode as a JS value (array of objects)
 *
 * This function avoids constructing `FunctionNameInfo` JS instances and instead
 * marshals the parsed metadata directly into a serde-friendly structure and
 * returns a `JsValue` via `JsValue::from_serde`.
 * @param {Uint8Array} bytecode
 * @returns {any}
 */
export function get_function_names(bytecode) {
    const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.get_function_names(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Get the count of public functions from bytecode header
 * @param {Uint8Array} bytecode
 * @returns {number}
 */
export function get_public_function_count(bytecode) {
    const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.get_public_function_count(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0];
}

/**
 * Get information about the WASM compiler capabilities
 * @returns {any}
 */
export function get_wasm_compiler_info() {
    const ret = wasm.get_wasm_compiler_info();
    return ret;
}

/**
 * Helper function to convert JS value to VM Value
 * @param {any} js_val
 * @param {number} value_type
 * @returns {any}
 */
export function js_value_to_vm_value(js_val, value_type) {
    const ret = wasm.js_value_to_vm_value(js_val, value_type);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} message
 */
export function log_to_console(message) {
    const ptr0 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.log_to_console(ptr0, len0);
}

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
export function parse_function_names(bytecode) {
    const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.parse_function_names(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * Utility: Validate optimized headers and mirror bytecode back to JS callers
 * @param {Uint8Array} bytecode
 * @returns {Uint8Array}
 */
export function wrap_with_script_header(bytecode) {
    const ptr0 = passArray8ToWasm0(bytecode, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.wrap_with_script_header(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

export function __wbg_Error_52673b7de5a0ca89(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return ret;
};

export function __wbg_String_fed4d24b68977888(arg0, arg1) {
    const ret = String(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg___wbindgen_bigint_get_as_i64_6e32f5e6aff02e1d(arg0, arg1) {
    const v = arg1;
    const ret = typeof(v) === 'bigint' ? v : undefined;
    getDataViewMemory0().setBigInt64(arg0 + 8 * 1, isLikeNone(ret) ? BigInt(0) : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
};

export function __wbg___wbindgen_boolean_get_dea25b33882b895b(arg0) {
    const v = arg0;
    const ret = typeof(v) === 'boolean' ? v : undefined;
    return isLikeNone(ret) ? 0xFFFFFF : ret ? 1 : 0;
};

export function __wbg___wbindgen_debug_string_adfb662ae34724b6(arg0, arg1) {
    const ret = debugString(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg___wbindgen_in_0d3e1e8f0c669317(arg0, arg1) {
    const ret = arg0 in arg1;
    return ret;
};

export function __wbg___wbindgen_is_bigint_0e1a2e3f55cfae27(arg0) {
    const ret = typeof(arg0) === 'bigint';
    return ret;
};

export function __wbg___wbindgen_is_function_8d400b8b1af978cd(arg0) {
    const ret = typeof(arg0) === 'function';
    return ret;
};

export function __wbg___wbindgen_is_object_ce774f3490692386(arg0) {
    const val = arg0;
    const ret = typeof(val) === 'object' && val !== null;
    return ret;
};

export function __wbg___wbindgen_is_string_704ef9c8fc131030(arg0) {
    const ret = typeof(arg0) === 'string';
    return ret;
};

export function __wbg___wbindgen_is_undefined_f6b95eab589e0269(arg0) {
    const ret = arg0 === undefined;
    return ret;
};

export function __wbg___wbindgen_jsval_eq_b6101cc9cef1fe36(arg0, arg1) {
    const ret = arg0 === arg1;
    return ret;
};

export function __wbg___wbindgen_jsval_loose_eq_766057600fdd1b0d(arg0, arg1) {
    const ret = arg0 == arg1;
    return ret;
};

export function __wbg___wbindgen_number_get_9619185a74197f95(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
};

export function __wbg___wbindgen_string_get_a2a31e16edf96e42(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg___wbindgen_throw_dd24417ed36fc46e(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

export function __wbg_call_abb4ff46ce38be40() { return handleError(function (arg0, arg1) {
    const ret = arg0.call(arg1);
    return ret;
}, arguments) };

export function __wbg_done_62ea16af4ce34b24(arg0) {
    const ret = arg0.done;
    return ret;
};

export function __wbg_error_7534b8e9a36f1ab4(arg0, arg1) {
    let deferred0_0;
    let deferred0_1;
    try {
        deferred0_0 = arg0;
        deferred0_1 = arg1;
        console.error(getStringFromWasm0(arg0, arg1));
    } finally {
        wasm.__wbindgen_free(deferred0_0, deferred0_1, 1);
    }
};

export function __wbg_error_7bc7d576a6aaf855(arg0) {
    console.error(arg0);
};

export function __wbg_getRandomValues_9b655bdd369112f2() { return handleError(function (arg0, arg1) {
    globalThis.crypto.getRandomValues(getArrayU8FromWasm0(arg0, arg1));
}, arguments) };

export function __wbg_get_6b7bd52aca3f9671(arg0, arg1) {
    const ret = arg0[arg1 >>> 0];
    return ret;
};

export function __wbg_get_af9dab7e9603ea93() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(arg0, arg1);
    return ret;
}, arguments) };

export function __wbg_get_with_ref_key_bb8f74a92cb2e784(arg0, arg1) {
    const ret = arg0[arg1];
    return ret;
};

export function __wbg_instanceof_ArrayBuffer_f3320d2419cd0355(arg0) {
    let result;
    try {
        result = arg0 instanceof ArrayBuffer;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_instanceof_Uint8Array_da54ccc9d3e09434(arg0) {
    let result;
    try {
        result = arg0 instanceof Uint8Array;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

export function __wbg_isArray_51fd9e6422c0a395(arg0) {
    const ret = Array.isArray(arg0);
    return ret;
};

export function __wbg_isSafeInteger_ae7d3f054d55fa16(arg0) {
    const ret = Number.isSafeInteger(arg0);
    return ret;
};

export function __wbg_iterator_27b7c8b35ab3e86b() {
    const ret = Symbol.iterator;
    return ret;
};

export function __wbg_length_22ac23eaec9d8053(arg0) {
    const ret = arg0.length;
    return ret;
};

export function __wbg_length_d45040a40c570362(arg0) {
    const ret = arg0.length;
    return ret;
};

export function __wbg_log_1d990106d99dacb7(arg0) {
    console.log(arg0);
};

export function __wbg_new_1ba21ce319a06297() {
    const ret = new Object();
    return ret;
};

export function __wbg_new_25f239778d6112b9() {
    const ret = new Array();
    return ret;
};

export function __wbg_new_6421f6084cc5bc5a(arg0) {
    const ret = new Uint8Array(arg0);
    return ret;
};

export function __wbg_new_8a6f238a6ece86ea() {
    const ret = new Error();
    return ret;
};

export function __wbg_new_b546ae120718850e() {
    const ret = new Map();
    return ret;
};

export function __wbg_new_from_slice_f9c22b9153b26992(arg0, arg1) {
    const ret = new Uint8Array(getArrayU8FromWasm0(arg0, arg1));
    return ret;
};

export function __wbg_new_no_args_cb138f77cf6151ee(arg0, arg1) {
    const ret = new Function(getStringFromWasm0(arg0, arg1));
    return ret;
};

export function __wbg_next_138a17bbf04e926c(arg0) {
    const ret = arg0.next;
    return ret;
};

export function __wbg_next_3cfe5c0fe2a4cc53() { return handleError(function (arg0) {
    const ret = arg0.next();
    return ret;
}, arguments) };

export function __wbg_now_2c95c9de01293173(arg0) {
    const ret = arg0.now();
    return ret;
};

export function __wbg_now_69d776cd24f5215b() {
    const ret = Date.now();
    return ret;
};

export function __wbg_parse_a09a54cf72639456() { return handleError(function (arg0, arg1) {
    const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
    return ret;
}, arguments) };

export function __wbg_performance_7a3ffd0b17f663ad(arg0) {
    const ret = arg0.performance;
    return ret;
};

export function __wbg_prototypesetcall_dfe9b766cdc1f1fd(arg0, arg1, arg2) {
    Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
};

export function __wbg_push_7d9be8f38fc13975(arg0, arg1) {
    const ret = arg0.push(arg1);
    return ret;
};

export function __wbg_set_3fda3bac07393de4(arg0, arg1, arg2) {
    arg0[arg1] = arg2;
};

export function __wbg_set_781438a03c0c3c81() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(arg0, arg1, arg2);
    return ret;
}, arguments) };

export function __wbg_set_7df433eea03a5c14(arg0, arg1, arg2) {
    arg0[arg1 >>> 0] = arg2;
};

export function __wbg_set_efaaf145b9377369(arg0, arg1, arg2) {
    const ret = arg0.set(arg1, arg2);
    return ret;
};

export function __wbg_stack_0ed75d68575b0f3c(arg0, arg1) {
    const ret = arg1.stack;
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

export function __wbg_static_accessor_GLOBAL_769e6b65d6557335() {
    const ret = typeof global === 'undefined' ? null : global;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_static_accessor_GLOBAL_THIS_60cf02db4de8e1c1() {
    const ret = typeof globalThis === 'undefined' ? null : globalThis;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_static_accessor_SELF_08f5a74c69739274() {
    const ret = typeof self === 'undefined' ? null : self;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_static_accessor_WINDOW_a8924b26aa92d024() {
    const ret = typeof window === 'undefined' ? null : window;
    return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
};

export function __wbg_value_57b7b035e117f7ee(arg0) {
    const ret = arg0.value;
    return ret;
};

export function __wbg_warn_6e567d0d926ff881(arg0) {
    console.warn(arg0);
};

export function __wbg_wasmcompilererror_new(arg0) {
    const ret = WasmCompilerError.__wrap(arg0);
    return ret;
};

export function __wbg_wasmsuggestion_new(arg0) {
    const ret = WasmSuggestion.__wrap(arg0);
    return ret;
};

export function __wbindgen_cast_2241b6af4c4b2941(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
};

export function __wbindgen_cast_4625c577ab2ec9ee(arg0) {
    // Cast intrinsic for `U64 -> Externref`.
    const ret = BigInt.asUintN(64, arg0);
    return ret;
};

export function __wbindgen_cast_9ae0607507abb057(arg0) {
    // Cast intrinsic for `I64 -> Externref`.
    const ret = arg0;
    return ret;
};

export function __wbindgen_cast_d6cd19b81560fd6e(arg0) {
    // Cast intrinsic for `F64 -> Externref`.
    const ret = arg0;
    return ret;
};

export function __wbindgen_init_externref_table() {
    const table = wasm.__wbindgen_externrefs;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
};
