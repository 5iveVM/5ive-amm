import { WasmCompilerService } from '../app/wasm-compiler';
import { readFileSync, existsSync } from 'fs';
import { resolve, join } from 'path';

describe('Opinion Market Verification Tests', () => {
    let wasmService: WasmCompilerService;
    let bytecode: Uint8Array;

    beforeAll(async () => {
        wasmService = new WasmCompilerService();
        await wasmService.initialize();

        // Path to the compiled bytecode
        // Assuming we run from five-wasm directory
        const bytecodePath = resolve(__dirname, '../../five-opinion/tests/test_lmsr_math.fbin');

        if (!existsSync(bytecodePath)) {
            throw new Error(`Bytecode file not found at: ${bytecodePath}`);
        }

        const buffer = readFileSync(bytecodePath);
        bytecode = new Uint8Array(buffer);
    });

    test('should execute test_lmsr_math.v successfully', async () => {
        // Execute with empty input data (defaults to function index 0 if script expects it, 
        // or execution from start if standard script)
        // Since we compiled with `compile-multi`, the entry point is test_lmsr_math
        // which was renamed to run_tests and is at index 0.
        // Input data 00 indicates function index 0.
        const inputData = new Uint8Array([0x00]);

        const result = await wasmService.testBytecodeExecution(
            bytecode,
            inputData
        );

        if (result.outcome !== 'completed') {
            console.error('Execution failed:', result.error_details);
            console.error('Stopped at:', result.stopped_at_operation);
        }

        expect(result.outcome).toBe('completed');
        expect(result.test_success).toBe(true);
        expect(result.final_state.has_result).toBe(true); // Void function usually pushes output or just returns
    });
});
