import { readFile } from 'fs/promises';

import { CommandContext, CommandDefinition } from '../types.js';
import {
  createFiveArtifact,
  normalizeArtifactAbi,
  readBytecodeFile,
  writePackagedArtifact,
} from '../utils/artifacts.js';

export const artifactCommand: CommandDefinition = {
  name: 'artifact',
  description: 'Package and normalize 5IVE artifact files',

  options: [
    {
      flags: '--bytecode <file>',
      description: 'Bytecode input file',
      required: false,
    },
    {
      flags: '--abi <file>',
      description: 'ABI JSON input file',
      required: false,
    },
    {
      flags: '-o, --output <file>',
      description: 'Packaged .five output path',
      required: false,
    },
    {
      flags: '--encoding <encoding>',
      description: 'Bytecode file encoding',
      choices: ['binary', 'hex'],
      defaultValue: 'binary',
    },
    {
      flags: '--types',
      description: 'Generate a .d.ts file next to the packaged artifact',
      defaultValue: false,
    },
  ],

  arguments: [
    {
      name: 'action',
      description: 'Artifact action (currently: pack)',
      required: true,
    },
  ],

  examples: [
    {
      command: '5ive artifact pack --bytecode build/script.bin --abi build/script.abi.json -o build/main.five',
      description: 'Package raw bytecode and ABI into a .five artifact',
    },
    {
      command: '5ive artifact pack --bytecode build/script.hex --encoding hex --abi build/script.abi.json -o build/main.five --types',
      description: 'Package a hex-encoded bytecode file and emit TypeScript types',
    },
  ],

  handler: async (args: any, options: any, context: CommandContext): Promise<void> => {
    const [action] = Array.isArray(args) ? args : [args];
    if (action !== 'pack') {
      throw new Error(`Unsupported artifact action '${action}'. Supported actions: pack`);
    }

    if (!options.bytecode || !options.abi || !options.output) {
      throw new Error('artifact pack requires --bytecode, --abi, and --output');
    }

    const bytecode = await readBytecodeFile(options.bytecode, options.encoding || 'binary');
    const rawAbi = JSON.parse(await readFile(options.abi, 'utf8'));
    const normalizedAbi = normalizeArtifactAbi(rawAbi);
    const artifact = createFiveArtifact({
      bytecode,
      abi: normalizedAbi,
      metadata: {},
    });

    await writePackagedArtifact(options.output, artifact, {
      emitTypes: Boolean(options.types),
    });

    if (context.options.verbose) {
      context.logger.info(`Packaged ${bytecode.length} bytes into ${options.output}`);
    }
  },
};
