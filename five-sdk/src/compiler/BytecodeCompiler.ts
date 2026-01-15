/**
 * Five Bytecode Compiler
 *
 * Handles compilation of Five script source (.v files) to bytecode (.bin files)
 * using the existing WASM compilation infrastructure.
 *
 * This maintains the real compilation capabilities while providing a clean SDK interface.
 */

import { readFile } from "fs/promises";
import {
  FiveScriptSource,
  FiveBytecode,
  CompilationOptions,
  CompilationResult,
  CompilationError,
  CompilationSDKError,
  FiveFunction,
  FiveParameter,
  FiveType,
  CompilationTarget,
} from "../types.js";
import { normalizeAbiFunctions } from "../utils/abi.js";

/**
 * Compiler configuration
 */
interface CompilerConfig {
  debug?: boolean;
  wasmPath?: string;
}

/**
 * WASM Compiler interface (matches existing implementation)
 */
interface WasmCompiler {
  compile(
    source: string,
    options?: any,
  ): Promise<{
    success: boolean;
    bytecode?: Uint8Array;
    abi?: any;
    errors?: any[];
    metadata?: any;
    disassembly?: string[];
  }>;

  getCompilerInfo(): { version: string; features: string[] };

  validateSource(source: string): Promise<{
    valid: boolean;
    errors: string[];
    warnings: string[];
  }>;

  generateABI(source: string): Promise<any>;

  getFunctionNames?(bytecode: Uint8Array): Promise<string>;

  compile_multi?(
    mainSource: string,
    modules: Array<{ name: string; source: string }>,
    options?: any,
  ): Promise<{
    success: boolean;
    bytecode?: Uint8Array;
    abi?: any;
    errors?: any[];
    metadata?: any;
    disassembly?: string[];
  }>;
}

/**
 * Bytecode compiler for Five scripts
 */
export class BytecodeCompiler {
  private debug: boolean;
  private wasmCompiler?: WasmCompiler;
  private wasmModule?: any;

  constructor(config: CompilerConfig = {}) {
    this.debug = config.debug || false;

    if (this.debug) {
      console.log("[BytecodeCompiler] Initialized");
    }
  }

  /**
   * Compile Five script source to bytecode
   */
  async compile(
    source: FiveScriptSource | string,
    options: CompilationOptions = {},
  ): Promise<CompilationResult> {
    const startTime = Date.now();
    const sourceContent = typeof source === 'string' ? source : source.content;
    const sourceFilename = typeof source === 'string' ? 'unknown.v' : source.filename || 'unknown.v';

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
        metricsFormat: (options as any).metricsFormat,
        metricsOutput: (options as any).metricsOutput,
        errorFormat: (options as any).errorFormat,
        includeMetrics: (options as any).includeMetrics,
        comprehensiveMetrics: (options as any).comprehensiveMetrics,
      };

      // Perform compilation
      const result = await this.wasmCompiler!.compile(sourceContent, compilerOptions);

      const compilationTime = Date.now() - startTime;

      if (result.success && result.bytecode) {
        let abiData = result.abi as any;
        if (!abiData) {
          abiData = await this.generateABI(source);
        }

        if (this.debug) {
          console.log(
            `[BytecodeCompiler] Compilation successful in ${compilationTime}ms`,
          );
          console.log(
            `[BytecodeCompiler] Bytecode size: ${result.bytecode.length} bytes`,
          );
        }

        const normalizedFunctions = normalizeAbiFunctions(
          (abiData as any)?.functions ?? abiData,
        );
        const normalizedAbi = {
          ...(abiData as any),
          functions: normalizedFunctions,
        };

        const compilerInfo = await this.getCompilerInfo();

        return {
          success: true,
          bytecode: result.bytecode,
          abi: normalizedAbi,
          disassembly: result.disassembly || [],
          metadata: {
            sourceFile: sourceFilename,
            timestamp: new Date().toISOString(),
            compilerVersion: compilerInfo.version || '1.0.0',
            target: (options.target || 'vm') as CompilationTarget,
            optimizations: [],
            originalSize: sourceContent.length,
            compressedSize: result.bytecode.length,
            compressionRatio: 1.0,
            sourceSize: sourceContent.length,
            bytecodeSize: result.bytecode.length,
            functions: this.extractFunctions(normalizedAbi),
            compilationTime,
          },
          metricsReport: (result as any).metricsReport,
        };
      } else {
        const errors = this.transformErrors(result.errors || []);

        if (this.debug) {
          console.log(
            `[BytecodeCompiler] Compilation failed with ${errors.length} errors`,
          );
          errors.forEach((error) => {
            console.log(
              `  - ${error.severity}: ${error.message} (${error.line}:${error.column})`,
            );
          });
        }

        return {
          success: false,
          errors,
          metricsReport: (result as any).metricsReport,
        };
      }
    } catch (error) {
      throw new CompilationSDKError(
        `Compilation error: ${error instanceof Error ? error.message : "Unknown error"}`,
        { source: sourceContent.substring(0, 200), options },
      );
    }
  }

  /**
   * Compile multiple modules (entry + dependencies)
   */
  async compileModules(
    mainSource: FiveScriptSource,
    modules: Array<{ name: string; source: string }>,
    options: CompilationOptions = {},
  ): Promise<CompilationResult> {
    const startTime = Date.now();

    try {
      if (!this.wasmCompiler) {
        await this.loadWasmCompiler();
      }

      if (!this.wasmCompiler?.compile_multi) {
        throw new CompilationSDKError("Multi-file compilation is not supported in this build");
      }

      const compilerOptions = {
        optimize: options.optimize || false,
        target: options.target || "vm",
        debug: options.debug || false,
        maxSize: options.maxSize || 1048576,
        optimizationLevel: options.optimizationLevel || "production",
        includeMetrics: options.includeMetrics || options.metricsOutput !== undefined,
        metricsFormat: options.metricsFormat || "json",
        errorFormat: options.errorFormat || "terminal",
        comprehensiveMetrics: options.comprehensiveMetrics || false,
      };

      const result = await this.wasmCompiler.compile_multi(
        mainSource.content,
        modules,
        compilerOptions,
      );

      const compilationTime = Date.now() - startTime;

      if (result.success && result.bytecode) {
        const compilerInfo = await this.getCompilerInfo();

        return {
          success: true,
          bytecode: result.bytecode,
          abi: result.abi,
          disassembly: result.disassembly || [],
          metadata: {
            sourceFile: mainSource.filename || 'main.v',
            timestamp: new Date().toISOString(),
            compilerVersion: compilerInfo.version || '1.0.0',
            target: (options.target || 'vm') as CompilationTarget,
            optimizations: [],
            originalSize: mainSource.content.length,
            compressedSize: result.bytecode.length,
            compressionRatio: 1.0,
            sourceSize: mainSource.content.length,
            bytecodeSize: result.bytecode.length,
            functions: [],
            compilationTime,
          },
          metricsReport: (result as any).metricsReport,
        };
      } else {
        const errors = this.transformErrors((result as any).errors || (result as any).compiler_errors || []);

        return {
          success: false,
          errors,
          metricsReport: (result as any).metricsReport,
        };
      }
    } catch (error) {
      throw new CompilationSDKError(
        `Compilation error: ${error instanceof Error ? error.message : "Unknown error"}`,
        { options },
      );
    }
  }

  /**
   * Compile script from file path
   */
  async compileFile(
    filePath: string,
    options: CompilationOptions = {},
  ): Promise<CompilationResult> {
    if (this.debug) {
      console.log(`[BytecodeCompiler] Reading file: ${filePath}`);
    }

    try {
      const source = await readFile(filePath, "utf-8");
      return this.compile(source, options);
    } catch (error) {
      throw new CompilationSDKError(
        `Failed to read file ${filePath}: ${error instanceof Error ? error.message : "Unknown error"}`,
        { filePath, options },
      );
    }
  }

  /**
   * Validate Five script source without compiling
   */
  async validateSource(source: FiveScriptSource | string): Promise<{
    valid: boolean;
    errors?: CompilationError[];
  }> {
    const code = typeof source === 'string' ? source : source.content;

    if (this.debug) {
      console.log(
        `[BytecodeCompiler] Validating source (${code.length} chars)...`,
      );
    }

    try {
      if (!this.wasmCompiler) {
        await this.loadWasmCompiler();
      }

      const result = await this.wasmCompiler!.validateSource(code);

      return {
        valid: result.valid,
        errors: result.errors ? this.transformErrors(result.errors) : undefined,
      };
    } catch (error) {
      if (this.debug) {
        console.log(`[BytecodeCompiler] Validation error: ${error}`);
      }

      return {
        valid: false,
        errors: [
          {
            message:
              error instanceof Error
                ? error.message
                : "Unknown validation error",
            severity: "error",
          } as CompilationError,
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

      let version = "1.0.0";
      if (this.wasmCompiler && typeof this.wasmCompiler.getCompilerInfo === 'function') {
        const info = this.wasmCompiler.getCompilerInfo();
        version = info.version;
      }

      return {
        version,
        wasmLoaded: !!this.wasmCompiler,
        debug: this.debug,
      };
    } catch (error) {
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
  private async loadWasmCompiler(): Promise<void> {
    try {
      // Load WASM compiler silently unless debug

      // Import existing WASM compiler from the SDK
      const wasmModule = await import("../wasm/compiler/index.js");
      const wasmInstance = new wasmModule.FiveCompiler(this.createWasmLogger());

      // Initialize the compiler
      await wasmInstance.initialize();

      this.wasmCompiler = wasmInstance as unknown as WasmCompiler;
      this.wasmModule = wasmModule;

      if (this.debug) {
        console.log(
          "[BytecodeCompiler] WASM compiler loaded and initialized successfully",
        );
      }
    } catch (error) {
      throw new CompilationSDKError(
        `Failed to load WASM compiler: ${error instanceof Error ? error.message : "Unknown error"}`,
        { debug: this.debug },
      );
    }
  }

  private createWasmLogger() {
    if (this.debug) {
      return console;
    }

    return {
      debug: () => {},
      info: () => {},
      warn: console.warn.bind(console),
      error: console.error.bind(console),
    };
  }

  /**
   * Transform compiler errors to SDK format
   */
  private transformErrors(errors: any[]): CompilationError[] {
    return errors.map((error) => ({
      type: 'compiler',
      message: error.message || error.toString(),
      line: error.line,
      column: error.column,
      severity: error.severity || "error",
    }));
  }

  /**
   * Extract function definitions from ABI
   */
  private extractFunctions(abi: any): FiveFunction[] {
    const functions = normalizeAbiFunctions(abi?.functions ?? abi);

    return functions.map((func) => ({
      name: func.name,
      index: func.index,
      parameters:
        func.parameters?.map((param: any) => ({
          name: param.name,
          type: param.type as FiveType,
          optional: param.optional || false,
        })) || [],
      returnType: func.returnType as FiveType | undefined,
    }));
  }

  /**
   * Generate ABI from Five script source code
   */
  async generateABI(source: FiveScriptSource | string): Promise<any> {
    const code = typeof source === 'string' ? source : source.content;

    if (this.debug) {
      console.log(
        `[BytecodeCompiler] Generating ABI for source (${code.length} chars)...`,
      );
    }

    try {
      if (!this.wasmCompiler) {
        await this.loadWasmCompiler();
      }

      const abi = await this.wasmCompiler!.generateABI(code);
      const normalizedFunctions = normalizeAbiFunctions(
        (abi as any)?.functions ?? abi,
      );
      return { ...(abi as any), functions: normalizedFunctions };

    } catch (error) {
      if (this.debug) {
        console.log(`[BytecodeCompiler] ABI generation error: ${error}`);
      }
      throw new CompilationSDKError(
        `ABI generation failed: ${error instanceof Error ? error.message : "Unknown error"}`,
        { source: code.substring(0, 100) },
      );
    }
  }

  /**
   * Extract function names from compiled bytecode
   */
  async getFunctionNames(bytecode: FiveBytecode): Promise<any> {
    if (this.debug) {
      console.log(
        `[BytecodeCompiler] Extracting function names from bytecode (${bytecode.length} bytes)...`,
      );
    }

    try {
      if (!this.wasmCompiler) {
        await this.loadWasmCompiler();
      }

      let namesJson: any = null;
      if (
        this.wasmCompiler &&
        typeof (this.wasmCompiler as any).getFunctionNames === "function"
      ) {
        namesJson = await (this.wasmCompiler as any).getFunctionNames(bytecode);
      } else if (
        this.wasmModule &&
        typeof this.wasmModule.get_function_names === "function"
      ) {
        namesJson = await this.wasmModule.get_function_names(bytecode);
      }
      // Fallback: direct call if previous attempts yielded no data
      if (!namesJson && this.wasmModule?.get_function_names) {
        namesJson = this.wasmModule.get_function_names(bytecode);
      }

      let parsedNames: any = namesJson;
      if (typeof namesJson === "string") {
        try {
          parsedNames = JSON.parse(namesJson);
        } catch (e) {
          if (this.debug) {
            console.log(`[BytecodeCompiler] Failed to parse function names JSON:`, e);
          }
        }
      }

      if (this.debug) {
        console.log(
          `[BytecodeCompiler] Function names extracted: ${JSON.stringify(parsedNames)}`,
        );
      }

      return parsedNames || [];
    } catch (error) {
      if (this.debug) {
        console.log(
          `[BytecodeCompiler] Function name extraction error: ${error}`,
        );
      }
      throw new CompilationSDKError(
        `Function name extraction failed: ${error instanceof Error ? error.message : "Unknown error"}`,
        { bytecodeLength: bytecode.length },
      );
    }
  }
}
