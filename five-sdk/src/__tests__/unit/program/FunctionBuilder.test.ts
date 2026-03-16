/**
 * Unit tests for FunctionBuilder class
 */

import { FunctionBuilder } from '../../../program/FunctionBuilder.js';
import { FiveProgram } from '../../../program/FiveProgram.js';
import { ProgramIdResolver } from '../../../config/ProgramIdResolver.js';
import type { ScriptABI, FunctionDefinition } from '../../../metadata/index.js';

describe('FunctionBuilder', () => {
  beforeEach(() => {
    ProgramIdResolver.setDefault('TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP');
  });

  afterEach(() => {
    ProgramIdResolver.clearDefault();
  });
  const mockABI: ScriptABI = {
    name: 'TestProgram',
    functions: [
      {
        name: 'transfer',
        index: 0,
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

  const SCRIPT_ACCOUNT = 'So11111111111111111111111111111111111111112';
  const FROM_ACCOUNT = 'SysvarRent111111111111111111111111111111111';
  const TO_ACCOUNT = 'SysvarC1ock11111111111111111111111111111111';

  describe('accounts', () => {
    it('should accept string addresses', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program.function('transfer');

      builder.accounts({
        from: FROM_ACCOUNT,
        to: TO_ACCOUNT,
      });

      const accounts = builder.getAccounts();
      expect(accounts.from).toBe(FROM_ACCOUNT);
      expect(accounts.to).toBe(TO_ACCOUNT);
    });

    it('should accept PublicKey-like objects', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program.function('transfer');

      const mockPublicKey = {
        toBase58: () => FROM_ACCOUNT,
      };

      builder.accounts({
        from: mockPublicKey as any,
        to: TO_ACCOUNT,
      });

      const accounts = builder.getAccounts();
      expect(accounts.from).toBe(FROM_ACCOUNT);
    });

    it('should support method chaining', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program.function('transfer');

      const result = builder.accounts({
        from: FROM_ACCOUNT,
        to: TO_ACCOUNT,
      });

      expect(result).toBe(builder);
    });

    it('should accumulate accounts across multiple calls', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program.function('transfer');

      builder.accounts({ from: FROM_ACCOUNT });
      builder.accounts({ to: TO_ACCOUNT });

      const accounts = builder.getAccounts();
      expect(accounts.from).toBe(FROM_ACCOUNT);
      expect(accounts.to).toBe(TO_ACCOUNT);
    });
  });

  describe('args', () => {
    it('should accept data parameters', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program.function('transfer');

      builder.args({ amount: 1000 });

      const args = builder.getArgs();
      expect(args.amount).toBe(1000);
    });

    it('should support various types', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program.function('transfer');

      builder.args({ amount: 1000 });

      const args = builder.getArgs();
      expect(typeof args.amount).toBe('number');
    });

    it('should support method chaining', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program.function('transfer');

      const result = builder.args({ amount: 100 });

      expect(result).toBe(builder);
    });
  });

  describe('instruction', () => {
    it('should throw error when required account is missing', async () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program
        .function('transfer')
        .accounts({ from: FROM_ACCOUNT })
        .args({ amount: 100 });

      await expect(builder.instruction()).rejects.toThrow(
        "Missing required account 'to'"
      );
    });

    it('should throw error when required argument is missing', async () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program
        .function('transfer')
        .accounts({ from: FROM_ACCOUNT, to: TO_ACCOUNT });

      await expect(builder.instruction()).rejects.toThrow(
        "Missing required argument 'amount'"
      );
    });

    it('should generate instruction when all parameters provided', async () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program
        .function('transfer')
        .accounts({ from: FROM_ACCOUNT, to: TO_ACCOUNT })
        .args({ amount: 100 });

      const instruction = await builder.instruction();

      expect(instruction).toBeDefined();
      expect(instruction.programId).toBe(program.getFiveVMProgramId());
      expect(instruction.keys).toBeDefined();
      expect(instruction.keys.length).toBeGreaterThan(0);
      expect(instruction.data).toBeDefined();
    });

    it('should include correct account metadata', async () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program
        .function('transfer')
        .accounts({ from: FROM_ACCOUNT, to: TO_ACCOUNT })
        .args({ amount: 100 });

      const instruction = await builder.instruction();

      const fromAcct = instruction.keys.find((a) => a.pubkey === FROM_ACCOUNT);
      const toAcct = instruction.keys.find((a) => a.pubkey === TO_ACCOUNT);

      expect(fromAcct).toBeDefined();
      expect(fromAcct?.isSigner).toBe(true);
      expect(fromAcct?.isWritable).toBe(true);

      expect(toAcct).toBeDefined();
      expect(toAcct?.isSigner).toBe(false);
      expect(toAcct?.isWritable).toBe(true);
    });
  });

  describe('getFunctionDef', () => {
    it('should return function definition', () => {
      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, mockABI);
      const builder = program.function('transfer');

      const funcDef = builder.getFunctionDef();
      expect(funcDef.name).toBe('transfer');
      expect(funcDef.index).toBe(0);
    });
  });

  describe('session auto-wiring', () => {
    it('auto-injects delegate and session accounts for play_ttt', async () => {
      const sessionAbi: ScriptABI = {
        name: 'BoardGames',
        functions: [
          {
            name: 'play_ttt',
            index: 1,
            parameters: [
              { name: 'match_state', type: 'Account', is_account: true, attributes: ['mut'] },
              { name: 'authority', type: 'Account', is_account: true, attributes: [] },
              { name: 'delegate', type: 'Account', is_account: true, attributes: ['signer'] },
              { name: 'session', type: 'Account', is_account: true, attributes: ['session'] },
              { name: 'cell_index', type: 'u64', is_account: false, attributes: [] },
            ],
            return_type: null,
            is_public: true,
            bytecode_offset: 0,
          },
        ],
      };

      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, sessionAbi).withSession({
        manager: { defaultTtlSlots: 3000 } as any,
        mode: 'auto',
        sessionAccountByFunction: { play_ttt: TO_ACCOUNT },
        delegateAccountByFunction: { play_ttt: FROM_ACCOUNT },
      });

      const instruction = await program
        .function('play_ttt')
        .accounts({
          match_state: TO_ACCOUNT,
          authority: FROM_ACCOUNT,
          delegate: FROM_ACCOUNT,
        })
        .payer(FROM_ACCOUNT)
        .args({ cell_index: 4 })
        .instruction();

      expect(instruction.keys.some((k) => k.pubkey === TO_ACCOUNT)).toBe(true);
      expect(instruction.keys.some((k) => k.pubkey === FROM_ACCOUNT)).toBe(true);
    });

    it('auto-wires compiler-injected implicit session account', async () => {
      const sessionAbi: ScriptABI = {
        name: 'BoardGames',
        functions: [
          {
            name: 'play_ttt',
            index: 1,
            parameters: [
              { name: 'match_state', type: 'Account', is_account: true, attributes: ['mut'] },
              { name: 'authority', type: 'Account', is_account: true, attributes: [] },
              { name: '__session', type: 'Account', is_account: true, implicit: true, source: 'compiler', attributes: ['session'] },
              { name: 'cell_index', type: 'u64', is_account: false, attributes: [] },
            ],
            return_type: null,
            is_public: true,
            bytecode_offset: 0,
          },
        ],
      };

      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, sessionAbi).withSession({
        manager: { defaultTtlSlots: 3000 } as any,
        mode: 'auto',
        sessionAccountByFunction: { play_ttt: TO_ACCOUNT },
        delegateAccountByFunction: { play_ttt: FROM_ACCOUNT },
      });

      const instruction = await program
        .function('play_ttt')
        .accounts({
          match_state: TO_ACCOUNT,
          authority: FROM_ACCOUNT,
        })
        .payer(FROM_ACCOUNT)
        .args({ cell_index: 4 })
        .instruction();

      expect(instruction.keys.some((k) => k.pubkey === TO_ACCOUNT)).toBe(true);
      expect(instruction.keys.some((k) => k.pubkey === FROM_ACCOUNT)).toBe(true);
    });

    it('falls back to owner for implicit __session in direct mode', async () => {
      const sessionAbi: ScriptABI = {
        name: 'BoardGames',
        functions: [
          {
            name: 'play_ttt',
            index: 1,
            parameters: [
              { name: 'match_state', type: 'Account', is_account: true, attributes: ['mut'] },
              { name: 'owner', type: 'Account', is_account: true, attributes: [] },
              { name: '__session', type: 'Account', is_account: true, implicit: true, source: 'compiler', attributes: ['session'] },
              { name: 'cell_index', type: 'u64', is_account: false, attributes: [] },
            ],
            return_type: null,
            is_public: true,
            bytecode_offset: 0,
          },
        ],
      };

      const program = FiveProgram.fromABI(SCRIPT_ACCOUNT, sessionAbi);
      const instruction = await program
        .function('play_ttt')
        .accounts({
          match_state: TO_ACCOUNT,
          owner: FROM_ACCOUNT,
        })
        .payer(FROM_ACCOUNT)
        .args({ cell_index: 4 })
        .instruction();

      const ownerKeyCount = instruction.keys.filter((k) => k.pubkey === FROM_ACCOUNT).length;
      expect(ownerKeyCount).toBeGreaterThan(0);
    });
  });
});
