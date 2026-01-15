import { CompilationContext } from "./types.js";
import { createCompilerError } from "./utils.js";

export async function analyzeBytecode(
  ctx: CompilationContext,
  bytecode: Uint8Array
): Promise<any> {
  if (!ctx.wasmModuleRef) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const analysis = ctx.BytecodeAnalyzer.analyze_semantic(bytecode);
    return JSON.parse(analysis);
  } catch (error) {
    throw createCompilerError(
      "Bytecode analysis failed",
      error as Error,
    );
  }
}

export async function analyzeInstructionAt(
  ctx: CompilationContext,
  bytecode: Uint8Array,
  offset: number,
): Promise<any> {
  if (!ctx.wasmModuleRef) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const instruction = ctx.BytecodeAnalyzer.analyze_instruction_at(
      bytecode,
      offset,
    );
    return JSON.parse(instruction);
  } catch (error) {
    throw createCompilerError(
      "Instruction analysis failed",
      error as Error,
    );
  }
}

export async function getBytecodeStats(
  ctx: CompilationContext,
  bytecode: Uint8Array
): Promise<any> {
  if (!ctx.wasmModuleRef) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const stats = ctx.BytecodeAnalyzer.get_bytecode_summary(bytecode);
    return JSON.parse(stats);
  } catch (error) {
    throw createCompilerError("Bytecode stats failed", error as Error);
  }
}

export async function getOpcodeUsage(
  ctx: CompilationContext,
  sourceCode: string
): Promise<any> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const usage = ctx.compiler.get_opcode_usage(sourceCode);
    return JSON.parse(usage);
  } catch (error) {
    throw createCompilerError(
      "Opcode usage analysis failed",
      error as Error,
    );
  }
}

export async function getOpcodeAnalysis(
  ctx: CompilationContext,
  sourceCode: string
): Promise<any> {
  if (!ctx.compiler) {
    throw createCompilerError("Compiler not initialized");
  }

  try {
    const analysis = ctx.compiler.get_opcode_analysis(sourceCode);
    return JSON.parse(analysis);
  } catch (error) {
    throw createCompilerError(
      "Comprehensive opcode analysis failed",
      error as Error,
    );
  }
}
