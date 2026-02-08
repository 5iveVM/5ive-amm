import { describe, it, expect } from '@jest/globals';

import { BytecodeEncoder } from '../../lib/bytecode-encoder.js';

describe('BytecodeEncoder', () => {
  describe('getTypeId', () => {
    it('maps types to protocol ids (case-insensitive)', () => {
      expect(BytecodeEncoder.getTypeId('u8')).toBe(1);
      expect(BytecodeEncoder.getTypeId('U32')).toBe(3);
      expect(BytecodeEncoder.getTypeId('bool')).toBe(9);
      expect(BytecodeEncoder.getTypeId('bytes')).toBe(11);
    });

    it('throws on unknown types', () => {
      expect(() => BytecodeEncoder.getTypeId('weird-type')).toThrow(/Unknown type/);
    });
  });

  describe('encodeU32', () => {
    it('encodes little-endian bytes', () => {
      const encoded = BytecodeEncoder.encodeU32(0x12345678);
      expect(Array.from(encoded)).toEqual([0x78, 0x56, 0x34, 0x12]);
    });

    it('handles zero', () => {
      const encoded = BytecodeEncoder.encodeU32(0);
      expect(Array.from(encoded)).toEqual([0, 0, 0, 0]);
    });
  });

  describe('parseParameters', () => {
    it('parses array-form parameters into definitions and values', () => {
      const payload = JSON.stringify([
        { name: 'amount', type: 'u64', value: 42 },
        { name: 'flag', type: 'bool', value: false },
        { name: 'zero', type: 'u32', value: 0 }
      ]);

      const { definitions, values } = BytecodeEncoder.parseParameters(payload);

      expect(definitions).toEqual([
        { name: 'amount', type: 'u64' },
        { name: 'flag', type: 'bool' },
        { name: 'zero', type: 'u32' }
      ]);
      expect(values).toEqual({ amount: 42, flag: false, zero: 0 });
    });

    it('throws when parameter fields are missing', () => {
      const payload = JSON.stringify([{ name: 'a', type: 'u8' }]);
      expect(() => BytecodeEncoder.parseParameters(payload))
        .toThrow(/Parameter must have name, type, and value fields/);
    });

    it('rejects non-array payloads', () => {
      const payload = JSON.stringify({ a: 1 });
      expect(() => BytecodeEncoder.parseParameters(payload))
        .toThrow(/Object format parameters require type definitions/);
    });

    it('wraps JSON parse errors with a helpful message', () => {
      expect(() => BytecodeEncoder.parseParameters('{bad json'))
        .toThrow(/Invalid parameters JSON/);
    });
  });
});
