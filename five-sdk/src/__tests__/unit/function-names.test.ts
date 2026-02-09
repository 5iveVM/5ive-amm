import { beforeEach, describe, expect, it, jest } from '@jest/globals';
import { FiveSDK } from '../../FiveSDK.js';

describe('function name utilities', () => {
  beforeEach(() => {
    jest.clearAllMocks();
    (FiveSDK as any).parameterEncoder = {};
  });

  it('returns function names when compiler yields an array', async () => {
    const mockCompiler = {
      getFunctionNames: jest.fn(async () => [
        { name: 'init', function_index: 0 },
        { name: 'transfer', function_index: 1 },
      ]),
    };
    (FiveSDK as any).compiler = mockCompiler;

    const names = await FiveSDK.getFunctionNames(new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]));

    expect(names).toHaveLength(2);
    expect(names[0].name).toBe('init');
    expect(names[1].function_index).toBe(1);
    expect(mockCompiler.getFunctionNames).toHaveBeenCalledTimes(1);
  });

  it('parses JSON string function names from compiler', async () => {
    const mockCompiler = {
      getFunctionNames: jest.fn(async () =>
        JSON.stringify([{ name: 'mint', function_index: 2 }])
      ),
    };
    (FiveSDK as any).compiler = mockCompiler;

    const names = await FiveSDK.getFunctionNames(new Uint8Array([9, 8, 7, 6, 5, 4, 3, 2]));

    expect(names).toEqual([{ name: 'mint', function_index: 2 }]);
  });

  it('falls back to empty list when compiler throws', async () => {
    const mockCompiler = {
      getFunctionNames: jest.fn(async () => {
        throw new Error('decode failed');
      }),
    };
    (FiveSDK as any).compiler = mockCompiler;

    const names = await FiveSDK.getFunctionNames(new Uint8Array([1, 1, 1, 1, 1, 1, 1, 1]));

    expect(names).toEqual([]);
  });
});
