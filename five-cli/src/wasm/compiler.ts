// Five compiler WASM integration.

import { readFile, writeFile } from "fs/promises";
import {
  CompilationOptions,
  CompilationResult,
  Logger,
  CLIError,
} from "../types.js";
import { existsSync, readFileSync } from "fs";
import { dirname, resolve } from "path";
import { fileURLToPath } from "url";
import { ConfigManager } from "../config/ConfigManager.js";
import { createRequire } from "module";

const require = createRequire(import.meta.url);

// Real Five VM WASM imports
let WasmFiveCompiler: any;
let FiveVMWasm: any;
let BytecodeAnalyzer: any;
let WasmCompilationOptions: any;
let wasmModuleRef: any | null = null;

export class FiveCompilerWasm {
  private compiler: any = null;
  private logger: Logger;
  private initialized = false;

  constructor(logger: Logger) {
    this.logger = logger;
  }

  async initialize(): Promise<void> {
    try {
      // Try multiple candidate locations (mirrors vm.ts logic)
      const cfg = await ConfigManager.getInstance().get();
      const prefer = cfg.wasm?.loader || "auto";
      const configured = Array.isArray(cfg.wasm?.modulePaths)
        ? cfg.wasm!.modulePaths!
        : [];
      const nodeCandidates = ["../../five_vm_wasm.js", "../five_vm_wasm.js"];
      const bundlerCandidates = [
        "../../assets/vm/five_vm_wasm.js",
        "../assets/vm/five_vm_wasm.js",
      ];
      let candidates: string[] = [];
      candidates.push(...configured);
      if (prefer === "node") {
        candidates.push(...nodeCandidates);
      } else if (prefer === "bundler") {
        candidates.push(...bundlerCandidates);
      } else {
        candidates.push(...nodeCandidates, ...bundlerCandidates);
      }
      let wasmModule: any | null = null;
      const tried: Array<{ path: string; error: unknown }> = [];
      for (const candidate of candidates) {
        try {
          // Use require for Node.js CommonJS WASM modules
          const mod = require(candidate as string);
          // The Node.js target (wasm-pack --target nodejs) is pre-initialized
          // No need to call initSync() - just check if it has the expected exports
          if (
            mod &&
            (mod.WasmFiveCompiler || mod.FiveCompilerWasm) &&
            mod.FiveVMWasm
          ) {
            wasmModule = mod;
            break;
          }
          if (!(mod.WasmFiveCompiler || mod.FiveCompilerWasm)) {
            tried.push({ path: candidate, error: "Missing WasmFiveCompiler export" });
          } else if (!mod.FiveVMWasm) {
            tried.push({ path: candidate, error: "Missing FiveVMWasm export" });
          } else {
            tried.push({ path: candidate, error: "Module loaded but invalid" });
          }
        } catch (e) {
          tried.push({ path: candidate, error: e });
        }
      }
      if (!wasmModule) {
        const attempted = tried
          .map(
            (t) =>
              `  - ${t.path}: ${t.error instanceof Error ? t.error.message : String(t.error)}`,
          )
          .join("\n");
        throw new Error(
          `Failed to load WASM compiler module. Attempted:\n${attempted}`,
        );
      }

      wasmModuleRef = wasmModule;
      WasmFiveCompiler =
        wasmModule.WasmFiveCompiler || wasmModule.FiveCompilerWasm;
      FiveVMWasm = wasmModule.FiveVMWasm;
      BytecodeAnalyzer = wasmModule.BytecodeAnalyzer;
      WasmCompilationOptions = wasmModule.WasmCompilationOptions;

      // Initialize the compiler instance
      this.compiler = new WasmFiveCompiler();
      this.initialized = true;

      // WASM compiler initialized silently
    } catch (error) {
      throw this.createCompilerError(
        `Five VM WASM modules not found. Please run "npm run build:wasm" to build the required WebAssembly modules. Error: ${error}`,
        error as Error,
      );
    }
  }

  /**
   * Compile Five DSL source code from string (SDK compatibility)
   */
  async compile(source: string, options?: any): Promise<any> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    const startTime = Date.now();

    // Extract metrics configuration at top level for scope
    const metricsFormat = options?.metricsFormat || "json";

    try {
      // Compile source silently

      // Use enhanced WASM compiler methods with rich error messages
      let result: any;

      // Compile using WASM bindings

      try {
        // Use unified compilation method (module already loaded in initialize)
        if (!WasmCompilationOptions && wasmModuleRef) {
          WasmCompilationOptions = wasmModuleRef.WasmCompilationOptions;
        }

        // Enable metrics if explicitly requested via metricsFormat or metricsOutput
        const includeMetrics = options?.includeMetrics || Boolean(options?.metricsOutput) || Boolean(options?.metricsFormat);
        const errorFormat = options?.errorFormat || "terminal";
        const comprehensiveMetrics = options?.comprehensiveMetrics || Boolean(options?.metricsOutput);

        const compilationOptions = new WasmCompilationOptions()
          .with_mode(options?.target || "deployment")
          .with_optimization_level(options?.optimizationLevel || "production")
          .with_v2_preview(true)
          .with_constraint_cache(false)
          .with_enhanced_errors(true)
          .with_metrics(includeMetrics)
          .with_comprehensive_metrics(comprehensiveMetrics)
          .with_metrics_format(metricsFormat)
          .with_error_format(errorFormat)
          .with_module_namespaces(!Boolean(options?.flatNamespace));

        // Execute compilation
        result = this.compiler.compile(source, compilationOptions);

        // Process compilation result silently

        // Log compiler errors for debugging if present
        if (result.compiler_errors && result.compiler_errors.length > 0) {
          this.logger.debug(
            `WASM returned ${result.compiler_errors.length} compiler errors`,
          );
          for (const error of result.compiler_errors) {
            this.logger.debug(`Compiler error details:`, error);

            // Extract error details using WASM getters
            try {
              const message = error.message ? error.message : "No message";
              const code = error.code ? error.code : "No code";
              const severity = error.severity ? error.severity : "Unknown";
              const category = error.category ? error.category : "Unknown";

              this.logger.error(
                `Detailed error: [${code}] ${severity} - ${message}`,
              );
              this.logger.error(`Category: ${category}`);

              // Try to get location information
              if (error.location) {
                const location = error.location;
                this.logger.error(
                  `Location: line ${location.line}, column ${location.column}`,
                );
              }
            } catch (e) {
              this.logger.debug(`Failed to extract error details:`, e);
              this.logger.debug(`Raw error object:`, error);
            }
          }
        }
      } catch (wasmError) {
        this.logger.debug("WASM compilation threw an error:", wasmError);
        throw wasmError;
      }

      const metricsPayload = this.extractMetrics(result, metricsFormat);

      // Transform result format to match expected SDK interface
      if (result.success && result.bytecode) {
        return {
          success: true,
          bytecode: result.bytecode,
          abi: this.extractAbi(result),
          metadata: result.metadata,
          metrics: metricsPayload,
          metricsReport: metricsPayload,
          formattedErrorsTerminal: typeof result.format_all_terminal === "function" ? result.format_all_terminal() : (result.formatted_errors_terminal || undefined),
          formattedErrorsJson: typeof result.format_all_json === "function" ? result.format_all_json() : (result.formatted_errors_json || undefined),
        };
      } else {
        return {
          success: false,
          errors: result.compiler_errors || [],
          metadata: result.metadata,
          metrics: metricsPayload,
          metricsReport: metricsPayload,
          formattedErrorsTerminal: typeof result.format_all_terminal === "function" ? result.format_all_terminal() : (result.formatted_errors_terminal || undefined),
          formattedErrorsJson: typeof result.format_all_json === "function" ? result.format_all_json() : (result.formatted_errors_json || undefined),
        };
      }
    } catch (error) {
      throw this.createCompilerError(
        `Compilation error: ${error instanceof Error ? error.message : "Unknown error"}`,
        error as Error,
      );
    }
  }

  /**
   * Compile Five DSL source code from file (CLI compatibility)
   */
  async compileFile(options: CompilationOptions): Promise<CompilationResult> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    const startTime = Date.now();

    // Extract metrics configuration at top level for scope
    const metricsFormat = (options as any).metricsFormat || "json";

    try {
      // Read source file
      const sourceCode = await readFile(options.sourceFile, "utf8");

      this.logger.debug(`Compiling source file: ${options.sourceFile}`);

      // Use enhanced WASM compiler methods with rich error messages
      let result: any;

      // Compile using WASM bindings

      try {
        // Use unified compilation method (module already loaded in initialize)
        if (!WasmCompilationOptions && wasmModuleRef) {
          WasmCompilationOptions = wasmModuleRef.WasmCompilationOptions;
        }
        // Enable metrics if explicitly requested via metricsFormat or metricsOutput
        const includeMetrics =
          (options as any).includeMetrics || Boolean((options as any).metricsOutput) || Boolean((options as any).metricsFormat);
        const errorFormat = (options as any).errorFormat || "terminal";
        const comprehensiveMetrics = (options as any).comprehensiveMetrics || Boolean((options as any).metricsOutput);
        const compilationOptions = new WasmCompilationOptions()
          .with_mode(options.target || "deployment")
          .with_optimization_level((options as any).optimizationLevel || "production")
          .with_v2_preview(true)
          .with_constraint_cache(
            (options as any).enable_constraint_cache !== false,
          )
          .with_enhanced_errors(true)
          .with_metrics(includeMetrics)
          .with_comprehensive_metrics(comprehensiveMetrics)
          .with_metrics_format(metricsFormat)
          .with_error_format(errorFormat)
          .with_source_file(options.sourceFile);

        // Execute compilation
        result = this.compiler.compile(sourceCode, compilationOptions);

        // Process compilation result silently

        // Log compiler errors for debugging if present
        if (result.compiler_errors && result.compiler_errors.length > 0) {
          this.logger.debug(
            `WASM returned ${result.compiler_errors.length} compiler errors`,
          );
          for (const error of result.compiler_errors) {
            this.logger.debug(`Compiler error details:`, error);

            // Extract error details using WASM getters
            try {
              const message = error.message ? error.message : "No message";
              const code = error.code ? error.code : "No code";
              const severity = error.severity ? error.severity : "Unknown";
              const category = error.category ? error.category : "Unknown";

              this.logger.error(
                `Detailed error: [${code}] ${severity} - ${message}`,
              );
              this.logger.error(`Category: ${category}`);

              // Try to get location information
              if (error.location) {
                const location = error.location;
                this.logger.error(
                  `Location: line ${location.line}, column ${location.column}`,
                );
              }
            } catch (e) {
              this.logger.debug(`Failed to extract error details:`, e);
              this.logger.debug(`Raw error object:`, error);
            }
          }
        }
      } catch (wasmError) {
        this.logger.error("WASM compiler threw exception:", wasmError);
        // If WASM throws exception, create a structured error response
        result = {
          success: false,
          bytecode: null,
          bytecode_size: 0,
          compilation_time: 0,
          compiler_errors: [
            {
              code: "E9999",
              severity: "error",
              category: "wasm_exception",
              message:
                wasmError instanceof Error
                  ? wasmError.message
                  : String(wasmError),
              description: "WASM compiler threw an exception",
              location: null,
              suggestions: [],
            },
          ],
          error_count: 1,
          warning_count: 0,
        };
      }

      // Check if compilation failed but we have enhanced error information
      if (
        !result.success &&
        result.compiler_errors &&
        result.compiler_errors.length > 0
      ) {
        // Handle compilation errors
        // Log enhanced error details using WASM formatting methods
        this.logger.debug(`Enhanced Error Count: ${result.error_count}`);

        // Try to get formatted output from WASM
        try {
          const terminalOutput = result.format_all_terminal
            ? result.format_all_terminal()
            : null;
          if (terminalOutput) {
            this.logger.debug("Terminal formatted errors:", terminalOutput);
          }

          const jsonOutput = result.format_all_json
            ? result.format_all_json()
            : null;
          if (jsonOutput) {
            this.logger.debug("JSON formatted errors:", jsonOutput);
          }
        } catch (formatError) {
          this.logger.debug("Failed to get formatted errors:", formatError);
        }
      }

      const compilationTime = Date.now() - startTime;
      const metricsPayload = this.extractMetrics(result, metricsFormat);

      // Use WASM-provided formatting methods to get structured error information
      let convertedErrors: any[] = [];

      if (result.compiler_errors && result.compiler_errors.length > 0) {
        try {
          // Try to get JSON formatted errors from WASM
          const jsonErrors = result.format_all_json
            ? result.format_all_json()
            : null;
          if (jsonErrors) {
            const parsedErrors = JSON.parse(jsonErrors);
            convertedErrors = parsedErrors.map((error: any) => ({
              type: "enhanced",
              ...error,
              // Ensure proper structure for CLI display
              code: error.code || "E0000",
              severity: error.severity || "error",
              category: error.category || "compilation",
              message: error.message || "Unknown error",
            }));
          } else {
            // Fallback: create basic errors from the result
            convertedErrors = [
              {
                type: "enhanced",
                code: "E0004",
                severity: "error",
                category: "compilation",
                message: "InvalidScript",
                description: "The script contains syntax or semantic errors",
                location: undefined,
                suggestions: [],
              },
            ];
          }
        } catch (parseError) {
          this.logger.debug(
            "Failed to parse JSON errors from WASM:",
            parseError,
          );
          // Fallback to basic error
          convertedErrors = [
            {
              type: "enhanced",
              code: "E0004",
              severity: "error",
              category: "compilation",
              message: "InvalidScript",
              description: "Compilation failed with enhanced error system",
              location: undefined,
              suggestions: [],
            },
          ];
        }
      }

      // Extract ABI - get_abi() returns JSON string, parse it
      let abi = undefined;
      try {
        const abiJson = result.get_abi();
        if (abiJson) {
          abi = JSON.parse(abiJson);
        }
      } catch (e) {
        this.logger.debug('Failed to parse ABI from get_abi():', e);
      }

      const compilationResult: CompilationResult = {
        success: result.success,
        bytecode: result.bytecode ? new Uint8Array(result.bytecode) : undefined,
        abi: abi,
        errors: convertedErrors,
        warnings:
          convertedErrors.filter((e: any) => e.severity === "warning") || [],
        metrics: {
          compilationTime: result.compilation_time || compilationTime,
          bytecodeSize: result.bytecode_size || 0,
          memoryUsed: 0, // Not available from WASM
          optimizationTime: 0,
          instructionCount: 0, // Would need analysis
          functionCount: 0, // Would need analysis
        },
        metricsReport: metricsPayload,
        formattedErrorsTerminal: typeof result.format_all_terminal === "function" ? result.format_all_terminal() : (result.formatted_errors_terminal || undefined),
        formattedErrorsJson: typeof result.format_all_json === "function" ? result.format_all_json() : (result.formatted_errors_json || undefined),
      };

      // Write output file if specified and compilation succeeded
      if (options.outputFile && compilationResult.bytecode) {
        await writeFile(options.outputFile, compilationResult.bytecode);
        this.logger.debug(`Bytecode written to: ${options.outputFile}`);
      }

      // Write ABI file if generated
      if (options.abiOutputFile && compilationResult.abi) {
        await writeFile(
          options.abiOutputFile,
          JSON.stringify(compilationResult.abi, null, 2),
        );
        this.logger.debug(`ABI written to: ${options.abiOutputFile}`);
      }

      this.logger.debug(`Compilation completed in ${compilationTime}ms`);
      return compilationResult;
    } catch (error) {
      const compilationTime = Date.now() - startTime;
      return {
        success: false,
        errors: [
          {
            type: "runtime",
            message: error instanceof Error ? error.message : String(error),
            sourceLocation: options.sourceFile,
          },
        ],
        warnings: [],
        metrics: {
          compilationTime,
          memoryUsed: 0,
          optimizationTime: 0,
          bytecodeSize: 0,
          instructionCount: 0,
          functionCount: 0,
        },
      };
    }
  }

  /**
   * Generate ABI from DSL source using real WASM compiler
   */
  async generateABI(sourceCode: string): Promise<any> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const abi = this.compiler.generate_abi(sourceCode);
      return JSON.parse(abi);
    } catch (error) {
      throw this.createCompilerError("ABI generation failed", error as Error);
    }
  }

  /**
   * Validate DSL syntax using real WASM compiler
   */
  async validateSource(
    sourceCode: string,
  ): Promise<{ valid: boolean; errors: string[]; warnings: string[] }> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const result = this.compiler.validate_syntax(sourceCode);
      return JSON.parse(result);
    } catch (error) {
      throw this.createCompilerError(
        "Syntax validation failed",
        error as Error,
      );
    }
  }

  /**
   * Optimize bytecode using real WASM optimizer
   */
  async optimizeBytecode(bytecode: Uint8Array): Promise<Uint8Array> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const optimized = this.compiler.optimize_bytecode(bytecode);
      return new Uint8Array(optimized);
    } catch (error) {
      throw this.createCompilerError(
        "Bytecode optimization failed",
        error as Error,
      );
    }
  }

  /**
   * Analyze bytecode using real WASM analyzer
   */
  async analyzeBytecode(bytecode: Uint8Array): Promise<any> {
    if (!this.initialized) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      // Use semantic analysis for detailed information
      const analysis = BytecodeAnalyzer.analyze_semantic(bytecode);
      return JSON.parse(analysis);
    } catch (error) {
      throw this.createCompilerError(
        "Bytecode analysis failed",
        error as Error,
      );
    }
  }

  /**
   * Get detailed instruction analysis at specific offset
   */
  async analyzeInstructionAt(
    bytecode: Uint8Array,
    offset: number,
  ): Promise<any> {
    if (!this.initialized) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const instruction = BytecodeAnalyzer.analyze_instruction_at(
        bytecode,
        offset,
      );
      return JSON.parse(instruction);
    } catch (error) {
      throw this.createCompilerError(
        "Instruction analysis failed",
        error as Error,
      );
    }
  }

  /**
   * Get bytecode summary statistics
   */
  async getBytecodeStats(bytecode: Uint8Array): Promise<any> {
    if (!this.initialized) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const stats = BytecodeAnalyzer.get_bytecode_summary(bytecode);
      return JSON.parse(stats);
    } catch (error) {
      throw this.createCompilerError("Bytecode stats failed", error as Error);
    }
  }

  /**
   * Extract account definitions from DSL source
   */
  async extractAccountDefinitions(sourceCode: string): Promise<any> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const definitions = this.compiler.extract_account_definitions(sourceCode);
      return JSON.parse(definitions);
    } catch (error) {
      throw this.createCompilerError(
        "Account definition extraction failed",
        error as Error,
      );
    }
  }

  /**
   * Extract function signatures from DSL source
   */
  async extractFunctionSignatures(sourceCode: string): Promise<any> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const signatures = this.compiler.extract_function_signatures(sourceCode);
      return JSON.parse(signatures);
    } catch (error) {
      throw this.createCompilerError(
        "Function signature extraction failed",
        error as Error,
      );
    }
  }

  /**
   * Validate account constraints
   */
  async validateAccountConstraints(
    sourceCode: string,
    functionName: string,
    accounts: any[],
  ): Promise<any> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const accountsJson = JSON.stringify(accounts);
      const validation = this.compiler.validate_account_constraints(
        sourceCode,
        functionName,
        accountsJson,
      );
      return JSON.parse(validation);
    } catch (error) {
      throw this.createCompilerError(
        "Account constraint validation failed",
        error as Error,
      );
    }
  }

  /**
   * Get compiler version and capabilities from real WASM
   */
  getCompilerInfo(): { version: string; features: string[] } {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const info = this.compiler.get_compiler_stats();
      return JSON.parse(info);
    } catch (error) {
      throw this.createCompilerError(
        "Failed to get compiler info",
        error as Error,
      );
    }
  }

  /**
   * Create a standardized compiler error
   */
  private createCompilerError(message: string, cause?: Error): CLIError {
    const error = new Error(message) as CLIError;
    error.name = "CompilerError";
    error.code = "COMPILER_ERROR";
    error.category = "wasm";
    error.exitCode = 1;

    if (cause) {
      error.details = {
        cause: cause.message,
        stack: cause.stack,
      };
    }

    return error;
  }

  private extractAbi(result: any): any | undefined {
    if (!result) {
      return undefined;
    }

    if (typeof result.get_abi === "function") {
      try {
        const abiJson = result.get_abi();
        if (abiJson) {
          return JSON.parse(abiJson);
        }
      } catch (error) {
        this.logger.debug("Failed to parse ABI from get_abi():", error);
      }
    }

    const directAbi = result.abi;
    if (!directAbi) {
      return undefined;
    }

    if (typeof directAbi === "string") {
      try {
        return JSON.parse(directAbi);
      } catch (error) {
        this.logger.debug("Failed to parse ABI from result.abi string:", error);
        return undefined;
      }
    }

    return directAbi;
  }

  /**
   * Get opcode usage statistics from compilation
   */
  async getOpcodeUsage(sourceCode: string): Promise<any> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const usage = this.compiler.get_opcode_usage(sourceCode);
      return JSON.parse(usage);
    } catch (error) {
      throw this.createCompilerError(
        "Opcode usage analysis failed",
        error as Error,
      );
    }
  }

  /**
   * Get comprehensive opcode analysis showing used vs unused opcodes
   */
  async getOpcodeAnalysis(sourceCode: string): Promise<any> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      const analysis = this.compiler.get_opcode_analysis(sourceCode);
      return JSON.parse(analysis);
    } catch (error) {
      throw this.createCompilerError(
        "Comprehensive opcode analysis failed",
        error as Error,
      );
    }
  }

  /**
   * Discover modules starting from entry point (tooling support)
   */
  async discoverModules(entryPoint: string): Promise<string[]> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    try {
      return this.compiler.discoverModules(entryPoint);
    } catch (error) {
      throw this.createCompilerError(
        `Module discovery failed: ${error instanceof Error ? error.message : String(error)}`,
        error as Error
      );
    }
  }

  /**
   * Compile multi-file project with automatic discovery
   */
  async compileWithDiscovery(
    entryPoint: string,
    options?: any
  ): Promise<any> {
    if (!this.initialized || !this.compiler) {
      throw this.createCompilerError("Compiler not initialized");
    }

    const startTime = Date.now();
    const metricsFormat = options?.metricsFormat || "json";

    try {
      if (!WasmCompilationOptions && wasmModuleRef) {
        WasmCompilationOptions = wasmModuleRef.WasmCompilationOptions;
      }

      const includeMetrics = options?.includeMetrics || Boolean(options?.metricsOutput);
      const errorFormat = options?.errorFormat || "terminal";
      const comprehensiveMetrics = options?.comprehensiveMetrics || false;

      const compilationOptions = new WasmCompilationOptions()
        .with_mode(options?.target || "deployment")
        .with_optimization_level(options?.optimizationLevel || "production")
        .with_v2_preview(true)
        .with_constraint_cache(false)
        .with_enhanced_errors(true)
        .with_metrics(includeMetrics)
        .with_comprehensive_metrics(comprehensiveMetrics)
        .with_metrics_format(metricsFormat)
        .with_error_format(errorFormat)
        .with_module_namespaces(!Boolean(options?.flatNamespace))
        .with_source_file(entryPoint);

      const result = this.compiler.compileMultiWithDiscovery(entryPoint, compilationOptions);

      const metricsPayload = this.extractMetrics(result, metricsFormat);

      if (result.success && result.bytecode) {
        // Extract ABI - get_abi() returns JSON string, parse it
        let abi = undefined;
        try {
          const abiJson = result.get_abi();
          if (abiJson) {
            abi = JSON.parse(abiJson);
          }
        } catch (e) {
          this.logger.debug('Failed to parse ABI from get_abi():', e);
        }

        return {
          success: true,
          bytecode: result.bytecode,
          abi: abi,
          metadata: result.metadata,
          metrics: metricsPayload,
          metricsReport: metricsPayload,
        };
      } else {
        return {
          success: false,
          errors: result.compiler_errors || [],
          formattedErrorsTerminal: typeof result.format_all_terminal === "function" ? result.format_all_terminal() : (result as any).formatted_errors_terminal,
          formattedErrorsJson: typeof result.format_all_json === "function" ? result.format_all_json() : (result as any).formatted_errors_json,
          metadata: result.metadata,
          metrics: metricsPayload,
          metricsReport: metricsPayload,
        };
      }
    } catch (error) {
      throw this.createCompilerError(
        `Compilation error: ${error instanceof Error ? error.message : "Unknown error"}`,
        error as Error
      );
    }
  }

  private extractMetrics(
    result: any,
    defaultFormat: string,
  ): { format: string; exported: string; detailed?: any } | undefined {
    if (!result || typeof result !== "object") {
      return undefined;
    }

    const exported = (result as any).metrics;

    if (!exported) {
      return undefined;
    }

    const format = (result as any).metrics_format || defaultFormat;
    let detailed: any | undefined;

    try {
      const getDetailed = (result as any).get_metrics_detailed;
      if (typeof getDetailed === "function") {
        detailed = getDetailed.call(result);
      } else {
        const getObject = (result as any).get_metrics_object;
        if (typeof getObject === "function") {
          detailed = getObject.call(result);
        }
      }
    } catch (metricError) {
      // Ignore metrics errors
    }

    return {
      format,
      exported,
      detailed,
    };
  }

  /**
   * Check if compiler is ready
   */
  isReady(): boolean {
    return this.initialized && this.compiler !== null;
  }
}
