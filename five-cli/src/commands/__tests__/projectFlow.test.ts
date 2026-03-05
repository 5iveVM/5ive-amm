import { mkdtemp, writeFile, mkdir, readFile } from 'fs/promises';
import { join } from 'path';
import { tmpdir } from 'os';

// Mocks for ESM-only dependencies
jest.mock('chalk', () => {
  const mockColorFunction = (s: string) => s;
  return {
    __esModule: true,
    default: {
      bold: mockColorFunction,
      green: mockColorFunction,
      red: mockColorFunction,
      gray: mockColorFunction,
      cyan: mockColorFunction,
      yellow: mockColorFunction,
      magenta: mockColorFunction,
      magentaBright: mockColorFunction,
      white: mockColorFunction,
      hex: () => mockColorFunction
    }
  };
});

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

// Mock SDK and managers to avoid real WASM/network calls
jest.mock('@5ive-tech/sdk', () => ({
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
  FiveSDK: {
    compile: jest.fn().mockResolvedValue({
      success: true,
      fiveFile: {},
      bytecode: new Uint8Array(),
      metadata: {}
    }),
    compileProject: jest.fn().mockResolvedValue({
      success: true,
      fiveFile: {},
      bytecode: new Uint8Array(),
      metadata: {}
    }),
    compileWithDiscovery: jest.fn().mockResolvedValue({
      success: true,
      fiveFile: {},
      bytecode: new Uint8Array(),
      metadata: {}
    }),
    validateBytecode: jest.fn().mockResolvedValue({ success: true, valid: true }),
    deployToSolana: jest.fn().mockResolvedValue({
      success: true,
      programId: 'mock-program',
      transactionId: 'tx',
      deploymentCost: 0
    }),
    executeOnSolana: jest.fn().mockResolvedValue({
      success: true,
      result: 0,
      computeUnitsUsed: 0,
      cost: 0
    }),
    create: jest.fn(() => ({ run: jest.fn() }))
  },
  ProgramIdResolver: {
    resolve: jest.fn((programId?: string) => programId ?? 'FmzLpEQryX1UDtNjDBPx9GDsXiThFtzjsZXtTLNLU7Vb')
  },
  FiveTestRunner: class {
    constructor() {}
    async discoverTestSuites() {
      return [];
    }
    async runTestSuites() {
      return [];
    }
  },
  TestDiscovery: {
    discoverTests: jest.fn().mockResolvedValue([])
  },
  compileScript: jest.fn(),
  executeLocally: jest.fn(),
  compileAndExecuteLocally: jest.fn()
}));

jest.mock('../../config/ConfigManager.js', () => ({
  ConfigManager: {
    getInstance: () => ({
      applyOverrides: jest.fn().mockResolvedValue({
        target: 'devnet',
        networks: { devnet: { rpcUrl: 'https://api.devnet.solana.com' } },
        keypairPath: '~/.config/solana/id.json',
        showConfig: false
      }),
      getProgramId: jest.fn().mockResolvedValue(undefined)
    }),
    getTargetPrefix: () => '[devnet]'
  }
}));

jest.mock('../../utils/FiveFileManager.js', () => ({
  FiveFileManager: {
    getInstance: () => ({
      loadFile: jest.fn().mockResolvedValue({
        bytecode: new Uint8Array(),
        format: 'five',
        abi: {}
      })
    })
  }
}));

jest.mock('../../wasm/compiler.js', () => ({
  FiveCompilerWasm: require('../../__tests__/mocks/wasm-compiler').FiveCompilerWasm
}));

import { FiveCompilerWasm } from '../../__tests__/mocks/wasm-compiler';
import { loadProjectConfig } from '../../project/ProjectLoader.js';
import { compileCommand } from '../compile.js';
import { deployCommand } from '../deploy.js';
import { executeCommand } from '../execute.js';
import { testCommand } from '../test.js';
import { buildCommand } from '../build.js';
import { artifactCommand } from '../artifact.js';

// Mock logger
const logger = {
  debug: jest.fn(),
  info: jest.fn(),
  warn: jest.fn(),
  error: jest.fn()
};

// Helper to create a sample project structure with five.toml and a stub .v file
async function createProject({ target = 'vm' }: { target?: string } = {}) {
  const root = await mkdtemp(join(tmpdir(), 'five-cli-project-'));
  await mkdir(join(root, 'src'), { recursive: true });
  await mkdir(join(root, 'build'), { recursive: true });
  await writeFile(
    join(root, 'five.toml'),
    `
schema_version = 1

[project]
name = "demo"
version = "0.1.0"
description = "Demo project"
source_dir = "src"
build_dir = "build"
target = "${target}"
entry_point = "src/main.v"

[build]
output_artifact_name = "demo"

[deploy]
cluster = "devnet"
rpc_url = "https://api.devnet.solana.com"
commitment = "confirmed"
keypair_path = "~/.config/solana/id.json"
`
  );
  await writeFile(
    join(root, 'src', 'main.v'),
    `pub main() -> u64 { return 1; }`
  );
  return root;
}

// Minimal command context for testing handlers without executing real SDK
function createContext() {
  return {
    config: { rootDir: process.cwd() },
    logger,
    wasmManager: null,
    options: { verbose: false }
  };
}

describe('project-aware commands', () => {
  it('loads project config from cwd and applies defaults', async () => {
    const root = await createProject();
    const loaded = await loadProjectConfig(undefined, root);
    expect(loaded?.config.name).toBe('demo');
    expect(loaded?.config.entryPoint).toBe('src/main.v');
  });

  it('compile handler errors when no sources found and no config', async () => {
    const ctx = createContext();
    await expect(
      compileCommand.handler([], { validate: true }, ctx as any)
    ).rejects.toThrow(/No 5ive source files found/);
  });

  it('deploy/execute handlers require artifact if manifest missing', async () => {
    const ctx = createContext();
    await expect(
      deployCommand.handler([], {}, ctx as any)
    ).rejects.toThrow(/Bytecode file is required/);
    await expect(
      executeCommand.handler([], {}, ctx as any)
    ).rejects.toThrow(/No bytecode or script account provided/);
  });

  it('deploy emits JSON with scriptAccount alias in dry-run mode', async () => {
    const root = await createProject();
    const ctx = createContext();
    const consoleSpy = jest.spyOn(console, 'log').mockImplementation(() => {});

    await deployCommand.handler(
      [join(root, 'build', 'demo.five')],
      {
        project: root,
        format: 'json',
        dryRun: true,
        programId: 'FmzLpEQryX1UDtNjDBPx9GDsXiThFtzjsZXtTLNLU7Vb',
      },
      ctx as any,
    );

    const payload = JSON.parse(String(consoleSpy.mock.calls.at(-1)?.[0] || '{}'));
    expect(payload.success).toBe(true);
    expect(payload.scriptAccount).toBeTruthy();
    expect(payload.programId).toBe(payload.scriptAccount);
    expect(payload.transactionId).toBeTruthy();

    consoleSpy.mockRestore();
  });

  it('test handler uses default tests path when project present', async () => {
    const root = await createProject();
    const ctx = createContext();
    // No tests present, expect a warning but no throw
    await expect(
      testCommand.handler([], { project: root }, ctx as any)
    ).resolves.toBeUndefined();
  });

  it('build handler loads project config and delegates to compile', async () => {
    const root = await createProject();
    const ctx = createContext();
    const discoverySpy = jest.spyOn(FiveCompilerWasm.prototype, 'compileWithDiscovery');

    await buildCommand.handler([], { project: root }, ctx as any);

    expect(discoverySpy).toHaveBeenCalledTimes(1);
    expect(discoverySpy.mock.calls[0][0]).toMatch(/src\/main\.v$/);
    discoverySpy.mockRestore();
  });

  it('compile handler uses discovery path for explicit input files', async () => {
    const root = await createProject();
    const ctx = createContext();
    const inputPath = join(root, 'src', 'main.v');
    const discoverySpy = jest.spyOn(FiveCompilerWasm.prototype, 'compileWithDiscovery');

    await compileCommand.handler([inputPath], { project: root }, ctx as any);

    expect(discoverySpy).toHaveBeenCalledTimes(1);
    expect(discoverySpy.mock.calls[0][0]).toMatch(/src\/main\.v$/);
    discoverySpy.mockRestore();
  });

  it('build emits packaged artifacts with ABI-derived account parameters and types', async () => {
    const root = await createProject();
    const ctx = createContext();

    await buildCommand.handler([], { project: root }, ctx as any);

    const artifact = JSON.parse(await readFile(join(root, 'build', 'demo.five'), 'utf8'));
    expect(Array.isArray(artifact.abi.functions)).toBe(true);
    expect(artifact.abi.functions[0].parameters[0]).toMatchObject({
      name: 'state',
      is_account: true,
      attributes: ['mut'],
    });
    expect(artifact.abi.functions[0].parameters[1]).toMatchObject({
      name: 'payer',
      is_account: true,
      attributes: ['mut', 'signer'],
    });

    const typeDefs = await readFile(join(root, 'build', 'demo.d.ts'), 'utf8');
    expect(typeDefs).toContain('generated for 1 functions');
  });

  it('compile writes abi output using the normalized packaged ABI', async () => {
    const root = await createProject();
    const ctx = createContext();
    const inputPath = join(root, 'src', 'main.v');
    const abiPath = join(root, 'build', 'demo.abi.json');

    await compileCommand.handler([inputPath], { project: root, abi: abiPath }, ctx as any);

    const abi = JSON.parse(await readFile(abiPath, 'utf8'));
    expect(Array.isArray(abi.functions)).toBe(true);
    expect(abi.functions[0].parameters[0]).toMatchObject({
      name: 'state',
      is_account: true,
    });
  });

  it('artifact pack normalizes object ABI functions and emits optional types', async () => {
    const root = await mkdtemp(join(tmpdir(), 'five-cli-artifact-pack-'));
    const ctx = createContext();
    const bytecodePath = join(root, 'script.bin');
    const abiPath = join(root, 'script.abi.json');
    const outputPath = join(root, 'build', 'main.five');

    await writeFile(bytecodePath, Buffer.from([0xde, 0xad, 0xbe, 0xef]));
    await writeFile(
      abiPath,
      JSON.stringify({
        name: 'PackDemo',
        functions: {
          settle: {
            index: 2,
            parameters: [{ name: 'amount', type: 'u64' }],
            accounts: [{ name: 'vault', writable: true, signer: false }],
          },
        },
      }),
    );

    await artifactCommand.handler(['pack'], {
      bytecode: bytecodePath,
      abi: abiPath,
      output: outputPath,
      encoding: 'binary',
      types: true,
    }, ctx as any);

    const artifact = JSON.parse(await readFile(outputPath, 'utf8'));
    expect(Array.isArray(artifact.abi.functions)).toBe(true);
    expect(artifact.abi.functions[0]).toMatchObject({
      name: 'settle',
      index: 2,
    });
    expect(artifact.abi.functions[0].parameters[0]).toMatchObject({
      name: 'vault',
      is_account: true,
      attributes: ['mut'],
    });

    const typeDefs = await readFile(join(root, 'build', 'main.d.ts'), 'utf8');
    expect(typeDefs).toContain('generated for 1 functions');
  });

  it('compile handler reports import-resolution requirement when discovery API is unavailable', async () => {
    const root = await mkdtemp(join(tmpdir(), 'five-cli-import-fallback-'));
    const sourcePath = join(root, 'imports.v');
    await writeFile(
      sourcePath,
      `use std::interfaces::spl_token;\n\npub run(source: account @mut, destination: account @mut, authority: account @signer, amount: u64) {\n  SPLToken.transfer(source, destination, authority, amount);\n}\n`
    );
    const ctx = createContext();
    const discoverySpy = jest
      .spyOn(FiveCompilerWasm.prototype, 'compileWithDiscovery')
      .mockRejectedValue(new Error('compileMultiWithDiscovery is unavailable'));
    const exitSpy = jest.spyOn(process, 'exit').mockImplementation(((code?: number) => {
      throw new Error(`exit:${code ?? 'undefined'}`);
    }) as never);
    try {
      await expect(
        compileCommand.handler([sourcePath], {}, ctx as any)
      ).rejects.toThrow(/exit:1/);
    } finally {
      discoverySpy.mockRestore();
      exitSpy.mockRestore();
    }
  });
});
