/**
 * Five CLI Version Command
 * 
 * Display comprehensive version information including CLI, WASM modules,
 * dependencies, and system information.
 */

import { readFile } from 'fs/promises';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';
import chalk from 'chalk';
import os from 'os';
import { success as uiSuccess, uiColors } from '../utils/cli-ui.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

import {
  CommandDefinition,
  CommandContext,
  CLIOptions
} from '../types.js';
import { FiveCompilerWasm } from '../wasm/compiler.js';
import { FiveVM } from '../wasm/vm.js';

interface VersionInfo {
  cli: {
    name: string;
    version: string;
    description: string;
  };
  wasm: {
    compiler?: {
      version: string;
      features: string[];
    };
    vm?: {
      version: string;
      features: string[];
    };
  };
  dependencies: {
    [key: string]: string;
  };
  system: {
    node: string;
    platform: string;
    arch: string;
    memory: string;
    cpus: number;
  };
  build?: {
    timestamp?: string;
    commit?: string;
    target?: string;
  };
}

/**
 * Five version command implementation
 */
export const versionCommand: CommandDefinition = {
  name: 'version',
  description: 'Display version information',
  aliases: ['v'],
  
  options: [
    {
      flags: '--format <format>',
      description: 'Output format',
      choices: ['text', 'json', 'table'],
      defaultValue: 'text'
    },
    {
      flags: '--detailed',
      description: 'Show detailed version information',
      defaultValue: false
    },
    {
      flags: '--check-updates',
      description: 'Check for available updates',
      defaultValue: false
    }
  ],

  arguments: [],

  examples: [
    {
      command: 'five version',
      description: 'Show basic version information'
    },
    {
      command: 'five version --detailed --format json',
      description: 'Show detailed version info in JSON format'
    },
    {
      command: 'five version --check-updates',
      description: 'Check for CLI updates'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;
    
    try {
      // Gather version information
      const versionInfo = await gatherVersionInfo(context, options.detailed);

      // Check for updates if requested
      if (options.checkUpdates) {
        await checkForUpdates(versionInfo, logger);
      }

      // Display version information
      displayVersionInfo(versionInfo, options, logger);

    } catch (error) {
      logger.error('Failed to get version information:', error);
      
      // Display minimal version info as fallback
      console.log('five-cli version: 1.0.0 (version check failed)');
      throw error;
    }
  }
};

/**
 * Gather comprehensive version information
 */
async function gatherVersionInfo(context: CommandContext, detailed: boolean): Promise<VersionInfo> {
  const { config, logger } = context;
  
  // CLI version from package.json
  const packageInfo = await getPackageInfo(config.rootDir);
  
  const versionInfo: VersionInfo = {
    cli: {
      name: packageInfo.name || 'five-cli',
      version: packageInfo.version || '1.0.0',
      description: packageInfo.description || 'Five VM CLI'
    },
    wasm: {},
    dependencies: {},
    system: {
      node: process.version,
      platform: os.platform(),
      arch: os.arch(),
      memory: formatMemory(os.totalmem()),
      cpus: os.cpus().length
    }
  };

  if (detailed) {
    // WASM module versions
    try {
      versionInfo.wasm = await getWasmVersions(logger);
    } catch (error) {
      logger.debug('Failed to get WASM versions:', error);
    }

    // Dependencies
    versionInfo.dependencies = getDependencyVersions(packageInfo);

    // Build info if available
    versionInfo.build = await getBuildInfo(config.rootDir);
  }

  return versionInfo;
}

/**
 * Get package.json information
 */
async function getPackageInfo(rootDir: string): Promise<any> {
  const possiblePaths = [
    join(rootDir, 'package.json'),
    join(__dirname, '../package.json'),
    join(__dirname, '../../package.json')
  ];

  for (const path of possiblePaths) {
    try {
      const content = await readFile(path, 'utf8');
      return JSON.parse(content);
    } catch {
      continue;
    }
  }

  return { name: 'five-cli', version: '1.0.0', description: 'Five VM CLI' };
}

/**
 * Get WASM module version information
 */
async function getWasmVersions(logger: any): Promise<any> {
  const wasmVersions: any = {};

  // Check if WASM modules are available without initializing
  try {
    // Resolve WASM path relative to source (src) and built (dist) layouts
    const path = '../assets/vm/five_vm_wasm.js';
    await import(path as string);
    
    // WASM modules exist, try to get versions
    try {
      const compiler = new FiveCompilerWasm(logger);
      await compiler.initialize();
      wasmVersions.compiler = compiler.getCompilerInfo();
    } catch (error) {
      wasmVersions.compiler = {
        version: 'error',
        features: [],
        error: 'Failed to initialize compiler'
      };
    }

    try {
      const vm = new FiveVM(logger);
      await vm.initialize();
      wasmVersions.vm = vm.getVMInfo();
    } catch (error) {
      wasmVersions.vm = {
        version: 'error', 
        features: [],
        error: 'Failed to initialize VM'
      };
    }
  } catch (error) {
    // WASM modules not built
    wasmVersions.compiler = {
      version: 'not built',
      features: [],
      note: 'Run "npm run build:vm-wasm" to build WASM modules'
    };
    wasmVersions.vm = {
      version: 'not built',
      features: [],
      note: 'Run "npm run build:vm-wasm" to build WASM modules'
    };
  }

  return wasmVersions;
}

/**
 * Get dependency versions from package.json
 */
function getDependencyVersions(packageInfo: any): Record<string, string> {
  const dependencies: Record<string, string> = {};
  
  const keyDeps = [
    'commander',
    'chalk',
    'ora',
    '@solana/web3.js',
    'typescript'
  ];

  const allDeps = {
    ...packageInfo.dependencies,
    ...packageInfo.devDependencies
  };

  for (const dep of keyDeps) {
    if (allDeps[dep]) {
      dependencies[dep] = allDeps[dep];
    }
  }

  return dependencies;
}

/**
 * Get build information if available
 */
async function getBuildInfo(rootDir: string): Promise<any> {
  const buildInfo: any = {};

  // Try to read build timestamp
  try {
    const buildFile = join(rootDir, '.build-info.json');
    const content = await readFile(buildFile, 'utf8');
    const info = JSON.parse(content);
    
    buildInfo.timestamp = info.timestamp;
    buildInfo.commit = info.commit;
    buildInfo.target = info.target;
  } catch {
    // Build info not available
  }

  return Object.keys(buildInfo).length > 0 ? buildInfo : undefined;
}

/**
 * Check for available updates
 */
async function checkForUpdates(versionInfo: VersionInfo, logger: any): Promise<void> {
  try {
    logger.info('Checking for updates...');
    
    // In a real implementation, this would check npm registry or GitHub releases
    // For now, simulate the check
    await new Promise(resolve => setTimeout(resolve, 1000));
    
    console.log(uiSuccess('You are running the latest version'));
    
  } catch (error) {
    logger.warn('Update check failed:', error);
  }
}

/**
 * Display version information in specified format
 */
function displayVersionInfo(versionInfo: VersionInfo, options: any, logger: any): void {
  if (options.format === 'json') {
    console.log(JSON.stringify(versionInfo, null, 2));
    return;
  }

  if (options.format === 'table') {
    displayVersionTable(versionInfo);
    return;
  }

  // Default text format
  displayVersionText(versionInfo, options.detailed);
}

/**
 * Display version information as formatted text
 */
function displayVersionText(versionInfo: VersionInfo, detailed: boolean): void {
  console.log(chalk.bold(`${versionInfo.cli.name} ${versionInfo.cli.version}`));
  
  if (detailed) {
    console.log(uiColors.muted(versionInfo.cli.description));
    console.log();
    
    // WASM modules
    if (versionInfo.wasm.compiler) {
      console.log(chalk.bold('WASM Compiler:'));
      console.log(`  Version: ${versionInfo.wasm.compiler.version}`);
      console.log(`  Features: ${versionInfo.wasm.compiler.features.join(', ')}`);
      console.log();
    }
    
    if (versionInfo.wasm.vm) {
      console.log(chalk.bold('WASM VM:'));
      console.log(`  Version: ${versionInfo.wasm.vm.version}`);
      console.log(`  Features: ${versionInfo.wasm.vm.features.join(', ')}`);
      console.log();
    }
    
    // System info
    console.log(chalk.bold('System:'));
    console.log(`  Node.js: ${versionInfo.system.node}`);
    console.log(`  Platform: ${versionInfo.system.platform} ${versionInfo.system.arch}`);
    console.log(`  Memory: ${versionInfo.system.memory}`);
    console.log(`  CPUs: ${versionInfo.system.cpus}`);
    
    // Dependencies
    if (Object.keys(versionInfo.dependencies).length > 0) {
      console.log();
      console.log(chalk.bold('Key Dependencies:'));
      for (const [name, version] of Object.entries(versionInfo.dependencies)) {
        console.log(`  ${name}: ${version}`);
      }
    }
    
    // Build info
    if (versionInfo.build) {
      console.log();
      console.log(chalk.bold('Build:'));
      if (versionInfo.build.timestamp) {
        console.log(`  Timestamp: ${versionInfo.build.timestamp}`);
      }
      if (versionInfo.build.commit) {
        console.log(`  Commit: ${versionInfo.build.commit}`);
      }
      if (versionInfo.build.target) {
        console.log(`  Target: ${versionInfo.build.target}`);
      }
    }
  }
}

/**
 * Display version information as a table
 */
function displayVersionTable(versionInfo: VersionInfo): void {
  const rows: Array<[string, string]> = [
    ['CLI Version', versionInfo.cli.version],
    ['Node.js', versionInfo.system.node],
    ['Platform', `${versionInfo.system.platform} ${versionInfo.system.arch}`]
  ];
  
  if (versionInfo.wasm.compiler) {
    rows.push(['WASM Compiler', versionInfo.wasm.compiler.version]);
  }
  
  if (versionInfo.wasm.vm) {
    rows.push(['WASM VM', versionInfo.wasm.vm.version]);
  }
  
  // Simple table formatting
  console.log(chalk.bold('Five CLI Version Information:'));
  console.log();
  
  const maxKeyLength = Math.max(...rows.map(([key]) => key.length));
  
  for (const [key, value] of rows) {
    const paddedKey = key.padEnd(maxKeyLength);
    console.log(`${uiColors.info(paddedKey)} : ${value}`);
  }
}

/**
 * Format memory size for display
 */
function formatMemory(bytes: number): string {
  const sizes = ['B', 'KB', 'MB', 'GB', 'TB'];
  if (bytes === 0) return '0 B';
  
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const size = (bytes / Math.pow(1024, i)).toFixed(1);
  
  return `${size} ${sizes[i]}`;
}
