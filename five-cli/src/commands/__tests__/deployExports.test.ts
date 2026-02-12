import { mkdtemp, readFile } from 'fs/promises';
import { join } from 'path';
import { tmpdir } from 'os';
import { parse as parseToml } from '@iarna/toml';

jest.mock('chalk', () => {
  const passthrough = (s: string) => s;
  return {
    __esModule: true,
    default: {
      bold: passthrough,
      green: passthrough,
      red: passthrough,
      gray: passthrough,
      cyan: passthrough,
      yellow: passthrough,
      magenta: passthrough,
      magentaBright: passthrough,
      white: passthrough,
      hex: () => passthrough,
    },
  };
});

jest.mock('ora', () => {
  const spinner = {
    start: () => spinner,
    succeed: () => spinner,
    fail: () => spinner,
    stop: () => spinner,
    text: '',
  };
  return () => spinner;
});

import { buildExportMetadataFromAbi, updateLockfileExports } from '../deploy.js';

describe('deploy export metadata helpers', () => {
  it('builds export metadata from ABI public functions', () => {
    const abi = {
      functions: [
        { name: 'public_a', is_public: true },
        { name: 'public_b', visibility: 'public' },
        { name: 'private_c', visibility: 'private' },
      ],
    };

    const out = buildExportMetadataFromAbi(abi);
    expect(out.methods).toEqual(['public_a', 'public_b']);
    expect(out.interfaces).toEqual([]);
  });

  it('writes and updates five.lock export cache entry', async () => {
    const root = await mkdtemp(join(tmpdir(), 'five-lock-'));
    const bytecode = new Uint8Array([0x35, 0x49, 0x56, 0x45, 0x00]);
    const address = '11111111111111111111111111111111';

    await updateLockfileExports(root, 'demo', address, bytecode, {
      methods: ['transfer'],
      interfaces: [{ name: 'TokenOps', methodMap: { transfer: 'transfer_checked' } }],
    });

    let lock = parseToml(await readFile(join(root, 'five.lock'), 'utf8')) as any;
    expect(Array.isArray(lock.packages)).toBe(true);
    expect(lock.packages.length).toBe(1);
    expect(lock.packages[0].name).toBe('demo');
    expect(lock.packages[0].address).toBe(address);
    expect(lock.packages[0].exports.methods).toEqual(['transfer']);
    expect(lock.packages[0].exports.interfaces[0].name).toBe('TokenOps');
    expect(lock.packages[0].exports.interfaces[0].method_map.transfer).toBe('transfer_checked');

    await updateLockfileExports(root, 'demo', address, bytecode, {
      methods: ['mint'],
      interfaces: [],
    });
    lock = parseToml(await readFile(join(root, 'five.lock'), 'utf8')) as any;
    expect(lock.packages.length).toBe(1);
    expect(lock.packages[0].exports.methods).toEqual(['mint']);
  });
});
