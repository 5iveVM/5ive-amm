
import { describe, it, expect } from '@jest/globals';
import {
  normalizeAbiFunctions,
  findFunctionInABI,
  resolveFunctionIndex
} from '../../utils/abi.js';

describe('ABI Utilities', () => {
  describe('normalizeAbiFunctions', () => {
    it('should normalize array format ABI functions', () => {
      const arrayAbi = [
        { name: 'func1', index: 0, parameters: [] },
        { name: 'func2', index: 1, parameters: [{ name: 'p1', type: 'u64' }] }
      ];

      const normalized = normalizeAbiFunctions(arrayAbi);

      expect(normalized).toHaveLength(2);
      expect(normalized[0]).toMatchObject({
        name: 'func1',
        index: 0,
        parameters: [],
        visibility: 'public'
      });
      expect(normalized[1]).toMatchObject({
        name: 'func2',
        index: 1,
        parameters: [{ name: 'p1', type: 'u64', optional: false, isAccount: false }],
        visibility: 'public'
      });
    });

    it('should normalize object format ABI functions', () => {
      const objectAbi = {
        'func1': { index: 0, parameters: [] },
        'func2': { index: 1, parameters: [{ name: 'p1', type: 'u64' }] }
      };

      const normalized = normalizeAbiFunctions(objectAbi);

      expect(normalized).toHaveLength(2);
      // Order is preserved or sorted by index? implementation sorts by index
      expect(normalized[0].name).toBe('func1');
      expect(normalized[1].name).toBe('func2');
    });

    it('should handle missing parameters', () => {
      const abi = [{ name: 'func1', index: 0 }];
      const normalized = normalizeAbiFunctions(abi);
      expect(normalized[0].parameters).toEqual([]);
    });

    it('should handle missing visibility', () => {
      const abi = [{ name: 'func1', index: 0, is_public: false }];
      const normalized = normalizeAbiFunctions(abi);
      expect(normalized[0].visibility).toBe('private');
    });

    it('should handle null or undefined input', () => {
        expect(normalizeAbiFunctions(null)).toEqual([]);
        expect(normalizeAbiFunctions(undefined)).toEqual([]);
    });
  });

  describe('findFunctionInABI', () => {
    const abi = [
      { name: 'module::func', index: 0 },
      { name: 'simple_func', index: 1 },
      { name: 'other::module::nested', index: 2 }
    ];

    it('should find exact match', () => {
      const func = findFunctionInABI(abi, 'simple_func');
      expect(func).toBeDefined();
      expect(func?.name).toBe('simple_func');
    });

    it('should find qualified name match', () => {
      const func = findFunctionInABI(abi, 'module::func');
      expect(func).toBeDefined();
      expect(func?.name).toBe('module::func');
    });

    it('should find function by unqualified name in qualified ABI', () => {
      const func = findFunctionInABI(abi, 'func'); // matches module::func
      expect(func).toBeDefined();
      expect(func?.name).toBe('module::func');
    });

    it('should find function by partial qualified name', () => {
        const func = findFunctionInABI(abi, 'module::nested'); // matches other::module::nested
        expect(func).toBeDefined();
        expect(func?.name).toBe('other::module::nested');
    });

    it('should return undefined if not found', () => {
      const func = findFunctionInABI(abi, 'non_existent');
      expect(func).toBeUndefined();
    });
  });

  describe('resolveFunctionIndex', () => {
    it('should resolve index from array ABI', () => {
      const abi = {
        functions: [
          { name: 'func1', index: 10 }
        ]
      };
      expect(resolveFunctionIndex(abi, 'func1')).toBe(10);
    });

    it('should resolve index from object ABI', () => {
      const abi = {
        functions: {
          'func1': { index: 20 }
        }
      };
      expect(resolveFunctionIndex(abi, 'func1')).toBe(20);
    });

    it('should throw if ABI is missing functions', () => {
      expect(() => resolveFunctionIndex({}, 'func1')).toThrow(/No ABI information available/);
    });

    it('should throw if function not found', () => {
      const abi = { functions: [{ name: 'func1', index: 0 }] };
      expect(() => resolveFunctionIndex(abi, 'func2')).toThrow(/Function 'func2' not found/);
    });
  });
});
