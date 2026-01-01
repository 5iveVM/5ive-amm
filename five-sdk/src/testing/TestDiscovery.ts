/**
 * Five CLI Test Discovery
 *
 * Enhanced test discovery for Five VM scripts supporting both .v source files
 * and compiled .test.json test suites. Automatically detects test functions,
 * extracts parameters from @test-params comments, and compiles source files.
 */

import { readFile, readdir, stat } from 'fs/promises';
import { join, extname, basename, relative } from 'path';
import { FiveSDK } from '../FiveSDK.js';

/**
 * Discovered test source from .v file
 */
export interface VSourceTest {
  type: 'v-source';
  file: string;
  functionName: string;
  parameters?: any[];
  description?: string;
}

/**
 * Compiled test case
 */
export interface CompiledTestCase {
  type: 'compiled';
  file: string;
  bytecode: Uint8Array;
  functionIndex?: number;
  parameters?: any[];
  description?: string;
}

/**
 * Test discovery result
 */
export interface DiscoveredTest {
  name: string;
  path: string;
  type: 'v-source' | 'json-suite' | 'bin-bytecode';
  source?: VSourceTest;
  description?: string;
  parameters?: any[];
}

/**
 * Discover tests from directory
 */
export class TestDiscovery {
  /**
   * Discover all tests in a directory
   */
  static async discoverTests(
    testDir: string,
    options: {
      pattern?: string;
      verbose?: boolean;
    } = {}
  ): Promise<DiscoveredTest[]> {
    const tests: DiscoveredTest[] = [];

    try {
      const stats = await stat(testDir);

      if (stats.isFile()) {
        // Single test file
        const tests = await this.discoverFromFile(testDir);
        return tests;
      } else if (stats.isDirectory()) {
        // Discover all test files
        const vSourceTests = await this.discoverVTests(testDir);
        const jsonTests = await this.discoverJsonTests(testDir);

        tests.push(...vSourceTests, ...jsonTests);
      }
    } catch (error) {
      if (options.verbose) {
        console.warn(`Failed to discover tests in ${testDir}: ${error}`);
      }
    }

    return tests;
  }

  /**
   * Discover .v source test files
   */
  private static async discoverVTests(testDir: string): Promise<DiscoveredTest[]> {
    const tests: DiscoveredTest[] = [];

    try {
      const files = await this.findFilesRecursive(testDir, (f) => f.endsWith('.v'));

      for (const file of files) {
        const sourceTests = await this.parseVFile(file);
        tests.push(...sourceTests);
      }
    } catch (error) {
      // Silently skip if directory doesn't exist
    }

    return tests;
  }

  /**
   * Discover .test.json test suites
   */
  private static async discoverJsonTests(testDir: string): Promise<DiscoveredTest[]> {
    const tests: DiscoveredTest[] = [];

    try {
      const files = await this.findFilesRecursive(testDir, (f) => f.endsWith('.test.json'));

      for (const file of files) {
        try {
          const content = await readFile(file, 'utf8');
          const data = JSON.parse(content);

          const testCases = data.tests || data.testCases || [];
          for (const testCase of testCases) {
            tests.push({
              name: testCase.name,
              path: file,
              type: 'json-suite',
              description: testCase.description,
              parameters: testCase.parameters
            });
          }
        } catch (error) {
          console.warn(`Failed to parse test file ${file}: ${error}`);
        }
      }
    } catch (error) {
      // Silently skip if directory doesn't exist
    }

    return tests;
  }

  /**
   * Discover from single file
   */
  private static async discoverFromFile(file: string): Promise<DiscoveredTest[]> {
    if (file.endsWith('.v')) {
      return this.parseVFile(file);
    } else if (file.endsWith('.test.json')) {
      try {
        const content = await readFile(file, 'utf8');
        const data = JSON.parse(content);

        const testCases = data.tests || data.testCases || [];
        return testCases.map((testCase: any) => ({
          name: testCase.name,
          path: file,
          type: 'json-suite' as const,
          description: testCase.description,
          parameters: testCase.parameters
        }));
      } catch (error) {
        console.warn(`Failed to parse test file ${file}: ${error}`);
        return [];
      }
    }

    return [];
  }

  /**
   * Parse .v source file for test functions and parameters
   */
  private static async parseVFile(file: string): Promise<DiscoveredTest[]> {
    const tests: DiscoveredTest[] = [];

    try {
      const content = await readFile(file, 'utf8');

      // Find all function definitions with potential test annotations
      // Pattern 1: pub function with #[test] annotation
      // Pattern 2: function with specific naming convention (test_*, *_test)

      const lines = content.split('\n');
      let currentFunction: string | null = null;
      let currentParams: any[] | null = null;
      let currentDescription: string | null = null;

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i].trim();

        // Check for @test-params comment
        const paramsMatch = line.match(/@test-params\s+(.*)/);
        if (paramsMatch) {
          try {
            const paramsStr = paramsMatch[1].trim();
            // Try to parse as JSON array first
            if (paramsStr.startsWith('[')) {
              currentParams = JSON.parse(paramsStr);
            } else {
              // Parse space-separated values
              currentParams = paramsStr.split(/\s+/).map(p => {
                // Try to parse as number
                if (!isNaN(Number(p))) {
                  return Number(p);
                }
                return p;
              });
            }
          } catch (error) {
            console.warn(`Failed to parse @test-params in ${file}:${i + 1}: ${line}`);
          }
          continue;
        }

        // Check for test annotation
        if (line.startsWith('#[test]') || line.includes('#[test]')) {
          // Next non-empty line should be the function definition
          for (let j = i + 1; j < lines.length; j++) {
            const nextLine = lines[j].trim();
            if (nextLine && !nextLine.startsWith('//')) {
              const funcMatch = nextLine.match(/(?:pub\s+)?(?:fn|instruction|script)\s+(\w+)\s*\(/);
              if (funcMatch) {
                currentFunction = funcMatch[1];
                break;
              }
            }
          }
          continue;
        }

        // Check for pub function that matches test naming convention
        const funcMatch = line.match(/pub\s+(?:fn|instruction|script)\s+(test_\w+|_?\w+_test)\s*\(/);
        if (funcMatch) {
          const functionName = funcMatch[1];

          // Check if we have parameters from @test-params comment
          if (currentParams || currentFunction) {
            const name = basename(file, '.v') + '::' + functionName;

            tests.push({
              name,
              path: file,
              type: 'v-source',
              source: {
                type: 'v-source',
                file,
                functionName,
                parameters: currentParams || undefined,
                description: currentDescription || undefined
              },
              parameters: currentParams || undefined,
              description: currentDescription || undefined
            });

            currentParams = null;
            currentDescription = null;
            currentFunction = null;
          }
        }
      }
    } catch (error) {
      console.warn(`Failed to parse V file ${file}: ${error}`);
    }

    return tests;
  }

  /**
   * Compile a .v test source file
   */
  static async compileVTest(file: string): Promise<{
    success: boolean;
    bytecode?: Uint8Array;
    errors?: string[];
  }> {
    try {
      const source = await readFile(file, 'utf8');

      // Check if file has use statements for multi-file compilation
      const hasUseStatements = /^\s*use\s+/m.test(source) || /^\s*import\s+/m.test(source);

      let compilation;
      if (hasUseStatements) {
        // Use multi-file compilation with auto-discovery
        // This would require access to WasmCompilerWrapper
        // For now, use single-file compilation
        compilation = await FiveSDK.compile(source, {
          debug: false,
          optimize: false
        });
      } else {
        compilation = await FiveSDK.compile(source, {
          debug: false,
          optimize: false
        });
      }

      if (!compilation.success) {
        const errors = compilation.errors || [];
        const errorMessages = errors.map(e =>
          typeof e === 'string' ? e : (e as any).message || 'Unknown error'
        );
        return {
          success: false,
          errors: errorMessages
        };
      }

      return {
        success: true,
        bytecode: compilation.bytecode
      };
    } catch (error) {
      return {
        success: false,
        errors: [error instanceof Error ? error.message : 'Compilation failed']
      };
    }
  }

  /**
   * Find files recursively matching predicate
   */
  private static async findFilesRecursive(
    dir: string,
    predicate: (file: string) => boolean
  ): Promise<string[]> {
    const files: string[] = [];

    try {
      const entries = await readdir(dir, { withFileTypes: true });

      for (const entry of entries) {
        const fullPath = join(dir, entry.name);

        if (entry.isDirectory() && !entry.name.startsWith('.')) {
          // Recursively search subdirectories
          const subFiles = await this.findFilesRecursive(fullPath, predicate);
          files.push(...subFiles);
        } else if (entry.isFile() && predicate(entry.name)) {
          files.push(fullPath);
        }
      }
    } catch (error) {
      // Silently skip inaccessible directories
    }

    return files;
  }
}
