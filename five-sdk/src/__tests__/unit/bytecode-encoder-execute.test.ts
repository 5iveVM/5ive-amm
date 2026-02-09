import { beforeAll, beforeEach, describe, expect, it, jest } from '@jest/globals';

const mockEncodeExecute = jest.fn();

jest.unstable_mockModule('../../wasm/loader.js', () => ({
  getWasmModule: jest.fn(async () => ({
    ParameterEncoder: {
      encode_execute: mockEncodeExecute
    }
  }))
}));

let BytecodeEncoder: any;

describe('BytecodeEncoder execute path', () => {
  beforeAll(async () => {
    const module = await import('../../lib/bytecode-encoder.js');
    BytecodeEncoder = module.BytecodeEncoder;
  });

  beforeEach(() => {
    mockEncodeExecute.mockReset();
    mockEncodeExecute.mockReturnValue(new Uint8Array([0x04]));
  });

  it('parses sized string type specifications', () => {
    expect(BytecodeEncoder.parseTypeSpec('string<64>')).toEqual({ baseType: 'string', maxLen: 64 });
    expect(BytecodeEncoder.parseTypeSpec(' String < 32 > ')).toEqual({ baseType: 'string', maxLen: 32 });
    expect(BytecodeEncoder.parseTypeSpec('string')).toEqual({ baseType: 'string' });
  });

  it('forwards canonical typed params with maxLen for string<n>', async () => {
    const definitions = [
      { name: 'name', type: 'string<32>' },
      { name: 'symbol', type: 'string<8>' },
      { name: 'uri', type: 'string' },
    ];
    const values = {
      name: 'TestToken',
      symbol: 'TEST',
      uri: 'https://example.com/token'
    };

    await BytecodeEncoder.encodeExecute(0, definitions, values);

    expect(mockEncodeExecute).toHaveBeenCalledTimes(1);
    const [, paramArray] = mockEncodeExecute.mock.calls[0];

    expect(paramArray[0].type).toBe('string');
    expect(paramArray[0].maxLen).toBe(32);
    expect(paramArray[1].type).toBe('string');
    expect(paramArray[1].maxLen).toBe(8);
    expect(paramArray[2].type).toBe('string');
    expect(paramArray[2].maxLen).toBeUndefined();
  });

  it('normalizes account-like DSL types to account', async () => {
    const definitions = [
      { name: 'mint_account', type: 'Mint' },
      { name: 'destination_account', type: 'TokenAccount' },
    ];
    const values = {
      mint_account: 1,
      destination_account: 2
    };

    await BytecodeEncoder.encodeExecute(2, definitions, values);

    const [, paramArray] = mockEncodeExecute.mock.calls[0];
    expect(paramArray[0].type).toBe('account');
    expect(paramArray[0].isAccount).toBe(true);
    expect(paramArray[1].type).toBe('account');
    expect(paramArray[1].isAccount).toBe(true);
  });
});
