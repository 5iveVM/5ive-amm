import { readFile, writeFile } from "fs/promises";
import { CompilationOptions, CompilationResult } from "../../types.js";
import { CompilationContext } from "./types.js";
import {
  createCompilerError,
  extractMetrics,
  extractAbi,
  extractFormattedErrors,
  extractDiagnostics,
} from "./utils.js";

export async function compile(
  ctx: CompilationContext,
  source: string,
  options?: any
): Promise<any> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  const startTime = Date.now();
  const metricsFormat = options?.metricsFormat || "json";

  try {
    let result: any;
    let WasmCompilationOptions = ctx.WasmCompilationOptions;

    // Use unified compilation method (module already loaded in initialize)
    if (!WasmCompilationOptions && ctx.wasmModuleRef) {
      WasmCompilationOptions = ctx.wasmModuleRef.WasmCompilationOptions;
    }

    const includeMetrics = options?.includeMetrics || Boolean(options?.metricsOutput) || Boolean(options?.metricsFormat);
    const errorFormat = options?.errorFormat || "terminal";
    const comprehensiveMetrics = options?.comprehensiveMetrics || Boolean(options?.metricsOutput);

    const compilationOptions = new WasmCompilationOptions()
      .with_mode(options?.target || "deployment")
      .with_optimization_level("production")
      .with_v2_preview(true)
      .with_constraint_cache(false)
      .with_enhanced_errors(true)
      .with_metrics(includeMetrics)
      .with_comprehensive_metrics(comprehensiveMetrics)
      .with_metrics_format(metricsFormat)
      .with_error_format(errorFormat)
      .with_source_file(options?.sourceFile || "input.v");

    result = ctx.compiler.compile(source, compilationOptions);

    if (result.compiler_errors && result.compiler_errors.length > 0) {
      ctx.logger.debug(
        `WASM returned ${result.compiler_errors.length} compiler errors`,
      );
      for (const error of result.compiler_errors) {
        ctx.logger.debug(`Compiler error details:`, error);
        try {
          const message = error.message ? error.message : "No message";
          const code = error.code ? error.code : "No code";
          const severity = error.severity ? error.severity : "Unknown";
          const category = error.category ? error.category : "Unknown";

          ctx.logger.error(
            `Detailed error: [${code}] ${severity} - ${message}`,
          );
          ctx.logger.error(`Category: ${category}`);

          if (error.location) {
            const location = error.location;
            ctx.logger.error(
              `Location: line ${location.line}, column ${location.column}`,
            );
          }
        } catch (e) {
          ctx.logger.debug(`Failed to extract error details:`, e);
          ctx.logger.debug(`Raw error object:`, error);
        }
      }
    }

    const metricsPayload = extractMetrics(result, metricsFormat);
    const formattedErrors = extractFormattedErrors(result);
    const diagnostics = extractDiagnostics(result, ctx.logger);

    if (result.success && result.bytecode) {
      return {
        success: true,
        bytecode: result.bytecode,
        abi: extractAbi(result, ctx.logger),
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
    if (
      error &&
      typeof error === "object" &&
      (error as any).code === "COMPILER_ERROR"
    ) {
      throw error;
    }

    throw createCompilerError(
      `Compilation error: ${error instanceof Error ? error.message : "Unknown error"}`,
      error as Error,
      {
        phase: "compile",
      },
    );
  }
}

export async function compileFile(
  ctx: CompilationContext,
  options: CompilationOptions
): Promise<CompilationResult> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  const startTime = Date.now();
  const metricsFormat = (options as any).metricsFormat || "json";

  try {
    const sourceCode = await readFile(options.sourceFile, "utf8");
    ctx.logger.debug(`Compiling source file: ${options.sourceFile}`);

    let result: any;
    try {
      let WasmCompilationOptions = ctx.WasmCompilationOptions;
      if (!WasmCompilationOptions && ctx.wasmModuleRef) {
        WasmCompilationOptions = ctx.wasmModuleRef.WasmCompilationOptions;
      }
      const includeMetrics =
        (options as any).includeMetrics || Boolean((options as any).metricsOutput) || Boolean((options as any).metricsFormat);
      const errorFormat = (options as any).errorFormat || "terminal";
      const comprehensiveMetrics = (options as any).comprehensiveMetrics || Boolean((options as any).metricsOutput);
      const compilationOptions = new WasmCompilationOptions()
        .with_mode(options.target || "deployment")
        .with_optimization_level("production")
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

      result = ctx.compiler.compile(sourceCode, compilationOptions);

      if (result.compiler_errors && result.compiler_errors.length > 0) {
        ctx.logger.debug(
          `WASM returned ${result.compiler_errors.length} compiler errors`,
        );
        for (const error of result.compiler_errors) {
          ctx.logger.debug(`Compiler error details:`, error);
          try {
            const message = error.message ? error.message : "No message";
            const code = error.code ? error.code : "No code";
            const severity = error.severity ? error.severity : "Unknown";
            const category = error.category ? error.category : "Unknown";
            ctx.logger.error(
              `Detailed error: [${code}] ${severity} - ${message}`,
            );
            ctx.logger.error(`Category: ${category}`);
            if (error.location) {
              const location = error.location;
              ctx.logger.error(
                `Location: line ${location.line}, column ${location.column}`,
              );
            }
          } catch (e) {
            ctx.logger.debug(`Failed to extract error details:`, e);
          }
        }
      }
    } catch (wasmError) {
      ctx.logger.error("WASM compiler threw exception:", wasmError);
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

    if (
      !result.success &&
      result.compiler_errors &&
      result.compiler_errors.length > 0
    ) {
      ctx.logger.debug(`Enhanced Error Count: ${result.error_count}`);
      try {
        const terminalOutput = result.format_all_terminal
          ? result.format_all_terminal()
          : null;
        if (terminalOutput) {
          ctx.logger.debug("Terminal formatted errors:", terminalOutput);
        }
        const jsonOutput = result.format_all_json
          ? result.format_all_json()
          : null;
        if (jsonOutput) {
          ctx.logger.debug("JSON formatted errors:", jsonOutput);
        }
      } catch (formatError) {
        ctx.logger.debug("Failed to get formatted errors:", formatError);
      }
    }

    const compilationTime = Date.now() - startTime;
    const metricsPayload = extractMetrics(result, metricsFormat);
    const formattedErrors = extractFormattedErrors(result);
    const diagnostics = extractDiagnostics(result, ctx.logger);

    const compilationResult: CompilationResult = {
      success: result.success,
      bytecode: result.bytecode ? new Uint8Array(result.bytecode) : undefined,
      abi: result.abi || undefined,
      errors: diagnostics,
      warnings: diagnostics.filter((diag) => diag.severity === "warning"),
      diagnostics,
      formattedErrorsTerminal: formattedErrors.terminal,
      formattedErrorsJson: formattedErrors.json,
      metrics: {
        compilationTime: result.compilation_time || compilationTime,
        bytecodeSize: result.bytecode_size || 0,
        memoryUsed: 0,
        optimizationTime: 0,
        instructionCount: 0,
        functionCount: 0,
      },
      metricsReport: metricsPayload,
    };

    if (options.outputFile && compilationResult.bytecode) {
      await writeFile(options.outputFile, compilationResult.bytecode);
      ctx.logger.debug(`Bytecode written to: ${options.outputFile}`);
    }

    if (options.abiOutputFile && compilationResult.abi) {
      await writeFile(
        options.abiOutputFile,
        JSON.stringify(compilationResult.abi, null, 2),
      );
      ctx.logger.debug(`ABI written to: ${options.abiOutputFile}`);
    }

    ctx.logger.debug(`Compilation completed in ${compilationTime}ms`);
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

export async function compileWithDiscovery(
  ctx: CompilationContext,
  entryPoint: string,
  options?: any
): Promise<any> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    let WasmCompilationOptions = ctx.WasmCompilationOptions;
    if (!WasmCompilationOptions && ctx.wasmModuleRef) {
      WasmCompilationOptions = ctx.wasmModuleRef.WasmCompilationOptions;
    }

    const metricsFormat = options?.metricsFormat || "json";
    const includeMetrics = options?.includeMetrics || Boolean(options?.metricsOutput);
    const errorFormat = options?.errorFormat || "terminal";
    const comprehensiveMetrics = options?.comprehensiveMetrics || false;

    const compilationOptions = new WasmCompilationOptions()
      .with_mode(options?.target || "deployment")
      .with_optimization_level("production")
      .with_v2_preview(true)
      .with_constraint_cache(false)
      .with_enhanced_errors(true)
      .with_metrics(includeMetrics)
      .with_comprehensive_metrics(comprehensiveMetrics)
      .with_metrics_format(metricsFormat)
      .with_error_format(errorFormat)
      .with_source_file(entryPoint);

    const result = ctx.compiler.compileMultiWithDiscovery(entryPoint, compilationOptions);

    const metricsPayload = extractMetrics(result, metricsFormat);
    const formattedErrors = extractFormattedErrors(result);
    const diagnostics = extractDiagnostics(result, ctx.logger);

    if (result.success && result.bytecode) {
      return {
        success: true,
        bytecode: result.bytecode,
        abi: result.abi,
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
    if (
      error &&
      typeof error === "object" &&
      (error as any).code === "COMPILER_ERROR"
    ) {
      throw error;
    }

    throw createCompilerError(
      `Compilation error: ${error instanceof Error ? error.message : "Unknown error"}`,
      error as Error,
      {
        phase: "compileWithDiscovery",
        entryPoint,
      },
    );
  }
}
