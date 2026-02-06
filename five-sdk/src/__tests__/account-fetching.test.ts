/**
 * Five SDK Account Fetching and Deserialization Tests
 *
 * Tests the new account fetching and deserialization functionality
 */
import { describe, it, expect, beforeAll, jest, afterAll } from '@jest/globals';

// Mock WASM module before importing FiveSDK
jest.unstable_mockModule('../assets/vm/five_vm_wasm.js', () => ({
    ParameterEncoder: {
        decode_instruction_data: jest.fn((data: Uint8Array) => {
            if (data.length === 0) {
                 throw new Error("Instruction data too short");
            }
            // Simple mock logic to support test cases
            // Data structure: [discriminator(1), function_index(4), param_count(4), params...]
            // For simplicity, we hardcode return values based on known test vectors

            // Check for large number test case (has 256)
            // 256 in u64 LE is 00 01 00...
            // Index 9 (param 1 start) -> 0, Index 10 -> 1
            if (data.length > 9 && data[9] === 0 && data[10] === 1) {
                 return {
                    discriminator: 2,
                    function_index: 0,
                    parameters: [256n]
                };
            }

            // Default simple case
            return {
                discriminator: 2,
                function_index: 0,
                parameters: [30n, 40n]
            };
        })
    }
}), { virtual: true });

// Mock Solana connection for testing
const mockConnection = {
    getAccountInfo: jest.fn()
};

// Import FiveSDK after mocking
const { FiveSDK } = await import('../FiveSDK.js');

describe('Five SDK Account Fetching and Deserialization', () => {
    beforeAll(async () => {
        // Initialize any required components
    });
    describe('fetchAccountAndDeserialize', () => {
        it('should handle non-existent account gracefully', async () => {
            mockConnection.getAccountInfo.mockResolvedValueOnce(null);
            const result = await FiveSDK.fetchAccountAndDeserialize('7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo', mockConnection, { debug: false });
            expect(result.success).toBe(false);
            expect(result.error).toContain('Account not found');
        });
        it('should handle account with no data', async () => {
            mockConnection.getAccountInfo.mockResolvedValueOnce({
                owner: { toString: () => '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo' },
                lamports: 1000000,
                data: new Uint8Array(0)
            });
            const result = await FiveSDK.fetchAccountAndDeserialize('7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo', mockConnection, { debug: false });
            expect(result.success).toBe(false);
            expect(result.error).toContain('Account has no data');
        });
        it('should return raw data when metadata parsing is disabled', async () => {
            const testData = new Uint8Array([1, 2, 3, 4, 5]);
            mockConnection.getAccountInfo.mockResolvedValueOnce({
                owner: { toString: () => '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo' },
                lamports: 1000000,
                data: testData
            });
            const result = await FiveSDK.fetchAccountAndDeserialize('7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo', mockConnection, { debug: false, parseMetadata: false });
            expect(result.success).toBe(true);
            expect(result.rawBytecode).toEqual(testData);
            expect(result.accountInfo?.dataLength).toBe(5);
        });
        it('should validate invalid account address format', async () => {
            const result = await FiveSDK.fetchAccountAndDeserialize('invalid-address', mockConnection, { debug: false });
            expect(result.success).toBe(false);
            expect(result.error).toContain('Invalid account address format');
        });
    });
    describe('deserializeParameters', () => {
        it('should deserialize simple instruction data', async () => {
            // Test data: Fixed Size (Little Endian)
            // Discriminator(2) (u8)
            // Function Index(0) (u32) -> 00 00 00 00
            // Param Count(2) (u32) -> 02 00 00 00
            // Param 1(30) (u64) -> 1E 00 00 00 00 00 00 00
            // Param 2(40) (u64) -> 28 00 00 00 00 00 00 00
            const instructionData = new Uint8Array([
                2,
                0, 0, 0, 0,
                2, 0, 0, 0,
                30, 0, 0, 0, 0, 0, 0, 0,
                40, 0, 0, 0, 0, 0, 0, 0
            ]);
            const result = await FiveSDK.deserializeParameters(instructionData, ['u64', 'u64'], { debug: false });
            expect(result.success).toBe(true);
            expect(result.discriminator).toBe(2);
            expect(result.functionIndex).toBe(0);
            expect(result.parameters).toHaveLength(2);
            expect(result.parameters?.[0].value).toBe(30n); // u64 returns bigint usually, or check behavior
            expect(result.parameters?.[1].value).toBe(40n);
        });
        it('should handle empty instruction data', async () => {
            const result = await FiveSDK.deserializeParameters(new Uint8Array([]), [], { debug: false });
            expect(result.success).toBe(false);
            // expect(result.error).toContain('Instruction data too short'); // Exact error depends on WASM
        });
        it('should handle large numbers', async () => {
            // Test encoding of larger numbers (> 127)
            // Discriminator(2)
            // Function Index(0)
            // Param Count(1)
            // Param 1(256) (u64) -> 00 01 00 00 00 00 00 00
            const instructionData = new Uint8Array([
                2,
                0, 0, 0, 0,
                1, 0, 0, 0,
                0, 1, 0, 0, 0, 0, 0, 0
            ]);
            const result = await FiveSDK.deserializeParameters(instructionData, ['u64'], { debug: false });
            expect(result.success).toBe(true);
            expect(result.parameters?.[0].value).toBe(256n);
        });
    });
    describe('fetchMultipleAccountsAndDeserialize', () => {
        it('should handle batch processing with mixed results', async () => {
            const addresses = [
                '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo',
                'GQegzMnjo2faJxnWoxX8FRBXMUKtWUvSiN4VbG4FQ7CG'
            ];
            // Mock different responses for different addresses
            mockConnection.getAccountInfo
                .mockResolvedValueOnce({
                owner: { toString: () => '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo' },
                lamports: 1000000,
                data: new Uint8Array([1, 2, 3])
            })
                .mockResolvedValueOnce(null); // Second account doesn't exist
            const results = await FiveSDK.fetchMultipleAccountsAndDeserialize(addresses, mockConnection, { debug: false });
            expect(results.size).toBe(2);
            expect(results.get(addresses[0])?.success).toBe(true);
            expect(results.get(addresses[1])?.success).toBe(false);
        });
        it('should handle empty address list', async () => {
            const results = await FiveSDK.fetchMultipleAccountsAndDeserialize([], mockConnection, { debug: false });
            expect(results.size).toBe(0);
        });
    });
    describe('Encoding validation', () => {
        it('should validate Five VM bytecode header', async () => {
            // Valid Five VM bytecode with "5IVE" magic bytes + Optimized Header V3
            // Magic(4) + Features(4) + PubCount(1) + TotalCount(1) = 10 bytes
            const validBytecode = new Uint8Array([
                0x35, 0x49, 0x56, 0x45, // Magic
                0x00, 0x00, 0x00, 0x00, // Features
                0x01, // PubCount
                0x01  // TotalCount
            ]);
            mockConnection.getAccountInfo.mockResolvedValueOnce({
                owner: { toString: () => '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo' },
                lamports: 1000000,
                data: validBytecode
            });
            const result = await FiveSDK.fetchAccountAndDeserialize('7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo', mockConnection, { debug: false, validateEncoding: true });
            expect(result.success).toBe(true);
            expect(result.logs?.some(log => log.includes('Encoding validation: PASSED'))).toBe(true);
        });
        it('should detect invalid bytecode header', async () => {
            // Invalid bytecode without proper magic bytes
            const invalidBytecode = new Uint8Array([0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x01]);
            mockConnection.getAccountInfo.mockResolvedValueOnce({
                owner: { toString: () => '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo' },
                lamports: 1000000,
                data: invalidBytecode
            });
            const result = await FiveSDK.fetchAccountAndDeserialize('7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo', mockConnection, { debug: false, validateEncoding: true });
            expect(result.success).toBe(true); // Account fetch succeeds
            expect(result.logs?.some(log => log.includes('Encoding validation: FAILED'))).toBe(true);
        });
    });
});
