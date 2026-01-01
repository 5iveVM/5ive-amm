import type { CommandDefinition } from '../../types.js';

export const versionCommand: CommandDefinition = {
  name: 'version',
  description: 'Display version information',
  aliases: ['v'],
  options: [],
  arguments: [],
  examples: [],
  handler: async () => {}
};
