import { CLIError } from "../../types.js";

export function createCompilerError(message: string, cause?: Error): CLIError {
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

export function extractMetrics(
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
  } catch {
    // Ignore metrics errors
  }

  return {
    format,
    exported,
    detailed,
  };
}

export function extractAbi(result: any, logger: any): any | undefined {
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
        logger.debug("Failed to parse ABI from get_abi():", error);
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
        logger.debug("Failed to parse ABI from result.abi string:", error);
        return undefined;
      }
    }

    return directAbi;
}
