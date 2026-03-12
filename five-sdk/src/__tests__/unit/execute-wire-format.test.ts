import { beforeAll, beforeEach, describe, expect, it, jest, afterEach } from '@jest/globals';

const mockEncodeExecute = jest.fn(async () => new Uint8Array([0xaa, 0xbb]));
const mockDeriveVMStatePDA = jest.fn(async () => ({
  address: '11111111111111111111111111111111',
  bump: 255,
}));

jest.unstable_mockModule('../../lib/bytecode-encoder.js', () => ({
  BytecodeEncoder: {
    encodeExecute: mockEncodeExecute,
  },
}));

jest.unstable_mockModule('../../crypto/index.js', () => ({
  PDAUtils: {
    deriveVMStatePDA: mockDeriveVMStatePDA,
  },
  Base58Utils: {
    encode: (value: string) => value,
  },
  RentCalculator: {
    calculateRentExemption: () => 0,
    formatSOL: () => '0',
  },
}));

let ExecuteModule: any;
let ProgramIdResolver: any;

describe('execute wire format', () => {
  beforeAll(async () => {
    ExecuteModule = await import('../../modules/execute.js');
    const resolver = await import('../../config/ProgramIdResolver.js');
    ProgramIdResolver = resolver.ProgramIdResolver;
  });

  beforeEach(() => {
    mockEncodeExecute.mockClear();
    mockDeriveVMStatePDA.mockClear();
    ProgramIdResolver.setDefault('TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP');
  });

  afterEach(() => {
    ProgramIdResolver.clearDefault();
  });

  it('encodes execute instruction envelope as discriminator + u32/u32 LE + payload', async () => {
    const scriptAccount = '11111111111111111111111111111111';
    const accountPubkey = '11111111111111111111111111111114';
    const abi = {
      functions: [
        {
          name: 'transfer',
          index: 1,
          parameters: [
            { name: 'from', type: 'account', is_account: true },
            { name: 'flag', type: 'bool', is_account: false },
          ],
        },
      ],
    };

    const result = await ExecuteModule.generateExecuteInstruction(
      scriptAccount,
      'transfer',
      [accountPubkey, true],
      [accountPubkey],
      undefined,
      {
        abi,
        estimateFees: false,
        fiveVMProgramId: '11111111111111111111111111111111',
        feeShardIndex: 0,
        payerAccount: '11111111111111111111111111111116',
      },
    );

    expect(mockEncodeExecute).toHaveBeenCalledTimes(1);
    const [functionIndex, paramDefs, paramValues] = mockEncodeExecute.mock.calls[0];
    expect(functionIndex).toBe(1);
    expect(paramDefs).toHaveLength(1);
    expect(paramValues.from).toBeUndefined();
    expect(paramValues.flag).toBe(true);

    const raw = Buffer.from(result.instruction.data, 'base64');
    expect(raw[0]).toBe(9);
    expect(raw[1]).toBe(0xff);
    expect(raw[2]).toBe(0x53);
    expect(raw[3]).toBe(0x00);
    expect(raw.readUInt32LE(4)).toBe(1);
    expect(raw.readUInt32LE(8)).toBe(1);
    expect(Array.from(raw.subarray(12))).toEqual([0xaa, 0xbb]);
    expect(raw[1] === 0xff && raw[2] === 0x53).toBe(true);
  });

  it('supports object-format ABI functions and coerces pubkey/account parameter values', async () => {
    const scriptAccount = '11111111111111111111111111111111';
    const ownerPubkey = '11111111111111111111111111111112';
    const counterPubkey = '11111111111111111111111111111113';
    const ownerLike = {
      toBase58: () => ownerPubkey,
    };
    const counterLike = {
      toBase58: () => counterPubkey,
    };

    const abi = {
      functions: {
        transfer: {
          index: 7,
          parameters: [
            { name: 'owner', type: 'pubkey', is_account: false },
            { name: 'counter', type: 'account', is_account: true },
          ],
        },
      },
    };

    const result = await ExecuteModule.generateExecuteInstruction(
      scriptAccount,
      'transfer',
      [ownerLike, counterLike],
      [counterPubkey],
      undefined,
      {
        abi,
        estimateFees: false,
        feeShardIndex: 0,
        payerAccount: '11111111111111111111111111111116',
      },
    );

    expect(mockEncodeExecute).toHaveBeenCalledTimes(1);
    const [functionIndex, , paramValues] = mockEncodeExecute.mock.calls[0];
    expect(functionIndex).toBe(7);
    expect(paramValues.owner).toBe(ownerPubkey);
    expect(paramValues.counter).toBeUndefined();

    const raw = Buffer.from(result.instruction.data, 'base64');
    expect(raw[0]).toBe(9);
    expect(raw[1]).toBe(0xff);
    expect(raw[2]).toBe(0x53);
    expect(raw[3]).toBe(0x00);
    expect(raw.readUInt32LE(4)).toBe(7);
    expect(raw.readUInt32LE(8)).toBe(1);
  });

  it('marks payer writable when function has @init + signer account', async () => {
    const scriptAccount = '11111111111111111111111111111111';
    const accountToInit = '11111111111111111111111111111112';
    const payer = '11111111111111111111111111111113';
    const abi = {
      functions: [
        {
          name: 'initialize',
          index: 3,
          parameters: [
            { name: 'state', type: 'account', is_account: true, attributes: ['init'] },
            { name: 'payer', type: 'account', is_account: true, attributes: ['signer'] },
          ],
        },
      ],
    };

    const result = await ExecuteModule.generateExecuteInstruction(
      scriptAccount,
      'initialize',
      [accountToInit, payer],
      [accountToInit, payer],
      undefined,
      {
        abi,
        estimateFees: false,
      },
    );

    const stateAccountMeta = result.instruction.accounts.find((a: any) => a.pubkey === accountToInit);
    const payerMeta = result.instruction.accounts.find((a: any) => a.pubkey === payer);

    expect(stateAccountMeta).toBeDefined();
    expect(stateAccountMeta.isWritable).toBe(true);
    expect(stateAccountMeta.isSigner).toBe(false);

    expect(payerMeta).toBeDefined();
    expect(payerMeta.isSigner).toBe(true);
    expect(payerMeta.isWritable).toBe(true);
  });

  it('rejects non-canonical vmStateAccount override', async () => {
    const scriptAccount = '11111111111111111111111111111111';
    const canonical = '11111111111111111111111111111111';
    const nonCanonical = '11111111111111111111111111111112';
    mockDeriveVMStatePDA.mockResolvedValueOnce({ address: canonical, bump: 255 });

    await expect(
      ExecuteModule.generateExecuteInstruction(
        scriptAccount,
        0,
        [],
        [],
        undefined,
        {
          vmStateAccount: nonCanonical,
          estimateFees: false,
          fiveVMProgramId: canonical,
        },
      ),
    ).rejects.toThrow(`vmStateAccount must be canonical PDA ${canonical}`);
  });

  it('always appends fee tail accounts in strict order', async () => {
    const scriptAccount = '11111111111111111111111111111111';
    const payer = '11111111111111111111111111111113';
    const result = await ExecuteModule.generateExecuteInstruction(
      scriptAccount,
      0,
      [],
      [payer],
      undefined,
      {
        estimateFees: false,
        feeShardIndex: 0,
        payerAccount: payer,
      },
    );

    const accounts = result.instruction.accounts;
    const feeTail = accounts.slice(-3);
    expect(feeTail[0]).toMatchObject({ pubkey: payer, isSigner: true, isWritable: true });
    expect(feeTail[2]).toMatchObject({
      pubkey: '11111111111111111111111111111111',
      isSigner: false,
      isWritable: false,
    });
    expect(feeTail[1].isSigner).toBe(false);
    expect(feeTail[1].isWritable).toBe(true);
    expect(feeTail[1].pubkey).not.toBe(payer);
  });

  it('marks execute core accounts (script + vm_state) readonly', async () => {
    const payer = '11111111111111111111111111111112';
    const result = await ExecuteModule.generateExecuteInstruction(
      '11111111111111111111111111111111',
      0,
      [],
      [payer],
      undefined,
      {
        estimateFees: false,
        payerAccount: payer,
        accountMetadata: new Map([
          [payer, { isSigner: true, isWritable: false }],
        ]),
      },
    );

    expect(result.instruction.accounts[0]).toMatchObject({
      pubkey: '11111111111111111111111111111111',
      isWritable: false,
    });
    expect(result.instruction.accounts[1]).toMatchObject({
      pubkey: '11111111111111111111111111111111',
      isWritable: false,
    });
  });

  it('derives account metadata in args-only mode using ABI account param order', async () => {
    const scriptAccount = '11111111111111111111111111111111';
    const stateAccount = '11111111111111111111111111111114';
    const warnSpy = jest.spyOn(console, 'warn').mockImplementation(() => {});
    const result = await ExecuteModule.generateExecuteInstruction(
      scriptAccount,
      'update',
      [42],
      [stateAccount],
      undefined,
      {
        estimateFees: false,
        abi: {
          functions: [
            {
              name: 'update',
              index: 9,
              parameters: [
                { name: 'state', type: 'account', is_account: true, attributes: ['mut'] },
                { name: 'value', type: 'u64', is_account: false },
              ],
            },
          ],
        },
        payerAccount: '11111111111111111111111111111116',
      },
    );

    // account layout: [script, vm_state, user_accounts..., fee_tail...]
    expect(result.instruction.accounts[2]).toMatchObject({
      pubkey: stateAccount,
      isWritable: true,
      isSigner: false,
    });
    expect(warnSpy).not.toHaveBeenCalled();
    warnSpy.mockRestore();
  });

  it('marks @close account writable when deriving metadata from ABI', async () => {
    const scriptAccount = '11111111111111111111111111111111';
    const vaultAccount = '11111111111111111111111111111117';
    const warnSpy = jest.spyOn(console, 'warn').mockImplementation(() => {});
    const result = await ExecuteModule.generateExecuteInstruction(
      scriptAccount,
      'close_vault',
      [],
      [vaultAccount],
      undefined,
      {
        estimateFees: false,
        abi: {
          functions: [
            {
              name: 'close_vault',
              index: 10,
              parameters: [
                { name: 'vault', type: 'account', is_account: true, attributes: ['close'] },
              ],
            },
          ],
        },
        payerAccount: '11111111111111111111111111111116',
      },
    );

    expect(result.instruction.accounts[2]).toMatchObject({
      pubkey: vaultAccount,
      isWritable: true,
      isSigner: false,
    });
    expect(warnSpy).not.toHaveBeenCalled();
    warnSpy.mockRestore();
  });

  it('defaults unknown user accounts to readonly and warns with hint', async () => {
    const scriptAccount = '11111111111111111111111111111111';
    const unknownAccount = '11111111111111111111111111111115';
    const warnSpy = jest.spyOn(console, 'warn').mockImplementation(() => {});
    const result = await ExecuteModule.generateExecuteInstruction(
      scriptAccount,
      0,
      [],
      [unknownAccount],
      undefined,
      {
        estimateFees: false,
        payerAccount: '11111111111111111111111111111116',
      },
    );

    expect(result.instruction.accounts[2]).toMatchObject({
      pubkey: unknownAccount,
      isWritable: false,
    });
    expect(warnSpy).toHaveBeenCalledWith(
      expect.stringContaining('Missing account metadata')
    );
    warnSpy.mockRestore();
  });

  it('does not match legacy varint envelope layout', async () => {
    const legacyLike = Buffer.from([9, 1, 1, 0xaa, 0xbb]);
    // Canonical flow requires execute fee header bytes and fixed-width u32 fields.
    expect(legacyLike.length).toBeLessThan(12);
    expect(legacyLike[1]).not.toBe(0xff);
    expect(legacyLike[2]).not.toBe(0x53);
  });

  it('fails fast when importing legacy source entrypoints', async () => {
    const srcIndexUrl = new URL('../../index.js', import.meta.url).href;
    const srcSdkUrl = new URL('../../FiveSDK.js', import.meta.url).href;
    await expect(import(srcIndexUrl)).rejects.toThrow(
      /Unsupported runtime import: `five-sdk\/src\/index\.js`/,
    );
    await expect(import(srcSdkUrl)).rejects.toThrow(
      /Unsupported runtime import: `five-sdk\/src\/FiveSDK\.js`/,
    );
  });
});
