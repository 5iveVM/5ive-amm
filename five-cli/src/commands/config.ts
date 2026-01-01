/**
 * Five CLI Config Command
 * 
 * Manage Five CLI configuration settings for networks, keypairs, and deployment options.
 */

import chalk from 'chalk';
import ora from 'ora';
import { readFile } from 'fs/promises';
import { resolve } from 'path';
import { homedir } from 'os';
import * as readline from 'readline';

import { CommandDefinition, CommandContext } from '../types.js';
import { ConfigManager } from '../config/ConfigManager.js';
import { ConfigTarget, FiveConfig } from '../config/types.js';
import { success as uiSuccess, warning as uiWarning, uiColors, section } from '../utils/cli-ui.js';

export const configCommand: CommandDefinition = {
  name: 'config',
  description: 'Manage Five CLI configuration',
  aliases: ['cfg'],

  options: [],

  arguments: [
    {
      name: 'action',
      description: 'Configuration action (init, get, set, reset)',
      required: false
    },
    {
      name: 'key',
      description: 'Configuration key for get operations',
      required: false
    }
  ],

  examples: [
    {
      command: 'five config init',
      description: 'Initialize configuration with interactive setup'
    },
    {
      command: 'five config get',
      description: 'Show all configuration values'
    },
    {
      command: 'five config get target',
      description: 'Show current target network'
    },
    {
      command: 'five config set --target devnet',
      description: 'Set target network to devnet'
    },
    {
      command: 'five config set --keypair ~/.solana/deployer.json',
      description: 'Set keypair file path'
    },
    {
      command: 'five config set --rpc-url https://api.custom.solana.com',
      description: 'Set custom RPC URL for current target'
    },
    {
      command: 'five config reset',
      description: 'Reset configuration to defaults'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;
    const configManager = ConfigManager.getInstance();

    const action = args[0] || 'get';

    try {
      switch (action) {
        case 'init':
          await handleInit(configManager, options, logger);
          break;

        case 'get':
          await handleGet(configManager, args[1], options, logger);
          break;

        case 'set':
          await handleSet(configManager, options, logger);
          break;

        case 'reset':
          await handleReset(configManager, options, logger);
          break;

        default:
          logger.error(`Unknown config action: ${action}`);
          logger.info('Available actions: init, get, set, reset');
          process.exit(1);
      }
    } catch (error) {
      logger.error('Configuration error:', error);
      throw error;
    }
  }
};

/**
 * Handle config init command with interactive setup
 */
async function handleInit(
  configManager: ConfigManager,
  options: any,
  logger: any
): Promise<void> {
  try {
    console.log('\n' + section('Configuration Setup'));

    const rl = readline.createInterface({
      input: process.stdin,
      output: process.stdout,
    });

    const question = (query: string): Promise<string> => {
      return new Promise((resolve) => rl.question(query, resolve));
    };

    // Interactive setup
    console.log('Let\'s configure your Five CLI environment:\n');

    // Target network selection
    const targetAnswer = await question(
      `Select target network (local/devnet/testnet/mainnet) [devnet]: `
    );
    const target = targetAnswer.trim() || 'devnet';

    if (!['local', 'devnet', 'testnet', 'mainnet'].includes(target)) {
      throw new Error(`Invalid target: ${target}`);
    }

    // Keypair file path
    const keypairAnswer = await question(
      `Keypair file path (optional) [~/.config/solana/id.json]: `
    );
    const keypairPath = keypairAnswer.trim() || '~/.config/solana/id.json';


    // Show config preference
    const showConfigAnswer = await question(
      `Show config details in command output? (y/n) [n]: `
    );
    const showConfig = showConfigAnswer.toLowerCase().startsWith('y');

    rl.close();

    // Initialize with default configuration first
    await configManager.init();

    // Apply user settings
    await configManager.setTarget(target as ConfigTarget);

    if (keypairPath && keypairPath !== '~/.config/solana/id.json') {
      await configManager.setKeypair(keypairPath);
    }


    await configManager.setShowConfig(showConfig);

    const config = await configManager.get();
    console.log('\n' + formatConfig(config));
    console.log(uiSuccess('Configuration initialized'));

  } catch (error) {
    logger.error('Failed to initialize configuration:', error);
    throw error;
  }
}

/**
 * Handle config get command
 */
async function handleGet(
  configManager: ConfigManager,
  key: string | undefined,
  options: any,
  logger: any
): Promise<void> {
  try {
    const config = await configManager.get();

    if (options.format === 'json') {
      if (key) {
        const value = getConfigValue(config, key);
        if (value === undefined) {
          console.log(uiWarning(`Configuration key '${key}' not found`));
          return;
        }
        console.log(JSON.stringify({ [key]: value }, null, 2));
      } else {
        console.log(JSON.stringify(config, null, 2));
      }
      return;
    }

    if (key) {
      const value = getConfigValue(config, key);
      if (value === undefined) {
        console.log(uiWarning(`Configuration key '${key}' not found`));
        console.log('Available keys: target, networks, keypair, showConfig');
        return;
      }
      console.log(`${uiColors.info(key)}: ${formatValue(value)}`);
    } else {
      // Display all configuration
      console.log(formatConfig(config));
    }

  } catch (error) {
    logger.error('Failed to get configuration:', error);
    throw error;
  }
}

/**
 * Handle config set command
 */
async function handleSet(
  configManager: ConfigManager,
  options: any,
  logger: any
): Promise<void> {
  let hasChanges = false;
  const changes: string[] = [];

  try {
    // Parse command line options and apply changes
    if (options.target) {
      if (!['local', 'devnet', 'testnet', 'mainnet'].includes(options.target)) {
        throw new Error(`Invalid target: ${options.target}. Must be one of: devnet, testnet, mainnet, local`);
      }
      await configManager.setTarget(options.target as ConfigTarget);
      changes.push(`${uiColors.info('Target:')} ${options.target}`);
      hasChanges = true;
    }

    if (options.keypair) {
      // Validate keypair file exists and is readable
      try {
        const expandedPath = expandPath(options.keypair);
        await readFile(expandedPath);
        await configManager.setKeypair(options.keypair);
        changes.push(`${uiColors.info('Keypair:')} ${options.keypair}`);
        hasChanges = true;
      } catch (error) {
        throw new Error(`Keypair file not found or unreadable: ${options.keypair}`);
      }
    }


    if (options.rpcUrl) {
      const config = await configManager.get();
      const currentTarget = config.target;
      await configManager.set({
        networks: {
          ...config.networks,
          [currentTarget]: {
            ...config.networks[currentTarget],
            rpcUrl: options.rpcUrl
          }
        }
      });
      changes.push(`${uiColors.info('RPC URL')} (${currentTarget}): ${options.rpcUrl}`);
      hasChanges = true;
    }

    if (options.showConfig !== undefined) {
      await configManager.setShowConfig(Boolean(options.showConfig));
      changes.push(`${uiColors.info('Show Config:')} ${Boolean(options.showConfig)}`);
      hasChanges = true;
    }

    if (!hasChanges) {
      logger.error('No configuration changes specified');
      logger.info('Available options: --target, --keypair, --rpc-url, --show-config');
      process.exit(1);
    }

    const spinner = ora('Updating configuration...').start();
    spinner.succeed('Configuration updated successfully');

    // Show what was changed
    console.log('\n' + chalk.bold('Updated configuration:'));
    changes.forEach(change => console.log(`  ${change}`));

    // Optionally show full config
    if (options.showConfig || options.verbose) {
      const config = await configManager.get();
      console.log('\n' + formatConfig(config));
    }

  } catch (error) {
    logger.error('Failed to update configuration:', error);
    throw error;
  }
}

/**
 * Handle config reset command
 */
async function handleReset(
  configManager: ConfigManager,
  options: any,
  logger: any
): Promise<void> {
  try {
    console.log('\n' + uiWarning('This will reset all configuration to defaults.'));

    if (!options.yes) {
      const rl = readline.createInterface({
        input: process.stdin,
        output: process.stdout,
      });

      const question = (query: string): Promise<string> => {
        return new Promise((resolve) => rl.question(query, resolve));
      };

      const confirmation = await question('Are you sure? (y/N): ');
      rl.close();

      if (!confirmation.toLowerCase().startsWith('y')) {
        console.log('Reset cancelled');
        return;
      }
    }

    const spinner = ora('Resetting configuration...').start();

    await configManager.reset();

    spinner.succeed('Configuration reset to defaults');

    const config = await configManager.get();
    console.log('\n' + formatConfig(config));

  } catch (error) {
    logger.error('Failed to reset configuration:', error);
    throw error;
  }
}

/**
 * Format configuration for display
 */
function formatConfig(config: FiveConfig): string {
  const lines: string[] = [];
  lines.push(chalk.bold('Five CLI Configuration:'));
  lines.push('');

  // Current target
  lines.push(`${uiColors.info('Target:')} ${config.target}`);

  // Current network endpoint
  const currentNetwork = config.networks[config.target];
  lines.push(`${uiColors.info('RPC URL:')} ${currentNetwork.rpcUrl}`);
  if (currentNetwork.wsUrl) {
    lines.push(`${uiColors.info('WebSocket URL:')} ${currentNetwork.wsUrl}`);
  }

  // Keypair
  if (config.keypair) {
    lines.push(`${uiColors.info('Keypair:')} ${config.keypair}`);
  } else {
    lines.push(`${uiColors.info('Keypair:')} ${uiColors.muted('(not set)')}`);
  }


  // Show config preference
  lines.push(`${uiColors.info('Show Config:')} ${config.showConfig}`);

  // All networks
  lines.push('');
  lines.push(chalk.bold('Available Networks:'));
  for (const [target, network] of Object.entries(config.networks)) {
    const isActive = target === config.target;
    const prefix = isActive ? '●' : '○';
    const color = isActive ? uiColors.success : uiColors.muted;
    lines.push(`  ${color(prefix)} ${target}: ${network.rpcUrl}`);
  }

  return lines.join('\n');
}

/**
 * Get a nested configuration value by key
 */
function getConfigValue(config: FiveConfig, key: string): any {
  const keys = key.split('.');
  let value: any = config;

  for (const k of keys) {
    if (value && typeof value === 'object' && k in value) {
      value = value[k];
    } else {
      return undefined;
    }
  }

  return value;
}

/**
 * Expand path with home directory substitution
 */
function expandPath(filePath: string): string {
  if (filePath.startsWith('~/')) {
    return resolve(homedir(), filePath.slice(2));
  }
  return resolve(filePath);
}

/**
 * Format configuration value for display
 */
function formatValue(value: any): string {
  if (typeof value === 'object' && value !== null) {
    return JSON.stringify(value, null, 2);
  }
  return String(value);
}

// Add config command options for the CLI parser
configCommand.options = [
  {
    flags: '--target <target>',
    description: 'Set target network (local, devnet, testnet, mainnet)',
    required: false
  },
  {
    flags: '--keypair <file>',
    description: 'Set keypair file path',
    required: false
  },
  {
    flags: '--rpc-url <url>',
    description: 'Set custom RPC URL for current target network',
    required: false
  },
  {
    flags: '--show-config [value]',
    description: 'Toggle config display in command output (true/false)',
    required: false
  },
  {
    flags: '--format <format>',
    description: 'Output format (json, text)',
    defaultValue: 'text'
  },
  {
    flags: '--yes',
    description: 'Skip confirmation prompts',
    defaultValue: false
  },
  {
    flags: '--verbose',
    description: 'Show detailed output',
    defaultValue: false
  }
];
