import { mkdtemp, mkdir, readFile, writeFile } from 'fs/promises';
import { join } from 'path';
import { tmpdir } from 'os';

jest.mock('chalk', () => ({
  __esModule: true,
  default: {
    bold: (s: string) => s,
    cyan: (s: string) => s,
    dim: (s: string) => s,
    hex: () => (s: string) => s,
    red: (s: string) => s,
    green: (s: string) => s,
    yellow: (s: string) => s,
    magenta: (s: string) => s,
    magentaBright: (s: string) => s,
    white: (s: string) => s,
    gray: (s: string) => s,
  }
}));

jest.mock('ora', () => {
  const spinner = {
    start: () => spinner,
    succeed: () => spinner,
    fail: () => spinner,
    stop: () => spinner,
    text: ''
  };
  return () => spinner;
});

jest.mock('five-sdk', () => ({
  normalizeAbiFunctions: (abiFunctions: any) => {
    if (!abiFunctions) return [];
    const functionsArray = Array.isArray(abiFunctions)
      ? abiFunctions
      : Object.entries(abiFunctions).map(([name, func]) => ({ name, ...(func as any || {}) }));

    return functionsArray.map((func: any, idx: number) => {
      const rawParameters = Array.isArray(func.parameters) ? func.parameters : [];
      const existingNames = new Set(rawParameters.map((param: any) => param.name));
      const accountParameters = Array.isArray(func.accounts)
        ? func.accounts
            .map((account: any, accountIdx: number) => ({
              name: account.name ?? `account${accountIdx}`,
              type: 'pubkey',
              param_type: 'pubkey',
              optional: false,
              is_account: true,
              isAccount: true,
              attributes: [
                ...(account.writable ? ['mut'] : []),
                ...(account.signer ? ['signer'] : []),
              ],
            }))
            .filter((param: any) => !existingNames.has(param.name))
        : [];

      return {
        name: func.name ?? `function_${idx}`,
        index: typeof func.index === 'number' ? func.index : idx,
        parameters: [
          ...accountParameters,
          ...rawParameters.map((param: any) => ({
            name: param.name,
            type: param.type ?? param.param_type ?? '',
            param_type: param.param_type,
            optional: param.optional ?? false,
            is_account: param.is_account ?? param.isAccount ?? false,
            isAccount: param.isAccount ?? param.is_account ?? false,
            attributes: Array.isArray(param.attributes) ? [...param.attributes] : [],
          })),
        ],
        accounts: func.accounts ?? [],
        visibility: func.visibility ?? 'public',
        returnType: func.returnType ?? func.return_type,
      };
    });
  },
  TypeGenerator: class {
    private abi: any;
    constructor(abi: any) {
      this.abi = abi;
    }
    generate() {
      return `// generated for ${Array.isArray(this.abi?.functions) ? this.abi.functions.length : 0} functions\n`;
    }
  },
}));

import { artifactCommand } from '../artifact.js';

const logger = {
  debug: jest.fn(),
  info: jest.fn(),
  warn: jest.fn(),
  error: jest.fn()
};

function createContext() {
  return {
    config: { rootDir: process.cwd() },
    logger,
    wasmManager: null,
    options: { verbose: false }
  };
}

describe('artifact command', () => {
  it('packs hex bytecode when --encoding hex is used', async () => {
    const root = await mkdtemp(join(tmpdir(), 'five-cli-artifact-hex-'));
    const ctx = createContext();
    const bytecodePath = join(root, 'script.hex');
    const abiPath = join(root, 'script.abi.json');
    const outputPath = join(root, 'build', 'packed.five');

    await mkdir(join(root, 'build'), { recursive: true });
    await writeFile(bytecodePath, 'deadbeef');
    await writeFile(
      abiPath,
      JSON.stringify({
        name: 'HexDemo',
        functions: {
          settle: {
            index: 0,
            parameters: [],
            accounts: [{ name: 'vault', writable: true, signer: false }],
          },
        },
      }),
    );

    await artifactCommand.handler(['pack'], {
      bytecode: bytecodePath,
      abi: abiPath,
      output: outputPath,
      encoding: 'hex',
      types: true,
    }, ctx as any);

    const artifact = JSON.parse(await readFile(outputPath, 'utf8'));
    expect(artifact.bytecode).toBe(Buffer.from([0xde, 0xad, 0xbe, 0xef]).toString('base64'));
    expect(artifact.abi.functions[0].parameters[0]).toMatchObject({
      name: 'vault',
      is_account: true,
      attributes: ['mut'],
    });

    const typeDefs = await readFile(join(root, 'build', 'packed.d.ts'), 'utf8');
    expect(typeDefs).toContain('generated for 1 functions');
  });

  it('errors when required pack flags are missing', async () => {
    const ctx = createContext();

    await expect(
      artifactCommand.handler(['pack'], { bytecode: '/tmp/demo.bin' }, ctx as any)
    ).rejects.toThrow(/requires --bytecode, --abi, and --output/);
  });

  it('errors on malformed hex input', async () => {
    const root = await mkdtemp(join(tmpdir(), 'five-cli-artifact-badhex-'));
    const ctx = createContext();
    const bytecodePath = join(root, 'script.hex');
    const abiPath = join(root, 'script.abi.json');
    const outputPath = join(root, 'build', 'packed.five');

    await mkdir(join(root, 'build'), { recursive: true });
    await writeFile(bytecodePath, 'abc');
    await writeFile(
      abiPath,
      JSON.stringify({
        name: 'BadHexDemo',
        functions: {
          noop: {
            index: 0,
            parameters: [],
            accounts: [],
          },
        },
      }),
    );

    await expect(
      artifactCommand.handler(['pack'], {
        bytecode: bytecodePath,
        abi: abiPath,
        output: outputPath,
        encoding: 'hex',
      }, ctx as any)
    ).rejects.toThrow(/even number of characters/);
  });
});
