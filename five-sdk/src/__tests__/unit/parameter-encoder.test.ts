
import { describe, it, expect, beforeEach, jest } from '@jest/globals';

// Mock VLEEncoder to allow controlling success/failure
const mockEncodeExecuteVLE = jest.fn();

jest.unstable_mockModule('../../lib/vle-encoder.js', () => ({
  VLEEncoder: {
    encodeExecuteVLE: mockEncodeExecuteVLE
  }
}));

// Dynamic import of the module under test after mocking
// We use beforeAll to handle the async import to avoid top-level await issues in some environments
let ParameterEncoder: any;
let ParameterEncodingError: any;

describe('ParameterEncoder', () => {
  let encoder: any;

  beforeAll(async () => {
      const module = await import('../../encoding/ParameterEncoder.js');
      ParameterEncoder = module.ParameterEncoder;

      const typesModule = await import('../../types.js');
      ParameterEncodingError = typesModule.ParameterEncodingError;
  });

  beforeEach(() => {
    encoder = new ParameterEncoder(false); // disable debug logs
    jest.clearAllMocks();
  });

  describe('coerceValue', () => {
    it('should coerce u8', () => {
      expect(encoder.coerceValue(10, 'u8')).toBe(10);
      expect(encoder.coerceValue('10', 'u8')).toBe(10);
      expect(() => encoder.coerceValue(256, 'u8')).toThrow();
      expect(() => encoder.coerceValue(-1, 'u8')).toThrow();
    });

    it('should coerce u64', () => {
      expect(encoder.coerceValue(123456789, 'u64')).toBe(123456789);
      expect(encoder.coerceValue(BigInt(123), 'u64')).toBe(BigInt(123));
      expect(() => encoder.coerceValue(-1, 'u64')).toThrow();
    });

    it('should coerce bool', () => {
      expect(encoder.coerceValue(true, 'bool')).toBe(true);
      expect(encoder.coerceValue(false, 'bool')).toBe(false);
      expect(encoder.coerceValue('true', 'bool')).toBe(true);
      expect(encoder.coerceValue('false', 'bool')).toBe(false);
      expect(encoder.coerceValue(1, 'bool')).toBe(true);
      expect(encoder.coerceValue(0, 'bool')).toBe(false);
    });

    it('should coerce string', () => {
        expect(encoder.coerceValue(123, 'string')).toBe('123');
        expect(encoder.coerceValue('hello', 'string')).toBe('hello');
    });

    it('should coerce array', () => {
        const arr = [1, 2, 3];
        expect(encoder.coerceValue(arr, 'array')).toBe(arr);
        expect(() => encoder.coerceValue('not array', 'array')).toThrow();
    });

    it('should coerce bytes', () => {
        const bytes = new Uint8Array([1, 2]);
        expect(encoder.coerceValue(bytes, 'bytes')).toBe(bytes);
        expect(encoder.coerceValue([1, 2], 'bytes')).toBeInstanceOf(Uint8Array);
        // hex string
        expect(encoder.coerceValue('0102', 'bytes')).toBeInstanceOf(Uint8Array);
    });
  });

  describe('encodeParametersWithABI', () => {
    const functionSignature = {
      name: 'test_func',
      index: 0,
      parameters: [
        { name: 'p1', type: 'u8' },
        { name: 'p2', type: 'bool' }
      ]
    };

    it('should encode parameters based on ABI', () => {
      const params = [10, true];
      const encoded = encoder.encodeParametersWithABI(params, functionSignature);

      expect(encoded).toHaveLength(2);
      expect(encoded[0]).toEqual({ type: 1, value: 10 }); // u8 -> 1
      expect(encoded[1]).toEqual({ type: 9, value: true }); // bool -> 9
    });

    it('should infer type if ABI type missing', () => {
      const incompleteSig = {
        name: 'test_func',
        index: 0,
        parameters: [] // No parameters defined
      };
      // But we pass parameters
      const params = [10];
      const encoded = encoder.encodeParametersWithABI(params, incompleteSig);

      // Should infer u64 (4) for integer
      expect(encoded[0].type).toBe(4);
      expect(encoded[0].value).toBe(10);
    });

    it('should throw if strict mode enabled and param mismatch', () => {
      const params = [10];
      const emptySig = { name: 'f', index: 0, parameters: [] };

      expect(() => {
        encoder.encodeParametersWithABI(params, emptySig, { strict: true });
      }).toThrow(ParameterEncodingError);
    });
  });

  describe('encodeParameterData (Manual Fallback)', () => {
    it('should fallback to manual encoding if VLE encoder throws or fails to load', async () => {
      // In this test environment, loading VLEEncoder fails due to import.meta usage.
      // This conveniently forces the fallback path which we want to test here.

      const params = [10, true]; // u64 (inferred), bool (inferred)
      // u64 -> type 4
      // bool -> type 9

      // Expected manual encoding:
      // VLE count (2) -> 0x02
      // Param 1: Type(4), Value(10 as u64 LE) -> 04 0A 00 00 00 00 00 00 00
      // Param 2: Type(9), Value(1) -> 09 01

      const result = await encoder.encodeParameterData(params);

      // We verify the result matches manual encoding expectation
      expect(result.length).toBe(1 + (1 + 8) + (1 + 1)); // 1 (count) + 9 (param1) + 2 (param2) = 12 bytes

      expect(result[0]).toBe(2); // Count
      expect(result[1]).toBe(4); // Type u64
      expect(result[2]).toBe(10); // Value 10
      expect(result[10]).toBe(9); // Type bool
      expect(result[11]).toBe(1); // Value true
    });
  });
});
