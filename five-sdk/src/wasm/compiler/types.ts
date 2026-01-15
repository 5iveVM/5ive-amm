import { Logger } from "../../types.js";

export interface CompilationContext {
  compiler: any;
  wasmModuleRef: any;
  WasmCompilationOptions: any;
  BytecodeAnalyzer: any; // Added this
  logger: Logger;
}
