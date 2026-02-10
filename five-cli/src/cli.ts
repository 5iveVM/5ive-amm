#!/usr/bin/env node
// Five CLI entry point.

import { Command } from 'commander';
import chalk from 'chalk';
import { join } from 'path';
import { readFileSync } from 'fs';

import {
  CLIConfig,
  CommandContext,
  Logger,
  CLIError
} from './types.js';
import { createLogger } from './utils/logger.js';
import { commands, getCommand, generateCommandsHelp } from './commands/index.js';
import {
  brandLine,
  section,
  commandExample,
  commandNotFound,
  error as uiError,
  hint
} from './utils/cli-ui.js';

export class FiveCLI {
  private program: Command;
  private config: CLIConfig;
  private logger: Logger;
  private context!: CommandContext;

  constructor(config: CLIConfig) {
    this.config = config;
    this.logger = createLogger({
      level: config.verbose ? 'debug' : 'info',
      enableColors: true
    });

    this.program = new Command();
    this.setupProgram();
    this.setupContext();
    this.registerCommands();
  }

  private setupProgram(): void {
    this.program
      .name('five')
      .description('')
      .version(this.getVersion(), '-V, --version', 'Display version information')
      .helpOption(false) // Disable default help to use our custom help
      .configureHelp({
        subcommandTerm: (cmd) => cmd.name()
      })
      .addHelpText('beforeAll', () => {
        return `${brandLine()}\n`;
      })
      .addHelpText('after', () => {
        return `
${section('Quick Start')}

${commandExample('five init my-project', 'Create a new Five project')}
${commandExample('five compile script.v', 'Compile Five source to bytecode')}
${commandExample('five execute script.v --local', 'Local WASM execution')}
${commandExample('five deploy build/script.bin --target mainnet', 'Deploy to Solana mainnet')}
${commandExample('five help <command>', 'Get help for specific command')}
`;
      });

    // Global options with styling
    this.program
      .option('-v, --verbose', 'Verbose output', false)
      .option('--debug', 'Debug mode', false)
      .option('--no-color', 'Disable colored output')
      .option('--config <file>', 'Use custom configuration file')
      .option('-h, --help', 'Display help information');

    // Global error handling
    this.program.exitOverride();
    this.program.configureOutput({
      outputError: (str, write) => {
        // Custom error output with colored formatting
        write(chalk.red(str));
      }
    });
  }

  private setupContext(): void {
    this.context = {
      config: this.config,
      logger: this.logger,
      wasmManager: null, // Will be initialized by individual commands
      options: {
        verbose: this.config.verbose,
        debug: this.config.debug
      }
    };
  }

  private registerCommands(): void {
    for (const commandDef of commands) {
      const command = this.program
        .command(commandDef.name)
        .description(commandDef.description);

      // Add aliases
      if (commandDef.aliases) {
        command.aliases(commandDef.aliases);
      }

      // Add options
      if (commandDef.options) {
        for (const option of commandDef.options) {
          command.option(option.flags, option.description, option.defaultValue);
        }
      }

      // Add arguments
      if (commandDef.arguments) {
        for (const arg of commandDef.arguments) {
          if (arg.variadic) {
            command.argument(`[${arg.name}...]`, arg.description);
          } else if (arg.required) {
            command.argument(`<${arg.name}>`, arg.description);
          } else {
            command.argument(`[${arg.name}]`, arg.description);
          }
        }
      }

      // Add examples to help text
      if (commandDef.examples) {
        const exampleText = commandDef.examples
          .map(ex => `  ${chalk.cyan(ex.command)}  ${ex.description}`)
          .join('\n');

        command.addHelpText('after', `\n${chalk.bold('Examples:')}\n${exampleText}\n`);
      }

      // Set command handler
      command.action(async (...args) => {
        try {
          // Commander always passes the command object as the last argument
          const commandInstance = args[args.length - 1] as Command;
          const commandArgs = args.slice(0, args.length - 1);
          const options =
            typeof commandInstance.optsWithGlobals === 'function'
              ? commandInstance.optsWithGlobals()
              : commandInstance.opts();

          // Update context with current options
          this.updateContextFromOptions(options);

          // Execute command
          await commandDef.handler(commandArgs, options, this.context);

        } catch (error) {
          await this.handleCommandError(error as Error, commandDef.name);
        }
      });
    }
  }

  private updateContextFromOptions(options: any): void {
    this.context.options = {
      ...this.context.options,
      verbose: options.verbose || this.config.verbose,
      debug: options.debug || this.config.debug,
      output: options.output,
      format: options.format,
      optimize: options.optimize,
      target: options.target
    };

    // Update logger level if verbose mode changed
    if (options.verbose && !this.config.verbose) {
      const disableColor = options.color === false || options.noColor === true;
      this.logger = createLogger({
        level: 'debug',
        enableColors: !disableColor
      });
      this.context.logger = this.logger;
    }
  }

  /**
   * Handle command execution errors with proper formatting and exit codes
   */
  private async handleCommandError(error: Error, commandName: string): Promise<void> {
    const cliError = error as CLIError;

    if (cliError.category === 'user') {
      // User errors (invalid input, missing files, etc.)
      console.error(uiError(`${commandName}: ${cliError.message}`));

      if (cliError.details && this.context.options.verbose) {
        console.error(hint(`Details: ${JSON.stringify(cliError.details)}`));
      }
    } else if (cliError.category === 'wasm') {
      // WASM-related errors
      console.error(uiError(`WASM error in ${commandName}: ${cliError.message}`));

      if (this.context.options.debug && cliError.details) {
        console.error(hint(`WASM stack: ${cliError.details.stack}`));
      }
    } else {
      // System errors
      console.error(uiError(`System error in ${commandName}: ${cliError.message}`));

      if (this.context.options.debug) {
        console.error(hint(`Stack trace: ${error.stack}`));
      }
    }

    // Exit with appropriate code
    const exitCode = cliError.exitCode || 1;
    process.exit(exitCode);
  }

  /**
   * Run the CLI with provided arguments
   */
  async run(argv: string[]): Promise<void> {
    try {
      // Handle special cases for help and version
      if (argv.includes('--help') || argv.includes('-h')) {
        this.program.help();
        return;
      }

      // Show welcome message if no command provided
      if (argv.length <= 2) {
        this.program.outputHelp();
        return;
      }

      // Check if command exists
      const commandName = argv[2];
      if (commandName && !commandName.startsWith('-')) {
        const command = getCommand(commandName);
        if (!command) {
          // Show styled command not found with suggestions
          const suggestions = rankCommandSuggestions(commandName, commands);

          console.error(commandNotFound(commandName, suggestions));
          console.error('\nAvailable commands:');
          console.error(generateCommandsHelp());
          process.exit(1);
        }
      }

      // Parse and execute
      await this.program.parseAsync(argv);

    } catch (error) {
      // Handle CLI parsing errors
      if (error instanceof Error) {
        if (error.name === 'CommanderError') {
          // Commander.js error - usually help or version display
          return;
        }

        this.logger.error('CLI Error:', error.message);
        if (this.context.options.debug) {
          console.error(hint(`Stack trace: ${error.stack}`));
        }
      }

      process.exit(1);
    }
  }

  /**
   * Get CLI version from package.json
   */
  private getVersion(): string {
    try {
      const possiblePaths = [
        join(this.config.rootDir, 'five-cli', 'package.json'), // From workspace root
        join(this.config.rootDir, 'package.json'), // Root level
      ];

      for (const path of possiblePaths) {
        try {
          const content = readFileSync(path, 'utf-8');
          const packageJson = JSON.parse(content);
          if (packageJson.version) {
            return packageJson.version;
          }
        } catch {
          continue;
        }
      }

      return '1.0.1'; // Fallback version
    } catch {
      return '1.0.1';
    }
  }

  /**
   * Get CLI program instance for testing
   */
  getProgram(): Command {
    return this.program;
  }

  /**
   * Get current configuration
   */
  getConfig(): CLIConfig {
    return this.config;
  }

  /**
   * Get current logger
   */
  getLogger(): Logger {
    return this.logger;
  }
}

function rankCommandSuggestions(input: string, available: typeof commands): string[] {
  const normalized = input.toLowerCase();
  const candidates = available.map(cmd => cmd.name);

  const ranked = candidates
    .map(name => {
      const lowerName = name.toLowerCase();
      const distance = levenshteinDistance(normalized, lowerName);
      const bonus = lowerName.startsWith(normalized) ? -2 : lowerName.includes(normalized) ? -1 : 0;
      const score = distance + bonus;
      const isClose = lowerName.includes(normalized) || distance <= 3;
      return { name, score, isClose };
    })
    .filter(candidate => candidate.isClose)
    .sort((a, b) => a.score - b.score)
    .slice(0, 3)
    .map(candidate => candidate.name);

  return ranked;
}

function levenshteinDistance(a: string, b: string): number {
  if (a === b) {
    return 0;
  }
  if (a.length === 0) {
    return b.length;
  }
  if (b.length === 0) {
    return a.length;
  }

  const matrix = Array.from({ length: a.length + 1 }, () => new Array(b.length + 1).fill(0));

  for (let i = 0; i <= a.length; i++) {
    matrix[i][0] = i;
  }
  for (let j = 0; j <= b.length; j++) {
    matrix[0][j] = j;
  }

  for (let i = 1; i <= a.length; i++) {
    for (let j = 1; j <= b.length; j++) {
      const cost = a[i - 1] === b[j - 1] ? 0 : 1;
      matrix[i][j] = Math.min(
        matrix[i - 1][j] + 1,
        matrix[i][j - 1] + 1,
        matrix[i - 1][j - 1] + cost
      );
    }
  }

  return matrix[a.length][b.length];
}

/**
 * Create and configure a new Five CLI instance
 */
export function createCLI(config: Partial<CLIConfig> = {}): FiveCLI {
  const defaultConfig: CLIConfig = {
    rootDir: process.cwd(),
    verbose: false,
    debug: false,
    wasmDir: join(process.cwd(), 'assets', 'vm'),
    tempDir: join(process.cwd(), '.five-tmp')
  };

  const finalConfig = { ...defaultConfig, ...config };
  return new FiveCLI(finalConfig);
}

/**
 * Default export for convenience
 */
export default FiveCLI;
