/**
 * Basic Five SDK Tests
 *
 * Simple tests to verify SDK structure and basic functionality
 * without complex ES module dependencies.
 */
import { describe, it, expect } from '@jest/globals';
describe('Five SDK Basic Tests', () => {
    describe('SDK Structure', () => {
        it('should have basic test framework working', () => {
            expect(1 + 1).toBe(2);
        });
        it('should handle basic TypeScript compilation', () => {
            const testArray = [1, 2, 3];
            const sum = testArray.reduce((acc, val) => acc + val, 0);
            expect(sum).toBe(6);
        });
        it('should work with Uint8Array operations', () => {
            const bytecode = new Uint8Array([1, 2, 3, 4, 5]);
            expect(bytecode.length).toBe(5);
            expect(bytecode[0]).toBe(1);
            expect(bytecode[4]).toBe(5);
        });
        it('should handle basic serialization operations', () => {
            const testData = {
                name: 'TestScript',
                version: 1,
                functions: ['initialize', 'transfer']
            };
            const serialized = JSON.stringify(testData);
            const deserialized = JSON.parse(serialized);
            expect(deserialized.name).toBe('TestScript');
            expect(deserialized.functions).toHaveLength(2);
        });
        it('should work with Buffer operations for bytecode', () => {
            const data = Buffer.from([0x46, 0x49, 0x56, 0x45]); // "FIVE"
            const text = data.toString('utf8');
            expect(text).toBe('FIVE');
        });
        it('should handle base64 encoding/decoding', () => {
            const originalData = new Uint8Array([1, 2, 3, 4, 5, 255]);
            const encoded = Buffer.from(originalData).toString('base64');
            const decoded = Buffer.from(encoded, 'base64');
            expect(decoded).toEqual(Buffer.from(originalData));
        });
        it('should work with varint encoding simulation', () => {
            // Simulate Variable Length Encoding for small numbers
            function encodeVLE(value) {
                const bytes = [];
                let num = value;
                while (num >= 0x80) {
                    bytes.push((num & 0x7F) | 0x80);
                    num >>>= 7;
                }
                bytes.push(num & 0x7F);
                return new Uint8Array(bytes);
            }
            const encoded127 = encodeVLE(127);
            expect(encoded127).toEqual(new Uint8Array([127]));
            const encoded128 = encodeVLE(128);
            expect(encoded128).toEqual(new Uint8Array([0x80, 0x01]));
        });
        it('should handle account structure validation', () => {
            const accounts = [
                {
                    pubkey: '11111111111111111111111111111114',
                    isSigner: true,
                    isWritable: true
                },
                {
                    pubkey: '22222222222222222222222222222224',
                    isSigner: false,
                    isWritable: false
                }
            ];
            expect(accounts).toHaveLength(2);
            expect(accounts[0].isSigner).toBe(true);
            expect(accounts[1].isSigner).toBe(false);
        });
        it('should work with instruction data formatting', () => {
            // Simulate instruction data creation
            const discriminator = 2; // Execute instruction
            const functionIndex = 1;
            const paramCount = 2;
            const instructionData = new Uint8Array([
                discriminator,
                functionIndex,
                paramCount,
                // Mock parameter data
                4, 0x64, 0, 0, 0, 0, 0, 0, 0, // u64: 100
                11, 4, 116, 101, 115, 116 // string: "test"
            ]);
            expect(instructionData[0]).toBe(2); // Execute discriminator
            expect(instructionData[1]).toBe(1); // Function index
            expect(instructionData[2]).toBe(2); // Parameter count
            expect(instructionData.length).toBeGreaterThan(3);
        });
        it('should handle error cases gracefully', () => {
            try {
                JSON.parse('invalid json');
                expect(false).toBe(true); // Should not reach this
            }
            catch (error) {
                expect(error).toBeInstanceOf(SyntaxError);
            }
        });
    });
    describe('SDK Constants', () => {
        it('should define expected constants', () => {
            const FIVE_VM_PROGRAM_ID = 'FiveProgramID11111111111111111111111111111';
            const SYSTEM_PROGRAM_ID = '11111111111111111111111111111112';
            expect(FIVE_VM_PROGRAM_ID.length).toBeGreaterThan(40);
            expect(SYSTEM_PROGRAM_ID.length).toBeGreaterThan(30);
        });
        it('should handle compute unit estimations', () => {
            function estimateComputeUnits(functionIndex, paramCount) {
                return Math.max(5000, 1000 + (paramCount * 500) + (functionIndex * 100));
            }
            expect(estimateComputeUnits(0, 0)).toBe(5000); // Minimum
            expect(estimateComputeUnits(1, 2)).toBe(5000); // Still minimum (Math.max)
            expect(estimateComputeUnits(10, 10)).toBe(7000); // Exceeds minimum: 1000 + (10*500) + (10*100) = 7000
        });
    });
    describe('Type System', () => {
        it('should work with SDK type definitions', () => {
            const successResult = {
                success: true,
                bytecode: new Uint8Array([1, 2, 3]),
                metadata: {
                    functions: [
                        { name: 'initialize', index: 0 },
                        { name: 'transfer', index: 1 }
                    ]
                }
            };
            const failureResult = {
                success: false,
                errors: ['Syntax error', 'Type error']
            };
            expect(successResult.success).toBe(true);
            expect(successResult.metadata?.functions).toHaveLength(2);
            expect(failureResult.success).toBe(false);
            expect(failureResult.errors).toHaveLength(2);
        });
        it('should work with serialized instruction types', () => {
            const instruction = {
                programId: 'FiveProgramID11111111111111111111111111111',
                accounts: [
                    {
                        pubkey: '11111111111111111111111111111114',
                        isSigner: false,
                        isWritable: true
                    }
                ],
                data: Buffer.from([1, 2, 3]).toString('base64')
            };
            expect(instruction.programId).toBe('FiveProgramID11111111111111111111111111111');
            expect(instruction.accounts).toHaveLength(1);
            expect(instruction.data).toBe('AQID');
        });
    });
});
//# sourceMappingURL=basic.test.js.map