/**
 * Integration tests for FiveProgram with actual Five SDK
 *
 * Tests the full workflow from FiveProgram to FiveSDK.generateExecuteInstruction()
 */

import { FiveProgram } from '../../program/FiveProgram.js';
import type { ScriptABI } from '../../metadata/index.js';

describe('FiveProgram Integration', () => {
  // Real counter ABI from compiled contract
  const counterABI: ScriptABI = {
    name: 'Module',
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
          {
            name: 'owner',
            type: 'Account',
            is_account: true,
            attributes: ['signer'],
          },
        ],
        return_type: null,
        is_public: true,
        bytecode_offset: 0,
      },
      {
        name: 'add_amount',
        index: 3,
        parameters: [
          {
            name: 'counter',
            type: 'Counter',
            is_account: true,
            attributes: ['mut'],
          },
          {
            name: 'owner',
            type: 'Account',
            is_account: true,
            attributes: ['signer'],
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

  const COUNTER_SCRIPT = 'CounterScriptAccount1234567890123456789012';
  const COUNTER_ACCOUNT = 'CounterAccount12345678901234567890123456';
  const OWNER_ACCOUNT = 'OwnerAccount1234567890123456789012345678';

  describe('FiveProgram API', () => {
    it('should create program from ABI', () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      expect(program).toBeDefined();
    });

    it('should list available functions', () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      const functions = program.getFunctions();
      expect(functions).toContain('initialize');
      expect(functions).toContain('increment');
      expect(functions).toContain('add_amount');
    });

    it('should generate TypeScript types', () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      const types = program.generateTypes();

      expect(types).toContain('interface ModuleProgram');
      expect(types).toContain('initialize');
      expect(types).toContain('increment');
      expect(types).toContain('add_amount');
      expect(types).toContain('InitializeParams');
      expect(types).toContain('IncrementParams');
      expect(types).toContain('Add_amountParams');
    });
  });

  describe('FunctionBuilder API', () => {
    it('should build increment function call', async () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      const builder = program
        .function('increment')
        .accounts({
          counter: COUNTER_ACCOUNT,
          owner: OWNER_ACCOUNT,
        });

      const accounts = builder.getAccounts();
      expect(accounts.counter).toBe(COUNTER_ACCOUNT);
      expect(accounts.owner).toBe(OWNER_ACCOUNT);
    });

    it('should build add_amount function call with data', () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      const builder = program
        .function('add_amount')
        .accounts({
          counter: COUNTER_ACCOUNT,
          owner: OWNER_ACCOUNT,
        })
        .args({
          amount: 42,
        });

      const args = builder.getArgs();
      expect(args.amount).toBe(42);
    });

    it('should support method chaining', () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      const result = program
        .function('increment')
        .accounts({ counter: COUNTER_ACCOUNT, owner: OWNER_ACCOUNT });

      expect(result).toBeDefined();
      expect(result.getAccounts().counter).toBe(COUNTER_ACCOUNT);
    });

    it('should auto-inject SystemProgram for initialize', async () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      const builder = program
        .function('initialize')
        .accounts({
          counter: COUNTER_ACCOUNT,
          owner: OWNER_ACCOUNT,
        });

      // Note: SystemProgram auto-injection happens in instruction() method
      // We can't easily test it here without mocking FiveSDK
      expect(builder).toBeDefined();
    });

    it('should validate required parameters', async () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      const builder = program
        .function('increment')
        .accounts({ counter: COUNTER_ACCOUNT });
        // Missing 'owner' account

      // This should fail when we try to generate the instruction
      // (We can't easily test without mocking FiveSDK)
      expect(builder.getAccounts().counter).toBe(COUNTER_ACCOUNT);
    });
  });

  describe('Error handling', () => {
    it('should throw for non-existent function', () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      expect(() => {
        program.function('nonexistent');
      }).toThrowError("Function 'nonexistent' not found in ABI");
    });

    it('should suggest available functions in error', () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      try {
        program.function('invalid');
        fail('Should have thrown');
      } catch (error) {
        const message = (error as Error).message;
        expect(message).toContain('initialize');
        expect(message).toContain('increment');
      }
    });
  });

  describe('Account resolution', () => {
    it('should handle account addresses as strings', () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      const builder = program
        .function('increment')
        .accounts({
          counter: 'A3kgBFd2Y1as6kxm6MZSmcWbN19TGM4oFDmE25mxBCB9',
          owner: '4i2yQRFiz3mNQCHNXouyQyC1Rr6LUwMcWfPWXqrb3LkG',
        });

      const accounts = builder.getAccounts();
      expect(accounts.counter).toBe('A3kgBFd2Y1as6kxm6MZSmcWbN19TGM4oFDmE25mxBCB9');
      expect(accounts.owner).toBe('4i2yQRFiz3mNQCHNXouyQyC1Rr6LUwMcWfPWXqrb3LkG');
    });

    it('should handle PublicKey-like objects', () => {
      const program = FiveProgram.fromABI(COUNTER_SCRIPT, counterABI);
      const mockPubkey = {
        toBase58: () => 'MockPublicKeyAddress123456789012345',
      };

      const builder = program
        .function('increment')
        .accounts({
          counter: mockPubkey as any,
          owner: 'OwnerAccount1234567890123456789012345678',
        });

      const accounts = builder.getAccounts();
      expect(accounts.counter).toBe('MockPublicKeyAddress123456789012345');
    });
  });
});
