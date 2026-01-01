/**
 * VLE Encoding Unit Tests
 *
 * These tests validate that VLE encoding produces the exact byte sequences
 * expected by the Five VM program and MitoVM.
 */
import { VLEEncoder } from '../../lib/vle-encoder';
describe('VLE Encoder', () => {
    describe('Basic Function Execution', () => {
        it('should encode function index 0 with no parameters', async () => {
            // Test the exact case that was failing: function 0, no parameters
            const parameters = [];
            const values = {};
            const result = await VLEEncoder.encodeExecuteVLE(0, parameters, values);
            // Expected: [0] (parameter count 0, VLE encoded)
            // The WASM encoder should return ONLY parameter data
            expect(Array.from(result)).toEqual([0]);
            expect(result.length).toBe(1);
        });
        it('should encode function index 1 with U64 parameter', async () => {
            // Test with a parameter to validate the encoding works
            const parameters = [{ name: 'value', type: 'u64' }];
            const values = { value: 42 };
            const result = await VLEEncoder.encodeExecuteVLE(1, parameters, values);
            // Expected: [1, 4, ...u64_bytes] 
            // [1] = parameter count 1 (VLE)
            // [4] = u64 type ID
            // [...] = 42 as little-endian u64
            expect(result.length).toBeGreaterThan(1);
            expect(result[0]).toBe(1); // parameter count = 1
        });
    });
    describe('SDK Integration', () => {
        it('should produce correct transaction data for execute instruction', async () => {
            // Test the complete flow: VLE encoding + SDK instruction building
            const { FiveSDK } = await import('../FiveSDK');
            // Mock the components needed for encodeExecuteInstruction
            const functionIndex = 0;
            const encodedParams = new Uint8Array([0]); // parameter count 0
            // Test the private encodeExecuteInstruction method
            const instructionData = FiveSDK.encodeExecuteInstruction(functionIndex, encodedParams);
            // Expected: [2, 0, 0] = [discriminator, function_index(VLE), param_count(VLE)]
            expect(Array.from(instructionData)).toEqual([2, 0, 0]);
            expect(instructionData.length).toBe(3);
        });
        it('should handle function index encoding correctly', async () => {
            const { FiveSDK } = await import('../FiveSDK');
            // Test VLE encoding of different function indices
            const vle0 = FiveSDK.encodeVLE(0);
            const vle1 = FiveSDK.encodeVLE(1);
            const vle127 = FiveSDK.encodeVLE(127);
            expect(Array.from(vle0)).toEqual([0]);
            expect(Array.from(vle1)).toEqual([1]);
            expect(Array.from(vle127)).toEqual([127]);
        });
    });
    describe('Parameter Encoding', () => {
        it('should encode multiple parameters correctly', async () => {
            const parameters = [
                { name: 'a', type: 'u64' },
                { name: 'b', type: 'u64' }
            ];
            const values = { a: 30, b: 40 };
            const result = await VLEEncoder.encodeExecuteVLE(0, parameters, values);
            // Expected: [2, 4, ...30_bytes, 4, ...40_bytes]
            // [2] = parameter count 2 (VLE)
            // [4] = u64 type ID for first param
            // [...] = 30 as little-endian u64
            // [4] = u64 type ID for second param  
            // [...] = 40 as little-endian u64
            expect(result[0]).toBe(2); // parameter count = 2
            expect(result.length).toBeGreaterThan(10); // At least 1 + 1 + 8 + 1 + 8 bytes
        });
        it('should handle empty parameters consistently', async () => {
            // Test various ways of specifying no parameters
            const cases = [
                { params: [], values: {} },
                { params: undefined, values: {} },
                { params: [], values: undefined }
            ];
            for (const testCase of cases) {
                const result = await VLEEncoder.encodeExecuteVLE(0, testCase.params, testCase.values);
                expect(Array.from(result)).toEqual([0]);
                expect(result.length).toBe(1);
            }
        });
    });
    describe('Error Cases', () => {
        it('should handle missing parameter values gracefully', async () => {
            const parameters = [{ name: 'missing', type: 'u64' }];
            const values = {}; // missing the 'missing' parameter
            await expect(VLEEncoder.encodeExecuteVLE(0, parameters, values)).rejects.toThrow(/Missing value for parameter/);
        });
    });
});
//# sourceMappingURL=vle-encoding.test.js.map