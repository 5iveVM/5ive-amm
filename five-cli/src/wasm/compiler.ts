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
const moduleDir = dirname(fileURLToPath(import.meta.url));

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
          .with_module_namespaces(!Boolean(options?.flatNamespace))
          .with_source_file(options?.sourceFile || "input.v");

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
      const formattedErrors = this.extractFormattedErrors(result);
      const diagnostics = this.extractDiagnostics(result);

      // Transform result format to match expected SDK interface
      if (result.success && result.bytecode) {
        return {
          success: true,
          bytecode: result.bytecode,
          abi: this.extractAbi(result),
          metadata: result.metadata,
          metrics: metricsPayload,
          metricsReport: metricsPayload,
          formattedErrorsTerminal: formattedErrors.terminal,
          formattedErrorsJson: formattedErrors.json,
        };
      } else {
        return {
          success: false,
          errors: diagnostics,
          warnings: diagnostics.filter((diag) => diag.severity === "warning"),
          diagnostics,
          metadata: result.metadata,
          metrics: metricsPayload,
          metricsReport: metricsPayload,
          formattedErrorsTerminal: formattedErrors.terminal,
          formattedErrorsJson: formattedErrors.json,
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
      const diagnostics = this.extractDiagnostics(result);
      const formattedErrors = this.extractFormattedErrors(result);

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
        errors: diagnostics,
        warnings: diagnostics.filter((diag) => diag.severity === "warning"),
        diagnostics,
        metrics: {
          compilationTime: result.compilation_time || compilationTime,
          bytecodeSize: result.bytecode_size || 0,
          memoryUsed: 0, // Not available from WASM
          optimizationTime: 0,
          instructionCount: 0, // Would need analysis
          functionCount: 0, // Would need analysis
        },
        metricsReport: metricsPayload,
        formattedErrorsTerminal: formattedErrors.terminal,
        formattedErrorsJson: formattedErrors.json,
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
    if (
      cause &&
      typeof cause === "object" &&
      (cause as any).code === "COMPILER_ERROR"
    ) {
      return cause as CLIError;
    }

    const error = new Error(message) as CLIError;
    error.name = "CompilerError";
    error.code = "COMPILER_ERROR";
    error.category = "wasm";
    error.exitCode = 1;

    if (cause) {
      const inheritedDetails =
        cause && typeof (cause as any).details === "object"
          ? (cause as any).details
          : undefined;
      error.details = {
        ...(inheritedDetails || {}),
        cause: cause.message,
        stack: cause.stack,
      };
    }

    return error;
  }

  private isNonEmptyString(value: unknown): value is string {
    return typeof value === "string" && value.trim().length > 0;
  }

  private toOptionalNumber(value: unknown): number | undefined {
    if (typeof value === "number" && Number.isFinite(value)) {
      return value;
    }
    if (typeof value === "string" && value.trim().length > 0) {
      const parsed = Number(value);
      if (Number.isFinite(parsed)) {
        return parsed;
      }
    }
    return undefined;
  }

  private extractFormattedErrors(result: any): {
    terminal?: string;
    json?: string;
  } {
    const safeCall = (methodName: string): string | undefined => {
      if (!result || typeof result !== "object") {
        return undefined;
      }
      const fn = (result as any)[methodName];
      if (typeof fn !== "function") {
        return undefined;
      }
      try {
        const value = fn.call(result);
        return this.isNonEmptyString(value) ? value : undefined;
      } catch {
        return undefined;
      }
    };

    const readProp = (name: string): string | undefined => {
      if (!result || typeof result !== "object") {
        return undefined;
      }
      const value = (result as any)[name];
      return this.isNonEmptyString(value) ? value : undefined;
    };

    const terminal =
      safeCall("get_formatted_errors_terminal") ||
      readProp("formattedErrorsTerminal") ||
      readProp("formatted_errors_terminal") ||
      safeCall("format_all_terminal");

    const json =
      safeCall("format_all_json") ||
      safeCall("get_formatted_errors_json") ||
      readProp("formattedErrorsJson") ||
      readProp("formatted_errors_json");

    return { terminal, json };
  }

  private normalizeDiagnosticSuggestion(suggestion: any):
    | { message: string; explanation?: string; confidence?: number; codeSuggestion?: string }
    | undefined {
    if (typeof suggestion === "string") {
      return { message: suggestion };
    }

    if (!suggestion || typeof suggestion !== "object") {
      return undefined;
    }

    const message = this.isNonEmptyString(suggestion.message)
      ? suggestion.message
      : this.isNonEmptyString(suggestion.explanation)
        ? suggestion.explanation
        : undefined;
    if (!message) {
      return undefined;
    }

    return {
      message,
      explanation: this.isNonEmptyString(suggestion.explanation)
        ? suggestion.explanation
        : undefined,
      confidence: this.toOptionalNumber(suggestion.confidence),
      codeSuggestion: this.isNonEmptyString(suggestion.code_suggestion)
        ? suggestion.code_suggestion
        : this.isNonEmptyString(suggestion.codeSuggestion)
          ? suggestion.codeSuggestion
          : undefined,
    };
  }

  private buildFallbackSuggestions(code: string): Array<{ message: string; confidence?: number }> {
    switch (code) {
      case "E2000":
        return [
          {
            message: "Declare the variable before use with `let <name> = ...`.",
            confidence: 0.8,
          },
          {
            message: "Check for spelling differences between parameter/field names and usages.",
            confidence: 0.7,
          },
        ];
      case "E0002":
        return [
          {
            message: "Check for missing closing `}`, `)`, or an incomplete function signature.",
            confidence: 0.75,
          },
        ];
      case "E0001":
      case "E0004":
        return [
          {
            message: "Check for missing punctuation (`;`, `{`, `}`) near the reported statement.",
            confidence: 0.7,
          },
        ];
      default:
        return [];
    }
  }

  private normalizeDiagnostic(error: any): any {
    let parsedFromFormatter: any;
    if (error && typeof error?.format_json === "function") {
      try {
        const parsed = JSON.parse(error.format_json());
        if (parsed && typeof parsed === "object") {
          parsedFromFormatter = parsed;
        }
      } catch {
        // Ignore formatter parsing issues and continue with direct field access.
      }
    }

    const merged = {
      ...(parsedFromFormatter || {}),
      ...(error && typeof error === "object" ? error : {}),
    };

    const location =
      merged.location && typeof merged.location === "object"
        ? merged.location
        : undefined;
    const line = this.toOptionalNumber(merged.line) ?? this.toOptionalNumber(location?.line);
    const column =
      this.toOptionalNumber(merged.column) ?? this.toOptionalNumber(location?.column);
    const file =
      this.isNonEmptyString(merged.sourceLocation)
        ? merged.sourceLocation
        : this.isNonEmptyString(location?.file)
          ? location.file
          : undefined;

    const code = this.isNonEmptyString(merged.code) ? merged.code : "E0000";
    const rawSuggestions = Array.isArray(merged.suggestions)
      ? merged.suggestions
      : [];
    const suggestions = rawSuggestions
      .map((item: any) => this.normalizeDiagnosticSuggestion(item))
      .filter(Boolean) as Array<{ message: string; explanation?: string; confidence?: number; codeSuggestion?: string }>;

    if (suggestions.length === 0) {
      suggestions.push(...this.buildFallbackSuggestions(code));
    }

    const rendered = this.isNonEmptyString(merged.rendered)
      ? merged.rendered
      : typeof merged.format_terminal === "function"
        ? merged.format_terminal()
        : undefined;

    return {
      type: this.isNonEmptyString(merged.type) ? merged.type : "enhanced",
      code,
      severity: this.isNonEmptyString(merged.severity) ? merged.severity : "error",
      category: this.isNonEmptyString(merged.category)
        ? merged.category
        : "compilation",
      message: this.isNonEmptyString(merged.message)
        ? merged.message
        : this.isNonEmptyString(merged.description)
          ? merged.description
          : "Unknown compiler error",
      description: this.isNonEmptyString(merged.description)
        ? merged.description
        : undefined,
      line,
      column,
      sourceLocation: file,
      location,
      suggestion: suggestions[0]?.message,
      suggestions,
      sourceLine: this.isNonEmptyString(merged.sourceLine)
        ? merged.sourceLine
        : this.isNonEmptyString(merged.source_line)
          ? merged.source_line
          : undefined,
      sourceSnippet: this.isNonEmptyString(merged.sourceSnippet)
        ? merged.sourceSnippet
        : this.isNonEmptyString(merged.source_snippet)
          ? merged.source_snippet
          : undefined,
      rendered: this.isNonEmptyString(rendered) ? rendered : undefined,
      raw: error,
    };
  }

  private extractDiagnostics(result: any): any[] {
    const formatted = this.extractFormattedErrors(result);

    if (formatted.json) {
      try {
        const parsed = JSON.parse(formatted.json);
        const parsedErrors = Array.isArray(parsed)
          ? parsed
          : Array.isArray(parsed?.errors)
            ? parsed.errors
            : [];

        if (parsedErrors.length > 0) {
          return parsedErrors.map((item: any) => this.normalizeDiagnostic(item));
        }
      } catch (parseError) {
        this.logger.debug("Failed to parse formatted JSON diagnostics:", parseError);
      }
    }

    if (Array.isArray(result?.compiler_errors)) {
      return result.compiler_errors.map((item: any) =>
        this.normalizeDiagnostic(item),
      );
    }

    if (Array.isArray(result?.errors)) {
      return result.errors.map((item: any) => this.normalizeDiagnostic(item));
    }

    return [];
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

      const compilationOptions = this.createDiscoveryCompilationOptions(entryPoint, options, metricsFormat);
      let result = this.compiler.compileMultiWithDiscovery(entryPoint, compilationOptions);

      if (!result.success && this.isDiscoveryFsFailure(result)) {
        result = await this.compileWithLocalModuleMap(entryPoint, options, compilationOptions);
      }

      return this.toCompilationOutput(result, metricsFormat);
    } catch (error) {
      if (this.isDiscoveryFsFailure(error)) {
        try {
          const compilationOptions = this.createDiscoveryCompilationOptions(entryPoint, options, metricsFormat);
          const fallbackResult = await this.compileWithLocalModuleMap(entryPoint, options, compilationOptions);
          return this.toCompilationOutput(fallbackResult, metricsFormat);
        } catch (fallbackError) {
          throw this.createCompilerError(
            `Compilation error: ${fallbackError instanceof Error ? fallbackError.message : "Unknown error"}`,
            fallbackError as Error
          );
        }
      }
      throw this.createCompilerError(
        `Compilation error: ${error instanceof Error ? error.message : "Unknown error"}`,
        error as Error
      );
    }
  }

  private createDiscoveryCompilationOptions(entryPoint: string, options: any, metricsFormat: string): any {
    const includeMetrics = options?.includeMetrics || Boolean(options?.metricsOutput);
    const errorFormat = options?.errorFormat || "terminal";
    const comprehensiveMetrics = options?.comprehensiveMetrics || false;

    return new WasmCompilationOptions()
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
  }

  private toCompilationOutput(result: any, metricsFormat: string): any {
    const metricsPayload = this.extractMetrics(result, metricsFormat);
    const diagnostics = this.extractDiagnostics(result);
    const formattedErrors = this.extractFormattedErrors(result);

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
        formattedErrorsTerminal: formattedErrors.terminal,
        formattedErrorsJson: formattedErrors.json,
      };
    }

    return {
      success: false,
      errors: diagnostics,
      warnings: diagnostics.filter((diag: any) => diag.severity === "warning"),
      diagnostics,
      formattedErrorsTerminal: formattedErrors.terminal,
      formattedErrorsJson: formattedErrors.json,
      metadata: result.metadata,
      metrics: metricsPayload,
      metricsReport: metricsPayload,
    };
  }

  private isDiscoveryFsFailure(errorOrResult: any): boolean {
    const messages: string[] = [];

    if (!errorOrResult) {
      return false;
    }

    if (typeof errorOrResult?.message === "string") {
      messages.push(errorOrResult.message);
    }

    if (Array.isArray(errorOrResult?.errors)) {
      for (const err of errorOrResult.errors) {
        if (typeof err?.message === "string") {
          messages.push(err.message);
        }
      }
    }

    if (Array.isArray(errorOrResult?.diagnostics)) {
      for (const diag of errorOrResult.diagnostics) {
        if (typeof diag?.message === "string") {
          messages.push(diag.message);
        }
      }
    }

    if (Array.isArray(errorOrResult?.compiler_errors)) {
      for (const err of errorOrResult.compiler_errors) {
        if (typeof err?.message === "string") {
          messages.push(err.message);
        }
      }
    }

    const formattedTerminal = errorOrResult?.formatted_errors_terminal || errorOrResult?.formattedErrorsTerminal;
    if (typeof formattedTerminal === "string") {
      messages.push(formattedTerminal);
    }

    const formattedJson = errorOrResult?.formatted_errors_json || errorOrResult?.formattedErrorsJson;
    if (typeof formattedJson === "string") {
      messages.push(formattedJson);
    }

    if (typeof errorOrResult?.toString === "function") {
      const value = String(errorOrResult);
      if (value && value !== "[object Object]") {
        messages.push(value);
      }
    }

    return messages.some((msg) =>
      msg.includes("Module discovery failed") &&
      msg.includes("Module") &&
      msg.includes("not found")
    );
  }

  private extractModuleImports(source: string): string[] {
    const imports = new Set<string>();
    const importPattern = /^\s*(?:use|import)\s+([^;\n]+)\s*;/gm;
    let match: RegExpExecArray | null;

    while ((match = importPattern.exec(source)) !== null) {
      let pathExpr = (match[1] || "").trim();
      if (!pathExpr || pathExpr.startsWith('"') || pathExpr.startsWith("'")) {
        continue;
      }

      if (pathExpr.includes(" as ")) {
        pathExpr = pathExpr.split(" as ")[0].trim();
      }

      const braceIndex = pathExpr.indexOf("::{");
      if (braceIndex >= 0) {
        pathExpr = pathExpr.slice(0, braceIndex).trim();
      }

      if (!pathExpr) {
        continue;
      }

      imports.add(pathExpr);
    }

    return [...imports];
  }

  private resolveLocalModuleFile(baseDir: string, modulePath: string): string | null {
    const relPath = modulePath.replace(/::/g, "/");
    const direct = resolve(baseDir, `${relPath}.v`);
    if (existsSync(direct)) {
      return direct;
    }
    const modFile = resolve(baseDir, relPath, "mod.v");
    if (existsSync(modFile)) {
      return modFile;
    }
    return null;
  }

  private resolveStdlibModuleFile(modulePath: string): string | null {
    if (!modulePath.startsWith("std::")) {
      return null;
    }

    const relPath = modulePath.replace(/^std::/, "").replace(/::/g, "/");
    const candidates = [
      resolve(moduleDir, "../../../assets/stdlib/std", `${relPath}.v`),
      resolve(moduleDir, "../../assets/stdlib/std", `${relPath}.v`),
      resolve(moduleDir, "../../../five-stdlib/std", `${relPath}.v`),
      resolve(process.cwd(), "five-stdlib/std", `${relPath}.v`)
    ];

    for (const candidate of candidates) {
      if (existsSync(candidate)) {
        return candidate;
      }
    }

    return null;
  }

  private async compileWithLocalModuleMap(entryPoint: string, _options: any, compilationOptions: any): Promise<any> {
    const entryPointAbs = resolve(entryPoint);
    const sourceDir = dirname(entryPointAbs);
    const mainSource = await readFile(entryPointAbs, "utf8");

    const modules: Array<{ name: string; content: string }> = [];
    const visited = new Set<string>();

    const visitModule = async (modulePath: string): Promise<void> => {
      if (visited.has(modulePath)) {
        return;
      }

      const moduleFile = modulePath.startsWith("std::")
        ? this.resolveStdlibModuleFile(modulePath)
        : this.resolveLocalModuleFile(sourceDir, modulePath);
      if (!moduleFile) {
        return;
      }

      visited.add(modulePath);
      const moduleContent = await readFile(moduleFile, "utf8");
      modules.push({
        name: `${modulePath.replace(/::/g, "/")}.v`,
        content: moduleContent,
      });

      for (const dep of this.extractModuleImports(moduleContent)) {
        await visitModule(dep);
      }
    };

    for (const dep of this.extractModuleImports(mainSource)) {
      await visitModule(dep);
    }

    const fallbackResult = this.compiler.compile_multi(mainSource, modules, compilationOptions);
    return fallbackResult;
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
