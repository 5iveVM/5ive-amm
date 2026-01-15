import { CompilationContext } from "./types.js";
import { createCompilerError } from "./utils.js";

export async function optimizeBytecode(
  ctx: CompilationContext,
  bytecode: Uint8Array
): Promise<Uint8Array> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const optimized = ctx.compiler.optimize_bytecode(bytecode);
    return new Uint8Array(optimized);
  } catch (error) {
    throw createCompilerError(
      "Bytecode optimization failed",
      error as Error,
    );
  }
}
