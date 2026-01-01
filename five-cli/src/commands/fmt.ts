/**
 * Five CLI Format Command
 *
 * Code formatting for Five VM source files with configurable style,
 * automatic fixing, and check mode.
 */

import { readFile, writeFile, readdir, stat } from 'fs/promises';
import { extname, basename, join } from 'path';
import ora from 'ora';
import { section, success as uiSuccess, error as uiError } from '../utils/cli-ui.js';

import {
  CommandDefinition,
  CommandContext
} from '../types.js';

/**
 * Five format command implementation
 */
export const fmtCommand: CommandDefinition = {
  name: 'fmt',
  description: 'Format Five VM source code',
  aliases: ['format'],

  options: [
    {
      flags: '--check',
      description: 'Check formatting without modifying files',
      defaultValue: false
    },
    {
      flags: '--style <style>',
      description: 'Code style preset',
      choices: ['default', 'compact', 'expanded', 'custom'],
      defaultValue: 'default'
    },
    {
      flags: '--indent <size>',
      description: 'Indentation size (spaces)',
      defaultValue: '4'
    },
    {
      flags: '--line-width <width>',
      description: 'Maximum line width',
      defaultValue: '100'
    },
    {
      flags: '--trailing-comma',
      description: 'Add trailing commas',
      defaultValue: false
    },
    {
      flags: '--verbose',
      description: 'Show formatting details',
      defaultValue: false
    },
    {
      flags: '--recursive',
      description: 'Format files recursively in directory',
      defaultValue: false
    }
  ],

  arguments: [
    {
      name: 'path',
      description: 'File or directory to format (.v files)',
      required: true
    }
  ],

  examples: [
    {
      command: 'five fmt src/main.v',
      description: 'Format a single file'
    },
    {
      command: 'five fmt src/ --recursive',
      description: 'Format all .v files in directory'
    },
    {
      command: 'five fmt src/main.v --check',
      description: 'Check formatting without modifying'
    },
    {
      command: 'five fmt src/ --style expanded --indent 2',
      description: 'Format with expanded style and 2-space indentation'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;

    try {
      const path = args[0];
      if (!path) {
        throw new Error('Path argument is required');
      }

      // Find files to format
      const spinner = ora('Finding files to format...').start();

      const files = await findFilesToFormat(path, options.recursive);

      if (files.length === 0) {
        spinner.warn('No .v files found to format');
        return;
      }

      spinner.succeed(`Found ${files.length} file(s) to format`);

      // Format configuration
      const formatConfig = {
        indentSize: parseInt(options.indent),
        lineWidth: parseInt(options.lineWidth),
        style: options.style,
        trailingComma: options.trailingComma,
        verbose: options.verbose
      };

      // Process files
      const results = {
        formatted: 0,
        unchanged: 0,
        errors: 0
      };

      for (let i = 0; i < files.length; i++) {
        const file = files[i];
        const spinnerMsg = `[${i + 1}/${files.length}] Formatting ${basename(file)}...`;
        spinner.start(spinnerMsg);

        try {
          const source = await readFile(file, 'utf8');
          const formatted = formatCode(source, formatConfig);

          if (source === formatted) {
            spinner.succeed(`[${i + 1}/${files.length}] ${basename(file)} (already formatted)`);
            results.unchanged++;
          } else {
            if (!options.check) {
              await writeFile(file, formatted);
              spinner.succeed(`[${i + 1}/${files.length}] ${basename(file)} (formatted)`);
            } else {
              spinner.warn(`[${i + 1}/${files.length}] ${basename(file)} (needs formatting)`);
            }
            results.formatted++;
          }
        } catch (error) {
          spinner.fail(`[${i + 1}/${files.length}] ${basename(file)} (error)`);
          if (options.verbose) {
            logger.error(`  Error: ${error instanceof Error ? error.message : 'Unknown error'}`);
          }
          results.errors++;
        }
      }

      // Display summary
      displayFormattingSummary(results, options, logger);
    } catch (error) {
      logger.error('Formatting failed:', error);
      throw error;
    }
  }
};

/**
 * Find .v files to format
 */
async function findFilesToFormat(path: string, recursive: boolean): Promise<string[]> {
  const files: string[] = [];

  try {
    const pathStats = await stat(path);

    if (pathStats.isFile()) {
      if (path.endsWith('.v')) {
        files.push(path);
      }
    } else if (pathStats.isDirectory()) {
      const entries = await readdir(path, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = join(path, entry.name);

        if (entry.isDirectory() && recursive && !entry.name.startsWith('.')) {
          const subFiles = await findFilesToFormat(fullPath, recursive);
          files.push(...subFiles);
        } else if (entry.isFile() && entry.name.endsWith('.v')) {
          files.push(fullPath);
        }
      }
    }
  } catch (error) {
    throw new Error(`Failed to access path: ${error instanceof Error ? error.message : 'Unknown error'}`);
  }

  return files.sort();
}

/**
 * Format source code
 */
function formatCode(source: string, config: any): string {
  let formatted = source;

  // Apply formatting transformations
  // 1. Normalize line endings
  formatted = formatted.replace(/\r\n/g, '\n');

  // 2. Remove trailing whitespace
  formatted = formatted
    .split('\n')
    .map((line) => line.trimRight())
    .join('\n');

  // 3. Fix indentation
  formatted = formatIndentation(formatted, config.indentSize);

  // 4. Format function declarations
  formatted = formatFunctions(formatted, config);

  // 5. Format blocks
  formatted = formatBlocks(formatted, config);

  // 6. Format function calls
  formatted = formatFunctionCalls(formatted, config.trailingComma);

  // 7. Format operators and spacing
  formatted = formatOperators(formatted);

  // 8. Add newlines at end if missing
  if (!formatted.endsWith('\n')) {
    formatted += '\n';
  }

  return formatted;
}

/**
 * Format indentation
 */
function formatIndentation(source: string, indentSize: number): string {
  const lines = source.split('\n');
  const indentStr = ' '.repeat(indentSize);
  let indentLevel = 0;
  const formatted: string[] = [];

  const bracketStack: string[] = [];

  for (let i = 0; i < lines.length; i++) {
    let line = lines[i].trim();

    if (!line) {
      formatted.push('');
      continue;
    }

    // Adjust indent for closing brackets
    if (line.startsWith('}') || line.startsWith(']') || line.startsWith(')')) {
      if (bracketStack.length > 0) {
        bracketStack.pop();
        indentLevel = Math.max(0, bracketStack.length);
      }
    }

    // Add indentation
    formatted.push(indentStr.repeat(indentLevel) + line);

    // Adjust indent for opening brackets
    const openCount = (line.match(/{|\[|\(/g) || []).length;
    const closeCount = (line.match(/}|\]|\)/g) || []).length;

    for (let j = 0; j < openCount; j++) {
      bracketStack.push('{');
    }

    indentLevel = bracketStack.length;
  }

  return formatted.join('\n');
}

/**
 * Format function declarations
 */
function formatFunctions(source: string, config: any): string {
  let formatted = source;

  // Ensure space before opening brace in function definitions
  formatted = formatted.replace(/pub\s+(\w+\s*\([^)]*\))\s*\{/g, 'pub $1 {');
  formatted = formatted.replace(/fn\s+(\w+\s*\([^)]*\))\s*\{/g, 'fn $1 {');

  // Format parameter lists
  formatted = formatted.replace(/,\s*(?=[^\s])/g, ', ');

  return formatted;
}

/**
 * Format blocks
 */
function formatBlocks(source: string, config: any): string {
  let formatted = source;

  // Ensure newlines after opening braces
  formatted = formatted.replace(/\{\s*([^\n])/g, '{\n$1');

  // Ensure newlines before closing braces (except for single-line)
  formatted = formatted.replace(/([^\n])\s*\}/g, '$1\n}');

  // Format if/else blocks
  formatted = formatted.replace(/\}\s*(else)/g, '} $1');

  return formatted;
}

/**
 * Format function calls
 */
function formatFunctionCalls(source: string, addTrailingComma: boolean): string {
  let formatted = source;

  // Remove spaces before opening parentheses in calls
  formatted = formatted.replace(/(\w+)\s+\(/g, '$1(');

  // Format argument spacing
  formatted = formatted.replace(/,\s*(?=[^\s])/g, ', ');

  if (addTrailingComma) {
    // Add trailing commas in multi-line function calls
    formatted = formatted.replace(/,(\s*[\n)])/g, ',$1');
  }

  return formatted;
}

/**
 * Format operators and spacing
 */
function formatOperators(source: string): string {
  let formatted = source;

  // Binary operators
  const binaryOps = ['=', '==', '!=', '<', '>', '<=', '>=', '+', '-', '*', '/', '%', '&&', '||'];

  for (const op of binaryOps) {
    const escaped = op.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
    const regex = new RegExp(`([^\\s${escaped}])\\s*${escaped}\\s*([^\\s=])`, 'g');
    formatted = formatted.replace(regex, `$1 ${op} $2`);
  }

  return formatted;
}

/**
 * Display formatting summary
 */
function displayFormattingSummary(results: any, options: any, logger: any): void {
  console.log('\n' + section('Formatting Summary'));

  const total = results.formatted + results.unchanged + results.errors;

  if (options.check) {
    console.log(`\nCheck Mode:`);
    console.log(`  Files to format: ${results.formatted}`);
    console.log(`  Already formatted: ${results.unchanged}`);

    if (results.errors > 0) {
      console.log(`  Errors: ${results.errors}`);
    }

    if (results.formatted > 0) {
      console.log('\nRun without --check flag to apply formatting');
      process.exit(1);
    } else {
      console.log(uiSuccess('All files are properly formatted'));
    }
  } else {
    console.log(`\nFormatted:`);
    console.log(`  Total processed: ${total}`);
    console.log(`  Formatted: ${results.formatted}`);
    console.log(`  Already formatted: ${results.unchanged}`);

    if (results.errors > 0) {
      console.log(`  Errors: ${results.errors}`);
    }

    if (results.formatted > 0) {
      console.log(uiSuccess('Formatting complete'));
    } else if (results.errors > 0) {
      console.log(uiError(`Formatting completed with ${results.errors} errors`));
    }
  }

  console.log('');
}
