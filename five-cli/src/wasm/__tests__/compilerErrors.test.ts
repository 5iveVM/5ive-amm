import { describe, expect, it, jest, beforeEach } from '@jest/globals';
import { FiveCompilerWasm } from '../compiler';

// 1. Mock the ConfigManager to force loading our virtual module
jest.mock('../../config/ConfigManager', () => ({
    ConfigManager: {
        getInstance: () => ({
            get: jest.fn().mockResolvedValue({
                wasm: {
                    loader: 'custom',
                    modulePaths: ['virtual_five_wasm']
                }
            } as any)
        })
    }
}));

// 2. Define mock WASM objects
const mockWasmCompilerInstance = {
    compile: jest.fn(),
    free: jest.fn(),
    compile_multi: jest.fn(),
    discoverModules: jest.fn(),
    extractFunctionMetadata: jest.fn()
};

const mockWasmMetricsCollectorInstance = {
    start_phase: jest.fn(),
    end_phase: jest.fn(),
    finalize: jest.fn(),
    reset: jest.fn(),
    get_metrics_object: jest.fn().mockReturnValue({})
};


// 3. Mock the virtual WASM module
const mockCompilationOptions = {
    with_mode: jest.fn().mockReturnThis(),
    with_optimization_level: jest.fn().mockReturnThis(),
    with_v2_preview: jest.fn().mockReturnThis(),
    with_constraint_cache: jest.fn().mockReturnThis(),
    with_enhanced_errors: jest.fn().mockReturnThis(),
    with_metrics: jest.fn().mockReturnThis(),
    with_comprehensive_metrics: jest.fn().mockReturnThis(),
    with_metrics_format: jest.fn().mockReturnThis(),
    with_error_format: jest.fn().mockReturnThis(),
    free: jest.fn()
};

jest.mock('virtual_five_wasm', () => {
    return {
        __esModule: true,
        WasmFiveCompiler: jest.fn(() => mockWasmCompilerInstance),
        WasmMetricsCollector: jest.fn(() => mockWasmMetricsCollectorInstance),
        WasmCompilationOptions: jest.fn(() => mockCompilationOptions),
        WasmCompilationResult: jest.fn(),
        FiveVMWasm: jest.fn(),
        BytecodeAnalyzer: jest.fn(),
        default: {}
    };
}, { virtual: true });

describe('FiveCompilerWasm Error Handling', () => {
    let compiler: FiveCompilerWasm;
    const mockLogger = {
        debug: jest.fn(),
        info: jest.fn(),
        warn: jest.fn(),
        error: jest.fn()
    };

    beforeEach(async () => {
        jest.clearAllMocks();
        compiler = new FiveCompilerWasm(mockLogger as any);
        await compiler.initialize();
    });

    const mockWasmCompilationResult = {
        success: false,
        bytecode: undefined,
        abi: undefined,
        metadata: undefined,
        compiler_errors: [],
        metrics: {},
        formatted_errors_terminal: undefined,
        formatted_errors_json: undefined,
        free: jest.fn(),
        get_metrics_object: jest.fn().mockReturnValue({})
    };

    it('propagates rich terminal errors when available', async () => {
        // Setup mock return with rich errors
        const richErrorResult = {
            ...mockWasmCompilationResult,
            success: false,
            formatted_errors_terminal: ' [31merror[E0001] [0m: test error\n  --> test.v:1:1',
            formatted_errors_json: '[{"code":"E0001"}]',
            compiler_errors: [{ message: 'raw error' }],
            free: jest.fn()
        };

        mockWasmCompilerInstance.compile.mockReturnValue(richErrorResult);

        const result = await compiler.compile('resource test {}');

        // Verify rich errors are present in result
        expect(result.success).toBe(false);
        expect(result.formattedErrorsTerminal).toBe(' [31merror[E0001] [0m: test error\n  --> test.v:1:1');
        expect(result.formattedErrorsJson).toBe('[{"code":"E0001"}]');

        // Verify legacy errors are still present
        expect(result.errors).toHaveLength(1);
    });

    it('falls back to legacy errors when rich errors are missing', async () => {
        // Setup mock return WITHOUT rich errors
        const legacyErrorResult = {
            ...mockWasmCompilationResult,
            success: false,
            formatted_errors_terminal: undefined,
            compiler_errors: [{
                code: 'E0001',
                message: 'legacy error',
                severity: 'error'
            }],
            free: jest.fn()
        };

        mockWasmCompilerInstance.compile.mockReturnValue(legacyErrorResult);

        const result = await compiler.compile('resource test {}');

        // Verify rich errors are undefined
        expect(result.success).toBe(false);
        expect(result.formattedErrorsTerminal).toBeUndefined();

        // Verify legacy errors are present
        expect(result.errors).toHaveLength(1);
        expect(result.errors[0].message).toBe('legacy error');
    });

    it('includes rich errors even on partial success (if warnings exist)', async () => {
        // Setup mock return with success BUT also warnings/errors
        const warningResult = {
            ...mockWasmCompilationResult,
            success: true,
            bytecode: new Uint8Array([1, 2, 3]),
            formatted_errors_terminal: 'warning: unused variable',
            compiler_errors: [],
            free: jest.fn()
        };

        mockWasmCompilerInstance.compile.mockReturnValue(warningResult);

        const result = await compiler.compile('resource test {}');

        expect(result.success).toBe(true);
        // Rich output should be preserved for warnings
        expect(result.formattedErrorsTerminal).toBe('warning: unused variable');
        expect(result.bytecode).toBeDefined();
    });
});
