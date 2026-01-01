/**
 * Five CLI Help Command
 * 
 * Provides comprehensive help with retro 2000s styling and proper
 * command-specific help functionality.
 */

import chalk from 'chalk';
import {
  CommandDefinition,
  CommandContext,
  CLIOptions
} from '../types.js';
import {
  brandLine,
  section,
  commandExample,
  keyValue,
  uiColors
} from '../utils/cli-ui.js';
import { commands, getCommand, getCommandsByCategory } from './index.js';

/**
 * Five help command implementation
 */
export const helpCommand: CommandDefinition = {
  name: 'help',
  description: 'Display help information for commands',
  aliases: ['h', '-h', '--help'],
  
  options: [
    {
      flags: '--detailed',
      description: 'Show detailed help with examples',
      defaultValue: false
    },
    {
      flags: '--no-banner',
      description: 'Skip the ASCII art banner',
      defaultValue: false
    }
  ],

  arguments: [
    {
      name: 'command',
      description: 'Command to get help for',
      required: false
    }
  ],

  examples: [
    {
      command: 'five help',
      description: 'Show general help'
    },
    {
      command: 'five help compile',
      description: 'Show help for compile command'
    },
    {
      command: 'five help --detailed',
      description: 'Show detailed help with examples'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const commandName = args[0];
    
    if (commandName) {
      // Show help for specific command
      await showCommandHelp(commandName, options, context);
    } else {
      // Show general help
      await showGeneralHelp(options, context);
    }
  }
};

/**
 * Show help for a specific command
 */
async function showCommandHelp(commandName: string, options: any, context: CommandContext): Promise<void> {
  const command = getCommand(commandName);
  
  if (!command) {
    console.log(uiColors.error(`Unknown command: ${commandName}`));
    console.log('\nAvailable commands:');
    showCommandList();
    return;
  }

  console.log(section(`Help: ${command.name}`));
  console.log();
  
  // Description
  console.log(chalk.bold('Description:'));
  console.log(`  ${command.description}`);
  console.log();
  
  // Aliases
  if (command.aliases && command.aliases.length > 0) {
    console.log(chalk.bold('Aliases:'));
    console.log(`  ${command.aliases.map(alias => uiColors.info(alias)).join(', ')}`);
    console.log();
  }
  
  // Usage
  console.log(chalk.bold('Usage:'));
  const usage = buildUsageString(command);
  console.log(`  ${uiColors.info('five')} ${uiColors.accent(command.name)} ${usage}`);
  console.log();
  
  // Arguments
  if (command.arguments && command.arguments.length > 0) {
    console.log(chalk.bold('Arguments:'));
    for (const arg of command.arguments) {
      const argName = arg.required ? `<${arg.name}>` : `[${arg.name}]`;
      const variadic = arg.variadic ? '...' : '';
      console.log(`  ${uiColors.info(argName + variadic).padEnd(20)} ${arg.description}`);
    }
    console.log();
  }
  
  // Options
  if (command.options && command.options.length > 0) {
    console.log(chalk.bold('Options:'));
    for (const option of command.options) {
      const flags = uiColors.info(option.flags);
      const desc = option.description;
      const defaultVal = option.defaultValue !== undefined ? 
        uiColors.muted(` (default: ${option.defaultValue})`) : '';
      console.log(`  ${flags.padEnd(30)} ${desc}${defaultVal}`);
    }
    console.log();
  }
  
  // Examples
  if (command.examples && command.examples.length > 0) {
    console.log(chalk.bold('Examples:'));
    for (const example of command.examples) {
      console.log(commandExample(example.command, example.description));
    }
    console.log();
  }
}

/**
 * Show general help with ASCII banner and command overview
 */
async function showGeneralHelp(options: any, context: CommandContext): Promise<void> {
  // Show simple header unless disabled
  if (!options.noBanner) {
    console.log(brandLine());
    console.log(uiColors.muted('Five VM CLI - Ultra-fast bytecode VM for Solana'));
    console.log();
  }
  
  // Show current configuration
  showCurrentConfig(context);
  
  // Command categories with styling
  console.log(section('Available Commands'));
  console.log();
  
  const categories = getCommandsByCategory();

  for (const [category, commandList] of Object.entries(categories)) {
    console.log(chalk.bold(uiColors.accent(`${category}:`)));
    
    for (const cmd of commandList) {
      const aliases = cmd.aliases ? uiColors.muted(` (${cmd.aliases.join(', ')})`) : '';
      console.log(`  ${uiColors.accent(cmd.name)}${aliases.padEnd(20)} ${cmd.description}`);
    }
    console.log();
  }
  
  // Quick start examples
  if (options.detailed) {
    console.log(section('Quick Start'));
    console.log();
    
    const quickStartExamples = [
      'five init my-project                    Create a new Five project',
      'five compile script.v                   Compile Five source to bytecode', 
      'five execute script.five --local        Test execution locally',
      'five deploy script.five --target devnet Deploy to Solana devnet',
      'five config get                         View current configuration'
    ];
    
    for (const example of quickStartExamples) {
      const [cmd, desc] = example.split('  ');
      console.log(commandExample(cmd.trim(), desc?.trim() || ''));
    }
    console.log();
  }
  
  console.log(section('Need More Help'));
  console.log(keyValue('five help <command>', 'Command-specific help'));
  console.log(keyValue('five --verbose', 'Detailed output'));
  console.log(keyValue('docs', 'https://github.com/five-vm/five-cli#readme'));
}

/**
 * Show current configuration status
 */
function showCurrentConfig(context: CommandContext): void {
  // For now, show a simple status. In the future, integrate with ConfigManager
  const configInfo = [
    `${chalk.bold('Status:')} ${uiColors.success('Ready')}`,
    `${chalk.bold('Version:')} ${uiColors.info('1.2.2')}`,
    // `${chalk.bold('Network:')} ${getNetworkDisplay('local')}`, // TODO: Get from config
    `${chalk.bold('Debug:')} ${context.options.debug ? uiColors.warn('ON') : uiColors.muted('OFF')}`
  ];
  
  console.log(section('Current Status'));
  for (const line of configInfo) {
    console.log(`  ${line}`);
  }
  console.log();
}

/**
 * Build usage string from command definition
 */
function buildUsageString(command: CommandDefinition): string {
  const parts: string[] = [];
  
  // Add options
  if (command.options && command.options.length > 0) {
    parts.push('[options]');
  }
  
  // Add arguments
  if (command.arguments && command.arguments.length > 0) {
    for (const arg of command.arguments) {
      if (arg.required) {
        parts.push(arg.variadic ? `<${arg.name}...>` : `<${arg.name}>`);
      } else {
        parts.push(arg.variadic ? `[${arg.name}...]` : `[${arg.name}]`);
      }
    }
  }
  
  return parts.join(' ');
}

/**
 * Show simplified command list
 */
function showCommandList(): void {
  const commandNames = commands.map(cmd => cmd.name).sort();
  const columns = 3;
  const rows = Math.ceil(commandNames.length / columns);
  
  for (let row = 0; row < rows; row++) {
    const rowCommands: string[] = [];
    for (let col = 0; col < columns; col++) {
      const index = row + col * rows;
      if (index < commandNames.length) {
        rowCommands.push(uiColors.info(commandNames[index].padEnd(15)));
      }
    }
    console.log(`  ${rowCommands.join('')}`);
  }
}
