/**
 * Five Bytecode Compiler
 *
 * Handles compilation of Five script source (.v files) to bytecode (.bin files)
 * using the existing WASM compilation infrastructure.
 *
 * This maintains the real compilation capabilities while providing a clean SDK interface.
 */
import { readFile } from "fs/promises";
import { FiveScriptSource, FiveBytecode, CompilationOptions, CompilationResult, CompilationError, CompilationSDKError, FiveFunction, FiveParameter, FiveType, } from "../types.js";
import { normalizeAbiFunctions } from "../utils/abi.js";
/**
 * Bytecode compiler for Five scripts
 */
export class BytecodeCompiler {
    debug;
    wasmCompiler;
    wasmModule;
    constructor(config = {}) {
        this.debug = config.debug || false;
        if (this.debug) {
            console.log("[BytecodeCompiler] Initialized");
        }
    }
    /**
     * Compile Five script source to bytecode
     */
    async compile(source, options = {}) {
        const startTime = Date.now();
        // Compile source (debug info available in this.debug mode)
        try {
            // Lazy load WASM compiler
            if (!this.wasmCompiler) {
                await this.loadWasmCompiler();
            }
            // Prepare compilation options - pass through all metrics options
            const compilerOptions = {
                optimize: options.optimize || false,
                target: options.target || "vm",
                debug: options.debug || false,
                maxSize: options.maxSize || 1048576, // 1MB default
                optimizationLevel: options.optimizationLevel || "production", // Default to Production
                // Pass through metrics options
                metricsFormat: options.metricsFormat,
                metricsOutput: options.metricsOutput,
                errorFormat: options.errorFormat,
                includeMetrics: options.includeMetrics,
                comprehensiveMetrics: options.comprehensiveMetrics,
            };
            // Perform compilation
            const result = await this.wasmCompiler.compile(source, compilerOptions);
            const compilationTime = Date.now() - startTime;
            if (result.success && result.bytecode) {
                let abiData = result.abi;
                if (!abiData) {
                    abiData = await this.generateABI(source);
                }
                if (this.debug) {
                    console.log(`[BytecodeCompiler] Compilation successful in ${compilationTime}ms`);
                    console.log(`[BytecodeCompiler] Bytecode size: ${result.bytecode.length} bytes`);
                }
                const normalizedFunctions = normalizeAbiFunctions(abiData?.functions ?? abiData);
                const normalizedAbi = {
                    ...abiData,
                    functions: normalizedFunctions,
                };
                return {
                    success: true,
                    bytecode: result.bytecode,
                    abi: normalizedAbi,
                    disassembly: result.disassembly || [],
                    metadata: {
                        sourceSize: source.length,
                        bytecodeSize: result.bytecode.length,
                        functions: this.extractFunctions(normalizedAbi),
                        compilationTime,
                    },
                    metricsReport: result.metricsReport,
                };
            }
            else {
                const errors = this.transformErrors(result.errors || []);
                if (this.debug) {
                    console.log(`[BytecodeCompiler] Compilation failed with ${errors.length} errors`);
                    errors.forEach((error) => {
                        console.log(`  - ${error.severity}: ${error.message} (${error.line}:${error.column})`);
                    });
                }
                return {
                    success: false,
                    errors,
                    metricsReport: result.metricsReport,
                };
            }
        }
        catch (error) {
            throw new CompilationSDKError(`Compilation error: ${error instanceof Error ? error.message : "Unknown error"}`, { source: source.substring(0, 200), options });
        }
    }
    async compileWithDiscovery(entryPoint, options = {}) {
        const startTime = Date.now();
        try {
            if (!this.wasmCompiler) {
                await this.loadWasmCompiler();
            }
            if (!this.wasmCompiler?.compileWithDiscovery) {
                throw new CompilationSDKError("Compiler discovery API is not supported in this build");
            }
            const result = await this.wasmCompiler.compileWithDiscovery(entryPoint, options);
            const compilationTime = Date.now() - startTime;
            if (result.success && result.bytecode) {
                return {
                    success: true,
                    bytecode: result.bytecode,
                    abi: result.abi,
                    disassembly: result.disassembly || [],
                    metadata: {
                        sourceFile: entryPoint,
                        sourceSize: 0,
                        bytecodeSize: result.bytecode.length,
                        functions: this.extractFunctions(result.abi || { functions: [] }),
                        compilationTime,
                    },
                    metricsReport: result.metricsReport,
                };
            }
            const errors = this.transformErrors(result.errors || []);
            return {
                success: false,
                errors,
                metricsReport: result.metricsReport,
            };
        }
        catch (error) {
            throw new CompilationSDKError(`Compilation error: ${error instanceof Error ? error.message : "Unknown error"}`, { entryPoint, options });
        }
    }
    /**
     * Compile script from file path
     */
    async compileFile(filePath, options = {}) {
        if (this.debug) {
            console.log(`[BytecodeCompiler] Reading file: ${filePath}`);
        }
        try {
            const source = await readFile(filePath, "utf-8");
            return this.compile(source, options);
        }
        catch (error) {
            throw new CompilationSDKError(`Failed to read file ${filePath}: ${error instanceof Error ? error.message : "Unknown error"}`, { filePath, options });
        }
    }
    /**
     * Validate Five script source without compiling
     */
    async validateSource(source) {
        if (this.debug) {
            console.log(`[BytecodeCompiler] Validating source (${source.length} chars)...`);
        }
        try {
            if (!this.wasmCompiler) {
                await this.loadWasmCompiler();
            }
            const result = await this.wasmCompiler.validateSource(source);
            return {
                valid: result.valid,
                errors: result.errors ? this.transformErrors(result.errors) : undefined,
            };
        }
        catch (error) {
            if (this.debug) {
                console.log(`[BytecodeCompiler] Validation error: ${error}`);
            }
            return {
                valid: false,
                errors: [
                    {
                        message: error instanceof Error
                            ? error.message
                            : "Unknown validation error",
                        severity: "error",
                    },
                ],
            };
        }
    }
    /**
     * Get compiler version and information
     */
    async getCompilerInfo() {
        try {
            if (!this.wasmCompiler) {
                await this.loadWasmCompiler();
            }
            return {
                version: "1.0.0", // TODO: Get from WASM module
                wasmLoaded: !!this.wasmCompiler,
                debug: this.debug,
            };
        }
        catch (error) {
            return {
                version: "unknown",
                wasmLoaded: false,
                debug: this.debug,
                error: error instanceof Error ? error.message : "Unknown error",
            };
        }
    }
    // ==================== Private Methods ====================
    /**
     * Load WASM compiler (reuse existing infrastructure)
     */
    async loadWasmCompiler() {
        try {
            // Load WASM compiler silently unless debug
            // Import existing WASM compiler from the CLI
            // This path points to the existing working compiler
            const wasmModule = await import("../../wasm/compiler.js");
            const wasmInstance = new wasmModule.FiveCompilerWasm(this.createWasmLogger());
            // CRITICAL: Initialize the compiler - this step was missing!
            await wasmInstance.initialize();
            this.wasmCompiler = wasmInstance;
            this.wasmModule = wasmModule;
            if (this.debug) {
                console.log("[BytecodeCompiler] WASM compiler loaded and initialized successfully");
            }
        }
        catch (error) {
            throw new CompilationSDKError(`Failed to load WASM compiler: ${error instanceof Error ? error.message : "Unknown error"}`, { debug: this.debug });
        }
    }

    createWasmLogger() {
        if (this.debug) {
            return console;
        }
        return {
            debug: () => { },
            info: () => { },
            warn: console.warn.bind(console),
            error: console.error.bind(console),
        };
    }

    /**
     * Transform compiler errors to SDK format
     */
    transformErrors(errors) {
        return errors.map((error) => ({
            message: error.message || error.toString(),
            line: error.line,
            column: error.column,
            severity: error.severity || "error",
        }));
    }
    /**
     * Extract function definitions from ABI
     */
    extractFunctions(abi) {
        const functions = normalizeAbiFunctions(abi?.functions ?? abi);
        return functions.map((func) => ({
            name: func.name,
            index: func.index,
            parameters: func.parameters?.map((param) => ({
                name: param.name,
                type: param.type,
                optional: param.optional || false,
            })) || [],
            returnType: func.returnType,
        }));
    }
    /**
     * Generate ABI from Five script source code
     */
    async generateABI(source) {
        if (this.debug) {
            console.log(`[BytecodeCompiler] Generating ABI for source (${source.length} chars)...`);
        }
        try {
            if (!this.wasmCompiler) {
                await this.loadWasmCompiler();
            }
            const abi = await this.wasmCompiler.generateABI(source);
            const normalizedFunctions = normalizeAbiFunctions(abi?.functions ?? abi);
            return { ...abi, functions: normalizedFunctions };
            if (this.debug) {
                console.log(`[BytecodeCompiler] ABI generated:`, JSON.stringify(abi, null, 2));
            }
            return { functions: [] };
        }
        catch (error) {
            if (this.debug) {
                console.log(`[BytecodeCompiler] ABI generation error: ${error}`);
            }
            throw new CompilationSDKError(`ABI generation failed: ${error instanceof Error ? error.message : "Unknown error"}`, { source });
        }
    }
    /**
     * Extract function names from compiled bytecode
     */
    async getFunctionNames(bytecode) {
        if (this.debug) {
            console.log(`[BytecodeCompiler] Extracting function names from bytecode (${bytecode.length} bytes)...`);
        }
        try {
            if (!this.wasmCompiler) {
                await this.loadWasmCompiler();
            }
            let namesJson = null;
            if (this.wasmCompiler &&
                typeof this.wasmCompiler.getFunctionNames === "function") {
                namesJson = await this.wasmCompiler.getFunctionNames(bytecode);
            }
            else if (this.wasmModule &&
                typeof this.wasmModule.get_function_names === "function") {
                namesJson = await this.wasmModule.get_function_names(bytecode);
            }
            // Fallback: direct call if previous attempts yielded no data
            if (!namesJson && this.wasmModule?.get_function_names) {
                namesJson = this.wasmModule.get_function_names(bytecode);
            }
            if (!namesJson) {
                try {
                    const directModule = (await import("../../assets/vm/five_vm_wasm.js"));
                    if (typeof directModule.get_function_names === "function") {
                        namesJson = directModule.get_function_names(bytecode);
                    }
                }
                catch (e) {
                    if (this.debug) {
                        console.log("[BytecodeCompiler] Direct import fallback for function names failed:", e);
                    }
                }
            }
            let parsedNames = namesJson;
            if (typeof namesJson === "string") {
                try {
                    parsedNames = JSON.parse(namesJson);
                }
                catch (e) {
                    if (this.debug) {
                        console.log(`[BytecodeCompiler] Failed to parse function names JSON:`, e);
                    }
                }
            }
            if (this.debug) {
                console.log(`[BytecodeCompiler] Function names extracted: ${JSON.stringify(parsedNames)}`);
            }
            return parsedNames || [];
        }
        catch (error) {
            if (this.debug) {
                console.log(`[BytecodeCompiler] Function name extraction error: ${error}`);
            }
            throw new CompilationSDKError(`Function name extraction failed: ${error instanceof Error ? error.message : "Unknown error"}`, { bytecodeLength: bytecode.length });
        }
    }
}
//# sourceMappingURL=BytecodeCompiler.js.map
