/**
 * Five SDK Account Fetching and VLE Deserialization Tests
 *
 * Tests the new account fetching and VLE deserialization functionality
 */
import { FiveSDK } from '../FiveSDK.js';
import { describe, it, expect, beforeAll } from '@jest/globals';
// Mock Solana connection for testing
const mockConnection = {
    getAccountInfo: jest.fn()
};
describe('Five SDK Account Fetching and VLE Deserialization', () => {
    beforeAll(async () => {
        // Initialize any required components
    });
    describe('fetchAccountAndDeserializeVLE', () => {
        it('should handle non-existent account gracefully', async () => {
            mockConnection.getAccountInfo.mockResolvedValueOnce(null);
            const result = await FiveSDK.fetchAccountAndDeserializeVLE('7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo', mockConnection, { debug: false });
            expect(result.success).toBe(false);
            expect(result.error).toContain('Account not found');
        });
        it('should handle account with no data', async () => {
            mockConnection.getAccountInfo.mockResolvedValueOnce({
                owner: { toString: () => '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo' },
                lamports: 1000000,
                data: new Uint8Array(0)
            });
            const result = await FiveSDK.fetchAccountAndDeserializeVLE('7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo', mockConnection, { debug: false });
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
            const result = await FiveSDK.fetchAccountAndDeserializeVLE('7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo', mockConnection, { debug: false, parseMetadata: false });
            expect(result.success).toBe(true);
            expect(result.rawBytecode).toEqual(testData);
            expect(result.accountInfo?.dataLength).toBe(5);
        });
        it('should validate invalid account address format', async () => {
            const result = await FiveSDK.fetchAccountAndDeserializeVLE('invalid-address', mockConnection, { debug: false });
            expect(result.success).toBe(false);
            expect(result.error).toContain('Invalid account address format');
        });
    });
    describe('deserializeVLEParameters', () => {
        it('should deserialize simple VLE instruction data', async () => {
            // Test data: [discriminator(2), function_index(0), param_count(2), param1(30), param2(40)]
            const instructionData = new Uint8Array([2, 0, 2, 30, 40]);
            const result = await FiveSDK.deserializeVLEParameters(instructionData, ['u64', 'u64'], { debug: false });
            expect(result.success).toBe(true);
            expect(result.discriminator).toBe(2);
            expect(result.functionIndex).toBe(0);
            expect(result.parameters).toHaveLength(2);
            expect(result.parameters?.[0].value).toBe(30);
            expect(result.parameters?.[1].value).toBe(40);
        });
        it('should handle empty instruction data', async () => {
            const result = await FiveSDK.deserializeVLEParameters(new Uint8Array([]), [], { debug: false });
            expect(result.success).toBe(false);
            expect(result.error).toContain('Instruction data too short');
        });
        it('should handle VLE with large numbers', async () => {
            // Test VLE encoding of larger numbers (> 127)
            const instructionData = new Uint8Array([2, 0, 1, 0x80, 0x02]); // 256 VLE-encoded
            const result = await FiveSDK.deserializeVLEParameters(instructionData, ['u64'], { debug: false });
            expect(result.success).toBe(true);
            expect(result.parameters?.[0].value).toBe(256);
        });
    });
    describe('fetchMultipleAccountsAndDeserializeVLE', () => {
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
            const results = await FiveSDK.fetchMultipleAccountsAndDeserializeVLE(addresses, mockConnection, { debug: false });
            expect(results.size).toBe(2);
            expect(results.get(addresses[0])?.success).toBe(true);
            expect(results.get(addresses[1])?.success).toBe(false);
        });
        it('should handle empty address list', async () => {
            const results = await FiveSDK.fetchMultipleAccountsAndDeserializeVLE([], mockConnection, { debug: false });
            expect(results.size).toBe(0);
        });
    });
    describe('VLE validation', () => {
        it('should validate Five VM bytecode header', async () => {
            // Valid Five VM bytecode with "5IVE" magic bytes
            const validBytecode = new Uint8Array([0x35, 0x49, 0x56, 0x45, 0x00, 0x01]); // "5IVE" + features + function_count
            mockConnection.getAccountInfo.mockResolvedValueOnce({
                owner: { toString: () => '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo' },
                lamports: 1000000,
                data: validBytecode
            });
            const result = await FiveSDK.fetchAccountAndDeserializeVLE('7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo', mockConnection, { debug: false, validateVLE: true });
            expect(result.success).toBe(true);
            expect(result.logs?.some(log => log.includes('VLE encoding validation: PASSED'))).toBe(true);
        });
        it('should detect invalid bytecode header', async () => {
            // Invalid bytecode without proper magic bytes
            const invalidBytecode = new Uint8Array([0x00, 0x00, 0x00, 0x00, 0x00, 0x01]);
            mockConnection.getAccountInfo.mockResolvedValueOnce({
                owner: { toString: () => '7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo' },
                lamports: 1000000,
                data: invalidBytecode
            });
            const result = await FiveSDK.fetchAccountAndDeserializeVLE('7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo', mockConnection, { debug: false, validateVLE: true });
            expect(result.success).toBe(true); // Account fetch succeeds
            expect(result.logs?.some(log => log.includes('VLE encoding validation: FAILED'))).toBe(true);
        });
    });
});
//# sourceMappingURL=account-fetching.test.js.map