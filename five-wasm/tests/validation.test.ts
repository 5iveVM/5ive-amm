/**
 * Comprehensive WASM VM validation tests
 * Tests WASM bindings against expected VM behavior and compatibility
 */

import { 
    StacksVMWrapper, 
    StacksAccount,
    BytecodeAnalyzer,
    validate_bytecode,
    get_constants
} from '../wrapper/index';

const loadConstants = () => JSON.parse(get_constants());

const u32ToBytes = (value: number): number[] => {
    return [
        value & 0xFF,
        (value >> 8) & 0xFF,
        (value >> 16) & 0xFF,
        (value >> 24) & 0xFF
    ];
};

const u64ToBytes = (value: number | bigint): number[] => {
    let v = BigInt(value);
    const bytes: number[] = [];
    for (let i = 0; i < 8; i++) {
        bytes.push(Number(v & BigInt(0xFF)));
        v >>= BigInt(8);
    }
    return bytes;
};

const buildHeader = (constants: any): number[] => [
    ...constants.FIVE_MAGIC,
    0x00, 0x00, 0x00, 0x00, // features
    0x00, 0x00              // public/total function counts
];

const buildBytecode = (constants: any, ops: number[]): Uint8Array => {
    return new Uint8Array([...buildHeader(constants), ...ops]);
};

const emitPushU64 = (constants: any, value: number | bigint): number[] => [
    constants.opcodes.PUSH_U64,
    ...u64ToBytes(value)
];

const emitPushU8 = (constants: any, value: number): number[] => [
    constants.opcodes.PUSH_U8,
    value & 0xFF
];

const emitAccountIndex = (constants: any, opcode: string, accountIndex: number): number[] => [
    constants.opcodes[opcode],
    accountIndex & 0xFF
];

const emitAccountField = (constants: any, opcode: string, accountIndex: number, offset: number): number[] => [
    constants.opcodes[opcode],
    accountIndex & 0xFF,
    ...u32ToBytes(offset >>> 0)
];

describe('WASM VM Validation Suite', () => {
    
    describe('Bytecode Compatibility', () => {
        test('should handle all VM opcodes correctly', () => {
            const constants = loadConstants();
            
            // Test each opcode category
            const opcodeCategories = {
                stack: ['PUSH_U64', 'POP', 'DUP', 'SWAP'],
                math: ['ADD', 'SUB', 'MUL', 'DIV', 'GT', 'LT', 'EQ'],
                logical: ['AND', 'OR', 'NOT'],
                memory: ['STORE', 'LOAD', 'STORE_FIELD', 'LOAD_FIELD'],
                system: ['INVOKE', 'INVOKE_SIGNED'],
                account: ['CREATE_ACCOUNT', 'LOAD_ACCOUNT', 'DERIVE_PDA'],
                control: ['HALT', 'JUMP', 'JUMP_IF', 'REQUIRE']
            };

            Object.entries(opcodeCategories).forEach(([category, opcodes]) => {
                opcodes.forEach(opcode => {
                    expect(constants.opcodes[opcode]).toBeDefined();
                    expect(typeof constants.opcodes[opcode]).toBe('number');
                });
            });
        });

        test('should validate magic bytes correctly', () => {
            const constants = loadConstants();
            const validBytecode = buildBytecode(constants, [constants.opcodes.HALT]);
            const invalidBytecode = new Uint8Array([0x00, 0x01, 0x02, 0x03, 0x00]);

            expect(validate_bytecode(validBytecode)).toBe(true);
            expect(validate_bytecode(invalidBytecode)).toBe(false);
        });

        test('should enforce size limits', () => {
            const constants = loadConstants();
            const maxSize = constants.MAX_SCRIPT_SIZE;

            // Valid size
            const validSize = new Uint8Array(maxSize - 1).fill(0);
            validSize.set([0x35, 0x49, 0x56, 0x45], 0); // Add magic bytes
            expect(validate_bytecode(validSize)).toBe(true);

            // Invalid size (too small)
            const tooSmall = new Uint8Array([0x53, 0x54]);
            expect(validate_bytecode(tooSmall)).toBe(false);

            // Invalid size (too large)
            const tooLarge = new Uint8Array(maxSize + 1).fill(0);
            tooLarge.set([0x35, 0x49, 0x56, 0x45], 0);
            expect(validate_bytecode(tooLarge)).toBe(false);
        });
    });

    describe('Value Type System', () => {
        test('should support all VM value types', () => {
            const constants = loadConstants();
            
            const expectedTypes = ['U64', 'BOOL', 'PUBKEY', 'I64', 'U8', 'STRING', 'ACCOUNT', 'ARRAY'];
            
            expectedTypes.forEach(type => {
                expect(constants.types[type]).toBeDefined();
                expect(typeof constants.types[type]).toBe('number');
            });
        });

        test('should convert JavaScript values to VM values', () => {
            const constants = loadConstants();

            // Test each value type conversion
            expect(() => StacksVMWrapper.createVMValue(42, 'U64')).not.toThrow();
            expect(() => StacksVMWrapper.createVMValue(true, 'BOOL')).not.toThrow();
            expect(() => StacksVMWrapper.createVMValue('test string', 'STRING')).not.toThrow();
            expect(() => StacksVMWrapper.createVMValue(255, 'U8')).not.toThrow();
            expect(() => StacksVMWrapper.createVMValue(-42, 'I64')).not.toThrow();
        });

        test('should validate value type constraints', () => {
            // U8 should not accept values > 255
            expect(() => StacksVMWrapper.createVMValue(256, 'U8')).toThrow();
            
            // PUBKEY should require 32 bytes
            const validPubkey = new Uint8Array(32);
            const invalidPubkey = new Uint8Array(31);
            
            expect(() => StacksVMWrapper.createVMValue(validPubkey, 'PUBKEY')).not.toThrow();
            expect(() => StacksVMWrapper.createVMValue(invalidPubkey, 'PUBKEY')).toThrow();
        });
    });

    describe('Account Interface Compatibility', () => {
        test('should create accounts compatible with Solana AccountInfo', () => {
            const account = StacksVMWrapper.createAccount(
                new Uint8Array(32).fill(1), // key
                new Uint8Array(1000), // data  
                BigInt(1000000), // lamports
                true, // writable
                false, // signer
                new Uint8Array(32).fill(2) // owner
            );

            // Verify Solana AccountInfo compatibility
            expect(account.key).toHaveLength(32);
            expect(account.owner).toHaveLength(32);
            expect(typeof account.lamports).toBe('bigint');
            expect(typeof account.isWritable).toBe('boolean');
            expect(typeof account.isSigner).toBe('boolean');
            expect(account.data instanceof Uint8Array).toBe(true);
        });

        test('should handle account mutations during execution', async () => {
            const constants = loadConstants();
            const bytecode = buildBytecode(constants, [
                ...emitPushU64(constants, 42), // value
                ...emitAccountField(constants, 'STORE_FIELD', 0, 8),
                constants.opcodes.HALT
            ]);

            const account = StacksVMWrapper.createAccount(
                new Uint8Array(32).fill(1),
                new Uint8Array(1000), // Large enough for test
                BigInt(1000000),
                true, // Must be writable
                false,
                new Uint8Array(32).fill(2)
            );

            const vm = new StacksVMWrapper(bytecode);
            const result = await vm.execute(new Uint8Array(), [account]);

            expect(result.success).toBe(true);
            expect(result.updatedAccounts).toHaveLength(1);
            
            // Account should maintain structure
            const updatedAccount = result.updatedAccounts[0];
            expect(updatedAccount.key).toEqual(account.key);
            expect(updatedAccount.owner).toEqual(account.owner);
            expect(updatedAccount.isWritable).toBe(account.isWritable);

            vm.dispose();
        });
    });

    describe('Error Handling and Safety', () => {
        test('should handle stack overflow gracefully', async () => {
            // Create bytecode that would cause stack overflow
            const constants = loadConstants();
            const pushes = Array(50).fill(0).flatMap(() => emitPushU64(constants, 1));
            const bytecode = buildBytecode(constants, [
                ...pushes,
                constants.opcodes.HALT
            ]);

            const vm = new StacksVMWrapper(bytecode);
            const result = await vm.execute(new Uint8Array(), []);

            if (!result.success) {
                expect(result.error).toContain('overflow'); // Should contain relevant error
            }

            vm.dispose();
        });

        test('should handle invalid account access', async () => {
            const constants = loadConstants();
            const bytecode = buildBytecode(constants, [
                ...emitAccountIndex(constants, 'LOAD_ACCOUNT', 99),
                constants.opcodes.HALT
            ]);

            const vm = new StacksVMWrapper(bytecode);
            const result = await vm.execute(new Uint8Array(), []); // No accounts provided

            expect(result.success).toBe(false);
            expect(result.error).toMatch(/account|index/i);

            vm.dispose();
        });

        test('should prevent memory access violations', async () => {
            const constants = loadConstants();
            const bytecode = buildBytecode(constants, [
                ...emitPushU64(constants, 42),
                ...emitAccountField(constants, 'STORE_FIELD', 0, 0xFFFFFFFF),
                constants.opcodes.HALT
            ]);

            const account = StacksVMWrapper.createAccount(
                new Uint8Array(32).fill(1),
                new Uint8Array(100), // Small data size
                BigInt(1000000),
                true,
                false,
                new Uint8Array(32).fill(2)
            );

            const vm = new StacksVMWrapper(bytecode);
            const result = await vm.execute(new Uint8Array(), [account]);

            expect(result.success).toBe(false);
            expect(result.error).toMatch(/memory|access|violation/i);

            vm.dispose();
        });

        test('should enforce compute unit limits', async () => {
            // Create bytecode with many operations to exceed compute limit
            const constants = loadConstants();
            const manyOps = Array(1000).fill(0).flatMap(() => [
                ...emitPushU64(constants, 1),
                ...emitPushU64(constants, 1),
                constants.opcodes.ADD,
                constants.opcodes.POP
            ]);

            const bytecode = buildBytecode(constants, [
                ...manyOps,
                constants.opcodes.HALT
            ]);

            const vm = new StacksVMWrapper(bytecode);
            const result = await vm.execute(new Uint8Array(), []);

            // Should either complete with high CU count or fail with limit exceeded
            if (result.success) {
                expect(result.computeUnits).toBeGreaterThan(BigInt(1000));
            } else {
                expect(result.error).toMatch(/compute|limit/i);
            }

            vm.dispose();
        });
    });

    describe('Bytecode Analysis', () => {
        test('should analyze instruction structure correctly', () => {
            const constants = loadConstants();
            const bytecode = buildBytecode(constants, [
                ...emitPushU64(constants, 42),
                ...emitPushU64(constants, 24),
                constants.opcodes.ADD,
                constants.opcodes.HALT
            ]);

            const analysis = StacksVMWrapper.analyzeBytecode(bytecode);

            expect(analysis.totalSize).toBe(bytecode.length);
            expect(analysis.instructionCount).toBeGreaterThan(0);
            expect(analysis.instructions).toHaveLength(analysis.instructionCount);

            // Verify instruction details
            if (analysis.instructions.length > 0) {
                const firstInstruction = analysis.instructions[0];
                expect(firstInstruction.offset).toBeGreaterThanOrEqual(10); // After optimized header
                expect(typeof firstInstruction.opcode).toBe('number');
                expect(typeof firstInstruction.name).toBe('string');
                expect(firstInstruction.size).toBeGreaterThan(0);
            }
        });

        test('should identify different instruction types', () => {
            const constants = loadConstants();
            
            // Create bytecode with different instruction types
            const bytecode = buildBytecode(constants, [
                ...emitPushU64(constants, 42),
                constants.opcodes.DUP,   // Stack op
                constants.opcodes.ADD,   // Math op
                constants.opcodes.POP,   // Stack op
                constants.opcodes.HALT   // Control op
            ]);

            const analysis = StacksVMWrapper.analyzeBytecode(bytecode);
            
            expect(analysis.instructions.length).toBeGreaterThanOrEqual(4);
            
            const instructionNames = analysis.instructions.map(i => i.name);
            expect(instructionNames).toContain('PUSH_U64');
            expect(instructionNames).toContain('DUP');
            expect(instructionNames).toContain('ADD');
            expect(instructionNames).toContain('HALT');
        });
    });

    describe('Performance Characteristics', () => {
        test('should maintain consistent execution times', async () => {
            const constants = loadConstants();
            const bytecode = buildBytecode(constants, [
                ...emitPushU64(constants, 42),
                ...emitPushU64(constants, 24),
                constants.opcodes.ADD,
                constants.opcodes.HALT
            ]);

            const executionTimes: number[] = [];
            const iterations = 50;

            for (let i = 0; i < iterations; i++) {
                const vm = new StacksVMWrapper(bytecode);
                const start = performance.now();
                
                await vm.execute(new Uint8Array(), []);
                
                const end = performance.now();
                executionTimes.push(end - start);
                vm.dispose();
            }

            // Calculate statistics
            const mean = executionTimes.reduce((a, b) => a + b, 0) / executionTimes.length;
            const stdDev = Math.sqrt(
                executionTimes.reduce((acc, time) => acc + Math.pow(time - mean, 2), 0) / executionTimes.length
            );

            // Performance should be consistent (low coefficient of variation)
            const coefficientOfVariation = stdDev / mean;
            expect(coefficientOfVariation).toBeLessThan(1.0); // Less than 100% variation

            // Execution should be fast
            expect(mean).toBeLessThan(100); // Less than 100ms average
        });

        test('should scale linearly with bytecode complexity', async () => {
            const constants = loadConstants();
            const createBytecode = (operations: number) => {
                const ops = Array(operations).fill(0).flatMap(() => [
                    ...emitPushU64(constants, 1),
                    ...emitPushU64(constants, 1),
                    constants.opcodes.ADD,
                    constants.opcodes.POP
                ]);

                return buildBytecode(constants, [
                    ...ops,
                    constants.opcodes.HALT
                ]);
            };

            const complexities = [10, 50, 100];
            const timings: number[] = [];

            for (const complexity of complexities) {
                const bytecode = createBytecode(complexity);
                const vm = new StacksVMWrapper(bytecode);
                
                const start = performance.now();
                const result = await vm.execute(new Uint8Array(), []);
                const end = performance.now();
                
                if (result.success) {
                    timings.push(end - start);
                }
                
                vm.dispose();
            }

            // Verify we have successful executions
            expect(timings.length).toBeGreaterThan(0);
            
            // Higher complexity should generally take longer (allowing for some variance)
            if (timings.length >= 2) {
                const trend = timings[timings.length - 1] / timings[0];
                expect(trend).toBeGreaterThan(0.5); // Some increase expected
            }
        });
    });

    describe('Constants Synchronization', () => {
        test('should match Rust VM constants exactly', () => {
            const constants = loadConstants();

            // These values should match the Rust constants exactly
            expect(constants.MAX_SCRIPT_SIZE).toBe(65536);
            expect(constants.MAX_COMPUTE_UNITS).toBe(1000000);
            
            // Magic bytes should match "5IVE"
            expect(constants.FIVE_MAGIC).toEqual([0x35, 0x49, 0x56, 0x45]);
            
            // Key opcodes should match expected values
            expect(constants.opcodes.PUSH_U64).toBe(0x1B);
            expect(constants.opcodes.POP).toBe(0x10);
            expect(constants.opcodes.ADD).toBe(0x20);
            expect(constants.opcodes.HALT).toBe(0x00);
            
            // Type constants should match
            expect(constants.types.U64).toBe(1);
            expect(constants.types.BOOL).toBe(2);
            expect(constants.types.STRING).toBe(6);
        });

        test('should provide complete opcode coverage', () => {
            const constants = loadConstants();
            
            // Verify all major opcode categories are present
            const requiredOpcodes = [
                // Stack operations
                'PUSH_U64', 'POP', 'DUP', 'SWAP',
                // Math operations  
                'ADD', 'SUB', 'MUL', 'DIV', 'GT', 'LT', 'EQ',
                // Memory operations
                'STORE', 'LOAD', 'STORE_FIELD', 'LOAD_FIELD',
                // System operations
                'INVOKE', 'INVOKE_SIGNED',
                // Account operations
                'CREATE_ACCOUNT', 'LOAD_ACCOUNT', 'DERIVE_PDA',
                // Control flow
                'HALT', 'JUMP', 'JUMP_IF', 'REQUIRE'
            ];

            requiredOpcodes.forEach(opcode => {
                expect(constants.opcodes[opcode]).toBeDefined();
                expect(typeof constants.opcodes[opcode]).toBe('number');
            });
        });
    });
});
