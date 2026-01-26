let wasm;

function _assertNum(n) {
    if (typeof(n) !== 'number') throw new Error(`expected a number argument, found ${typeof(n)}`);
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

function logError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        let error = (function () {
            try {
                return e instanceof Error ? `${e.message}\n\nStack:\n${e.stack}` : e.toString();
            } catch(_) {
                return "<failed to stringify thrown value>";
            }
        }());
        console.error("wasm-bindgen: imported JS function that was not marked as `catch` threw an error:", error);
        throw e;
    }
}

function passStringToWasm0(arg, malloc, realloc) {
    if (typeof(arg) !== 'string') throw new Error(`expected a string argument, found ${typeof(arg)}`);
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
        if (ret.read !== arg.length) throw new Error('failed to pass whole string');
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

const FiveLspWasmFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_fivelspwasm_free(ptr >>> 0, 1));

/**
 * WASM wrapper for the Five LSP compiler bridge
 *
 * This is the main entry point for WASM clients. It wraps the Rust
 * CompilerBridge and exposes it to JavaScript via wasm-bindgen.
 */
export class FiveLspWasm {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        FiveLspWasmFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_fivelspwasm_free(ptr, 0);
    }
    /**
     * Create a new LSP instance
     *
     * This initializes the compiler bridge and prepares it for use.
     */
    constructor() {
        const ret = wasm.fivelspwasm_new();
        this.__wbg_ptr = ret >>> 0;
        FiveLspWasmFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get diagnostics for a Five DSL file
     *
     * # Arguments
     * * `uri` - File URI (e.g., "file:///test.v")
     * * `source` - The source code to analyze
     *
     * # Returns
     * A JSON string containing an array of diagnostics, or an error message
     *
     * # Example
     * ```typescript
     * const lsp = FiveLspWasm.new();
     * const result = lsp.get_diagnostics('file:///test.v', 'init { let x = 5; }');
     * const diagnostics = JSON.parse(result);
     * ```
     * @param {string} uri
     * @param {string} source
     * @returns {string}
     */
    get_diagnostics(uri, source) {
        let deferred4_0;
        let deferred4_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ptr0 = passStringToWasm0(uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.fivelspwasm_get_diagnostics(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Get hover information for a symbol at the given position
     *
     * # Arguments
     * * `uri` - File URI (e.g., "file:///test.v")
     * * `source` - The source code
     * * `line` - 0-indexed line number
     * * `character` - 0-indexed character position
     *
     * # Returns
     * A JSON string containing hover information, or error message
     *
     * # Example
     * ```typescript
     * const lsp = FiveLspWasm.new();
     * const result = lsp.get_hover('file:///test.v', 'let x = 5;', 0, 4);
     * const hover = result ? JSON.parse(result) : null;
     * ```
     * @param {string} uri
     * @param {string} source
     * @param {number} line
     * @param {number} character
     * @returns {string | undefined}
     */
    get_hover(uri, source, line, character) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        _assertNum(line);
        _assertNum(character);
        const ret = wasm.fivelspwasm_get_hover(this.__wbg_ptr, ptr0, len0, ptr1, len1, line, character);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        let v3;
        if (ret[0] !== 0) {
            v3 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v3;
    }
    /**
     * Get completion suggestions at the given position
     *
     * # Arguments
     * * `uri` - File URI (e.g., "file:///test.v")
     * * `source` - The source code
     * * `line` - 0-indexed line number
     * * `character` - 0-indexed character position
     *
     * # Returns
     * A JSON string containing CompletionList with suggestions
     *
     * # Example
     * ```typescript
     * const lsp = FiveLspWasm.new();
     * const result = lsp.get_completions('file:///test.v', 'let x = ', 0, 8);
     * const completions = JSON.parse(result);
     * ```
     * @param {string} uri
     * @param {string} source
     * @param {number} line
     * @param {number} character
     * @returns {string}
     */
    get_completions(uri, source, line, character) {
        let deferred4_0;
        let deferred4_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ptr0 = passStringToWasm0(uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            _assertNum(line);
            _assertNum(character);
            const ret = wasm.fivelspwasm_get_completions(this.__wbg_ptr, ptr0, len0, ptr1, len1, line, character);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Get go-to-definition location for a symbol at the given position
     *
     * # Arguments
     * * `uri` - File URI (e.g., "file:///test.v")
     * * `source` - The source code
     * * `line` - 0-indexed line number
     * * `character` - 0-indexed character position
     *
     * # Returns
     * A JSON string containing Location if definition found, null otherwise
     *
     * # Example
     * ```typescript
     * const lsp = FiveLspWasm.new();
     * const result = lsp.get_definition('file:///test.v', 'function foo() {}', 0, 9);
     * const location = result ? JSON.parse(result) : null;
     * ```
     * @param {string} uri
     * @param {string} source
     * @param {number} line
     * @param {number} character
     * @returns {string | undefined}
     */
    get_definition(uri, source, line, character) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        _assertNum(line);
        _assertNum(character);
        const ret = wasm.fivelspwasm_get_definition(this.__wbg_ptr, ptr0, len0, ptr1, len1, line, character);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        let v3;
        if (ret[0] !== 0) {
            v3 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v3;
    }
    /**
     * Find all references to a symbol at the given position
     *
     * # Arguments
     * * `uri` - File URI (e.g., "file:///test.v")
     * * `source` - The source code
     * * `line` - 0-indexed line number
     * * `character` - 0-indexed character position
     *
     * # Returns
     * A JSON string containing an array of Locations where the symbol is referenced
     *
     * # Example
     * ```typescript
     * const lsp = FiveLspWasm.new();
     * const result = lsp.find_references('file:///test.v', 'let x = 5; let y = x;', 0, 4);
     * const references = JSON.parse(result);  // Array of Location objects
     * ```
     * @param {string} uri
     * @param {string} source
     * @param {number} line
     * @param {number} character
     * @returns {string}
     */
    find_references(uri, source, line, character) {
        let deferred4_0;
        let deferred4_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ptr0 = passStringToWasm0(uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            _assertNum(line);
            _assertNum(character);
            const ret = wasm.fivelspwasm_find_references(this.__wbg_ptr, ptr0, len0, ptr1, len1, line, character);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Get semantic tokens for syntax highlighting
     *
     * Returns an array of semantic tokens for AST-based syntax highlighting.
     * Provides more accurate highlighting than regex-based approaches.
     * @param {string} uri
     * @param {string} source
     * @returns {string}
     */
    get_semantic_tokens(uri, source) {
        let deferred4_0;
        let deferred4_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ptr0 = passStringToWasm0(uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.fivelspwasm_get_semantic_tokens(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Get document symbols for outline view
     *
     * Returns all top-level definitions (functions, variables, accounts) for
     * display in the editor's outline/navigator panel.
     * @param {string} uri
     * @param {string} source
     * @returns {string}
     */
    get_document_symbols(uri, source) {
        let deferred4_0;
        let deferred4_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ptr0 = passStringToWasm0(uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ret = wasm.fivelspwasm_get_document_symbols(this.__wbg_ptr, ptr0, len0, ptr1, len1);
            var ptr3 = ret[0];
            var len3 = ret[1];
            if (ret[3]) {
                ptr3 = 0; len3 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred4_0 = ptr3;
            deferred4_1 = len3;
            return getStringFromWasm0(ptr3, len3);
        } finally {
            wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
        }
    }
    /**
     * Get code actions for a diagnostic
     *
     * Provides quick fix suggestions for a diagnostic at the given position.
     * @param {string} uri
     * @param {string} source
     * @param {string} diagnostic_json
     * @returns {string}
     */
    get_code_actions(uri, source, diagnostic_json) {
        let deferred5_0;
        let deferred5_1;
        try {
            if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
            _assertNum(this.__wbg_ptr);
            const ptr0 = passStringToWasm0(uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len0 = WASM_VECTOR_LEN;
            const ptr1 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len1 = WASM_VECTOR_LEN;
            const ptr2 = passStringToWasm0(diagnostic_json, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
            const len2 = WASM_VECTOR_LEN;
            const ret = wasm.fivelspwasm_get_code_actions(this.__wbg_ptr, ptr0, len0, ptr1, len1, ptr2, len2);
            var ptr4 = ret[0];
            var len4 = ret[1];
            if (ret[3]) {
                ptr4 = 0; len4 = 0;
                throw takeFromExternrefTable0(ret[2]);
            }
            deferred5_0 = ptr4;
            deferred5_1 = len4;
            return getStringFromWasm0(ptr4, len4);
        } finally {
            wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
        }
    }
    /**
     * Prepare a rename operation
     *
     * Validates that a symbol at the given position can be renamed and returns its name.
     * @param {string} source
     * @param {number} line
     * @param {number} character
     * @returns {string | undefined}
     */
    static prepare_rename(source, line, character) {
        const ptr0 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        _assertNum(line);
        _assertNum(character);
        const ret = wasm.fivelspwasm_prepare_rename(ptr0, len0, line, character);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        let v2;
        if (ret[0] !== 0) {
            v2 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v2;
    }
    /**
     * Rename a symbol across all occurrences
     *
     * Performs a safe rename of a symbol, updating all references to it.
     * @param {string} uri
     * @param {string} source
     * @param {number} line
     * @param {number} character
     * @param {string} new_name
     * @returns {string | undefined}
     */
    rename(uri, source, line, character, new_name) {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        const ptr0 = passStringToWasm0(uri, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        _assertNum(line);
        _assertNum(character);
        const ptr2 = passStringToWasm0(new_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.fivelspwasm_rename(this.__wbg_ptr, ptr0, len0, ptr1, len1, line, character, ptr2, len2);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        let v4;
        if (ret[0] !== 0) {
            v4 = getStringFromWasm0(ret[0], ret[1]).slice();
            wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        }
        return v4;
    }
    /**
     * Clear all caches
     *
     * Useful after large changes or when memory needs to be freed.
     * This forces recompilation on the next analysis.
     */
    clear_caches() {
        if (this.__wbg_ptr == 0) throw new Error('Attempt to use a moved value');
        _assertNum(this.__wbg_ptr);
        wasm.fivelspwasm_clear_caches(this.__wbg_ptr);
    }
}
if (Symbol.dispose) FiveLspWasm.prototype[Symbol.dispose] = FiveLspWasm.prototype.free;

const EXPECTED_RESPONSE_TYPES = new Set(['basic', 'cors', 'default']);

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && EXPECTED_RESPONSE_TYPES.has(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg___wbindgen_throw_dd24417ed36fc46e = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_cast_2241b6af4c4b2941 = function() { return logError(function (arg0, arg1) {
        // Cast intrinsic for `Ref(String) -> Externref`.
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_externrefs;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
    };

    return imports;
}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('five_lsp_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
