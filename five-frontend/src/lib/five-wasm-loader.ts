import { useEffect, useState } from 'react';

// Type definition for the WASM module
type FiveWasmModule = typeof import('five-vm-wasm');

let wasmModule: FiveWasmModule | null = null;

export const loadFiveWasm = async (): Promise<FiveWasmModule> => {
    if (wasmModule) return wasmModule;

    try {
        // Dynamically import the WASM module
        // This expects the bundler to handle the WASM loading or the file-loader to be configured
        // Since we are using Next.js with Webpack, we might need async import
        const module = await import('five-vm-wasm');

        // Some initialization might be required depending on how wasm-pack built it
        // Usually standard import triggers init in newer setups

        wasmModule = module;
        return module;
    } catch (error) {
        console.error("Failed to load five-vm-wasm:", error);
        throw error;
    }
};

export const useFiveWasm = () => {
    const [wasm, setWasm] = useState<FiveWasmModule | null>(null);
    const [error, setError] = useState<Error | null>(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        loadFiveWasm()
            .then(setWasm)
            .catch(setError)
            .finally(() => setLoading(false));
    }, []);

    return { wasm, loading, error };
};
