/**
 * Unit tests for FiveProgram class
 */

import { FiveProgram } from '../../../program/FiveProgram.js';
import { ProgramIdResolver } from '../../../config/ProgramIdResolver.js';
import type { ScriptABI, FunctionDefinition } from '../../../metadata/index.js';

describe('FiveProgram', () => {
  // Set a valid default program ID for tests
  beforeEach(() => {
    ProgramIdResolver.setDefault('TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP');
  });

  afterEach(() => {
    ProgramIdResolver.clearDefault();
  });
  // Mock ABI for testing
  const mockABI: ScriptABI = {
    name: 'TestProgram',
    functions: [
      {
        name: 'initialize',
        index: 0,
        parameters: [
          {
            name: 'account',
            type: 'Account',
            is_account: true,
            attributes: ['mut', 'init'],
          },
          {
            name: 'signer',
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
        name: 'transfer',
        index: 1,
        parameters: [
          {
            name: 'from',
            type: 'Account',
            is_account: true,
            attributes: ['mut', 'signer'],
          },
          {
            name: 'to',
            type: 'Account',
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

  const SCRIPT_ACCOUNT = '5w4epP8qZS4STiUDhj1jgJL4yqYPJnNvQTYYKVWfRQSZ';

  describe('fromABI', () => {
    it('should create FiveProgram from ABI', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      expect(program.getScriptAccount()).toBe(SCRIPT_ACCOUNT);
      expect(program.getABI()).toBe(mockABI);
      expect(program.getFunctions()).toEqual(['initialize', 'transfer']);
    });

    it('should use provided options', () => {
      const validProgramId = 'ATokenGPvbdGVqstVQmcLsNZAqeEbtQvvHta7h1Vvta';
      const options = { debug: true, fiveVMProgramId: validProgramId };
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI, options);
      expect(program.getOptions().debug).toBe(true);
      expect(program.getFiveVMProgramId()).toBe(validProgramId);
    });

    it('should use default Five VM program ID if not provided', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      expect(program.getFiveVMProgramId()).toBe('TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP');
    });
  });

  describe('getFunctions', () => {
    it('should return list of function names', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const functions = program.getFunctions();
      expect(functions).toEqual(['initialize', 'transfer']);
    });

    it('should return empty array for ABI with no functions', () => {
      const emptyABI: ScriptABI = { name: 'Empty', functions: [] };
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, emptyABI);
      expect(program.getFunctions()).toEqual([]);
    });
  });

  describe('function', () => {
    it('should return FunctionBuilder for valid function', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program.function('initialize');
      expect(builder.getFunctionDef().name).toBe('initialize');
      expect(builder.getFunctionDef().index).toBe(0);
    });

    it('should throw error for non-existent function', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      expect(() => program.function('nonexistent')).toThrowError(
        "Function 'nonexistent' not found in ABI"
      );
    });

    it('should list available functions in error message', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      try {
        program.function('invalid');
        throw new Error('Should have thrown');
      } catch (error) {
        expect((error as Error).message).toContain('Available: initialize, transfer');
      }
    });
  });

  describe('getFunction', () => {
    it('should return function definition by name', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const func = program.getFunction('initialize');
      expect(func).toBeDefined();
      expect(func?.name).toBe('initialize');
      expect(func?.index).toBe(0);
    });

    it('should return undefined for non-existent function', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const func = program.getFunction('nonexistent');
      expect(func).toBeUndefined();
    });
  });

  describe('getAllFunctions', () => {
    it('should return all function definitions', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const functions = program.getAllFunctions();
      expect(functions).toHaveLength(2);
      expect(functions[0].name).toBe('initialize');
      expect(functions[1].name).toBe('transfer');
    });
  });

  describe('generateTypes', () => {
    it('should generate TypeScript types', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const types = program.generateTypes();
      expect(types).toContain('interface');
      expect(types).toContain('TestProgramProgram');
      expect(types).toContain('initialize');
      expect(types).toContain('transfer');
    });

    it('should include function parameters in generated types', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const types = program.generateTypes();
      expect(types).toContain('InitializeParams');
      expect(types).toContain('TransferParams');
    });
  });

  describe('getScriptAccount', () => {
    it('should return script account address', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      expect(program.getScriptAccount()).toBe(SCRIPT_ACCOUNT);
    });
  });

  describe('getABI', () => {
    it('should return the ABI', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      expect(program.getABI()).toBe(mockABI);
    });
  });

  describe('findAddress', () => {
    it('derives script-scoped PDA with implicit script seed prefix', async () => {
      const { PublicKey } = await import('@solana/web3.js');
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const vmProgramId = program.getFiveVMProgramId();

      const [derived, bump] = await program.findAddress(['vault']);
      const [expected, expectedBump] = PublicKey.findProgramAddressSync(
        [new PublicKey(SCRIPT_ACCOUNT).toBuffer(), Buffer.from('vault')],
        new PublicKey(vmProgramId)
      );

      expect(derived).toBe(expected.toBase58());
      expect(bump).toBe(expectedBump);
    });

    it('changes PDA when script account changes even with same user seeds', async () => {
      const programA = FiveProgram.fromABI(
        '5w4epP8qZS4STiUDhj1jgJL4yqYPJnNvQTYYKVWfRQSZ',
        mockABI
      );
      const programB = FiveProgram.fromABI(
        '11111111111111111111111111111111',
        mockABI
      );

      const [a] = await programA.findAddress(['vault']);
      const [b] = await programB.findAddress(['vault']);

      expect(a).not.toBe(b);
    });
  });
});
