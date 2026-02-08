#!/usr/bin/env node
// Five CLI entrypoint.

import chalk from 'chalk';
import { createCLI } from './cli.js';

export async function main() {
  try {
    const cli = createCLI({
      verbose: process.argv.includes('--verbose') || process.argv.includes('-v'),
      debug: process.argv.includes('--debug')
    });
    
    await cli.run(process.argv);
  } catch (error) {
    console.error(chalk.red('Fatal error:'), error);
    process.exit(1);
  }
}

// Execute main when run directly (npx/global installs included).
const isMainModule = (
  import.meta.url === `file://${process.argv[1]}` ||
  process.argv[1].endsWith('dist/index.js') ||
  process.argv[1].endsWith('/five') ||
  process.argv[1].includes('five-cli')
);

if (isMainModule) {
  main().catch(error => {
    console.error('Fatal CLI error:', error);
    process.exit(1);
  });
}
