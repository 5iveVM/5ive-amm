// Build command.
import { CommandDefinition, CommandContext } from '../types.js';
import { loadProjectConfig } from '../project/ProjectLoader.js';
import { compileCommand } from './compile.js';

export const buildCommand: CommandDefinition = {
  name: 'build',
  description: 'Build a Five project from five.toml',
  aliases: ['b'],

  options: [
    {
      flags: '--project <path>',
      description: 'Project directory or five.toml path',
      required: false
    },
    {
      flags: '-t, --target <target>',
      description: 'Override project target (vm, solana, debug, test)',
      choices: ['vm', 'solana', 'debug', 'test'],
      required: false
    },
    {
      flags: '--debug',
      description: 'Enable debug output during build',
      defaultValue: false
    },
    {
      flags: '--no-metrics',
      description: 'Disable metrics collection during build',
      defaultValue: false
    }
  ],

  arguments: [],

  examples: [
    {
      command: 'five build',
      description: 'Build the project in the current directory (discovers five.toml)'
    },
    {
      command: 'five build --project ../my-app',
      description: 'Build a project at the given path'
    },
    {
      command: 'five build --target solana',
      description: 'Override target from five.toml during build'
    }
  ],

  handler: async (_args: any, options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;

    const projectContext = await loadProjectConfig(options.project, process.cwd());
    if (!projectContext) {
      throw new Error(
        'No five.toml found. Specify --project or run from a directory containing five.toml.'
      );
    }

    if (context.options.verbose) {
      logger.info(
        `Building project ${projectContext.config.name} (${projectContext.configPath})`
      );
    }

    // Delegate to compile command with project-aware options.
    const compileOptions = {
      ...options,
      project: projectContext.configPath,
      target: options.target ?? projectContext.config.target,
      // Respect --no-metrics but default to metrics enabled for builds
      includeMetrics: options.metrics ?? options.metrics !== false,
      comprehensiveMetrics: options.comprehensiveMetrics ?? false
    };

    await compileCommand.handler([], compileOptions, context);
  }
};
