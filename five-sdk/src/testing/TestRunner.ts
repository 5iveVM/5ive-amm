/**
 * Five SDK Test Runner
 * 
 * SDK-based test utilities that replace shell script approaches with programmatic
 * Five SDK usage. Provides comprehensive testing capabilities for Five VM scripts.
 */

import { readFile } from 'fs/promises';
import { basename } from 'path';
import { FiveSDK } from '../FiveSDK.js';
import { FiveBytecode, FiveScriptSource } from '../types.js';
import { TestDiscovery } from './TestDiscovery.js';

/**
 * Test case definition
 */
export interface TestCase {
  name: string;
  description?: string;
  bytecode?: string;       // Path to bytecode file
  source?: string;         // Path to source file (will be compiled)
  function?: string | number; // Function name or index to test
  parameters?: any[];      // Function parameters
  expected: {
    success: boolean;
    result?: any;
    error?: string;
    maxComputeUnits?: number;
    minComputeUnits?: number;
  };
  timeout?: number;
}

/**
 * Test suite definition
 */
export interface TestSuite {
  name: string;
  description?: string;
  setup?: () => Promise<void>;
  teardown?: () => Promise<void>;
  testCases: TestCase[];
}

/**
 * Test result
 */
export interface TestResult {
  name: string;
  passed: boolean;
  duration: number;
  error?: string;
  computeUnitsUsed?: number;
  result?: any;
  logs?: string[];
  trace?: any[];
}

/**
 * Test suite result
 */
export interface TestSuiteResult {
  suite: TestSuite;
  results: TestResult[];
  duration: number;
  passed: number;
  failed: number;
  skipped: number;
}

/**
 * Test runner options
 */
export interface TestRunnerOptions {
  timeout?: number;        // Default timeout per test (ms)
  maxComputeUnits?: number; // Max CU per test
  parallel?: number;       // Number of parallel tests (0 = serial)
  verbose?: boolean;       // Verbose output
  debug?: boolean;         // Debug mode
  trace?: boolean;         // Enable execution tracing
  pattern?: string;        // Test name pattern filter
  failFast?: boolean;      // Stop on first failure
}

/**
 * Five SDK-based test runner
 */
export class FiveTestRunner {
  private options: Required<TestRunnerOptions>;
  private compilationCache = new Map<string, { bytecode: Uint8Array; abi?: any }>();

  constructor(options: TestRunnerOptions = {}) {
    this.options = {
      timeout: 30000,
      maxComputeUnits: 1000000,
      parallel: 0,
      verbose: false,
      debug: false,
      trace: false,
      pattern: '*',
      failFast: false,
      ...options
    };
  }

  /**
   * Run a single test case
   */
  async runTestCase(testCase: TestCase): Promise<TestResult> {
    const startTime = Date.now();
    
    try {
      // Apply test name pattern filter
      if (!this.matchesPattern(testCase.name, this.options.pattern)) {
        return {
          name: testCase.name,
          passed: false,
          duration: 0,
          error: 'Skipped by pattern'
        };
      }

      let bytecode: Uint8Array;
      let abi: any | undefined;

      // Get bytecode (compile source or load existing)
      if (testCase.source) {
        if (!this.compilationCache.has(testCase.source)) {
          const result = await this.compileToBytecode(testCase.source);
          if (!result.success || !result.bytecode) {
            throw new Error(`Compilation failed: ${result.errors?.join(', ')}`);
          }
          this.compilationCache.set(testCase.source, {
            bytecode: result.bytecode,
            abi: result.abi
          });
        }
        const cached = this.compilationCache.get(testCase.source)!;
        bytecode = cached.bytecode;
        abi = cached.abi;
      } else if (testCase.bytecode) {
        const data = await readFile(testCase.bytecode);
        bytecode = new Uint8Array(data);
      } else {
        throw new Error('Test case must specify either source or bytecode');
      }

      // Validate bytecode
      const validation = await FiveSDK.validateBytecode(bytecode, {
        debug: this.options.debug
      });

      if (!validation.valid) {
        throw new Error(`Invalid bytecode: ${validation.errors?.join(', ')}`);
      }

      // Execute with timeout
      const executionTimeout = testCase.timeout || this.options.timeout;
      const executionPromise = FiveSDK.executeLocally(
        bytecode,
        testCase.function || 0,
        testCase.parameters || [],
        {
          debug: this.options.debug,
          trace: this.options.trace,
          computeUnitLimit: this.options.maxComputeUnits,
          abi
        }
      );

      const timeoutPromise = new Promise<never>((_, reject) => 
        setTimeout(() => reject(new Error('Test timeout')), executionTimeout)
      );

      const result = await Promise.race([executionPromise, timeoutPromise]);
      const duration = Date.now() - startTime;

      // Validate result
      const passed = this.validateResult(result, testCase.expected);

      return {
        name: testCase.name,
        passed,
        duration,
        computeUnitsUsed: result.computeUnitsUsed,
        result: result.result,
        logs: result.logs,
        trace: result.trace,
        error: passed ? undefined : this.getValidationError(result, testCase.expected)
      };

    } catch (error) {
      const duration = Date.now() - startTime;
      
      // Check if error was expected
      const errorMessage = error instanceof Error ? error.message : 'Unknown error';
      const passed = testCase.expected.success === false && 
                    testCase.expected.error !== undefined &&
                    errorMessage.includes(testCase.expected.error);

      return {
        name: testCase.name,
        passed,
        duration,
        error: errorMessage
      };
    }
  }

  /**
   * Run a complete test suite
   */
  async runTestSuite(suite: TestSuite): Promise<TestSuiteResult> {
    const startTime = Date.now();
    const results: TestResult[] = [];

    try {
      // Run setup if provided
      if (suite.setup) {
        await suite.setup();
      }

      // Run tests (serial or parallel)
      if (this.options.parallel > 0) {
        // Parallel execution
        const chunks = this.chunkArray(suite.testCases, this.options.parallel);
        
        for (const chunk of chunks) {
          const chunkResults = await Promise.all(
            chunk.map(testCase => this.runTestCase(testCase))
          );
          results.push(...chunkResults);
          
          // Fail fast check
          if (this.options.failFast && chunkResults.some(r => !r.passed)) {
            break;
          }
        }
      } else {
        // Serial execution
        for (const testCase of suite.testCases) {
          const result = await this.runTestCase(testCase);
          results.push(result);
          
          if (this.options.verbose) {
            console.log(this.formatTestResult(result));
          }
          
          // Fail fast check
          if (this.options.failFast && !result.passed) {
            break;
          }
        }
      }

      // Run teardown if provided
      if (suite.teardown) {
        await suite.teardown();
      }

    } catch (error) {
      console.error(`Test suite setup/teardown error: ${error}`);
    }

    const duration = Date.now() - startTime;
    const passed = results.filter(r => r.passed).length;
    const failed = results.filter(r => !r.passed && r.error !== 'Skipped by pattern').length;
    const skipped = results.filter(r => r.error === 'Skipped by pattern').length;

    return {
      suite,
      results,
      duration,
      passed,
      failed,
      skipped
    };
  }

  /**
   * Run multiple test suites
   */
  async runTestSuites(suites: TestSuite[]): Promise<TestSuiteResult[]> {
    const results: TestSuiteResult[] = [];
    
    for (const suite of suites) {
      if (this.options.verbose) {
        console.log(`\n📋 Running test suite: ${suite.name}`);
        if (suite.description) {
          console.log(`   ${suite.description}`);
        }
      }
      
      const result = await this.runTestSuite(suite);
      results.push(result);
      
      if (this.options.verbose) {
        console.log(this.formatSuiteResult(result));
      }
      
      // Fail fast for suites
      if (this.options.failFast && result.failed > 0) {
        break;
      }
    }
    
    return results;
  }

  /**
   * Discover and load test suites from directory
   */
  async discoverTestSuites(directory: string, pattern: string = '**/*.test.json'): Promise<TestSuite[]> {
    const discovered = await TestDiscovery.discoverTests(directory, { verbose: this.options.verbose });
    const suites: TestSuite[] = [];
    const byFile = new Map<string, TestCase[]>();
    const loadedJsonSuites = new Set<string>();

    for (const test of discovered) {
      if (test.type === 'json-suite') {
        if (loadedJsonSuites.has(test.path)) {
          continue;
        }
        try {
          const content = await readFile(test.path, 'utf8');
          const data = JSON.parse(content);
          suites.push({
            name: data.name || basename(test.path, '.test.json'),
            description: data.description,
            testCases: data.tests || data.testCases || []
          });
          loadedJsonSuites.add(test.path);
        } catch (error) {
          console.warn(`Failed to load test suite ${test.path}: ${error}`);
        }
        continue;
      }

      if (test.type === 'v-source' && test.source) {
        const cases = byFile.get(test.path) || [];
        cases.push({
          name: test.name,
          source: test.path,
          function: test.source.functionName,
          parameters: test.parameters || [],
          expected: {
            success: true,
            result: test.expectsResult ? test.expectedResult : undefined
          }
        });
        byFile.set(test.path, cases);
      }
    }

    for (const [file, testCases] of byFile.entries()) {
      suites.push({
        name: basename(file, '.v'),
        description: `Tests from ${file}`,
        testCases
      });
    }

    return suites;
  }

  /**
   * Compile and execute test - convenience method
   */
  static async compileAndTest(
    source: FiveScriptSource,
    functionName?: string | number,
    parameters: any[] = [],
    options: {
      debug?: boolean;
      trace?: boolean;
      computeUnitLimit?: number;
    } = {}
  ): Promise<{
    compilation: any;
    execution: any;
    success: boolean;
    error?: string;
  }> {
    try {
      // Compile
      const compilation = await FiveSDK.compile(source, {
        debug: options.debug,
        optimize: false
      });

      if (!compilation.success || !compilation.bytecode) {
        return {
          compilation,
          execution: null,
          success: false,
          error: `Compilation failed: ${compilation.errors?.join(', ')}`
        };
      }

      // Execute
      const execution = await FiveSDK.executeLocally(
        compilation.bytecode,
        functionName || 0,
        parameters,
        options
      );

      return {
        compilation,
        execution,
        success: execution.success || false,
        error: execution.error
      };

    } catch (error) {
      return {
        compilation: null,
        execution: null,
        success: false,
        error: error instanceof Error ? error.message : 'Unknown error'
      };
    }
  }

  // Private helper methods

  private async compileToBytecode(sourcePath: string): Promise<any> {
    const source = await readFile(sourcePath, 'utf8');
    return FiveSDK.compile({ filename: sourcePath, content: source }, { debug: this.options.debug });
  }

  private validateResult(result: any, expected: any): boolean {
    // Check success/failure
    if (result.success !== expected.success) {
      return false;
    }

    // If expecting success, check result value
    if (expected.success && expected.result !== undefined) {
      if (JSON.stringify(result.result) !== JSON.stringify(expected.result)) {
        return false;
      }
    }

    // Check compute units constraints
    if (expected.maxComputeUnits && result.computeUnitsUsed > expected.maxComputeUnits) {
      return false;
    }

    if (expected.minComputeUnits && result.computeUnitsUsed < expected.minComputeUnits) {
      return false;
    }

    return true;
  }

  private getValidationError(result: any, expected: any): string {
    const errors: string[] = [];

    if (result.success !== expected.success) {
      errors.push(`Expected success=${expected.success}, got ${result.success}`);
    }

    if (expected.success && expected.result !== undefined) {
      if (JSON.stringify(result.result) !== JSON.stringify(expected.result)) {
        errors.push(`Expected result ${JSON.stringify(expected.result)}, got ${JSON.stringify(result.result)}`);
      }
    }

    if (expected.maxComputeUnits && result.computeUnitsUsed > expected.maxComputeUnits) {
      errors.push(`Exceeded max compute units: ${result.computeUnitsUsed} > ${expected.maxComputeUnits}`);
    }

    if (expected.minComputeUnits && result.computeUnitsUsed < expected.minComputeUnits) {
      errors.push(`Below min compute units: ${result.computeUnitsUsed} < ${expected.minComputeUnits}`);
    }

    return errors.join('; ');
  }

  private matchesPattern(name: string, pattern: string): boolean {
    if (pattern === '*') return true;
    
    const regex = new RegExp(
      pattern.replace(/\*/g, '.*').replace(/\?/g, '.'),
      'i'
    );
    return regex.test(name);
  }

  private chunkArray<T>(array: T[], chunkSize: number): T[][] {
    const chunks: T[][] = [];
    for (let i = 0; i < array.length; i += chunkSize) {
      chunks.push(array.slice(i, i + chunkSize));
    }
    return chunks;
  }

  private formatTestResult(result: TestResult): string {
    const icon = result.passed ? '✅' : '❌';
    const duration = `${result.duration}ms`;
    const cu = result.computeUnitsUsed ? `${result.computeUnitsUsed} CU` : '';
    
    let line = `${icon} ${result.name} (${duration}`;
    if (cu) line += `, ${cu}`;
    line += ')';
    
    if (!result.passed && result.error) {
      line += `\n   Error: ${result.error}`;
    }
    
    return line;
  }

  private formatSuiteResult(result: TestSuiteResult): string {
    const { passed, failed, skipped } = result;
    const total = passed + failed + skipped;
    const duration = `${result.duration}ms`;
    
    let line = `📊 Results: ${passed}/${total} passed`;
    if (failed > 0) line += `, ${failed} failed`;
    if (skipped > 0) line += `, ${skipped} skipped`;
    line += ` (${duration})`;
    
    return line;
  }
}
