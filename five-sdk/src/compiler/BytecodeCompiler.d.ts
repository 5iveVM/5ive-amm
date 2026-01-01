/**
 * Five Bytecode Compiler
 *
 * Handles compilation of Five script source (.v files) to bytecode (.bin files)
 * using the existing WASM compilation infrastructure.
 *
 * This maintains the real compilation capabilities while providing a clean SDK interface.
 */
import { FiveScriptSource, FiveBytecode, CompilationOptions, CompilationResult, CompilationError } from "../types.js";
/**
 * Compiler configuration
 */
interface CompilerConfig {
    debug?: boolean;
    wasmPath?: string;
}
/**
 * Bytecode compiler for Five scripts
 */
export declare class BytecodeCompiler {
    private debug;
    private wasmCompiler?;
    private wasmModule?;
    constructor(config?: CompilerConfig);
    /**
     * Compile Five script source to bytecode
     */
    compile(source: FiveScriptSource, options?: CompilationOptions): Promise<CompilationResult>;
    /**
     * Compile multiple modules (entry + dependencies)
     */
    compileModules(mainSource: FiveScriptSource, modules: Array<{
        name: string;
        source: string;
    }>, options?: CompilationOptions): Promise<CompilationResult>;
    /**
     * Compile script from file path
     */
    compileFile(filePath: string, options?: CompilationOptions): Promise<CompilationResult>;
    /**
     * Validate Five script source without compiling
     */
    validateSource(source: FiveScriptSource): Promise<{
        valid: boolean;
        errors?: CompilationError[];
    }>;
    /**
     * Get compiler version and information
     */
    getCompilerInfo(): Promise<{
        version: string;
        wasmLoaded: boolean;
        debug: boolean;
        error?: never;
    } | {
        version: string;
        wasmLoaded: boolean;
        debug: boolean;
        error: string;
    }>;
    /**
     * Load WASM compiler (reuse existing infrastructure)
     */
    private loadWasmCompiler;
    /**
     * Transform compiler errors to SDK format
     */
    private transformErrors;
    /**
     * Extract function definitions from ABI
     */
    private extractFunctions;
    /**
     * Generate ABI from Five script source code
     */
    generateABI(source: FiveScriptSource): Promise<any>;
    /**
     * Extract function names from compiled bytecode
     */
    getFunctionNames(bytecode: FiveBytecode): Promise<any>;
}
export {};
//# sourceMappingURL=BytecodeCompiler.d.ts.map