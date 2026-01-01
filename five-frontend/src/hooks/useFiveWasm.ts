import { useEffect, useRef, useState } from 'react';
import { loadFiveWasm } from '@/lib/five-wasm-loader';

// Define the interface for the WASM module
interface FiveWasmModule {
    WasmFiveCompiler: new () => any;
    WasmCompilationOptions: new () => any;
    FiveVMWasm: new (bytecode: Uint8Array) => any;
    ParameterEncoder: any;
}

export interface CompilationResult {
    success: boolean;
    bytecode: Uint8Array | null;
    error: string | null;
    logs: string[];
}

export interface ExecutionResult {
    success: boolean;
    logs: string[];
    computeUnits: number;
    error: string | null;
}

// Singleton compiler instance shared across all hook usages (Removed)

export function useFiveWasm() {
    const [isReady, setIsReady] = useState(false);
    const [isLoading, setIsLoading] = useState(true);
    const wasmModuleRef = useRef<FiveWasmModule | null>(null);

    useEffect(() => {
        const loadWasm = async () => {
            try {
                const wasm = await loadFiveWasm();

                if (typeof wasm.default === 'function') {
                    try {
                        await wasm.default();
                    } catch (initErr) {
                        console.warn("useFiveWasm: Init failed or already initialized:", initErr);
                    }
                }

                wasmModuleRef.current = wasm;
                setIsReady(true);
            } catch (err) {
                console.error('Failed to load 5IVE WASM:', err);
            } finally {
                setIsLoading(false);
            }
        };

        loadWasm();
    }, []);

    const compile = async (code: string): Promise<CompilationResult> => {
        if (!wasmModuleRef.current) {
            console.error("useFiveWasm: Module not loaded");
            return { success: false, bytecode: null, error: "WASM module not loaded", logs: [] };
        }

        let compiler;
        let options;

        try {
            // Instantiate fresh compiler and options for every request to avoid memory corruption
            compiler = new wasmModuleRef.current.WasmFiveCompiler();
            options = new wasmModuleRef.current.WasmCompilationOptions();

            const result = compiler.compile(code, options);

            if (result.success) {
                const bytes = result.bytecode || (typeof result.get_bytecode === 'function' ? result.get_bytecode() : null);
                // Important: Copy the bytes because the original memory might be freed
                const safeBytes = bytes ? new Uint8Array(bytes) : null;

                return {
                    success: true,
                    bytecode: safeBytes,
                    error: null,
                    logs: ["Compilation successful"]
                };
            } else {
                let errorMsg = "Unknown error";
                if (result.errors && result.errors.length > 0) {
                    errorMsg = Array.from(result.errors).join('\n');
                } else if (result.error_message) {
                    errorMsg = result.error_message;
                }
                return {
                    success: false,
                    bytecode: null,
                    error: errorMsg,
                    logs: []
                };
            }
        } catch (e: any) {
            return {
                success: false,
                bytecode: null,
                error: e.toString(),
                logs: []
            };
        } finally {
            // ALWAYS free WASM objects to prevent leaks and memory corruption
            if (compiler && compiler.free) compiler.free();
            if (options && options.free) options.free();
        }
    };

    const execute = async (bytecode: Uint8Array, functionIndex: number = 0, params: any[] = []): Promise<ExecutionResult> => {
        if (!wasmModuleRef.current) {
            return { success: false, logs: [], computeUnits: 0, error: "VM not ready" };
        }

        try {
            const vm = new wasmModuleRef.current.FiveVMWasm(bytecode);

            // Simple execution payload for now (calling init or 0-index function)
            // For docs examples, we mostly just want to init or run the first function.
            // Construct payload: [Discriminator(9)] + [VLE(Index)] + [EncodedParams]

            // For simplicity in docs, let's assume no params or handle them simply if needed later.
            // Using 0 as default function index (usually 'main' or first defined)

            // VLE encoding helper for index
            const encodeVLE = (value: number) => {
                const bytes: number[] = [];
                do {
                    let byte = value & 0x7f;
                    value >>>= 7;
                    if (value !== 0) byte |= 0x80;
                    bytes.push(byte);
                } while (value !== 0);
                return new Uint8Array(bytes);
            };

            const discriminator = new Uint8Array([9]);
            const indexBytes = encodeVLE(functionIndex);

            // Empty params for now
            const encodedParams = new Uint8Array([]);

            const payload = new Uint8Array(discriminator.length + indexBytes.length + encodedParams.length);
            payload.set(discriminator, 0);
            payload.set(indexBytes, discriminator.length);

            const result = await vm.execute_partial(payload, []);

            return {
                success: !result.error_message,
                logs: [], // We could extract logs if VM returns them
                computeUnits: result.compute_units_used || 0,
                error: result.error_message || null
            };

        } catch (e: any) {
            return {
                success: false,
                logs: [],
                computeUnits: 0,
                error: e.toString()
            };
        }
    };

    return {
        isReady,
        isLoading,
        compile,
        execute
    };
}
