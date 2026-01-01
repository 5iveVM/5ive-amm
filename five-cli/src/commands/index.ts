/**
 * Five CLI Commands Index
 * 
 * Central registry for all Five CLI commands with automatic discovery
 * and registration capabilities.
 */

import { CommandDefinition } from '../types.js';
import { section, uiColors } from '../utils/cli-ui.js';

// Import all command definitions
import { compileCommand } from './compile.js';
import { executeCommand } from './execute.js';
import { initCommand } from './init.js';
import { analyzeCommand } from './analyze.js';
import { optimizeCommand } from './optimize.js';
import { fmtCommand } from './fmt.js';
import { deployCommand } from './deploy.js';
import { deployAndExecuteCommand } from './deploy-and-execute.js';
// onchainCommand removed - replaced by config-driven deploy/execute
import { testCommand } from './test.js';
import { versionCommand } from './version.js';
import { configCommand } from './config.js';
import { localCommand } from './local.js';
import { helpCommand } from './help.js';
import { templateCommand } from './template.js';
import { donateCommand } from './donate.js';
import { buildCommand } from './build.js';

/**
 * Registry of all available commands
 */
export const commands: CommandDefinition[] = [
  helpCommand, // Put help first for priority
  templateCommand,
  donateCommand,
  compileCommand,
  buildCommand,
  executeCommand,
  deployCommand,
  deployAndExecuteCommand,
  localCommand,
  configCommand,
  initCommand,
  testCommand,
  analyzeCommand,
  optimizeCommand,
  fmtCommand,
  versionCommand
  // onchainCommand removed - replaced by config-driven deploy/execute
];

/**
 * Get command by name or alias
 */
export function getCommand(name: string): CommandDefinition | undefined {
  return commands.find(cmd => 
    cmd.name === name || (cmd.aliases && cmd.aliases.includes(name))
  );
}

/**
 * Get all command names and aliases
 */
export function getAllCommandNames(): string[] {
  const names: string[] = [];
  
  for (const cmd of commands) {
    names.push(cmd.name);
    if (cmd.aliases) {
      names.push(...cmd.aliases);
    }
  }
  
  return names;
}

/**
 * Get commands by category
 */
export function getCommandsByCategory(): Record<string, CommandDefinition[]> {
  return {
    'Development': [compileCommand, buildCommand, executeCommand, localCommand, testCommand, templateCommand, initCommand],
    'Deployment': [deployCommand, deployAndExecuteCommand],
    'Support': [donateCommand],
    'Configuration': [configCommand],
    'Utility': [versionCommand, helpCommand],
    // 'Legacy': [onchainCommand] - removed
  };
}

/**
 * Generate help text for all commands with retro styling
 */
export function generateCommandsHelp(): string {
  const categories = getCommandsByCategory();
  const helpSections: string[] = [];
  
  for (const [category, cmds] of Object.entries(categories)) {
    helpSections.push(`\n${section(category)}`);
    
    for (const cmd of cmds) {
      const aliases = cmd.aliases ? uiColors.muted(` (${cmd.aliases.join(', ')})`) : '';
      const cmdName = uiColors.accent(cmd.name);
      const desc = uiColors.text(cmd.description);
      helpSections.push(`  ${cmdName}${aliases.padEnd(20)} ${desc}`);
    }
  }
  
  return helpSections.join('\n');
}

/**
 * Validate command definition
 */
export function validateCommand(cmd: CommandDefinition): boolean {
  if (!cmd.name || typeof cmd.name !== 'string') {
    return false;
  }
  
  if (!cmd.description || typeof cmd.description !== 'string') {
    return false;
  }
  
  if (!cmd.handler || typeof cmd.handler !== 'function') {
    return false;
  }
  
  return true;
}

/**
 * Register a new command dynamically
 */
export function registerCommand(cmd: CommandDefinition): boolean {
  if (!validateCommand(cmd)) {
    return false;
  }
  
  // Check for name conflicts
  if (getCommand(cmd.name)) {
    return false;
  }
  
  commands.push(cmd);
  return true;
}

// Export individual commands for direct access
export {
  helpCommand,
  compileCommand,
  buildCommand,
  executeCommand,
  deployCommand,
  deployAndExecuteCommand,
  localCommand,
  configCommand,
  initCommand,
  analyzeCommand,
  optimizeCommand,
  fmtCommand,
  testCommand,
  versionCommand
  // onchainCommand removed
};
