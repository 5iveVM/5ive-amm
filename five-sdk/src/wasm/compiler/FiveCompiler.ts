/**
 * Five Compiler WASM Integration
 *
 * Real integration with Five VM WASM bindings for DSL compilation,
 * ABI generation, and bytecode optimization.
 */

import {
  CompilationOptions,
  CompilationResult,
  Logger,
  CLIError,
} from "../../types.js";
import { ConfigManager } from "../../config/ConfigManager.js";
import { CompilationContext } from "./types.js";
import * as CompilationLogic from "./CompilationLogic.js";
import * as AnalysisLogic from "./AnalysisLogic.js";
import * as AbiLogic from "./AbiLogic.js";
import * as ValidationLogic from "./ValidationLogic.js";
import * as OptimizationLogic from "./OptimizationLogic.js";
import * as InfoLogic from "./InfoLogic.js";
import { createCompilerError } from "./utils.js";

// Global refs to loaded modules
let wasmModuleRef: any | null = null;
let FiveCompilerWasm: any;
let FiveVMWasm: any;
let BytecodeAnalyzer: any;
let WasmCompilationOptions: any;

export class FiveCompiler {
  private compiler: any = null;
  private logger: Logger;
  private initialized = false;

  constructor(logger: Logger) {
    this.logger = logger;
  }

  /**
   * Initialize the compiler with real Five VM WASM module
   * The Node.js target (from wasm-pack --target nodejs) is pre-initialized
   */
  async initialize(): Promise<void> {
    try {
      // Try multiple candidate locations (mirrors vm.ts logic)
      const cfg = await ConfigManager.getInstance().get();
      const prefer = cfg.wasm?.loader || "auto";
      const configured = Array.isArray(cfg.wasm?.modulePaths)
        ? cfg.wasm!.modulePaths!
        : [];
      const nodeCandidates = [
        "../../five_vm_wasm.cjs",
        "../five_vm_wasm.cjs",
        "../../five_vm_wasm.js",
        "../five_vm_wasm.js",
      ];
      const bundlerCandidates = [
        "../../assets/vm/five_vm_wasm.cjs",
        "../assets/vm/five_vm_wasm.cjs",
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
          // eslint-disable-next-line no-await-in-loop
          const mod = await import(candidate as string);
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
      FiveCompilerWasm =
        wasmModule.WasmFiveCompiler || wasmModule.FiveCompilerWasm;
      FiveVMWasm = wasmModule.FiveVMWasm;
      BytecodeAnalyzer = wasmModule.BytecodeAnalyzer;
      WasmCompilationOptions = wasmModule.WasmCompilationOptions;

      // Initialize the compiler instance
      this.compiler = new FiveCompilerWasm();
      this.initialized = true;

      // WASM compiler initialized silently
    } catch (error) {
      throw createCompilerError(
        `Five VM WASM modules not found. Please run "npm run build:wasm" to build the required WebAssembly modules. Error: ${error}`,
        error as Error,
      );
    }
  }

  private getContext(): CompilationContext {
    return {
      compiler: this.compiler,
      wasmModuleRef,
      WasmCompilationOptions,
      BytecodeAnalyzer,
      logger: this.logger,
    };
  }

  // --- Compilation ---

  async compile(source: string, options?: any): Promise<any> {
    return CompilationLogic.compile(this.getContext(), source, options);
  }

  async compileFile(options: CompilationOptions): Promise<CompilationResult> {
    return CompilationLogic.compileFile(this.getContext(), options);
  }

  async compileWithDiscovery(entryPoint: string, options?: any): Promise<any> {
    return CompilationLogic.compileWithDiscovery(this.getContext(), entryPoint, options);
  }

  // --- ABI & Extraction ---

  async generateABI(sourceCode: string): Promise<any> {
    return AbiLogic.generateABI(this.getContext(), sourceCode);
  }

  async extractAccountDefinitions(sourceCode: string): Promise<any> {
    return AbiLogic.extractAccountDefinitions(this.getContext(), sourceCode);
  }

  async extractFunctionSignatures(sourceCode: string): Promise<any> {
    return AbiLogic.extractFunctionSignatures(this.getContext(), sourceCode);
  }

  // --- Validation ---

  async validateSource(sourceCode: string): Promise<{ valid: boolean; errors: string[]; warnings: string[] }> {
    return ValidationLogic.validateSource(this.getContext(), sourceCode);
  }

  async validateAccountConstraints(sourceCode: string, functionName: string, accounts: any[]): Promise<any> {
    return ValidationLogic.validateAccountConstraints(this.getContext(), sourceCode, functionName, accounts);
  }

  // --- Optimization ---

  async optimizeBytecode(bytecode: Uint8Array): Promise<Uint8Array> {
    return OptimizationLogic.optimizeBytecode(this.getContext(), bytecode);
  }

  // --- Analysis ---

  async analyzeBytecode(bytecode: Uint8Array): Promise<any> {
    return AnalysisLogic.analyzeBytecode(this.getContext(), bytecode);
  }

  async analyzeInstructionAt(bytecode: Uint8Array, offset: number): Promise<any> {
    return AnalysisLogic.analyzeInstructionAt(this.getContext(), bytecode, offset);
  }

  async getBytecodeStats(bytecode: Uint8Array): Promise<any> {
    return AnalysisLogic.getBytecodeStats(this.getContext(), bytecode);
  }

  async getOpcodeUsage(sourceCode: string): Promise<any> {
    return AnalysisLogic.getOpcodeUsage(this.getContext(), sourceCode);
  }

  async getOpcodeAnalysis(sourceCode: string): Promise<any> {
    return AnalysisLogic.getOpcodeAnalysis(this.getContext(), sourceCode);
  }

  // --- Info ---

  getCompilerInfo(): { version: string; features: string[] } {
    return InfoLogic.getCompilerInfo(this.getContext());
  }

  async discoverModules(entryPoint: string): Promise<string[]> {
    return InfoLogic.discoverModules(this.getContext(), entryPoint);
  }

  isReady(): boolean {
    return this.initialized && this.compiler !== null;
  }
}
