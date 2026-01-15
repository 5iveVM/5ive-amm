import { FiveSDKError } from "../types.js";

let wasmVMInstance: any = null;

export async function loadWasmVM(): Promise<any> {
  if (wasmVMInstance) {
    return wasmVMInstance;
  }

  try {
    // Import existing WASM VM from five-cli infrastructure
    const { FiveVM } = await import('./vm.js');

    // Create a simple logger for WASM VM
    const logger = {
      debug: (msg: string) => console.debug("[WASM VM]", msg),
      info: (msg: string) => console.info("[WASM VM]", msg),
      warn: (msg: string) => console.warn("[WASM VM]", msg),
      error: (msg: string) => console.error("[WASM VM]", msg),
    };
    wasmVMInstance = new FiveVM(logger); // Initialize WASM VM
    if (wasmVMInstance.initialize) {
      await wasmVMInstance.initialize();
    }

    return wasmVMInstance;
  } catch (error) {
    throw new FiveSDKError(
      `Failed to load WASM VM: ${error instanceof Error ? error.message : "Unknown error"}`,
      "WASM_LOAD_ERROR",
    );
  }
}
