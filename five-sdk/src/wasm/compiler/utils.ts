import { CLIError } from "../../types.js";

export interface NormalizedCompilationSuggestion {
  message: string;
  explanation?: string;
  confidence?: number;
  codeSuggestion?: string;
}

export interface NormalizedCompilationDiagnostic {
  type: string;
  code: string;
  severity: string;
  category: string;
  message: string;
  description?: string;
  line?: number;
  column?: number;
  sourceLocation?: string;
  location?: any;
  suggestion?: string;
  suggestions?: NormalizedCompilationSuggestion[];
  sourceLine?: string;
  sourceSnippet?: string;
  rendered?: string;
  raw?: any;
}

export interface FormattedErrors {
  terminal?: string;
  json?: string;
}

const COMPILER_ERROR_CODE = "COMPILER_ERROR";

function isObject(value: unknown): value is Record<string, any> {
  return typeof value === "object" && value !== null;
}

function toOptionalString(value: unknown): string | undefined {
  return typeof value === "string" && value.trim().length > 0
    ? value
    : undefined;
}

function toOptionalNumber(value: unknown): number | undefined {
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

function readStringMethod(target: any, methodName: string): string | undefined {
  if (!target || typeof target !== "object") {
    return undefined;
  }

  const maybeMethod = (target as any)[methodName];
  if (typeof maybeMethod !== "function") {
    return undefined;
  }

  try {
    const value = maybeMethod.call(target);
    return toOptionalString(value);
  } catch {
    return undefined;
  }
}

function readStringProperty(target: any, propName: string): string | undefined {
  if (!target || typeof target !== "object") {
    return undefined;
  }

  return toOptionalString((target as any)[propName]);
}

function normalizeSuggestion(
  suggestion: any,
): NormalizedCompilationSuggestion | undefined {
  if (typeof suggestion === "string") {
    return { message: suggestion };
  }

  if (!isObject(suggestion)) {
    return undefined;
  }

  const message =
    toOptionalString(suggestion.message) ??
    toOptionalString(suggestion.explanation);
  if (!message) {
    return undefined;
  }

  return {
    message,
    explanation: toOptionalString(suggestion.explanation),
    confidence: toOptionalNumber(suggestion.confidence),
    codeSuggestion:
      toOptionalString(suggestion.code_suggestion) ??
      toOptionalString(suggestion.codeSuggestion),
  };
}

function pickSuggestions(
  rawSuggestions: unknown,
): NormalizedCompilationSuggestion[] | undefined {
  if (!Array.isArray(rawSuggestions)) {
    return undefined;
  }

  const normalized = rawSuggestions
    .map((item) => normalizeSuggestion(item))
    .filter((item): item is NormalizedCompilationSuggestion => Boolean(item));

  return normalized.length > 0 ? normalized : undefined;
}

export function createCompilerError(
  message: string,
  cause?: Error,
  details?: Record<string, unknown>,
): CLIError {
  if (
    cause &&
    isObject(cause) &&
    (cause as any).code === COMPILER_ERROR_CODE
  ) {
    return cause as CLIError;
  }

  const error = new Error(message) as CLIError;
  error.name = "CompilerError";
  error.code = COMPILER_ERROR_CODE;
  error.category = "wasm";
  error.exitCode = 1;

  if (cause || details) {
    const inheritedDetails =
      cause && isObject((cause as any).details)
        ? ((cause as any).details as Record<string, unknown>)
        : undefined;

    error.details = {
      ...(inheritedDetails || {}),
      ...(details || {}),
      ...(cause
        ? {
            cause: cause.message,
            stack: cause.stack,
          }
        : {}),
    };
  }

  return error;
}

export function extractFormattedErrors(result: any): FormattedErrors {
  const terminal =
    readStringMethod(result, "get_formatted_errors_terminal") ||
    readStringProperty(result, "formattedErrorsTerminal") ||
    readStringProperty(result, "formatted_errors_terminal") ||
    readStringMethod(result, "format_all_terminal");

  const json =
    readStringMethod(result, "format_all_json") ||
    readStringMethod(result, "get_formatted_errors_json") ||
    readStringProperty(result, "formattedErrorsJson") ||
    readStringProperty(result, "formatted_errors_json");

  return { terminal, json };
}

export function normalizeDiagnostic(error: any): NormalizedCompilationDiagnostic {
  let parsedJson: Record<string, any> | undefined;
  if (error && typeof error?.format_json === "function") {
    try {
      const raw = error.format_json();
      const parsed = JSON.parse(raw);
      if (isObject(parsed)) {
        parsedJson = parsed;
      }
    } catch {
      // Ignore formatter/parsing failures and continue with direct field access.
    }
  }

  const merged = {
    ...(parsedJson || {}),
    ...(isObject(error) ? error : {}),
  };

  const location = isObject(merged.location) ? merged.location : undefined;
  const line = toOptionalNumber(merged.line) ?? toOptionalNumber(location?.line);
  const column =
    toOptionalNumber(merged.column) ?? toOptionalNumber(location?.column);
  const file =
    toOptionalString(merged.sourceLocation) ??
    toOptionalString(location?.file);
  const suggestionList = pickSuggestions(merged.suggestions);
  const rendered = toOptionalString(merged.rendered) ||
    (typeof merged.format_terminal === "function"
      ? toOptionalString(merged.format_terminal())
      : undefined);

  return {
    type: toOptionalString(merged.type) || "enhanced",
    code: toOptionalString(merged.code) || "E0000",
    severity: toOptionalString(merged.severity) || "error",
    category: toOptionalString(merged.category) || "compilation",
    message:
      toOptionalString(merged.message) ||
      toOptionalString(merged.description) ||
      "Unknown compiler error",
    description: toOptionalString(merged.description),
    line,
    column,
    sourceLocation: file,
    location,
    suggestion: suggestionList?.[0]?.message,
    suggestions: suggestionList,
    sourceLine:
      toOptionalString(merged.source_line) ||
      toOptionalString(merged.sourceLine),
    sourceSnippet:
      toOptionalString(merged.source_snippet) ||
      toOptionalString(merged.sourceSnippet),
    rendered,
    raw: error,
  };
}

function parseDiagnosticsJson(
  jsonPayload: string,
  logger?: { debug?: (...args: any[]) => void },
): NormalizedCompilationDiagnostic[] {
  try {
    const parsed = JSON.parse(jsonPayload);
    const arr = Array.isArray(parsed)
      ? parsed
      : Array.isArray((parsed as any)?.errors)
        ? (parsed as any).errors
        : [];

    return arr.map((item) => normalizeDiagnostic(item));
  } catch (error) {
    logger?.debug?.("[Compilation] Failed to parse formatted JSON diagnostics", error);
    return [];
  }
}

export function extractDiagnostics(
  result: any,
  logger?: { debug?: (...args: any[]) => void },
): NormalizedCompilationDiagnostic[] {
  const formatted = extractFormattedErrors(result);

  if (formatted.json) {
    const parsed = parseDiagnosticsJson(formatted.json, logger);
    if (parsed.length > 0) {
      return parsed;
    }
  }

  if (Array.isArray(result?.compiler_errors)) {
    const fromCompiler = result.compiler_errors.map((item: any) =>
      normalizeDiagnostic(item),
    );
    if (fromCompiler.length > 0) {
      return fromCompiler;
    }
  }

  if (Array.isArray(result?.errors)) {
    const fromErrors = result.errors.map((item: any) =>
      normalizeDiagnostic(item),
    );
    if (fromErrors.length > 0) {
      return fromErrors;
    }
  }

  return [];
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
