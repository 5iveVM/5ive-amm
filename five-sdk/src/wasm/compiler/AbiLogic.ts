import { CompilationContext } from "./types.js";
import { createCompilerError } from "./utils.js";

export async function generateABI(
  ctx: CompilationContext,
  sourceCode: string
): Promise<any> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const abi = ctx.compiler.generate_abi(sourceCode);
    return JSON.parse(abi);
  } catch (error) {
    throw createCompilerError("ABI generation failed", error as Error);
  }
}

export async function extractAccountDefinitions(
  ctx: CompilationContext,
  sourceCode: string
): Promise<any> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const definitions = ctx.compiler.extract_account_definitions(sourceCode);
    return JSON.parse(definitions);
  } catch (error) {
    throw createCompilerError(
      "Account definition extraction failed",
      error as Error,
    );
  }
}

export async function extractFunctionSignatures(
  ctx: CompilationContext,
  sourceCode: string
): Promise<any> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const signatures = ctx.compiler.extract_function_signatures(sourceCode);
    return JSON.parse(signatures);
  } catch (error) {
    throw createCompilerError(
      "Function signature extraction failed",
      error as Error,
    );
  }
}
