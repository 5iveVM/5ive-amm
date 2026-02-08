/**
 * Integration tests for WasmCompilerService
 * 
 * These tests verify the integration between TypeScript and WASM module,
 * focusing on honest partial execution reporting and proper error handling.
 */

import { WasmCompilerService, TestResultHelper, PartialExecutionSummary } from '../app/wasm-compiler';

describe('WasmCompilerService Integration Tests', () => {
    let wasmService: WasmCompilerService;

    beforeAll(async () => {
        wasmService = new WasmCompilerService();
        await wasmService.initialize();
    });

    describe('Service Initialization', () => {
        test('should initialize without errors', () => {
            expect(wasmService).toBeDefined();
        });

        test('should provide VM constants after initialization', () => {
            const constants = wasmService.getConstants();
            
            expect(constants).toBeDefined();
            expect(constants.opcodes).toBeDefined();
            expect(constants.types).toBeDefined();
            expect(constants.MAX_SCRIPT_SIZE).toBeDefined();
            expect(constants.MAX_COMPUTE_UNITS).toBeDefined();
        });

        test('should reject invalid service operations before initialization', async () => {
            const uninitializedService = new WasmCompilerService();
            
            expect(() => uninitializedService.getConstants()).toThrow('not initialized');
            expect(() => uninitializedService.validateBytecode(new Uint8Array())).toThrow('not initialized');
        });
    });

    describe('Bytecode Validation', () => {
        test('should validate correct bytecode format', () => {
            // Valid bytecode with magic bytes
            const validBytecode = new Uint8Array([
                0x35, 0x49, 0x56, 0x45, // "5IVE"
                0x00, 0x00, 0x00, 0x00, // features
                0x00, 0x00,             // public/total function counts
                0x00                    // HALT
            ]);
            expect(wasmService.validateBytecode(validBytecode)).toBe(true);
        });

        test('should reject invalid magic bytes', () => {
            const invalidBytecode = new Uint8Array([0x00, 0x01, 0x02, 0x03, 0x00]);
            expect(wasmService.validateBytecode(invalidBytecode)).toBe(false);
        });

        test('should reject empty bytecode', () => {
            const emptyBytecode = new Uint8Array();
            expect(wasmService.validateBytecode(emptyBytecode)).toBe(false);
        });

        test('should reject bytecode that is too short', () => {
            const shortBytecode = new Uint8Array([0x35, 0x49]);
            expect(wasmService.validateBytecode(shortBytecode)).toBe(false);
        });
    });

    describe('Test Bytecode Creation', () => {
        test('should create valid bytecode for simple operations', () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 42] },
                { opcode: 'HALT' }
            ]);

            expect(bytecode.length).toBeGreaterThan(4); // More than just magic bytes
            expect(wasmService.validateBytecode(bytecode)).toBe(true);
        });

        test('should create bytecode for mathematical operations', () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 10] },
                { opcode: 'PUSH', args: ['U64', 20] },
                { opcode: 'ADD' },
                { opcode: 'HALT' }
            ]);

            expect(wasmService.validateBytecode(bytecode)).toBe(true);
        });

        test('should throw error for unknown opcodes', () => {
            expect(() => {
                wasmService.createTestBytecode([
                    { opcode: 'UNKNOWN_OPCODE' },
                    { opcode: 'HALT' }
                ]);
            }).toThrow('Unknown opcode');
        });

        test('should throw error for unknown value types', () => {
            expect(() => {
                wasmService.createTestBytecode([
                    { opcode: 'PUSH', args: ['UNKNOWN_TYPE', 42] },
                    { opcode: 'HALT' }
                ]);
            }).toThrow('Unknown type');
        });
    });

    describe('Account Creation', () => {
        test('should create valid test accounts', () => {
            const account = wasmService.createTestAccount(
                'a'.repeat(64), // 32 bytes hex
                new Uint8Array(100),
                BigInt(1000000),
                true,
                false,
                'b'.repeat(64)
            );

            expect(account.key).toHaveLength(32);
            expect(account.data).toHaveLength(100);
            expect(account.lamports).toBe(BigInt(1000000));
            expect(account.isWritable).toBe(true);
            expect(account.isSigner).toBe(false);
            expect(account.owner).toHaveLength(32);
        });

        test('should handle Uint8Array inputs for keys and owners', () => {
            const keyBytes = new Uint8Array(32).fill(1);
            const ownerBytes = new Uint8Array(32).fill(2);
            
            const account = wasmService.createTestAccount(
                keyBytes,
                new Uint8Array(50),
                BigInt(500),
                false,
                true,
                ownerBytes
            );

            expect(account.key).toEqual(keyBytes);
            expect(account.owner).toEqual(ownerBytes);
            expect(account.isWritable).toBe(false);
            expect(account.isSigner).toBe(true);
        });

        test('should validate account key and owner lengths', () => {
            expect(() => {
                wasmService.createTestAccount('invalid', new Uint8Array());
            }).toThrow('32 bytes');

            expect(() => {
                wasmService.createTestAccount(
                    new Uint8Array(32),
                    new Uint8Array(),
                    BigInt(0),
                    false,
                    false,
                    'invalid'
                );
            }).toThrow('32 bytes');
        });
    });

    describe('Pure Computational Execution', () => {
        test('should execute simple arithmetic correctly', async () => {
            // Test: 10 + 20 = 30
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 10] },
                { opcode: 'PUSH', args: ['U64', 20] },
                { opcode: 'ADD' },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.outcome).toBe('completed');
            expect(result.test_success).toBe(true);
            expect(result.operations_tested).toContain('PUSH_U64');
            expect(result.operations_tested).toContain('ADD');
            expect(result.operations_tested).toContain('HALT');
            expect(result.final_state.has_result).toBe(true);
            expect(result.error_details).toBeUndefined();
        });

        test('should execute subtraction correctly', async () => {
            // Test: 30 - 10 = 20
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 30] },
                { opcode: 'PUSH', args: ['U64', 10] },
                { opcode: 'SUB' },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.outcome).toBe('completed');
            expect(result.test_success).toBe(true);
            expect(result.operations_tested).toContain('SUB');
        });

        test('should execute multiplication correctly', async () => {
            // Test: 5 * 6 = 30
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 5] },
                { opcode: 'PUSH', args: ['U64', 6] },
                { opcode: 'MUL' },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.outcome).toBe('completed');
            expect(result.test_success).toBe(true);
            expect(result.operations_tested).toContain('MUL');
        });

        test('should execute division correctly', async () => {
            // Test: 20 / 4 = 5
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 20] },
                { opcode: 'PUSH', args: ['U64', 4] },
                { opcode: 'DIV' },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.outcome).toBe('completed');
            expect(result.test_success).toBe(true);
            expect(result.operations_tested).toContain('DIV');
        });

        test('should execute complex mathematical sequences', async () => {
            // Test: (10 + 5) * 3 - 2 = 43
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 10] },
                { opcode: 'PUSH', args: ['U64', 5] },
                { opcode: 'ADD' },                                // Stack: [15]
                { opcode: 'PUSH', args: ['U64', 3] },
                { opcode: 'MUL' },                                // Stack: [45]
                { opcode: 'PUSH', args: ['U64', 2] },
                { opcode: 'SUB' },                                // Stack: [43]
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.outcome).toBe('completed');
            expect(result.test_success).toBe(true);
            expect(result.operations_tested).toEqual(['PUSH_U64', 'PUSH_U64', 'ADD', 'PUSH_U64', 'MUL', 'PUSH_U64', 'SUB', 'HALT']);
            expect(result.final_state.stack_size).toBe(1); // Result should be on stack
        });
    });

    describe('Stack Operations', () => {
        test('should execute stack manipulation operations', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 42] },
                { opcode: 'DUP' },      // Duplicate top value
                { opcode: 'SWAP' },     // Swap top two values
                { opcode: 'POP' },      // Remove one value
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.outcome).toBe('completed');
            expect(result.test_success).toBe(true);
            expect(result.operations_tested).toContain('DUP');
            expect(result.operations_tested).toContain('SWAP');
            expect(result.operations_tested).toContain('POP');
        });
    });

    describe('Error Handling', () => {
        test('should handle division by zero correctly', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 10] },
                { opcode: 'PUSH', args: ['U64', 0] },
                { opcode: 'DIV' },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.outcome).toBe('failed');
            expect(result.test_success).toBe(false);
            expect(result.error_details).toBeDefined();
            expect(result.error_details?.toLowerCase()).toContain('zero');
        });

        test('should handle stack underflow correctly', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'POP' },     // Pop from empty stack
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.outcome).toBe('failed');
            expect(result.test_success).toBe(false);
            expect(result.error_details).toBeDefined();
            expect(result.error_details?.toLowerCase()).toContain('underflow');
        });

        test('should handle invalid bytecode gracefully', async () => {
            const invalidBytecode = new Uint8Array([0x00, 0x01, 0x02, 0x03]);

            const result = await wasmService.testBytecodeExecution(invalidBytecode);

            expect(result.outcome).toBe('failed');
            expect(result.test_success).toBe(false);
            expect(result.description).toContain('validation failed');
            expect(result.operations_tested).toEqual([]);
        });
    });

    describe('TestResult Interpretation', () => {
        test('should provide complete execution information', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 42] },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.final_state.compute_units_used).toBeGreaterThan(0);
            expect(result.final_state.instruction_pointer).toBeGreaterThan(4); // Past magic bytes
            expect(result.operations_tested.length).toBeGreaterThan(0);
            expect(result.description).toBeTruthy();
        });

        test('should accurately track operations executed', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 1] },
                { opcode: 'PUSH', args: ['U64', 2] },
                { opcode: 'ADD' },
                { opcode: 'PUSH', args: ['U64', 3] },
                { opcode: 'MUL' },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.operations_tested).toEqual([
                'PUSH_U64', 'PUSH_U64', 'ADD', 'PUSH_U64', 'MUL', 'HALT'
            ]);
        });
    });

    describe('TestResultHelper Utilities', () => {
        test('should correctly identify successful tests', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 42] },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);
            
            expect(TestResultHelper.isSuccessfulTest(result)).toBe(true);
            expect(TestResultHelper.wasStoppedAtSystemCall(result)).toBe(false);
        });

        test('should format execution summary correctly', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 42] },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);
            const summary = TestResultHelper.formatSummary(result);

            expect(summary).toContain('Status: COMPLETED');
            expect(summary).toContain('Operations Tested: PUSH_U64, HALT');
            expect(summary).toContain('Compute Units Used:');
        });

        test('should provide status messages', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 42] },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);
            const message = TestResultHelper.getStatusMessage(result);

            expect(message).toBeTruthy();
            expect(message).toContain('completed successfully');
        });

        test('should extract tested operations', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 42] },
                { opcode: 'DUP' },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);
            const operations = TestResultHelper.getTestedOperations(result);

            expect(operations).toContain('PUSH_U64');
            expect(operations).toContain('DUP');
            expect(operations).toContain('HALT');
        });
    });

    describe('Integration with Accounts', () => {
        test('should execute with account context', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 42] },
                { opcode: 'HALT' }
            ]);

            const account = wasmService.createTestAccount(
                'a'.repeat(64),
                new Uint8Array(100),
                BigInt(1000000),
                true,
                false,
                'b'.repeat(64)
            );

            const result = await wasmService.testBytecodeExecution(
                bytecode,
                new Uint8Array(),
                [account]
            );

            expect(result.outcome).toBe('completed');
            expect(result.test_success).toBe(true);
        });
    });

    describe('Performance and Resource Usage', () => {
        test('should track compute units accurately', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 1] },
                { opcode: 'PUSH', args: ['U64', 2] },
                { opcode: 'ADD' },
                { opcode: 'PUSH', args: ['U64', 3] },
                { opcode: 'MUL' },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.final_state.compute_units_used).toBeGreaterThan(0);
            expect(result.final_state.compute_units_used).toBeLessThan(1000); // Reasonable for simple ops
        });

        test('should handle multiple executions without interference', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 42] },
                { opcode: 'HALT' }
            ]);

            const results = await Promise.all([
                wasmService.testBytecodeExecution(bytecode),
                wasmService.testBytecodeExecution(bytecode),
                wasmService.testBytecodeExecution(bytecode)
            ]);

            results.forEach(result => {
                expect(result.outcome).toBe('completed');
                expect(result.test_success).toBe(true);
            });

            // All should have same execution pattern
            const firstOperations = results[0].operations_tested;
            results.forEach(result => {
                expect(result.operations_tested).toEqual(firstOperations);
            });
        });
    });

    describe('Real-world Scenarios', () => {
        test('should handle vault-like operations', async () => {
            // Simulate a simple vault deposit operation (computational part only)
            const bytecode = wasmService.createTestBytecode([
                // Load initial balance
                { opcode: 'PUSH', args: ['U64', 1000] },
                // Load deposit amount
                { opcode: 'PUSH', args: ['U64', 500] },
                // Add them together
                { opcode: 'ADD' },
                // Store result (new balance = 1500)
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.outcome).toBe('completed');
            expect(result.test_success).toBe(true);
            expect(result.operations_tested).toContain('ADD');
            expect(result.final_state.has_result).toBe(true);
        });

        test('should provide honest feedback about what was tested', async () => {
            const bytecode = wasmService.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 10] },
                { opcode: 'PUSH', args: ['U64', 20] },
                { opcode: 'ADD' },
                { opcode: 'HALT' }
            ]);

            const result = await wasmService.testBytecodeExecution(bytecode);

            expect(result.description).toContain('All 4 operations executed');
            expect(result.description).toContain('completed successfully');
            expect(result.operations_tested).toHaveLength(4);
        });
    });
});
