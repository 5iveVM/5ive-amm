import { CompilationContext } from "./types.js";
import { createCompilerError } from "./utils.js";

export async function validateSource(
  ctx: CompilationContext,
  sourceCode: string,
): Promise<{ valid: boolean; errors: string[]; warnings: string[] }> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const result = ctx.compiler.validate_syntax(sourceCode);
    return JSON.parse(result);
  } catch (error) {
    throw createCompilerError(
      "Syntax validation failed",
      error as Error,
    );
  }
}

export async function validateAccountConstraints(
  ctx: CompilationContext,
  sourceCode: string,
  functionName: string,
  accounts: any[],
): Promise<any> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const accountsJson = JSON.stringify(accounts);
    const validation = ctx.compiler.validate_account_constraints(
      sourceCode,
      functionName,
      accountsJson,
    );
    return JSON.parse(validation);
  } catch (error) {
    throw createCompilerError(
      "Account constraint validation failed",
      error as Error,
    );
  }
}
