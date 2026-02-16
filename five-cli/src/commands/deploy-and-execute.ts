// Deploy-and-execute command.

import { readFile } from 'fs/promises';
import { extname } from 'path';
import ora from 'ora';

import {
  CommandDefinition,
  CommandContext,
  CLIOptions
} from '../types.js';
import { FiveSDK, ProgramIdResolver } from '@5ive-tech/sdk';
import { ConfigManager } from '../config/ConfigManager.js';
import { ConfigOverrides } from '../config/types.js';
import { VmClusterConfigResolver } from '../config/VmClusterConfigResolver.js';
import { FiveFileManager } from '../utils/FiveFileManager.js';
import { section, success as uiSuccess, error as uiError, isTTY } from '../utils/cli-ui.js';

export const deployAndExecuteCommand: CommandDefinition = {
  name: 'deploy-and-execute',
  description: 'Deploy bytecode to Solana and execute immediately (perfect for testing)',
  aliases: ['dae', 'test-onchain'],

  options: [
    {
      flags: '-f, --function <index>',
      description: 'Execute specific function by index (default: 0)',
      defaultValue: 0
    },
    {
      flags: '-p, --params <params>',
      description: 'Function parameters as JSON array (e.g., "[10, 5]")',
      defaultValue: '[]'
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
      flags: '--max-cu <units>',
      description: 'Maximum compute units for execution (default: 1400000)',
      defaultValue: 1400000
    },
    {
      flags: '--compute-budget <units>',
      description: 'Compute budget for deployment transaction',
      defaultValue: 1400000
    },
    {
      flags: '--format <format>',
      description: 'Output format',
      choices: ['text', 'json'],
      defaultValue: 'text'
    },
    {
      flags: '--debug',
      description: 'Enable debug output',
      defaultValue: false
    },
    {
      flags: '--skip-deployment-verification',
      description: 'Skip deployment verification',
      defaultValue: false
    },
    {
      flags: '--cleanup',
      description: 'Clean up deployed account after execution (for testing)',
      defaultValue: false
    },
    {
      flags: '--program-id <address>',
      description: 'Override Five VM program ID for this run',
      defaultValue: undefined
    }
  ],

  arguments: [
    {
      name: 'bytecode',
      description: 'Five VM bytecode file (.bin, .five, or .v source)',
      required: true
    }
  ],

  examples: [
    {
      command: 'five deploy-and-execute script.five',
      description: 'Deploy and execute function 0 on configured target'
    },
    {
      command: 'five deploy-and-execute script.five --target local -f 1 -p "[10, 5]"',
      description: 'Deploy to localnet and execute function 1 with parameters [10, 5]'
    },
    {
      command: 'five deploy-and-execute src/main.v --target local --debug',
      description: 'Compile, deploy, and execute source file on localnet with debug output'
    },
    {
      command: 'five dae script.bin --target devnet --max-cu 2000000',
      description: 'Deploy and execute on devnet with higher compute limit'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;

    try {
      const requestedTarget = options.target || 'devnet';
      const normalizedTarget = requestedTarget;

      // Prefer explicit network flag, then env overrides for local testing, else defaults
      const networkOverride =
        options.network ||
        process.env.FIVE_LOCAL_RPC_URL ||
        process.env.SOLANA_URL ||
        (normalizedTarget === 'localnet' ? 'http://127.0.0.1:8900' : undefined);

      // Apply config with CLI overrides
      const configManager = ConfigManager.getInstance();
      const overrides: ConfigOverrides = {
        target: normalizedTarget as any,
        network: networkOverride,
        keypair: options.keypair
      };

      const config = await configManager.applyOverrides(overrides);

      // Resolve program ID with precedence: CLI flag → CLI config per-target → vm cluster constants
      const configuredProgramId = await configManager.getProgramId(config.target as any);
      const programIdOverride =
        options.programId ||
        configuredProgramId ||
        VmClusterConfigResolver.loadClusterConfig({
          cluster: VmClusterConfigResolver.fromCliTarget(config.target as any),
        }).programId;

      // Show target context prefix
      const targetPrefix = ConfigManager.getTargetPrefix(config.target);
      console.log(`${targetPrefix} Deploy-and-Execute workflow starting`);

      if (context.options.verbose || options.debug) {
        logger.info(`Target: ${config.target}`);
        const targetNetwork = config.networks[config.target];
        logger.info(`Network: ${targetNetwork?.rpcUrl || networkOverride || 'unknown'}`);
        logger.info(`Keypair: ${config.keypairPath}`);
        if (programIdOverride) {
          logger.info(`Program ID override: ${programIdOverride}`);
        }
      }

      const inputFile = args[0];

      // STEP 1: Load/Compile bytecode
      let bytecode: Uint8Array;
      let abi: any = null;

      if (extname(inputFile) === '.v') {
        // Compile source file
        console.log('Compiling source...');
        const sourceCode = await readFile(inputFile, 'utf8');

        const compilationResult = await FiveSDK.compile(sourceCode, {
          optimize: true,
          debug: options.debug
        });

        if (!compilationResult.success || !compilationResult.bytecode) {
          console.log(uiError('Compilation failed'));
          if (compilationResult.errors) {
            compilationResult.errors.forEach(error => {
              console.log(`  - ${error.message}`);
            });
          }
          process.exit(1);
        }

        bytecode = compilationResult.bytecode;
        abi = compilationResult.abi;
        console.log(uiSuccess(`Compiled (${bytecode.length} bytes)`));

      } else {
        // Load existing bytecode file
        console.log('Loading bytecode...');
        const fileManager = FiveFileManager.getInstance();
        const loadedFile = await fileManager.loadFile(inputFile, {
          validateFormat: true
        });

        bytecode = loadedFile.bytecode;
        abi = loadedFile.abi;
        console.log(uiSuccess(`Loaded ${loadedFile.format.toUpperCase()} (${bytecode.length} bytes)`));
      }

      // Setup Solana connection and keypair
      const { Connection, Keypair } = await import('@solana/web3.js');
      const rpcUrl = config.networks[config.target].rpcUrl;
      // Guard against non-HTTP(S) endpoints (e.g., wasm://) when doing on-chain work
      if (!/^https?:\/\//.test(rpcUrl)) {
        throw new TypeError(
          `Invalid RPC URL for on-chain execution: ${rpcUrl}. Endpoint URL must start with http: or https:. ` +
          `Use --target local/devnet/testnet/mainnet or override with --network <http(s) URL>.`
        );
      }
      const connection = new Connection(rpcUrl, 'confirmed');

      // Load deployer keypair
      const keypairPath = config.keypairPath.startsWith('~/')
        ? config.keypairPath.replace('~', process.env.HOME || '')
        : config.keypairPath;
      const keypairData = JSON.parse(await readFile(keypairPath, 'utf8'));
      const deployerKeypair = Keypair.fromSecretKey(new Uint8Array(keypairData));

      if (options.debug) {
        console.log(`Deployer: ${deployerKeypair.publicKey.toString()}`);
      }

      // STEP 2: Deploy bytecode
      console.log('Deploying to Solana...');
      const deploySpinner = isTTY() ? ora('Deploying bytecode...').start() : null;

      const deploymentResult = await FiveSDK.deployToSolana(
        bytecode,
        connection,
        deployerKeypair,
        {
          debug: options.debug,
          network: config.target,
          computeBudget: options.computeBudget,
          maxRetries: 3,
          fiveVMProgramId: programIdOverride
        }
      );

      if (!deploymentResult.success) {
        if (deploySpinner) {
          deploySpinner.fail('Deployment failed');
        }
        try {
          console.log(uiError(`Deployment error: ${deploymentResult.error || 'unknown'}`));
          if (options.debug) {
            console.log('Deployment result:', JSON.stringify(deploymentResult, null, 2));
          }
        } catch { }
        process.exit(1);
      }

      const scriptAccount = deploymentResult.programId!;
      if (deploySpinner) {
        deploySpinner.succeed(`Deployed: ${scriptAccount}`);
      } else {
        console.log(uiSuccess(`Deployed: ${scriptAccount}`));
      }

      if (deploymentResult.logs && (context.options.verbose || options.debug)) {
        console.log(section('Deployment Logs'));
        deploymentResult.logs.forEach((log: string) => {
          console.log(`  ${log}`);
        });
      }

      // STEP 3: Wait for deployment to fully propagate, then execute
      console.log('Waiting for deployment to propagate...');
      await new Promise(resolve => setTimeout(resolve, 2000)); // 2 second delay

      console.log('Executing deployed script...');

      // Parse function index and parameters
      const functionIndex = parseInt(options.function) || 0;
      let parameters: any[] = [];

      try {
        if (options.params && options.params !== '[]') {
          parameters = JSON.parse(options.params);
        }
      } catch (error) {
        console.log(uiError(`Invalid parameters JSON: ${options.params}`));
        process.exit(1);
      }

      if (options.debug) {
        console.log(`Function ${functionIndex} params: ${JSON.stringify(parameters)}`);
      }

      const executeSpinner = isTTY() ? ora('Executing function...').start() : null;
      if (options.debug) {
        console.log(`VM state account: ${(deploymentResult as any).vmStateAccount || '(derived)'}`);
      }

      const executionResult = await FiveSDK.executeScriptAccount(
        scriptAccount,
        functionIndex,
        parameters,
        connection,
        deployerKeypair,
        {
          debug: options.debug,
          network: config.target,
          computeBudget: options.maxCu,
          maxRetries: 3,
          // If deployment returned a VM state account, prefer it
          vmStateAccount: (deploymentResult as any).vmStateAccount,
          fiveVMProgramId: programIdOverride
        }
      );

      if (executionResult.success) {
        if (executeSpinner) {
          executeSpinner.succeed('Execution completed');
        }

        // Display results
        displayExecutionResults(executionResult, options, targetPrefix);

      } else {
        if (executeSpinner) {
          executeSpinner.fail('Execution failed');
        }
        console.log(uiError(`Execution error: ${executionResult.error}`));

        if (executionResult.logs && executionResult.logs.length > 0) {
          console.log(section('Error Logs'));
          executionResult.logs.forEach(log => {
            console.log(`  ${log}`);
          });
        }

        process.exit(1);
      }

      // STEP 4: Optional cleanup (for testing)
      if (options.cleanup) {
        console.log('Cleanup requested (not implemented yet)');
        console.log(`  Script account ${scriptAccount} will remain on chain`);
      }

      console.log(uiSuccess('Deploy-and-execute completed'));

    } catch (error) {
      logger.error('Deploy-and-Execute workflow failed:', error);
      throw error;
    }
  }
};

/**
 * Display execution results in a formatted way
 */
function displayExecutionResults(result: any, options: any, targetPrefix: string): void {
  if (options.format === 'json') {
    console.log(JSON.stringify(result, null, 2));
    return;
  }

  console.log('\n' + section(`${targetPrefix} Execution`));

  if (result.transactionId) {
    console.log(`Transaction: ${result.transactionId}`);
  }

  if (result.computeUnitsUsed !== undefined) {
    console.log(`Compute units: ${result.computeUnitsUsed.toLocaleString()}`);
  }

  if (result.result !== undefined) {
    console.log(`Result: ${JSON.stringify(result.result)}`);
  }

  if (result.logs && result.logs.length > 0) {
    console.log('\n' + section('Transaction Logs'));
    result.logs.forEach((log: string) => {
      // Filter out system logs and show only relevant ones
      if (log.includes('Five') || log.includes('success') || log.includes('error') || log.includes('Program log:')) {
        console.log(`  ${log}`);
      }
    });
  }
}
