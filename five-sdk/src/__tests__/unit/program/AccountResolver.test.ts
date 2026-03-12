/**
 * Unit tests for AccountResolver class
 */

import { describe, expect, it, jest } from '@jest/globals';
import { AccountResolver } from '../../../program/AccountResolver.js';
import type { FunctionDefinition } from '../../../metadata/index.js';

describe('AccountResolver', () => {
  const SYSTEM_PROGRAM_ID = '11111111111111111111111111111111';

  describe('resolveSystemAccounts', () => {
    it('should auto-inject SystemProgram for @init constraint', () => {
      const resolver = new AccountResolver({});

      const funcDef: FunctionDefinition = {
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
      };

      const providedAccounts = new Map<string, string>([
        ['counter', 'counterAccount123'],
        ['owner', 'ownerAccount123'],
      ]);

      const resolved = resolver.resolveSystemAccounts(funcDef, providedAccounts);

      expect(resolved.systemProgram).toBe(SYSTEM_PROGRAM_ID);
    });

    it('should not auto-inject SystemProgram if no @init constraint', () => {
      const resolver = new AccountResolver({});

      const funcDef: FunctionDefinition = {
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
      };

      const providedAccounts = new Map<string, string>();
      const resolved = resolver.resolveSystemAccounts(funcDef, providedAccounts);

      expect(resolved.systemProgram).toBeUndefined();
    });

    it('should not auto-inject if user already provided systemProgram', () => {
      const resolver = new AccountResolver({});

      const funcDef: FunctionDefinition = {
        name: 'initialize',
        index: 0,
        parameters: [
          {
            name: 'account',
            type: 'Account',
            is_account: true,
            attributes: ['mut', 'init'],
          },
        ],
        return_type: null,
        is_public: true,
        bytecode_offset: 0,
      };

      const customSystemProgram = 'customProgram123';
      const providedAccounts = new Map<string, string>([
        ['systemProgram', customSystemProgram],
      ]);

      const resolved = resolver.resolveSystemAccounts(funcDef, providedAccounts);

      expect(resolved.systemProgram).toBeUndefined();
    });

    it('should return empty object for non-init functions', () => {
      const resolver = new AccountResolver({});

      const funcDef: FunctionDefinition = {
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
      };

      const providedAccounts = new Map<string, string>();
      const resolved = resolver.resolveSystemAccounts(funcDef, providedAccounts);

      expect(Object.keys(resolved)).toHaveLength(0);
    });

    it('should support debug logging', () => {
      const consoleSpy = jest.spyOn(console, 'log').mockImplementation();
      const resolver = new AccountResolver({ debug: true });

      const funcDef: FunctionDefinition = {
        name: 'initialize',
        index: 0,
        parameters: [
          {
            name: 'account',
            type: 'Account',
            is_account: true,
            attributes: ['mut', 'init'],
          },
        ],
        return_type: null,
        is_public: true,
        bytecode_offset: 0,
      };

      const providedAccounts = new Map<string, string>();
      resolver.resolveSystemAccounts(funcDef, providedAccounts);

      expect(consoleSpy).toHaveBeenCalledWith(
        expect.stringContaining('Auto-injecting SystemProgram')
      );

      consoleSpy.mockRestore();
    });
  });

  describe('getAccountMetadata', () => {
    it('should return correct metadata for signer account', () => {
      const resolver = new AccountResolver({});

      const param = {
        name: 'owner',
        type: 'Account',
        is_account: true,
        attributes: ['signer'],
      };

      const metadata = resolver.getAccountMetadata(param);
      expect(metadata.isSigner).toBe(true);
      expect(metadata.isWritable).toBe(false);
    });

    it('should return correct metadata for mutable account', () => {
      const resolver = new AccountResolver({});

      const param = {
        name: 'account',
        type: 'Account',
        is_account: true,
        attributes: ['mut'],
      };

      const metadata = resolver.getAccountMetadata(param);
      expect(metadata.isSigner).toBe(false);
      expect(metadata.isWritable).toBe(true);
    });

    it('should return correct metadata for init account', () => {
      const resolver = new AccountResolver({});

      const param = {
        name: 'account',
        type: 'Account',
        is_account: true,
        attributes: ['mut', 'init'],
      };

      const metadata = resolver.getAccountMetadata(param);
      expect(metadata.isSigner).toBe(false);
      expect(metadata.isWritable).toBe(true);
    });

    it('should treat close-constrained account as writable', () => {
      const resolver = new AccountResolver({});

      const param = {
        name: 'vault',
        type: 'Account',
        is_account: true,
        attributes: ['close'],
      };

      const metadata = resolver.getAccountMetadata(param);
      expect(metadata.isSigner).toBe(false);
      expect(metadata.isWritable).toBe(true);
    });

    it('should return correct metadata for readonly account', () => {
      const resolver = new AccountResolver({});

      const param = {
        name: 'account',
        type: 'Account',
        is_account: true,
        attributes: [],
      };

      const metadata = resolver.getAccountMetadata(param);
      expect(metadata.isSigner).toBe(false);
      expect(metadata.isWritable).toBe(false);
    });

    it('should handle missing attributes', () => {
      const resolver = new AccountResolver({});

      const param = {
        name: 'account',
        type: 'Account',
        is_account: true,
      };

      const metadata = resolver.getAccountMetadata(param as any);
      expect(metadata.isSigner).toBe(false);
      expect(metadata.isWritable).toBe(false);
    });

    it('should recognize both signer and mutable', () => {
      const resolver = new AccountResolver({});

      const param = {
        name: 'owner',
        type: 'Account',
        is_account: true,
        attributes: ['signer', 'mut'],
      };

      const metadata = resolver.getAccountMetadata(param);
      expect(metadata.isSigner).toBe(true);
      expect(metadata.isWritable).toBe(true);
    });
  });

  describe('validateResolvedAccounts', () => {
    it('should not throw when all required accounts provided', () => {
      const resolver = new AccountResolver({});

      const funcDef: FunctionDefinition = {
        name: 'transfer',
        index: 0,
        parameters: [
          {
            name: 'from',
            type: 'Account',
            is_account: true,
            attributes: [],
          },
          {
            name: 'to',
            type: 'Account',
            is_account: true,
            attributes: [],
          },
        ],
        return_type: null,
        is_public: true,
        bytecode_offset: 0,
      };

      const accounts = new Map<string, string>([
        ['from', 'from123'],
        ['to', 'to123'],
      ]);

      expect(() => {
        resolver.validateResolvedAccounts(funcDef, accounts);
      }).not.toThrow();
    });

    it('should throw when required account is missing', () => {
      const resolver = new AccountResolver({});

      const funcDef: FunctionDefinition = {
        name: 'transfer',
        index: 0,
        parameters: [
          {
            name: 'from',
            type: 'Account',
            is_account: true,
            attributes: [],
          },
          {
            name: 'to',
            type: 'Account',
            is_account: true,
            attributes: [],
          },
        ],
        return_type: null,
        is_public: true,
        bytecode_offset: 0,
      };

      const accounts = new Map<string, string>([['from', 'from123']]);

      expect(() => {
        resolver.validateResolvedAccounts(funcDef, accounts);
      }).toThrowError("Required account 'to' not provided");
    });

    it('should not validate data parameters', () => {
      const resolver = new AccountResolver({});

      const funcDef: FunctionDefinition = {
        name: 'transfer',
        index: 0,
        parameters: [
          {
            name: 'from',
            type: 'Account',
            is_account: true,
            attributes: [],
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
      };

      const accounts = new Map<string, string>([['from', 'from123']]);

      expect(() => {
        resolver.validateResolvedAccounts(funcDef, accounts);
      }).not.toThrow();
    });
  });
});
