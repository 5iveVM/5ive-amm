#!/usr/bin/env node
// Five CLI entrypoint.

export async function main() {
  // Keep CLI output clean unless explicitly requested.
  if (!process.env.FIVE_SHOW_DEPRECATIONS) {
    process.noDeprecation = true;
  }

  try {
    const { createCLI } = await import('./cli.js');
    const cli = createCLI({
      verbose: process.argv.includes('--verbose') || process.argv.includes('-v'),
      debug: process.argv.includes('--debug')
    });
    
    await cli.run(process.argv);
  } catch (error) {
    const chalk = (await import('chalk')).default;
    console.error(chalk.red('Fatal error:'), error);
    process.exit(1);
  }
}

// Execute main when run directly (npx/global installs included).
const isMainModule = (
  import.meta.url === `file://${process.argv[1]}` ||
  process.argv[1].endsWith('dist/index.js') ||
  process.argv[1].endsWith('/five') ||
  process.argv[1].endsWith('/5ive') ||
  process.argv[1].includes('five-cli')
);

if (isMainModule) {
  main().catch(error => {
    console.error('Fatal CLI error:', error);
    process.exit(1);
  });
}
