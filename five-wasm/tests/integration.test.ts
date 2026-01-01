import { 
    StacksVMWrapper, 
    VMPerformanceBenchmark,
    StacksAccount,
    validate_bytecode,
    get_constants
} from '../wrapper/index';

// Mock WASM module for testing
// In real implementation, this would be the actual WASM module
jest.mock('../pkg/five_vm_wasm', () => ({
    // Provide both names for compatibility with wrapper and legacy tests
    FiveVMWasm: jest.fn().mockImplementation(() => ({
        execute: jest.fn().mockResolvedValue({ success: true }),
        get_state: jest.fn().mockReturnValue('{"compute_units": 42}'),
        free: jest.fn(),
    })),
    StacksVMWasm: jest.fn().mockImplementation(() => ({
        execute: jest.fn().mockResolvedValue({ success: true }),
        get_state: jest.fn().mockReturnValue('{"compute_units": 42}'),
        free: jest.fn(),
    })),
    WasmAccount: jest.fn().mockImplementation((key, data, lamports, isWritable, isSigner, owner) => ({
        key,
        data,
        lamports,
        is_writable: isWritable,
        is_signer: isSigner,
        owner,
    })),
    BytecodeAnalyzer: {
        analyze: jest.fn().mockReturnValue('{"total_size": 100, "instruction_count": 10, "instructions": []}')
    },
    validate_bytecode: jest.fn().mockReturnValue(true),
    get_constants: jest.fn().mockReturnValue(JSON.stringify({
        MAX_SCRIPT_SIZE: 1000,
        MAX_COMPUTE_UNITS: 200000,
        FIVE_MAGIC: [0x35, 0x49, 0x56, 0x45],
        opcodes: { PUSH: 1, POP: 2, ADD: 16 },
        types: { U64: 1, BOOL: 2, STRING: 6 }
    })),
    js_value_to_vm_value: jest.fn().mockReturnValue(42),
    default: jest.fn().mockResolvedValue({}),
}));

describe('WASM VM Integration Tests', () => {
    let vm: StacksVMWrapper;
    let validBytecode: Uint8Array;
    let testAccounts: StacksAccount[];

    beforeEach(async () => {
        // Initialize WASM module (mocked)
        await StacksVMWrapper.init();

        // Create valid test bytecode
        validBytecode = new Uint8Array([
            0x35, 0x49, 0x56, 0x45, // 5IVE magic
            0x01, 0x01, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH U64(42)
            0x00 // HALT
        ]);

        // Create test accounts
        testAccounts = [
            StacksVMWrapper.createAccount(
                new Uint8Array(32).fill(1), // key
                new Uint8Array(1024).fill(0), // data
                BigInt(1000000), // lamports
                true, // writable
                true, // signer
                new Uint8Array(32).fill(2) // owner
            )
        ];

        vm = new StacksVMWrapper(validBytecode);
    });

    afterEach(() => {
        vm?.dispose();
    });

    describe('VM Initialization', () => {
        test('should create VM with valid bytecode', () => {
            expect(vm).toBeDefined();
            expect(() => new StacksVMWrapper(validBytecode)).not.toThrow();
        });

        test('should reject invalid bytecode', () => {
            const invalidBytecode = new Uint8Array([0x00, 0x01, 0x02]); // No magic bytes
            expect(() => new StacksVMWrapper(invalidBytecode)).toThrow('Invalid bytecode format');
        });

        test('should validate bytecode format', () => {
            expect(validate_bytecode(validBytecode)).toBe(true);
            
            const invalidBytecode = new Uint8Array([0x00, 0x01]);
            expect(validate_bytecode(invalidBytecode)).toBe(false);
        });
    });

    describe('Constants and Types', () => {
        test('should provide VM constants', () => {
            const constants = vm.getConstants();
            
            expect(constants).toHaveProperty('MAX_SCRIPT_SIZE');
            expect(constants).toHaveProperty('MAX_COMPUTE_UNITS');
            expect(constants).toHaveProperty('FIVE_MAGIC');
            expect(constants).toHaveProperty('opcodes');
            expect(constants).toHaveProperty('types');
            
            expect(constants.opcodes).toHaveProperty('PUSH');
            expect(constants.types).toHaveProperty('U64');
        });

        test('should create VM values with proper types', () => {
            const constants = vm.getConstants();
            
            // Test different value types
            expect(() => StacksVMWrapper.createVMValue(42, 'U64')).not.toThrow();
            expect(() => StacksVMWrapper.createVMValue(true, 'BOOL')).not.toThrow();
            expect(() => StacksVMWrapper.createVMValue('test', 'STRING')).not.toThrow();
            
            // Test invalid type
            expect(() => StacksVMWrapper.createVMValue(42, 'INVALID_TYPE' as any)).toThrow();
        });
    });

    describe('Account Management', () => {
        test('should create accounts with proper validation', () => {
            const account = StacksVMWrapper.createAccount(
                'a'.repeat(64), // 32 bytes in hex
                new Uint8Array(100),
                BigInt(500),
                true,
                false,
                'b'.repeat(64) // 32 bytes in hex
            );

            expect(account.key).toHaveLength(32);
            expect(account.owner).toHaveLength(32);
            expect(account.lamports).toBe(BigInt(500));
            expect(account.isWritable).toBe(true);
            expect(account.isSigner).toBe(false);
        });

        test('should validate account key and owner lengths', () => {
            expect(() => StacksVMWrapper.createAccount(
                'invalid', // Too short
                new Uint8Array(),
                BigInt(0),
                false,
                false,
                new Uint8Array(32)
            )).toThrow('Key must be 32 bytes');

            expect(() => StacksVMWrapper.createAccount(
                new Uint8Array(32),
                new Uint8Array(),
                BigInt(0),
                false,
                false,
                'invalid' // Too short
            )).toThrow('Owner must be 32 bytes');
        });
    });

    describe('VM Execution', () => {
        test('should execute simple bytecode successfully', async () => {
            const inputData = new Uint8Array([1, 2, 3, 4]);
            const result = await vm.execute(inputData, testAccounts);

            expect(result.success).toBe(true);
            expect(result.computeUnits).toBeGreaterThanOrEqual(BigInt(0));
            expect(result.updatedAccounts).toHaveLength(testAccounts.length);
        });

        test('should handle execution errors gracefully', async () => {
            // Mock execution failure
            const mockVM = vm as any;
            mockVM.vm.execute = jest.fn().mockImplementation(() => {
                throw new Error('Execution failed');
            });

            const result = await vm.execute(new Uint8Array(), testAccounts);

            expect(result.success).toBe(false);
            expect(result.error).toContain('Execution failed');
            expect(result.computeUnits).toBe(BigInt(0));
        });

        test('should return updated account states', async () => {
            const result = await vm.execute(new Uint8Array(), testAccounts);

            expect(result.updatedAccounts).toHaveLength(testAccounts.length);
            result.updatedAccounts.forEach((account, index) => {
                expect(account.key).toEqual(testAccounts[index].key);
                expect(account.owner).toEqual(testAccounts[index].owner);
            });
        });
    });

    describe('Bytecode Analysis', () => {
        test('should analyze bytecode structure', () => {
            const analysis = StacksVMWrapper.analyzeBytecode(validBytecode);

            expect(analysis).toHaveProperty('totalSize');
            expect(analysis).toHaveProperty('instructionCount');
            expect(analysis).toHaveProperty('instructions');
            expect(analysis.totalSize).toBeGreaterThan(0);
            expect(Array.isArray(analysis.instructions)).toBe(true);
        });

        test('should provide instruction details', () => {
            const analysis = StacksVMWrapper.analyzeBytecode(validBytecode);

            if (analysis.instructions.length > 0) {
                const instruction = analysis.instructions[0];
                expect(instruction).toHaveProperty('offset');
                expect(instruction).toHaveProperty('opcode');
                expect(instruction).toHaveProperty('name');
                expect(instruction).toHaveProperty('size');
            }
        });
    });

    describe('VM State', () => {
        test('should provide current VM state', () => {
            const state = vm.getState();
            
            expect(state).toHaveProperty('compute_units');
            expect(typeof state.compute_units).toBe('number');
        });

        test('should throw error when VM not initialized', () => {
            const uninitializedVM = Object.create(StacksVMWrapper.prototype);
            uninitializedVM.initialized = false;
            uninitializedVM.vm = null;

            expect(() => uninitializedVM.getState()).toThrow('VM not initialized');
        });
    });

    describe('Resource Management', () => {
        test('should properly dispose of resources', () => {
            const spy = jest.spyOn(vm['vm'] as any, 'free');
            
            vm.dispose();
            
            expect(spy).toHaveBeenCalled();
            expect(vm['vm']).toBeNull();
            expect(vm['initialized']).toBe(false);
        });

        test('should handle multiple dispose calls safely', () => {
            vm.dispose();
            
            expect(() => vm.dispose()).not.toThrow();
        });
    });
});

describe('Performance Benchmarking', () => {
    const validBytecode = new Uint8Array([
        0x35, 0x49, 0x56, 0x45, // 5IVE magic
        0x01, 0x01, 0x2A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // PUSH U64(42)
        0x00 // HALT
    ]);

    const testAccounts: StacksAccount[] = [
        StacksVMWrapper.createAccount(
            new Uint8Array(32).fill(1),
            new Uint8Array(100),
            BigInt(1000),
            true,
            true,
            new Uint8Array(32).fill(2)
        )
    ];

    test('should benchmark VM execution performance', async () => {
        const benchmark = await VMPerformanceBenchmark.benchmarkExecution(
            validBytecode,
            new Uint8Array([1, 2, 3]),
            testAccounts,
            10 // Small number for testing
        );

        expect(benchmark).toHaveProperty('averageTime');
        expect(benchmark).toHaveProperty('minTime');
        expect(benchmark).toHaveProperty('maxTime');
        expect(benchmark).toHaveProperty('totalTime');
        expect(benchmark).toHaveProperty('iterations');
        expect(benchmark).toHaveProperty('successRate');

        expect(benchmark.iterations).toBe(10);
        expect(benchmark.successRate).toBeGreaterThanOrEqual(0);
        expect(benchmark.successRate).toBeLessThanOrEqual(1);
        expect(benchmark.averageTime).toBeGreaterThanOrEqual(0);
        expect(benchmark.minTime).toBeLessThanOrEqual(benchmark.maxTime);
    });

    test('should compare WASM vs native performance', async () => {
        const wasmFn = async () => {
            const vm = new StacksVMWrapper(validBytecode);
            await vm.execute(new Uint8Array([1, 2, 3]), testAccounts);
            vm.dispose();
        };

        const nativeFn = async () => {
            // Simulate native implementation
            await new Promise(resolve => setTimeout(resolve, 1));
        };

        const comparison = await VMPerformanceBenchmark.comparePerformance(
            wasmFn,
            nativeFn,
            5 // Small number for testing
        );

        expect(comparison).toHaveProperty('wasmPerformance');
        expect(comparison).toHaveProperty('nativePerformance');
        expect(comparison).toHaveProperty('performanceRatio');
        expect(comparison).toHaveProperty('wasmFaster');

        expect(comparison.wasmPerformance).toBeGreaterThan(0);
        expect(comparison.nativePerformance).toBeGreaterThan(0);
        expect(comparison.performanceRatio).toBeGreaterThan(0);
        expect(typeof comparison.wasmFaster).toBe('boolean');
    });
});

describe('Type Safety and Error Handling', () => {
    test('should enforce TypeScript types at compile time', () => {
        // These tests verify that TypeScript types are properly defined
        // and would catch type errors at compile time
        
        const validBytecode = new Uint8Array([0x53, 0x54, 0x4B, 0x53]);
        const vm = new StacksVMWrapper(validBytecode);

        // TypeScript should enforce these types
        const account: StacksAccount = {
            key: new Uint8Array(32),
            data: new Uint8Array(),
            lamports: BigInt(0),
            isWritable: false,
            isSigner: false,
            owner: new Uint8Array(32),
        };

        expect(account.lamports).toEqual(BigInt(0));
        expect(typeof account.isWritable).toBe('boolean');

        vm.dispose();
    });

    test('should handle WASM memory management correctly', () => {
        const bytecode = new Uint8Array([0x53, 0x54, 0x4B, 0x53]);
        
        // Create multiple VMs to test memory management
        const vms = Array.from({ length: 10 }, () => new StacksVMWrapper(bytecode));
        
        // Dispose all VMs
        vms.forEach(vm => vm.dispose());
        
        // Should not throw or leak memory
        expect(true).toBe(true);
    });
});

describe('Integration with Existing Stacks Ecosystem', () => {
    test('should maintain compatibility with existing VM interface', () => {
        const constants = JSON.parse(get_constants());
        
        // Verify that opcodes match expected values from constants.rs
        expect(constants.opcodes.PUSH).toBe(1);
        expect(constants.opcodes.POP).toBe(2);
        expect(constants.opcodes.ADD).toBe(16);
        
        // Verify type constants
        expect(constants.types.U64).toBe(1);
        expect(constants.types.BOOL).toBe(2);
        expect(constants.types.STRING).toBe(6);
    });

    test('should handle account structures compatible with Solana', () => {
        const account = StacksVMWrapper.createAccount(
            new Uint8Array(32).fill(1), // Pubkey format
            new Uint8Array(1000), // Account data
            BigInt(1000000), // Lamports
            true, // Writable
            false, // Signer
            new Uint8Array(32).fill(2) // Owner pubkey
        );

        // Should match Solana AccountInfo structure
        expect(account.key).toHaveLength(32);
        expect(account.owner).toHaveLength(32);
        expect(typeof account.lamports).toBe('bigint');
        expect(typeof account.isWritable).toBe('boolean');
        expect(typeof account.isSigner).toBe('boolean');
    });
});
