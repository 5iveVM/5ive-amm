import { mkdtemp, writeFile, mkdir } from 'fs/promises';
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
jest.mock('five-sdk', () => ({
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
    validateBytecode: jest.fn().mockResolvedValue({ success: true }),
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

import { FiveSDK } from '@5ive-tech/sdk';
import { loadProjectConfig } from '../../project/ProjectLoader.js';
import { compileCommand } from '../compile.js';
import { deployCommand } from '../deploy.js';
import { executeCommand } from '../execute.js';
import { testCommand } from '../test.js';
import { buildCommand } from '../build.js';

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
  beforeEach(() => {
    (FiveSDK.compileProject as jest.Mock).mockClear();
    (FiveSDK.compileWithDiscovery as jest.Mock).mockClear();
  });

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

    await buildCommand.handler([], { project: root }, ctx as any);

    expect((FiveSDK.compileWithDiscovery as jest.Mock)).toHaveBeenCalledTimes(1);
    expect((FiveSDK.compileWithDiscovery as jest.Mock).mock.calls[0][0]).toBe('src/main.v');
  });

  it('compile handler uses discovery path for explicit input files', async () => {
    const root = await createProject();
    const ctx = createContext();
    const inputPath = join(root, 'src', 'main.v');

    await compileCommand.handler([inputPath], { project: root }, ctx as any);

    expect((FiveSDK.compileWithDiscovery as jest.Mock)).toHaveBeenCalledTimes(1);
    expect((FiveSDK.compileWithDiscovery as jest.Mock).mock.calls[0][0]).toBe('main.v');
  });

  it('compile handler reports import-resolution requirement when discovery API is unavailable', async () => {
    const root = await mkdtemp(join(tmpdir(), 'five-cli-import-fallback-'));
    const sourcePath = join(root, 'imports.v');
    await writeFile(
      sourcePath,
      `use std::interfaces::spl_token;\n\npub run(source: account @mut, destination: account @mut, authority: account @signer, amount: u64) {\n  SPLToken.transfer(source, destination, authority, amount);\n}\n`
    );
    const ctx = createContext();

    const sdkAny = FiveSDK as any;
    const original = sdkAny.compileWithDiscovery;
    const exitSpy = jest.spyOn(process, 'exit').mockImplementation(((code?: number) => {
      throw new Error(`exit:${code ?? 'undefined'}`);
    }) as never);
    try {
      delete sdkAny.compileWithDiscovery;

      await expect(
        compileCommand.handler([sourcePath], {}, ctx as any)
      ).rejects.toThrow(/exit:1/);
    } finally {
      sdkAny.compileWithDiscovery = original;
      exitSpy.mockRestore();
    }
  });
});
