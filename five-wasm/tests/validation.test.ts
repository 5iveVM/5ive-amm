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

describe('WASM VM Validation Suite', () => {
    
    describe('Bytecode Compatibility', () => {
        test('should handle all VM opcodes correctly', () => {
            const constants = JSON.parse(get_constants());
            
            // Test each opcode category
            const opcodeCategories = {
                stack: ['PUSH', 'POP', 'DUP', 'SWAP'],
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
            const validBytecode = new Uint8Array([0x35, 0x49, 0x56, 0x45, 0x00]); // 5IVE + HALT
            const invalidBytecode = new Uint8Array([0x00, 0x01, 0x02, 0x03, 0x00]);

            expect(validate_bytecode(validBytecode)).toBe(true);
            expect(validate_bytecode(invalidBytecode)).toBe(false);
        });

        test('should enforce size limits', () => {
            const constants = JSON.parse(get_constants());
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
            const constants = JSON.parse(get_constants());
            
            const expectedTypes = ['U64', 'BOOL', 'PUBKEY', 'I64', 'U8', 'STRING', 'ACCOUNT', 'ARRAY'];
            
            expectedTypes.forEach(type => {
                expect(constants.types[type]).toBeDefined();
                expect(typeof constants.types[type]).toBe('number');
            });
        });

        test('should convert JavaScript values to VM values', () => {
            const constants = JSON.parse(get_constants());

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
            const bytecode = new Uint8Array([
                0x35, 0x49, 0x56, 0x45, // 5IVE magic
                0x01, 0x05, 0, // PUSH U8(0) - account index
                0x01, 0x01, 42, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(42) - value
                0x01, 0x01, 8, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(8) - offset
                0x32, // STORE_FIELD
                0x00  // HALT
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
            const bytecode = new Uint8Array([
                0x35, 0x49, 0x56, 0x45, // 5IVE magic
                // Push many values to overflow stack
                ...Array(50).fill([0x01, 0x01, 1, 0, 0, 0, 0, 0, 0, 0]).flat(), // PUSH U64(1) x50
                0x00 // HALT
            ]);

            const vm = new StacksVMWrapper(bytecode);
            const result = await vm.execute(new Uint8Array(), []);

            expect(result.success).toBe(false);
            expect(result.error).toContain('overflow'); // Should contain relevant error

            vm.dispose();
        });

        test('should handle invalid account access', async () => {
            const bytecode = new Uint8Array([
                0x35, 0x49, 0x56, 0x45, // 5IVE magic
                0x01, 0x05, 99, // PUSH U8(99) - invalid account index
                0x51, // LOAD_ACCOUNT
                0x00  // HALT
            ]);

            const vm = new StacksVMWrapper(bytecode);
            const result = await vm.execute(new Uint8Array(), []); // No accounts provided

            expect(result.success).toBe(false);
            expect(result.error).toMatch(/account|index/i);

            vm.dispose();
        });

        test('should prevent memory access violations', async () => {
            const bytecode = new Uint8Array([
                0x35, 0x49, 0x56, 0x45, // 5IVE magic
                0x01, 0x05, 0, // PUSH U8(0) - account index
                0x01, 0x01, 42, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(42) - value
                0x01, 0x01, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, // PUSH U64(MAX) - invalid offset
                0x32, // STORE_FIELD
                0x00  // HALT
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
            const manyOps = Array(1000).fill([
                0x01, 0x01, 1, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(1)
                0x01, 0x01, 1, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(1)
                0x10, // ADD
                0x02  // POP
            ]).flat();

            const bytecode = new Uint8Array([
                0x35, 0x49, 0x56, 0x45, // 5IVE magic
                ...manyOps,
                0x00 // HALT
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
            const bytecode = new Uint8Array([
                0x35, 0x49, 0x56, 0x45, // 5IVE magic
                0x01, 0x01, 42, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(42)
                0x01, 0x01, 24, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(24)
                0x10, // ADD
                0x00  // HALT
            ]);

            const analysis = StacksVMWrapper.analyzeBytecode(bytecode);

            expect(analysis.totalSize).toBe(bytecode.length);
            expect(analysis.instructionCount).toBeGreaterThan(0);
            expect(analysis.instructions).toHaveLength(analysis.instructionCount);

            // Verify instruction details
            if (analysis.instructions.length > 0) {
                const firstInstruction = analysis.instructions[0];
                expect(firstInstruction.offset).toBeGreaterThanOrEqual(4); // After magic bytes
                expect(typeof firstInstruction.opcode).toBe('number');
                expect(typeof firstInstruction.name).toBe('string');
                expect(firstInstruction.size).toBeGreaterThan(0);
            }
        });

        test('should identify different instruction types', () => {
            const constants = JSON.parse(get_constants());
            
            // Create bytecode with different instruction types
            const bytecode = new Uint8Array([
                0x35, 0x49, 0x56, 0x45, // 5IVE magic
                constants.opcodes.PUSH, constants.types.U64, 42, 0, 0, 0, 0, 0, 0, 0, // PUSH
                constants.opcodes.DUP,   // Stack op
                constants.opcodes.ADD,   // Math op
                constants.opcodes.POP,   // Stack op
                constants.opcodes.HALT   // Control op
            ]);

            const analysis = StacksVMWrapper.analyzeBytecode(bytecode);
            
            expect(analysis.instructions.length).toBeGreaterThanOrEqual(4);
            
            const instructionNames = analysis.instructions.map(i => i.name);
            expect(instructionNames).toContain('PUSH');
            expect(instructionNames).toContain('DUP');
            expect(instructionNames).toContain('ADD');
            expect(instructionNames).toContain('HALT');
        });
    });

    describe('Performance Characteristics', () => {
        test('should maintain consistent execution times', async () => {
            const bytecode = new Uint8Array([
                0x35, 0x49, 0x56, 0x45, // 5IVE magic
                0x01, 0x01, 42, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(42)
                0x01, 0x01, 24, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(24)
                0x10, // ADD
                0x00  // HALT
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
            const createBytecode = (operations: number) => {
                const ops = Array(operations).fill([
                    0x01, 0x01, 1, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(1)
                    0x01, 0x01, 1, 0, 0, 0, 0, 0, 0, 0, // PUSH U64(1)
                    0x10, // ADD
                    0x02  // POP (keep stack clean)
                ]).flat();

                return new Uint8Array([
                    0x35, 0x49, 0x56, 0x45, // 5IVE magic
                    ...ops,
                    0x00 // HALT
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
            const constants = JSON.parse(get_constants());

            // These values should match the Rust constants exactly
            expect(constants.MAX_SCRIPT_SIZE).toBe(1000);
            expect(constants.MAX_COMPUTE_UNITS).toBe(200000);
            
            // Magic bytes should match "5IVE"
            expect(constants.FIVE_MAGIC).toEqual([0x35, 0x49, 0x56, 0x45]);
            
            // Key opcodes should match expected values
            expect(constants.opcodes.PUSH).toBe(1);
            expect(constants.opcodes.POP).toBe(2);
            expect(constants.opcodes.ADD).toBe(16);
            expect(constants.opcodes.HALT).toBe(0);
            
            // Type constants should match
            expect(constants.types.U64).toBe(1);
            expect(constants.types.BOOL).toBe(2);
            expect(constants.types.STRING).toBe(6);
        });

        test('should provide complete opcode coverage', () => {
            const constants = JSON.parse(get_constants());
            
            // Verify all major opcode categories are present
            const requiredOpcodes = [
                // Stack operations
                'PUSH', 'POP', 'DUP', 'SWAP',
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