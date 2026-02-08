// Deploy command.

import { readFile } from 'fs/promises';
import { extname, isAbsolute, join } from 'path';
import ora from 'ora';
import { Connection, Keypair } from '@solana/web3.js';

import {
  CommandDefinition,
  CommandContext,
  DeploymentOptions,
  DeploymentResult,
  CLIOptions
} from '../types.js';
import { FiveSDK } from 'five-sdk';
import { ConfigManager } from '../config/ConfigManager.js';
import { ConfigOverrides } from '../config/types.js';
import { FiveFileManager } from '../utils/FiveFileManager.js';
import { loadBuildManifest, loadProjectConfig } from '../project/ProjectLoader.js';
import {
  section,
  keyValue,
  success as uiSuccess,
  error as uiError,
  isTTY
} from '../utils/cli-ui.js';

/**
 * Five deploy command implementation
 */
export const deployCommand: CommandDefinition = {
  name: 'deploy',
  description: 'Deploy bytecode to Solana',
  aliases: ['d'],

  options: [
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
      flags: '--program-id <pubkey>',
      description: 'Specify the Five VM program ID to be the owner of the new script account',
      required: false
    },
    {
      flags: '--vm-state-account <pubkey>',
      description: 'Specify an existing VM State Account (skips creation)',
      required: false
    },
    {
      flags: '--max-data-size <size>',
      description: 'Maximum data size for program account (bytes)',
      defaultValue: 1000000
    },
    {
      flags: '--compute-budget <units>',
      description: 'Compute budget for deployment transaction',
      defaultValue: 1400000
    },
    {
      flags: '--skip-verification',
      description: 'Skip deployment verification',
      defaultValue: false
    },
    {
      flags: '--dry-run',
      description: 'Simulate deployment without executing',
      defaultValue: false
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
      flags: '--chunk-size <bytes>',
      description: 'Chunk size for large program deployment (default: 750)',
      defaultValue: 750
    },
    {
      flags: '--progress',
      description: 'Show progress for large program deployments',
      defaultValue: false
    },
    {
      flags: '--force-chunked',
      description: 'Force chunked deployment even for small programs',
      defaultValue: false
    },
    {
      flags: '--optimized',
      description: 'Use optimized deployment instructions (50-70% fewer transactions)',
      defaultValue: false
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
      description: 'Five VM bytecode file (.bin)',
      required: true
    }
  ],

  examples: [
    {
      command: 'five deploy program.bin',
      description: 'Deploy to configured target (uses config defaults)'
    },
    {
      command: 'five deploy program.bin --target mainnet',
      description: 'Deploy to mainnet (overrides config)'
    },
    {
      command: 'five deploy program.bin --keypair deployer.json --target devnet',
      description: 'Deploy to devnet with specific keypair'
    },
    {
      command: 'five deploy program.bin --dry-run --format json',
      description: 'Simulate deployment with JSON output'
    },
    {
      command: 'five deploy large-program.bin --optimized --progress',
      description: 'Deploy large program with optimized instructions (50-70% fewer transactions)'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;

    if (context.options.debug) {
      logger.debug(`deploy args: ${JSON.stringify(args)}`);
      logger.debug(`deploy options: ${JSON.stringify(options)}`);
    }

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
      if (!options.programId) {
        options.programId = projectContext?.config.programId || process.env.FIVE_PROGRAM_ID;
      }

      if (context.options.verbose) {
        logger.info('Applying configuration overrides...');
      }
      // Apply config with CLI overrides
      const configManager = ConfigManager.getInstance();
      const overrides: ConfigOverrides = {
        target: options.target,
        network: options.network,
        keypair: options.keypair
      };
      if (context.options.debug) {
        logger.debug(`overrides: ${JSON.stringify(overrides)}`);
      }

      const config = await configManager.applyOverrides(overrides);
      if (context.options.debug) {
        logger.debug(`config: ${JSON.stringify(config, null, 2)}`);
      }

      // Show target context prefix
      const targetPrefix = ConfigManager.getTargetPrefix(config.target);
      if (context.options.verbose) {
        logger.info(`${targetPrefix} Deploying Five VM bytecode`);
      }

      // Show config details if enabled
      if (config.showConfig && context.options.verbose) {
        logger.info(`Target: ${config.target}`);
        logger.info(`Network: ${config.networks[config.target].rpcUrl}`);
        logger.info(`Keypair: ${config.keypairPath}`);
      }

      // Load bytecode using centralized file manager
      let bytecodeFile = args[0] || manifest?.artifact_path;
      if (bytecodeFile && projectContext && !isAbsolute(bytecodeFile)) {
        bytecodeFile = join(projectContext.rootDir, bytecodeFile);
      }

      if (!bytecodeFile) {
        throw new Error('Bytecode file is required (pass a path or compile to generate .five/.bin and manifest)');
      }
      if (context.options.verbose) {
        logger.info(`Loading bytecode: ${bytecodeFile}`);
      }

      const spinner = isTTY() ? ora('Loading bytecode...').start() : null;
      const fileManager = FiveFileManager.getInstance();
      const loadedFile = await fileManager.loadFile(bytecodeFile, {
        validateFormat: true
      });

      if (spinner) {
        spinner.succeed(`Loaded ${loadedFile.format.toUpperCase()} (${loadedFile.bytecode.length} bytes)`);
      }
      if (context.options.verbose) {
        logger.info(`Loaded ${loadedFile.bytecode.length} bytes`);
      }

      // Show additional info for .five files
      if (loadedFile.abi) {
        const functionCount = Object.keys(loadedFile.abi.functions || {}).length;
        if (context.options.verbose) {
          logger.info(`ABI functions: ${functionCount}`);
        }
        if (functionCount > 0) {
          const functionNames = Object.keys(loadedFile.abi.functions).join(', ');
          if (context.options.debug) {
            logger.debug(`ABI functions: ${functionNames}`);
          }
        }
      }

      // Validate bytecode using Five SDK
      if (context.options.verbose) {
        logger.info('Validating bytecode...');
      }
      if (spinner) {
        spinner.start('Validating bytecode...');
      }
      const validation = await FiveSDK.validateBytecode(loadedFile.bytecode, {
        debug: options.debug || false
      });
      if (context.options.debug) {
        logger.debug(`validation: ${JSON.stringify(validation)}`);
      }

      if (!validation.valid) {
        if (spinner) {
          spinner.fail('Bytecode validation failed');
        }
        logger.error(`Validation failed: ${validation.errors?.join(', ')}`);
        throw new Error(`Invalid bytecode: ${validation.errors?.join(', ')}`);
      }
      if (spinner) {
        spinner.succeed('Bytecode validation passed');
      }
      if (context.options.verbose) {
        logger.info('Bytecode validation passed');
      }

      // Validate bytecode size
      if (context.options.debug) {
        logger.debug(`bytecode size: ${loadedFile.bytecode.length} bytes (max: ${options.maxDataSize})`);
      }
      if (loadedFile.bytecode.length > options.maxDataSize) {
        throw new Error(`Bytecode too large: ${loadedFile.bytecode.length} bytes (max: ${options.maxDataSize})`);
      }

      // Setup deployment options
      if (context.options.verbose) {
        logger.info('Preparing deployment options...');
      }
      const deploymentOptions = await setupDeploymentOptions(loadedFile.bytecode, options, config, logger);
      if (context.options.debug) {
        logger.debug(`deployment options: ${JSON.stringify(deploymentOptions, null, 2)}`);
      }

      // Fetch current fees from VM state to determine if extra accounts are needed
      let fees;
      try {
        const rpcUrl = config.networks[config.target].rpcUrl;
        const connection = new Connection(rpcUrl, 'confirmed');
        fees = await FiveSDK.getFees(connection, options.programId);

        if (fees.deployFeeBps > 0 || fees.executeFeeBps > 0) {
          if (context.options.verbose || options.debug) {
            console.log('\n' + section('VM Fees'));
            if (fees.deployFeeBps > 0) {
              console.log(keyValue('Deployment Fee', `${(fees.deployFeeBps / 100).toFixed(2)}%`));
            }
            if (fees.executeFeeBps > 0) {
              console.log(keyValue('Execution Fee', `${(fees.executeFeeBps / 100).toFixed(2)}%`));
            }
            if (fees.adminAccount) {
              console.log(keyValue('Admin Account', fees.adminAccount));
            }
          }

          // Attach admin account to options for use in deployment
          options.adminAccount = fees.adminAccount;
        }
      } catch (e) {
        if (context.options.debug) {
          logger.debug(`Could not fetch VM fees: ${e instanceof Error ? e.message : String(e)}`);
        }
      }

      // Execute deployment
      let result: DeploymentResult;
      if (options.dryRun) {
        if (context.options.verbose) {
          logger.info('Simulating deployment...');
        }
        result = await simulateDeployment(deploymentOptions, options, context);
      } else {
        if (context.options.verbose) {
          logger.info('Deploying to network...');
        }
        if (context.options.debug) {
          logger.debug(`deployment options: ${JSON.stringify(deploymentOptions)}`);
          logger.debug(`config: ${JSON.stringify(config)}`);
        }

        // Ensure admin account is passed into deployment options
        const execDeploymentOptions = {
          ...deploymentOptions,
          adminAccount: options.adminAccount
        };

        result = await executeDeployment(execDeploymentOptions, options, context, config);
      }

      // Display results
      if (context.options.verbose) {
        logger.info('Deployment completed');
      }
      displayDeploymentResult(result, options, logger);

      if (!result.success) {
        logger.error('Deployment failed');
        process.exit(1);
      }

      if (context.options.verbose) {
        logger.info('Deploy command completed successfully');
      }

    } catch (error) {
      logger.error('Deploy failed:', error);
      throw error;
    }
  }
};

/**
 * Setup deployment options from CLI arguments
 */
async function setupDeploymentOptions(
  bytecode: Uint8Array,
  options: any,
  config: any,
  logger: any
): Promise<DeploymentOptions> {
  const deploymentOptions: DeploymentOptions = {
    bytecode,
    network: config.target,
    maxDataSize: parseInt(options.maxDataSize),
    computeBudget: parseInt(options.computeBudget),
    fiveVMProgramId: options.programId, // Pass the programId
    vmStateAccount: options.vmStateAccount,
  };


  return deploymentOptions;
}

/**
 * Execute actual deployment to Solana network
 */
async function executeDeployment(
  deploymentOptions: DeploymentOptions,
  options: any,
  context: CommandContext,
  config: any
): Promise<DeploymentResult> {
  const { logger } = context;

  if (context.options.debug) {
    logger.debug(`deployment options: ${JSON.stringify(deploymentOptions)}`);
    logger.debug(`options: ${JSON.stringify(options)}`);
    logger.debug(`config: ${JSON.stringify(config)}`);
  }

  const targetPrefix = ConfigManager.getTargetPrefix(config.target);
  if (context.options.verbose) {
    logger.info(`${targetPrefix} Deploying to ${deploymentOptions.network}...`);
  }

  try {
    // Get network RPC endpoint from config
    const rpcUrl = config.networks[config.target].rpcUrl;
    if (context.options.debug) {
      logger.debug(`rpc url: ${rpcUrl}`);
    }

    const connection = new Connection(rpcUrl, 'confirmed');

    // Load deployer keypair from config
    const deployerKeypair = await loadKeypair(config.keypairPath, logger);
    if (context.options.debug) {
      logger.debug(`deployer: ${deployerKeypair.publicKey.toString()}`);
    }

    // Deploy using Five SDK
    const spinner = isTTY() ? ora('Deploying via Five SDK...').start() : null;

    // Auto-detect large programs and use appropriate deployment method
    const bytecodeArray = new Uint8Array(deploymentOptions.bytecode);
    const isLargeProgram = bytecodeArray.length > 800 || options.forceChunked;

    if (isLargeProgram) {
      if (options.optimized) {
        if (spinner) {
          spinner.text = 'Deploying large program via optimized chunked deployment...';
        }
      } else {
        if (spinner) {
          spinner.text = 'Deploying large program via chunked deployment...';
        }
      }
    }

    let result;
    if (isLargeProgram) {
      if (options.optimized) {
        // Use the new optimized deployment method
        result = await FiveSDK.deployLargeProgramOptimizedToSolana(
          bytecodeArray,
          connection,
          deployerKeypair,
          {
            debug: options.debug || false,
            network: deploymentOptions.network,
            fiveVMProgramId: deploymentOptions.fiveVMProgramId,
            // vmStateAccount: deploymentOptions.vmStateAccount, // Optimized method does not support this yet
            maxRetries: 3,
            chunkSize: options.chunkSize || 950, // Higher default for optimized method
            progressCallback: options.progress ? (transaction: number, total: number) => {
              if (spinner) {
                spinner.text = `Optimized deployment: transaction ${transaction}/${total}...`;
              }
            } : undefined
          }
        );
      } else {
        // Use traditional large program deployment
        result = await FiveSDK.deployLargeProgramToSolana(
          bytecodeArray,
          connection,
          deployerKeypair,
          {
            debug: options.debug || false,
            network: deploymentOptions.network,
            fiveVMProgramId: deploymentOptions.fiveVMProgramId,
            vmStateAccount: deploymentOptions.vmStateAccount,
            maxRetries: 3,
            chunkSize: options.chunkSize || 750,
            progressCallback: options.progress ? (chunk: number, total: number) => {
              if (spinner) {
                spinner.text = `Deploying chunk ${chunk}/${total}...`;
              }
            } : undefined
          }
        );
      }
    } else {
      // Use regular deployment for small programs
      result = await FiveSDK.deployToSolana(
        bytecodeArray,
        connection,
        deployerKeypair,
        {
          debug: options.debug || false,
          network: deploymentOptions.network,
          computeBudget: deploymentOptions.computeBudget,
          fiveVMProgramId: deploymentOptions.fiveVMProgramId, // Pass programId
          maxRetries: 3
        }
      );
    }

    if (result.success) {
      if (isLargeProgram && 'chunksUsed' in result && result.chunksUsed) {
        const largeResult = result as any; // Type assertion for large deployment result
        if (options.optimized && 'optimizationSavings' in result) {
          const optimizedResult = result as any;
          const savingsPercent = Math.round(optimizedResult.optimizationSavings.transactionsSaved / (optimizedResult.optimizationSavings.transactionsSaved + optimizedResult.totalTransactions) * 100);
          if (spinner) {
            spinner.succeed(`Optimized deployment completed (${largeResult.chunksUsed} chunks, ${largeResult.totalTransactions} transactions, ${savingsPercent}% reduction)`);
          }
        } else {
          if (spinner) {
            spinner.succeed(`Large program deployment completed (${largeResult.chunksUsed} chunks, ${largeResult.totalTransactions} transactions)`);
          }
        }
      } else {
        if (spinner) {
          spinner.succeed('Deployment completed');
        }
      }
    } else {
      if (spinner) {
        spinner.fail('Deployment failed');
      }
    }

    if (context.options.debug) {
      logger.debug(`deploy result: ${JSON.stringify(result, null, 2)}`);
    }
    return result;

  } catch (error) {
    logger.error('Deployment failed:', error);

    const errorResult = {
      success: false,
      error: error instanceof Error ? error.message : 'Unknown deployment error',
      logs: []
    };

    if (context.options.debug) {
      logger.debug(`deploy error result: ${JSON.stringify(errorResult, null, 2)}`);
    }
    return errorResult;
  }
}




/**
 * Simulate deployment without executing
 */
async function simulateDeployment(
  deploymentOptions: DeploymentOptions,
  options: any,
  context: CommandContext
): Promise<DeploymentResult> {
  const { logger } = context;

  if (context.options.verbose) {
    logger.info('Simulating deployment...');
  }

  // Simulate validation and cost calculation
  await new Promise(resolve => setTimeout(resolve, 1000));

  const estimatedCost = Math.ceil(deploymentOptions.bytecode.length / 1000) * 1000000; // Rough estimate

  return {
    success: true,
    programId: 'SIMULATED_PROGRAM_ID_' + Date.now(),
    transactionId: 'SIMULATED_TX_' + Date.now(),
    deploymentCost: estimatedCost,
    logs: [
      'Deployment simulation completed',
      `Estimated cost: ${estimatedCost / 1e9} SOL`,
      `Bytecode size: ${deploymentOptions.bytecode.length} bytes`,
      `Target network: ${deploymentOptions.network}`
    ]
  };
}

/**
 * Display deployment result in specified format
 */
function displayDeploymentResult(result: DeploymentResult, options: any, logger: any): void {
  if (options.format === 'json') {
    console.log(JSON.stringify(result, null, 2));
    return;
  }

  console.log('\n' + section('Deployment'));

  if (result.success) {
    console.log(uiSuccess('Deployment succeeded'));

    if (result.programId) {
      console.log(keyValue('Program', result.programId));
    }

    if (result.transactionId) {
      console.log(keyValue('Transaction', result.transactionId));
    }

    if (result.deploymentCost !== undefined) {
      const costSOL = result.deploymentCost / 1e9;
      console.log(keyValue('Cost', `${costSOL.toFixed(6)} SOL`));
    }

    if (result.logs && result.logs.length > 0 && options.verbose) {
      console.log(section('Logs'));
      result.logs.forEach((log: string) => {
        console.log(`  ${log}`);
      });
    }
  } else {
    console.log(uiError(result.error || 'Deployment failed'));
  }
}

/**
 * Get RPC URL for network
 */
function getNetworkRpcUrl(network: string): string {
  const endpoints: Record<string, string> = {
    'devnet': 'https://api.devnet.solana.com',
    'testnet': 'https://api.testnet.solana.com',
    'mainnet': 'https://api.mainnet-beta.solana.com',
    'local': 'http://127.0.0.1:8899'
  };

  return endpoints[network] || endpoints['devnet'];
}

/**
 * Load Solana keypair from file
 */
async function loadKeypair(keypairPath: string, logger: any): Promise<Keypair> {
  // Expand tilde in path
  const path = keypairPath.startsWith('~/')
    ? keypairPath.replace('~', process.env.HOME || '')
    : keypairPath;

  try {
    const keypairData = await readFile(path, 'utf8');
    const secretKey = Uint8Array.from(JSON.parse(keypairData));
    const keypair = Keypair.fromSecretKey(secretKey);

    logger.debug(`Loaded keypair from: ${path}`);
    logger.debug(`Public key: ${keypair.publicKey.toString()}`);

    return keypair;
  } catch (error) {
    throw new Error(`Failed to load keypair from ${path}: ${error}`);
  }
}
