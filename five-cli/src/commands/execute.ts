// Execute command.

import { readFile } from 'fs/promises';
import { readFileSync, existsSync } from 'fs';
import { extname, isAbsolute, join, resolve } from 'path';
import ora from 'ora';

import {
  CommandDefinition,
  CommandContext,
  VMExecutionOptions,
  AccountInfo,
  CLIOptions
} from '../types.js';
import { FiveSDK, ProgramIdResolver } from 'five-sdk';
import { ConfigManager } from '../config/ConfigManager.js';
import { ConfigOverrides } from '../config/types.js';
import { FiveFileManager } from '../utils/FiveFileManager.js';
import { loadBuildManifest, loadProjectConfig } from '../project/ProjectLoader.js';
import { section, success as uiSuccess, error as uiError, hint, keyValue } from '../utils/cli-ui.js';

/**
 * Five execute command implementation
 */
export const executeCommand: CommandDefinition = {
  name: 'execute',
  description: 'Execute Five VM bytecode',
  aliases: ['exec', 'run'],

  options: [
    {
      flags: '-i, --input <file>',
      description: 'Input data file (JSON format)',
      required: false
    },
    {
      flags: '-a, --accounts <file>',
      description: 'Accounts configuration file (JSON format) [local execution only] ',
      required: false
    },
    {
      flags: '--accounts-json <json>',
      description: 'Additional accounts for on-chain execution as JSON array or object of pubkeys',
      required: false
    },
    {
      flags: '-f, --function <index>',
      description: 'Execute specific function by index',
      required: false
    },
    {
      flags: '-p, --params <file>',
      description: 'Function parameters file (JSON format)',
      required: false
    },
    {
      flags: '--max-cu <units>',
      description: 'Maximum compute units (default: 1000000)',
      defaultValue: 1000000
    },
    {
      flags: '--validate',
      description: 'Validate bytecode before execution',
      defaultValue: false
    },
    {
      flags: '--partial',
      description: 'Enable partial execution (stops at system calls)',
      defaultValue: true
    },
    {
      flags: '--format <format>',
      description: 'Output format',
      choices: ['text', 'json', 'table'],
      defaultValue: 'text'
    },
    {
      flags: '--trace',
      description: 'Show execution trace',
      defaultValue: false
    },
    {
      flags: '--state',
      description: 'Show VM state after execution',
      defaultValue: false
    },
    {
      flags: '-t, --target <target>',
      description: 'Override target network (devnet, testnet, mainnet, local)',
      required: false
    },
    {
      flags: '-n, --network <url>',
      description: 'Override network RPC URL',
      required: false
    },
    {
      flags: '-k, --keypair <file>',
      description: 'Override keypair file path',
      required: false
    },
    {
      flags: '--local',
      description: 'Force local execution (overrides config)',
      defaultValue: false
    },
    {
      flags: '--script-account <account>',
      description: 'Execute deployed script by account ID (on-chain execution)',
      required: false
    },
    {
      flags: '--vm-state-account <account>',
      description: 'VM state account address (optional, required for on-chain execution if known)',
      required: false
    },
    {
      flags: '--program-id <id>',
      description: 'Override Five VM program ID (for custom deployments)',
      required: false
    },
    {
      flags: '--project <path>',
      description: 'Project directory or five.toml path',
      required: false
    }
  ],

  arguments: [
    {
      name: 'bytecode',
      description: 'Five VM artifact file (.five/.bin) or script account ID',
      required: false
    }
  ],

  examples: [
    {
      command: 'five execute program.five',
      description: 'Execute using configured target (default)'
    },
    {
      command: 'five execute program.five --local',
      description: 'Force local execution (overrides config)'
    },
    {
      command: 'five execute program.five --target devnet',
      description: 'Execute on devnet (overrides config)'
    },
    {
      command: 'five execute program.five -f 0 -p params.json',
      description: 'Execute function 0 with parameters'
    },
    {
      command: 'five execute program.five --validate --trace --format json',
      description: 'Validate and execute with JSON trace output'
    },
    {
      command: 'five execute src/main.v -f 0 --local --params "[10, 5]"',
      description: 'Compile and execute Five source locally with parameters'
    },
    {
      command: 'five execute --script-account 459SanqV8nQDDYW3gWq5JZZAPCMYs78Z5ZnrtH4eFffw -f 0',
      description: 'Execute deployed script by account ID on-chain'
    },
    {
      command: 'five execute --script-account 459SanqV8nQDDYW3gWq5JZZAPCMYs78Z5ZnrtH4eFffw -f 0 --program-id 9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH',
      description: 'Execute with custom Five VM program ID (for custom deployments)'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;

    try {
      const projectContext = await loadProjectConfig(options.project, process.cwd());
      const manifest = projectContext ? await loadBuildManifest(projectContext.rootDir) : null;

      // Apply project defaults to options if not provided
      if (!options.target && projectContext?.config.cluster) {
        options.target = projectContext.config.cluster;
      }
      if (!options.network && projectContext?.config.rpcUrl) {
        options.network = projectContext.config.rpcUrl;
      }
      if (!options.keypair && projectContext?.config.keypairPath) {
        options.keypair = projectContext.config.keypairPath;
      }

      let inputFile = args[0] || manifest?.artifact_path;
      if (inputFile && projectContext && !isAbsolute(inputFile)) {
        inputFile = join(projectContext.rootDir, inputFile);
      }
      const scriptAccount = options.scriptAccount;

      // Debug: Log received options
      if (context.options.verbose) {
        console.log('[execute] Received options:', Object.keys(options));
        console.log('[execute] scriptAccount value:', scriptAccount);
      }

      // Validate input - either bytecode file OR script account required
      if (!inputFile && !scriptAccount) {
        throw new Error(
          'No bytecode or script account provided. Pass a .five/.bin file, use --project to load the last build artifact, or provide --script-account for on-chain execution.'
        );
      }

      // Apply config with CLI overrides
      const configManager = ConfigManager.getInstance();
      const overrides: ConfigOverrides = {
        target: options.target,
        network: options.network,
        keypair: options.keypair
      };

      const config = await configManager.applyOverrides(overrides);

      // Resolve program ID AFTER target override is applied, using the correct target
      // Precedence: CLI flag → project config → config file (per-target) → env var
      if (!options.programId) {
        const configuredProgramId = await configManager.getProgramId(config.target as any);
        options.programId = projectContext?.config.programId || configuredProgramId || process.env.FIVE_PROGRAM_ID;
      }

      // Show target context prefix
      const targetPrefix = ConfigManager.getTargetPrefix(config.target);

      // Force local execution if --local flag is used, but not if script account is specified
      const forceLocal = options.local || false;
      const executeLocally = (config.target === 'wasm' || forceLocal) && !scriptAccount;

      // Log execution mode only in verbose
      if (context.options.verbose) {
        if (scriptAccount) {
          logger.info(`${targetPrefix} Executing deployed script account on-chain`);
        } else if (executeLocally) {
          logger.info(`${ConfigManager.getTargetPrefix('wasm')} Executing Five VM bytecode locally`);
        } else {
          logger.info(`${targetPrefix} Executing Five VM bytecode`);
        }
      }

      // Show config details only if explicitly enabled and verbose
      if (config.showConfig && !executeLocally && context.options.verbose) {
        logger.info(`Target: ${config.target}`);
        logger.info(`Network: ${config.networks[config.target].rpcUrl}`);
        logger.info(`Keypair: ${config.keypairPath}`);
      }

      if (scriptAccount) {
        await executeScriptAccount(scriptAccount, options, context, config);
      } else if (executeLocally) {
        await executeLocallyWithSDK(inputFile!, options, context, config);
      } else {
        await executeOnChain(inputFile!, options, context, config);
      }

    } catch (error) {
      logger.error('Execution failed:', error);
      throw error;
    }
  }
};

/**
 * Execute locally using Five SDK
 */
async function executeLocallyWithSDK(inputFile: string, options: any, context: CommandContext, config: any): Promise<void> {
  const { logger } = context;

  // Initialize for local execution
  if (context.options.verbose) {
    const spinner = ora('Preparing local execution...').start();
    spinner.succeed('Five SDK ready for local execution');
  }

  try {
    let result;

    if (extname(inputFile) === '.v') {
      // Compile and execute Five source file
      logger.info(`Compiling and executing Five source: ${inputFile}`);

      const sourceCode = await readFile(inputFile, 'utf8');

      // For .v files, we don't have ABI yet, so we'll just use the provided function or default to 0
      // The real fix here would be to compile first, extract ABI, then auto-detect the public function
      const functionIndex = parseFunctionIndex(options.function) || 0;
      const parameters = parseParameters(options.params);

      // Parse accounts if provided
      const accounts = options.accounts ? parseAccounts(options.accounts) : undefined;

      result = await FiveSDK.execute(
        sourceCode,
        functionIndex,
        parameters,
        {
          debug: options.trace || context.options.debug,
          trace: options.trace,
          optimize: true,
          computeUnitLimit: options.maxCu,
          accounts
        }
      );

      // Display compilation info with proper type guard
      if ('compilation' in result && result.compilation) {
        console.log(`Compilation: ${result.compilation.success ? 'OK' : 'FAIL'}`);
        if ('bytecodeSize' in result && result.bytecodeSize) {
          console.log(`Bytecode size: ${result.bytecodeSize} bytes`);
        }
      }

    } else {
      // Execute existing bytecode file
      logger.info(`Executing bytecode: ${inputFile}`);

      // Load file using centralized manager
      const fileManager = FiveFileManager.getInstance();
      const loadedFile = await fileManager.loadFile(inputFile, {
        validateFormat: true
      });

      const bytecode = loadedFile.bytecode;
      const abi = loadedFile.abi;

      if (options.trace || context.options.debug) {
        console.log(`[CLI] Loaded ${loadedFile.format.toUpperCase()} file: bytecode=${bytecode.length} bytes`);
        if (abi && abi.functions) {
          console.log(`[CLI] Available functions: ${Object.keys(abi.functions).length}`);
        }
      }

      // FUNCTION RESOLUTION FIX: Auto-detect public function when no -f flag provided
      let functionIndex = parseFunctionIndex(options.function);

      if (functionIndex === undefined && abi && abi.functions) {
        // SYSTEMATIC APPROACH: Public functions always have the lowest indices (0, 1, 2...)
        // So we always default to function index 0, which is guaranteed to be public if any functions exist
        functionIndex = 0;

        if (context.options.verbose) {
          let functionName = 'function_0';

          if (Array.isArray(abi.functions)) {
            // FIVEABI format: find function with index 0
            const func0 = abi.functions.find((f: any) => f.index === 0);
            if (func0 && func0.name) functionName = func0.name;
          } else {
            // SimpleABI format: find function with index 0  
            const func0Entry = Object.entries(abi.functions).find(([_, f]: [string, any]) => f.index === 0);
            if (func0Entry) functionName = func0Entry[0];
          }

          logger.info(`Auto-detected public function: ${functionName} (index 0 - first public function)`);
        }
      } else if (functionIndex === undefined) {
        functionIndex = 0; // No ABI available, use legacy default
      }

      // Ensure functionIndex is never undefined
      if (functionIndex === undefined) {
        functionIndex = 0;
      }

      const parameters = parseParameters(options.params);

      // Parse accounts if provided
      const accounts = options.accounts ? parseAccounts(options.accounts) : undefined;

      result = await FiveSDK.executeLocally(
        bytecode,
        functionIndex,
        parameters,
        {
          debug: options.trace || context.options.debug,
          trace: options.trace,
          computeUnitLimit: options.maxCu,
          abi: abi, // Pass ABI for function name resolution
          accounts
        }
      );
    }

    // Display execution results
    displayLocalExecutionResult(result, options, logger);

  } catch (error) {
    throw new Error(`Local execution failed: ${error}`);
  }
}

/**
 * Execute deployed script account on-chain using Five SDK
 */
async function executeScriptAccount(scriptAccount: string, options: any, context: CommandContext, config: any): Promise<void> {
  const { logger } = context;
  const { Connection, Keypair } = await import('@solana/web3.js');

  const targetPrefix = ConfigManager.getTargetPrefix(config.target);
  logger.info(`${targetPrefix} Executing script account: ${scriptAccount}`);

  try {
    // Show configuration
    console.log('\n' + section('Execution Configuration'));
    console.log(`  Script Account: ${scriptAccount}`);
    console.log(`  Target: ${config.target}`);
    console.log(`  Network: ${config.networks[config.target].rpcUrl}`);
    console.log(`  Keypair: ${config.keypairPath}`);

    // Set up connection and keypair
    const rpcUrl = config.networks[config.target].rpcUrl;
    const connection = new Connection(rpcUrl, 'confirmed');

    // Load keypair
    const keypairPath = config.keypairPath;
    const keypairData = JSON.parse(await readFile(keypairPath, 'utf8'));
    const deployerKeypair = Keypair.fromSecretKey(new Uint8Array(keypairData));

    console.log(`  Executor: ${deployerKeypair.publicKey.toBase58()}`);

    const spinner = ora('Executing script account on-chain...').start();

    try {
      // Get function index, parameters, and additional accounts
      const functionIndex = options.function ? parseInt(options.function) : 0;
      const parameters = parseParameters(options.params);

      // Parse additional accounts for on-chain execution
      let additionalAccounts: string[] = [];
      if (options.accountsJson) {
        try {
          const parsed = JSON.parse(options.accountsJson);
          if (Array.isArray(parsed)) {
            additionalAccounts = parsed;
          } else if (typeof parsed === 'object' && parsed !== null) {
            // Preserve insertion order from the JSON string
            const orderedKeys = Object.keys(parsed);
            additionalAccounts = orderedKeys.map(k => parsed[k]);
          }
        } catch (e) {
          console.warn('Warning: Failed to parse --accounts-json, ignoring.');
        }
      }

      // Validate program ID for script account execution
      let resolvedProgramId: string | undefined;
      if (config.target !== 'wasm') {
        try {
          resolvedProgramId = ProgramIdResolver.resolve(options.programId);
        } catch (error) {
          throw new Error(
            `Program ID required for script account execution on ${config.target}. ` +
            `Provide via: --program-id <pubkey>, five.toml programId, ` +
            `or: five config set --program-id <pubkey>`
          );
        }
        options.programId = resolvedProgramId;
      }

      // Fetch current fees from VM state
      let fees;
      try {
        fees = await FiveSDK.getFees(connection, options.programId);

        if (fees.executeFeeBps > 0) {
          if (context.options.verbose || options.debug) {
            console.log('\n' + section('VM Fees'));
            console.log(keyValue('Execution Fee', `${(fees.executeFeeBps / 100).toFixed(2)}%`));
            if (fees.adminAccount) {
              console.log(keyValue('Admin Account', fees.adminAccount));
            }
          }

          // Attach admin account to options
          options.adminAccount = fees.adminAccount;
        }
      } catch (e) {
        if (context.options.debug) {
          logger.debug(`Could not fetch VM fees: ${e instanceof Error ? e.message : String(e)}`);
        }
      }

      // Execute script account on-chain with additional accounts
      const executeOptions: any = {
        debug: options.trace || context.options.debug,
        network: config.target,
        computeUnitLimit: options.maxCu || 1400000,
        maxRetries: 3,
        vmStateAccount: options.vmStateAccount,
        adminAccount: options.adminAccount // Pass admin account
      };

      // Add program ID override if provided
      if (options.programId) {
        executeOptions.fiveVMProgramId = options.programId;
      }

      const result = await FiveSDK.executeOnSolana(
        scriptAccount,
        connection,
        deployerKeypair,
        functionIndex,
        parameters,
        additionalAccounts,
        executeOptions
      );

      spinner.succeed('Script account execution completed');

      // Display results
      displayOnChainExecutionResult(result, options, logger);

    } catch (error) {
      spinner.fail('Script account execution failed');
      throw error;
    }

  } catch (error) {
    throw new Error(`Script account execution failed: ${error}`);
  }
}

/**
 * Execute on-chain using Five SDK
 */
async function executeOnChain(inputFile: string, options: any, context: CommandContext, config: any): Promise<void> {
  const { logger } = context;
  const { Connection, Keypair } = await import('@solana/web3.js');

  const targetPrefix = ConfigManager.getTargetPrefix(config.target);
  logger.info(`${targetPrefix} On-chain execution using Five SDK`);

  try {
    // Show configuration
    console.log('\n' + section('Execution Configuration'));
    console.log(`  Target: ${config.target}`);
    console.log(`  Network: ${config.networks[config.target].rpcUrl}`);
    console.log(`  Keypair: ${config.keypairPath}`);

    // Check if input is a script account (base58 string) or bytecode file
    let scriptAccount: string;

    if (inputFile.length > 30 && inputFile.length < 50 && !inputFile.includes('/') && !inputFile.includes('.')) {
      // Looks like a base58 script account address
      scriptAccount = inputFile;
      console.log(`Using script account: ${scriptAccount}`);
    } else {
      // It's a file - need to deploy first or prompt user
      console.log(uiError('On-chain execution requires a deployed script account.'));
      console.log(hint(`deploy first: five deploy ${inputFile}`));
      console.log(hint(`then execute: five execute <SCRIPT_ACCOUNT> --target ${config.target}`));
      console.log(hint(`or run locally: five execute ${inputFile} --local`));
      return;
    }

    // Setup connection
    const rpcUrl = config.networks[config.target].rpcUrl;
    const connection = new Connection(rpcUrl, 'confirmed');

    // Load signer keypair
    const signerKeypair = await loadKeypair(config.keypairPath, logger);

    // Parse execution options
    const functionName = parseFunctionIndex(options.function) || 0;
    const parameters = parseParameters(options.params);
    const accounts: string[] = []; // No additional accounts for simple execution

    console.log(`\nExecuting function ${functionName} with ${parameters.length} parameters...`);

    // Fetch current fees from VM state
    let fees;
    try {
      fees = await FiveSDK.getFees(connection, options.programId);

      if (fees.executeFeeBps > 0) {
        if (context.options.verbose || options.debug) {
          console.log('\n' + section('VM Fees'));
          console.log(keyValue('Execution Fee', `${(fees.executeFeeBps / 100).toFixed(2)}%`));
          if (fees.adminAccount) {
            console.log(keyValue('Admin Account', fees.adminAccount));
          }
        }

        // Attach admin account to options
        options.adminAccount = fees.adminAccount;
      }
    } catch (e) {
      if (context.options.debug) {
        logger.debug(`Could not fetch VM fees: ${e instanceof Error ? e.message : String(e)}`);
      }
    }

    // Execute using Five SDK
    const spinner = ora('Executing on-chain via Five SDK...').start();

    const executeOptions: any = {
      debug: options.debug || context.options.debug || false,
      network: config.target,
      computeUnitLimit: options.maxCu,
      maxRetries: 3,
      vmStateAccount: options.vmStateAccount,
      adminAccount: options.adminAccount // Pass admin account
    };

    // Add program ID override if provided
    if (options.programId) {
      executeOptions.fiveVMProgramId = options.programId;
    }

    const result = await FiveSDK.executeOnSolana(
      scriptAccount,
      connection,
      signerKeypair,
      functionName,
      parameters,
      accounts,
      executeOptions
    );

    if (result.success) {
      spinner.succeed('On-chain execution completed successfully!');
      displayOnChainExecutionResult(result, options, logger);
    } else {
      spinner.fail('On-chain execution failed');
      displayOnChainExecutionResult(result, options, logger);
      process.exit(1);
    }

  } catch (error) {
    logger.error('On-chain execution failed:', error);
    throw error;
  }
}

/**
 * Parse function index - handle both numeric strings and function names
 */
function parseFunctionIndex(functionOption: any): string | number | undefined {
  if (!functionOption) {
    return undefined;
  }

  // If it's already a number, return it
  if (typeof functionOption === 'number') {
    return functionOption;
  }

  // If it's a string that looks like a number, convert it
  if (typeof functionOption === 'string') {
    const numericValue = parseInt(functionOption, 10);
    if (!isNaN(numericValue) && numericValue.toString() === functionOption) {
      return numericValue;
    }
    // Otherwise, treat it as a function name
    return functionOption;
  }

  return functionOption;
}

/**
 * Parse parameters from JSON string or file
 */
function parseParameters(paramsOption: any): any[] {
  if (!paramsOption) {
    return [];
  }

  try {
    // Try to parse as JSON directly
    if (typeof paramsOption === 'string' && paramsOption.startsWith('[')) {
      return JSON.parse(paramsOption);
    }

    // Handle parameter files
    if (typeof paramsOption === 'string') {
      const filePath = isAbsolute(paramsOption) ? paramsOption : resolve(process.cwd(), paramsOption);
      if (existsSync(filePath)) {
        const fileContent = readFileSync(filePath, 'utf-8');
        return JSON.parse(fileContent);
      }
    }

    // If it looks like a file path but didn't exist, and doesn't look like JSON
    if (typeof paramsOption === 'string' && (paramsOption.endsWith('.json') || paramsOption.includes('/'))) {
      throw new Error(`Parameter file not found or invalid: ${paramsOption}`);
    }

    return [];
  } catch (error) {
    throw new Error(`Failed to parse parameters: ${error}`);
  }
}

/**
 * Parse accounts from comma-separated string
 */
function parseAccounts(accountsOption: any): string[] | undefined {
  if (!accountsOption) {
    return undefined;
  }

  try {
    // If it's a string, split by comma and trim whitespace
    if (typeof accountsOption === 'string') {
      const accounts = accountsOption
        .split(',')
        .map(account => account.trim())
        .filter(account => account.length > 0);
      return accounts.length > 0 ? accounts : undefined;
    }

    // If it's already an array, return as-is
    if (Array.isArray(accountsOption)) {
      return accountsOption;
    }

    return undefined;
  } catch (error) {
    throw new Error(`Failed to parse accounts: ${error}`);
  }
}

/**
 * Display on-chain execution results
 */
function displayOnChainExecutionResult(result: any, options: any, logger: any): void {
  if (options.format === 'json') {
    console.log(JSON.stringify(result, null, 2));
    return;
  }

  console.log('\n' + section('On-Chain Execution'));

  if (result.success) {
    console.log(uiSuccess('Execution succeeded'));

    if (result.transactionId) {
      console.log(`Transaction: ${result.transactionId}`);
    }

    if (result.computeUnitsUsed !== undefined) {
      console.log(`Compute units: ${result.computeUnitsUsed}`);
    }

    if (result.result !== undefined) {
      console.log(`Result: ${result.result}`);
    }

    if (result.logs && result.logs.length > 0) {
      console.log('\n' + section('Logs'));
      result.logs.forEach((log: string) => {
        // Filter out system logs and show only Five VM logs
        if (log.includes('Five') || log.includes('success') || log.includes('error')) {
          console.log(`  ${log}`);
        }
      });
    }

  } else {
    console.log(uiError('Execution failed'));

    if (result.error) {
      console.log(`Error: ${result.error}`);
    }

    if (result.transactionId) {
      console.log(`Transaction: ${result.transactionId}`);
    }

    if (result.logs && result.logs.length > 0) {
      console.log('\n' + section('Error Logs'));
      result.logs.forEach((log: string) => {
        console.log(`  ${log}`);
      });
    }
  }
}

/**
 * Load Solana keypair from file
 */
async function loadKeypair(keypairPath: string, logger: any): Promise<any> {
  const { readFile } = await import('fs/promises');
  const { Keypair } = await import('@solana/web3.js');

  // Expand tilde in path
  const path = keypairPath.startsWith('~/')
    ? keypairPath.replace('~', process.env.HOME || '')
    : keypairPath;

  try {
    const keypairData = await readFile(path, 'utf8');
    const secretKey = Uint8Array.from(JSON.parse(keypairData));
    const keypair = Keypair.fromSecretKey(secretKey);

    if (logger.debug) {
      logger.debug(`Loaded keypair from: ${path}`);
      logger.debug(`Public key: ${keypair.publicKey.toString()}`);
    }

    return keypair;
  } catch (error) {
    throw new Error(`Failed to load keypair from ${path}: ${error}`);
  }
}

/**
 * Display local execution results
 */
function displayLocalExecutionResult(result: any, options: any, logger: any): void {
  console.log('\n' + section('Local Execution'));

  if (result.success) {
    console.log(uiSuccess('Execution succeeded'));

    if (result.result !== undefined) {
      console.log(`  Result: ${JSON.stringify(result.result)}`);
    }

    if (result.executionTime) {
      console.log(`  Time: ${result.executionTime}ms`);
    }

    if (result.computeUnitsUsed) {
      console.log(`  Compute units: ${result.computeUnitsUsed}`);
    }

    if (result.logs && result.logs.length > 0) {
      console.log('\n' + section('Logs'));
      result.logs.forEach((log: string) => {
        console.log(`  ${log}`);
      });
    }

    if (result.trace && options.trace) {
      console.log('\n' + section('Trace'));
      // Display trace information if available
      if (Array.isArray(result.trace)) {
        result.trace.slice(0, 10).forEach((step: any, i: number) => {
          console.log(`  ${i}: ${JSON.stringify(step)}`);
        });
        if (result.trace.length > 10) {
          console.log(`  ... and ${result.trace.length - 10} more steps`);
        }
      }
    }

  } else {
    console.log(uiError('Execution failed'));

    if (result.error) {
      if (typeof result.error === 'object' && result.error.message) {
        console.log(`  Error: ${result.error.message}`);
        if (result.error.type) {
          console.log(`  Type: ${result.error.type}`);
        }
      } else {
        const errorMessage = typeof result.error === 'object'
          ? JSON.stringify(result.error, null, 2)
          : result.error;
        console.log(`  Error: ${errorMessage}`);
      }
    }

    if (result.compilationErrors && result.compilationErrors.length > 0) {
      console.log('\n  Compilation errors:');
      result.compilationErrors.forEach((error: any) => {
        console.log(`    - ${error.message || error}`);
      });
    }
  }

  // Display output format
  if (options.format === 'json') {
    console.log('\n' + section('JSON Output'));
    console.log(JSON.stringify(result, null, 2));
  }
}
