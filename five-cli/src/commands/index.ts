// Commands index.

import { CommandDefinition } from '../types.js';
import { section, uiColors } from '../utils/cli-ui.js';

import { compileCommand } from './compile.js';
import { executeCommand } from './execute.js';
import { initCommand } from './init.js';
import { deployCommand } from './deploy.js';
import { testCommand } from './test.js';
import { versionCommand } from './version.js';
import { configCommand } from './config.js';
import { helpCommand } from './help.js';
import { buildCommand } from './build.js';
import { namespaceCommand } from './namespace.js';

/**
 * Registry of all available commands
 */
export const commands: CommandDefinition[] = [
  helpCommand, // Put help first for priority
  initCommand,
  configCommand,
  compileCommand,
  buildCommand,
  executeCommand,
  deployCommand,
  namespaceCommand,
  testCommand,
  versionCommand
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
    'Development': [initCommand, compileCommand, buildCommand, executeCommand, testCommand],
    'Deployment': [deployCommand, namespaceCommand],
    'Configuration': [configCommand],
    'Utility': [versionCommand, helpCommand],
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
  initCommand,
  configCommand,
  compileCommand,
  buildCommand,
  executeCommand,
  deployCommand,
  namespaceCommand,
  testCommand,
  versionCommand
};
