import { configCommand } from '../config.js';

const mockSetProgramId = jest.fn();

jest.mock('../../config/ConfigManager.js', () => ({
  ConfigManager: {
    getInstance: () => ({
      get: jest.fn().mockResolvedValue({
        target: 'devnet',
        networks: {
          devnet: { rpcUrl: 'https://api.devnet.solana.com' }
        },
        keypair: '~/.config/solana/id.json',
        showConfig: false
      }),
      setTarget: jest.fn(),
      setProgramId: mockSetProgramId,
      setKeypair: jest.fn(),
      setShowConfig: jest.fn(),
      set: jest.fn(),
      init: jest.fn()
    })
  }
}));

jest.mock('chalk', () => ({
  __esModule: true,
  default: {
    bold: (s: string) => s,
    cyan: (s: string) => s,
    dim: (s: string) => s,
    hex: () => (s: string) => s
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

describe('config command', () => {
  beforeEach(() => {
    mockSetProgramId.mockReset();
  });

  it('prints JSON for a specific key', async () => {
    const ctx = createContext();
    const logSpy = jest.spyOn(console, 'log').mockImplementation(() => undefined);

    await configCommand.handler(['get', 'target'], { format: 'json' }, ctx as any);

    const output = logSpy.mock.calls.map(call => call.join(' ')).join('\n');
    expect(output).toContain('"target"');
    expect(output).toContain('devnet');
  });

  it('exits when no changes are provided for set', async () => {
    const ctx = createContext();
    const exitSpy = jest.spyOn(process, 'exit').mockImplementation(((code?: number) => {
      throw new Error(`exit:${code ?? 'undefined'}`);
    }) as never);

    await expect(
      configCommand.handler(['set'], {}, ctx as any)
    ).rejects.toThrow('exit:1');

    exitSpy.mockRestore();
  });

  it('registers --program-id option and routes it through set', async () => {
    const hasProgramId = (configCommand.options || []).some(
      (opt: any) => opt.flags === '--program-id <id>'
    );
    expect(hasProgramId).toBe(true);

    const ctx = createContext();
    await configCommand.handler(
      ['set'],
      { programId: '11111111111111111111111111111112', target: 'devnet' },
      ctx as any
    );

    expect(mockSetProgramId).toHaveBeenCalledWith(
      '11111111111111111111111111111112',
      'devnet'
    );
  });
});
