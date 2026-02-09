import { beforeAll, beforeEach, describe, expect, it, jest } from '@jest/globals';

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

describe('execute wire format', () => {
  beforeAll(async () => {
    ExecuteModule = await import('../../modules/execute.js');
  });

  beforeEach(() => {
    mockEncodeExecute.mockClear();
    mockDeriveVMStatePDA.mockClear();
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
      },
    );

    expect(mockEncodeExecute).toHaveBeenCalledTimes(1);
    const [functionIndex, paramDefs, paramValues] = mockEncodeExecute.mock.calls[0];
    expect(functionIndex).toBe(1);
    expect(paramDefs).toHaveLength(2);
    // Account pubkey should be normalized to VM account index+1.
    expect(paramValues.from).toBe(1);
    expect(paramValues.flag).toBe(true);

    const raw = Buffer.from(result.instruction.data, 'base64');
    expect(raw[0]).toBe(9);
    expect(raw.readUInt32LE(1)).toBe(1);
    expect(raw.readUInt32LE(5)).toBe(2);
    expect(Array.from(raw.subarray(9))).toEqual([0xaa, 0xbb]);
  });
});
