import { createCLI } from '../index.js';

describe('CLI entry', () => {
  const exitError = (code?: number) => new Error(`exit:${code ?? 'undefined'}`);
  let exitSpy: jest.SpyInstance;

  beforeEach(() => {
    exitSpy = jest.spyOn(process, 'exit').mockImplementation(((code?: number) => {
      throw exitError(code);
    }) as never);
  });

  afterEach(() => {
    exitSpy.mockRestore();
    jest.restoreAllMocks();
  });

  it('shows help when no command is provided', async () => {
    const cli = createCLI({ rootDir: process.cwd() });
    const helpSpy = jest
      .spyOn(cli.getProgram(), 'outputHelp')
      .mockImplementation(() => undefined);

    await cli.run(['node', 'five']);

    expect(helpSpy).toHaveBeenCalled();
  });

  it('errors on unknown command', async () => {
    const cli = createCLI({ rootDir: process.cwd() });
    const errorSpy = jest.spyOn(console, 'error').mockImplementation(() => undefined);

    await expect(cli.run(['node', 'five', 'buidl'])).rejects.toThrow('exit:1');
    expect(errorSpy).toHaveBeenCalled();
  });
});
