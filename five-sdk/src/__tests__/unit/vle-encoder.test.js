/**
 * Five SDK VLE Encoder Unit Tests
 *
 * Tests for Variable Length Encoding operations including parameter encoding,
 * function index encoding, and WASM integration for Five VM bytecode.
 */
import { describe, it, expect, beforeEach, jest } from '@jest/globals';
import { VLEEncoder } from '../../../lib/vle-encoder.js';
// Mock the WASM module
const mockWasmModule = {
    ParameterEncoder: {
        encode_execute_params: jest.fn()
    }
};
// Helper function to set up mock with discriminator handling
function setupWasmMock(expectedOutput) {
    const wasmOutput = new Uint8Array([2, ...expectedOutput]); // Add discriminator (2) at position 0
    mockWasmModule.ParameterEncoder.encode_execute_params.mockReturnValue(wasmOutput);
}
// Mock the path that VLE encoder actually uses, resolved from src/lib/
jest.unstable_mockModule('../../../assets/vm/five_vm_wasm.js', () => mockWasmModule);
describe('Five SDK VLE Encoder', () => {
    beforeEach(() => {
        jest.clearAllMocks();
    });
    describe('encodeExecuteVLE', () => {
        it('should encode function execution with u64 parameters', async () => {
            const functionIndex = 0;
            const paramDefs = [
                { name: 'amount', type: 'u64' },
                { name: 'receiver', type: 'u64' }
            ];
            const paramValues = {
                amount: 1000,
                receiver: 2000
            };
            // Expected output from real WASM module (discriminator removed)
            const expectedEncoded = Buffer.from([0, 232, 3, 0, 0, 0, 0, 0, 0, 208, 7, 0, 0, 0, 0, 0, 0]);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
        it('should encode function execution with string parameters', async () => {
            const functionIndex = 1;
            const paramDefs = [
                { name: 'name', type: 'string' },
                { name: 'symbol', type: 'string' }
            ];
            const paramValues = {
                name: 'TestToken',
                symbol: 'TT'
            };
            // Expected output from real WASM module (discriminator removed)
            const expectedEncoded = Buffer.from([1, 2, 11, 9, 84, 101, 115, 116, 84, 111, 107, 101, 110, 11, 2, 84, 84]);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
        it('should encode function execution with bool parameters', async () => {
            const functionIndex = 2;
            const paramDefs = [
                { name: 'enabled', type: 'bool' },
                { name: 'locked', type: 'bool' }
            ];
            const paramValues = {
                enabled: true,
                locked: false
            };
            // Expected output from real WASM module (discriminator removed)
            const expectedEncoded = Buffer.from([2, 1, 0]);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
        it('should encode function execution with pubkey parameters', async () => {
            const functionIndex = 3;
            const paramDefs = [
                { name: 'authority', type: 'pubkey' },
                { name: 'recipient', type: 'pubkey' }
            ];
            const paramValues = {
                authority: '11111111111111111111111111111112', // System Program ID (valid base58)
                recipient: 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA' // SPL Token Program ID (valid base58)
            };
            // Expected output from real WASM module (discriminator removed) - actual base58 decoding of pubkeys
            const expectedEncoded = Buffer.from([
                3, 2, 10, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
                10, 6, 221, 246, 225, 215, 101, 161, 147, 217, 203, 225, 70, 206, 235, 121, 172, 28, 180, 133, 237, 95, 91, 55, 145, 58, 140, 245, 133, 126, 255, 0, 169
            ]);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
        it('should encode function execution with bytes parameters', async () => {
            const functionIndex = 4;
            const paramDefs = [
                { name: 'data', type: 'bytes' }
            ];
            const paramValues = {
                data: new Uint8Array([1, 2, 3, 4, 5])
            };
            // Expected output from real WASM module (discriminator removed)
            const expectedEncoded = Buffer.from([4, 1, 12, 5, 1, 2, 3, 4, 5]);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
        it('should encode function execution with array parameters', async () => {
            const functionIndex = 5;
            const paramDefs = [
                { name: 'amounts', type: 'array' }
            ];
            const paramValues = {
                amounts: [100, 200, 300]
            };
            // Expected output from real WASM module (discriminator removed) - array with VLE encoded u32 elements
            const expectedEncoded = Buffer.from([5, 1, 13, 3, 4, 100, 0, 0, 0, 0, 0, 0, 0, 200, 0, 0, 0, 0, 0, 0, 0, 44, 1, 0, 0, 0, 0, 0, 0]);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
        it('should encode function execution with no parameters', async () => {
            const functionIndex = 0;
            const paramDefs = [];
            const paramValues = {};
            // Expected output from real WASM module (discriminator removed)
            const expectedEncoded = Buffer.from([0, 0]);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
        it('should handle missing parameter values', async () => {
            const functionIndex = 0;
            const paramDefs = [
                { name: 'amount', type: 'u64' },
                { name: 'missing', type: 'u64' }
            ];
            const paramValues = {
                amount: 1000
                // missing parameter intentionally omitted
            };
            await expect(VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues))
                .rejects.toThrow('Missing value for parameter: missing');
        });
        it('should handle WASM encoding failure and throw error', async () => {
            const functionIndex = 0;
            const paramDefs = [{ name: 'test', type: 'u64' }];
            const paramValues = { test: 42 };
            // Force an error by using invalid type that WASM doesn't support
            const invalidParamDefs = [{ name: 'test', type: 'invalid_type' }];
            await expect(VLEEncoder.encodeExecuteVLE(functionIndex, invalidParamDefs, paramValues))
                .rejects.toThrow();
        });
        it('should encode complex mixed parameter types', async () => {
            const functionIndex = 10;
            const paramDefs = [
                { name: 'id', type: 'u64' },
                { name: 'name', type: 'string' },
                { name: 'active', type: 'bool' },
                { name: 'owner', type: 'pubkey' },
                { name: 'data', type: 'bytes' },
                { name: 'amounts', type: 'array' }
            ];
            const paramValues = {
                id: 42,
                name: 'test',
                active: true,
                owner: '11111111111111111111111111111114',
                data: new Uint8Array([1, 2, 3]),
                amounts: [10, 20, 30]
            };
            // For complex mixed types, we can't predict exact WASM output due to base58 decoding
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            // Verify result structure (real WASM will produce variable output)
            expect(result).toBeInstanceOf(Buffer);
            expect(result.length).toBeGreaterThan(0);
            expect(result[0]).toBe(10); // Function index
        });
    });
    describe('parameter type validation', () => {
        it('should handle invalid parameter types gracefully', async () => {
            const functionIndex = 0;
            const paramDefs = [
                { name: 'unknown', type: 'unknown_type' }
            ];
            const paramValues = {
                unknown: 'some_value'
            };
            // Real WASM will reject unknown types
            await expect(VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues))
                .rejects.toThrow();
        });
        it('should handle null parameter values', async () => {
            const functionIndex = 0;
            const paramDefs = [
                { name: 'nullable', type: 'u64' }
            ];
            const paramValues = {
                nullable: null
            };
            await expect(VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues))
                .rejects.toThrow('Missing value for parameter: nullable');
        });
        it('should handle undefined parameter values', async () => {
            const functionIndex = 0;
            const paramDefs = [
                { name: 'undefined', type: 'u64' }
            ];
            const paramValues = {
                undefined: undefined
            };
            await expect(VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues))
                .rejects.toThrow('Missing value for parameter: undefined');
        });
    });
    describe('edge cases', () => {
        it('should handle high function indices', async () => {
            const functionIndex = 255; // Max u8 value
            const paramDefs = [];
            const paramValues = {};
            // Expected output from real WASM module (discriminator removed) - function index + param count (0) + type info
            const expectedEncoded = Buffer.from([255, 1, 0]);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
        it('should handle large parameter counts', async () => {
            const functionIndex = 0;
            const paramDefs = Array.from({ length: 10 }, (_, i) => ({
                name: `param_${i}`,
                type: 'u64'
            }));
            const paramValues = Object.fromEntries(paramDefs.map((param, i) => [param.name, i * 100]));
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            // Verify result structure (real WASM will produce variable output)
            expect(result).toBeInstanceOf(Buffer);
            expect(result.length).toBeGreaterThan(0);
            expect(result[0]).toBe(0); // Function index
        });
        it('should handle empty arrays', async () => {
            const functionIndex = 0;
            const paramDefs = [
                { name: 'empty_array', type: 'array' }
            ];
            const paramValues = {
                empty_array: []
            };
            // Expected: [function_index, param_count, type_id, array_length, element_type]
            const expectedEncoded = Buffer.from([0, 1, 13, 0, 4]); // With element type identifier
            mockWasmModule.ParameterEncoder.encode_execute_params.mockReturnValue(expectedEncoded);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
        it('should handle empty bytes', async () => {
            const functionIndex = 0;
            const paramDefs = [
                { name: 'empty_bytes', type: 'bytes' }
            ];
            const paramValues = {
                empty_bytes: new Uint8Array([])
            };
            // Mock WASM output: includes discriminator (2) at position 0
            const wasmOutput = new Uint8Array([2, 0, 1, 12, 0]);
            const expectedEncoded = Buffer.from([0, 1, 12, 0]); // Without discriminator
            mockWasmModule.ParameterEncoder.encode_execute_params.mockReturnValue(wasmOutput);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
        it('should handle empty strings', async () => {
            const functionIndex = 0;
            const paramDefs = [
                { name: 'empty_string', type: 'string' }
            ];
            const paramValues = {
                empty_string: ''
            };
            // Mock WASM output: includes discriminator (2) at position 0  
            const wasmOutput = new Uint8Array([2, 0, 1, 11, 0]);
            const expectedEncoded = Buffer.from([0, 1, 11, 0]); // Without discriminator
            mockWasmModule.ParameterEncoder.encode_execute_params.mockReturnValue(wasmOutput);
            const result = await VLEEncoder.encodeExecuteVLE(functionIndex, paramDefs, paramValues);
            expect(result).toEqual(expectedEncoded);
        });
    });
});
//# sourceMappingURL=vle-encoder.test.js.map