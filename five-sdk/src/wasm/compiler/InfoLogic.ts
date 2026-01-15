import { CompilationContext } from "./types.js";
import { createCompilerError } from "./utils.js";

export function getCompilerInfo(
  ctx: CompilationContext
): { version: string; features: string[] } {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const info = ctx.compiler.get_compiler_stats();
    return JSON.parse(info);
  } catch (error) {
    throw createCompilerError(
      "Failed to get compiler info",
      error as Error,
    );
  }
}

export async function discoverModules(
  ctx: CompilationContext,
  entryPoint: string
): Promise<string[]> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    return ctx.compiler.discoverModules(entryPoint);
  } catch (error) {
    throw createCompilerError(
      `Module discovery failed: ${error instanceof Error ? error.message : String(error)}`,
      error as Error
    );
  }
}
