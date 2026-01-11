/**
 * Unit tests for TypeGenerator class
 */

import { TypeGenerator } from '../../../program/TypeGenerator.js';
import type { ScriptABI } from '../../../metadata/index.js';

describe('TypeGenerator', () => {
  const mockABI: ScriptABI = {
    name: 'Counter',
    functions: [
      {
        name: 'initialize',
        index: 0,
        parameters: [
          {
            name: 'counter',
            type: 'Counter',
            is_account: true,
            attributes: ['mut', 'init'],
          },
          {
            name: 'owner',
            type: 'Account',
            is_account: true,
            attributes: ['signer'],
          },
        ],
        return_type: 'pubkey',
        is_public: true,
        bytecode_offset: 0,
      },
      {
        name: 'increment',
        index: 1,
        parameters: [
          {
            name: 'counter',
            type: 'Counter',
            is_account: true,
            attributes: ['mut'],
          },
        ],
        return_type: null,
        is_public: true,
        bytecode_offset: 0,
      },
      {
        name: 'add_amount',
        index: 2,
        parameters: [
          {
            name: 'counter',
            type: 'Counter',
            is_account: true,
            attributes: ['mut'],
          },
          {
            name: 'amount',
            type: 'u64',
            is_account: false,
            attributes: [],
          },
        ],
        return_type: null,
        is_public: true,
        bytecode_offset: 0,
      },
    ],
  };

  describe('generate', () => {
    it('should generate TypeScript interface', () => {
      const generator = new TypeGenerator(mockABI);
      const types = generator.generate();

      expect(types).toContain('interface CounterProgram');
      expect(types).toContain('initialize');
      expect(types).toContain('increment');
      expect(types).toContain('add_amount');
    });

    it('should generate parameter types for each function', () => {
      const generator = new TypeGenerator(mockABI);
      const types = generator.generate();

      expect(types).toContain('interface InitializeParams');
      expect(types).toContain('interface IncrementParams');
      expect(types).toContain('interface Add_amountParams');
    });

    it('should include accounts object for functions with account parameters', () => {
      const generator = new TypeGenerator(mockABI);
      const types = generator.generate();

      expect(types).toContain('accounts: {');
      expect(types).toContain('counter:');
      expect(types).toContain('owner:');
    });

    it('should include args object for functions with data parameters', () => {
      const generator = new TypeGenerator(mockABI);
      const types = generator.generate();

      expect(types).toContain('args: {');
      expect(types).toContain('amount:');
    });

    it('should use custom script name in interface', () => {
      const generator = new TypeGenerator(mockABI, { scriptName: 'MyScript' });
      const types = generator.generate();

      expect(types).toContain('interface MyScriptProgram');
    });

    it('should include JSDoc comments by default', () => {
      const generator = new TypeGenerator(mockABI, { includeJSDoc: true });
      const types = generator.generate();

      expect(types).toContain('/**');
      expect(types).toContain('* Call initialize()');
    });

    it('should exclude JSDoc comments when disabled', () => {
      const generator = new TypeGenerator(mockABI, { includeJSDoc: false });
      const types = generator.generate();

      expect(types).toContain('initialize(params: InitializeParams)');
      // Should not have the comment line above it
      const lines = types.split('\n');
      const initLineIndex = lines.findIndex((l) =>
        l.includes('initialize(params: InitializeParams)')
      );
      const commentLine = initLineIndex > 0 ? lines[initLineIndex - 1] : '';
      expect(commentLine).not.toContain('* Call initialize()');
    });

    it('should handle empty ABI', () => {
      const emptyABI: ScriptABI = { name: 'Empty', functions: [] };
      const generator = new TypeGenerator(emptyABI);
      const types = generator.generate();

      expect(types).toContain('interface EmptyProgram');
      expect(types).toContain('}');
    });

    it('should include header comment', () => {
      const generator = new TypeGenerator(mockABI);
      const types = generator.generate();

      expect(types).toContain('Auto-generated types');
      expect(types).toContain('Generated from ABI');
    });
  });

  describe('type conversion', () => {
    it('should convert u64 to number | bigint', () => {
      const abi: ScriptABI = {
        name: 'Test',
        functions: [
          {
            name: 'test',
            index: 0,
            parameters: [
              {
                name: 'value',
                type: 'u64',
                is_account: false,
                attributes: [],
              },
            ],
            return_type: null,
            is_public: true,
            bytecode_offset: 0,
          },
        ],
      };

      const generator = new TypeGenerator(abi);
      const types = generator.generate();

      expect(types).toContain('value: number | bigint;');
    });

    it('should convert bool to boolean', () => {
      const abi: ScriptABI = {
        name: 'Test',
        functions: [
          {
            name: 'test',
            index: 0,
            parameters: [
              {
                name: 'flag',
                type: 'bool',
                is_account: false,
                attributes: [],
              },
            ],
            return_type: null,
            is_public: true,
            bytecode_offset: 0,
          },
        ],
      };

      const generator = new TypeGenerator(abi);
      const types = generator.generate();

      expect(types).toContain('flag: boolean;');
    });

    it('should convert string to string', () => {
      const abi: ScriptABI = {
        name: 'Test',
        functions: [
          {
            name: 'test',
            index: 0,
            parameters: [
              {
                name: 'text',
                type: 'string',
                is_account: false,
                attributes: [],
              },
            ],
            return_type: null,
            is_public: true,
            bytecode_offset: 0,
          },
        ],
      };

      const generator = new TypeGenerator(abi);
      const types = generator.generate();

      expect(types).toContain('text: string;');
    });

    it('should convert account types to pubkey union', () => {
      const abi: ScriptABI = {
        name: 'Test',
        functions: [
          {
            name: 'test',
            index: 0,
            parameters: [
              {
                name: 'account',
                type: 'Account',
                is_account: true,
                attributes: [],
              },
            ],
            return_type: null,
            is_public: true,
            bytecode_offset: 0,
          },
        ],
      };

      const generator = new TypeGenerator(abi);
      const types = generator.generate();

      expect(types).toContain('account: string | { toBase58(): string };');
    });

    it('should handle unknown types as any', () => {
      const abi: ScriptABI = {
        name: 'Test',
        functions: [
          {
            name: 'test',
            index: 0,
            parameters: [
              {
                name: 'unknown',
                type: 'UnknownType',
                is_account: false,
                attributes: [],
              },
            ],
            return_type: null,
            is_public: true,
            bytecode_offset: 0,
          },
        ],
      };

      const generator = new TypeGenerator(abi);
      const types = generator.generate();

      expect(types).toContain('unknown: any;');
    });
  });

  describe('getABIStructure', () => {
    it('should return ABI structure', () => {
      const generator = new TypeGenerator(mockABI);
      const structure = generator.getABIStructure();

      expect(structure.programName).toBe('CounterProgram');
      expect(structure.functions).toHaveLength(3);
      expect(structure.functions[0].name).toBe('initialize');
    });

    it('should use custom script name', () => {
      const generator = new TypeGenerator(mockABI, { scriptName: 'Custom' });
      const structure = generator.getABIStructure();

      expect(structure.programName).toBe('CustomProgram');
    });
  });
});
