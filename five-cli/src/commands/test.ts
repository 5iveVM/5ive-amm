// Test command.

import { readFile, readdir, stat } from 'fs/promises';
import { join, basename } from 'path';
import ora from 'ora';

import {
  CommandDefinition,
  CommandContext
} from '../types.js';
import { FiveSDK, FiveTestRunner, TestDiscovery } from '@5ive-tech/sdk';
import { ConfigManager } from '../config/ConfigManager.js';
import { ConfigOverrides } from '../config/types.js';
import { Connection, Keypair } from '@solana/web3.js';
import { FiveFileManager } from '../utils/FiveFileManager.js';
import { loadProjectConfig } from '../project/ProjectLoader.js';
import { section, success as uiSuccess, error as uiError } from '../utils/cli-ui.js';

interface TestCase {
  name: string;
  bytecode: string;
  input?: string;
  accounts?: string;
  sourceFile?: string;
  functionRef?: string | number;
  parameters?: any[];
  inlineBytecode?: Uint8Array;
  inlineAbi?: any;
  expected: {
    success: boolean;
    result?: any;
    error?: string;
    maxComputeUnits?: number;
  };
}

interface TestSuite {
  name: string;
  description?: string;
  testCases: TestCase[];
}

interface TestResult {
  name: string;
  passed: boolean;
  duration: number;
  error?: string;
  computeUnits?: number;
  details?: any;
}

interface OnChainTestResult {
  scriptFile: string;
  passed: boolean;
  deployResult?: {
    success: boolean;
    scriptAccount?: string;
    transactionId?: string;
    cost?: number;
    error?: string;
  };
  executeResult?: {
    success: boolean;
    transactionId?: string;
    computeUnitsUsed?: number;
    cost?: number;
    result?: any;
    error?: string;
  };
  totalDuration: number;
  totalCost: number;
  error?: string;
}

interface OnChainTestSummary {
  totalScripts: number;
  passed: number;
  failed: number;
  totalCost: number;
  totalDuration: number;
  results: OnChainTestResult[];
}

interface OnChainFixtureAccount {
  owner?: string; // "system" or base58 pubkey
  lamports?: number;
  data_len?: number;
  is_signer?: boolean;
  is_writable?: boolean;
}

interface OnChainFixtureTestSpec {
  accounts?: string[];
  parameters?: any[];
  expected?: {
    success?: boolean;
    errorContains?: string;
  };
}

interface OnChainFixtureFile {
  accounts?: Record<string, OnChainFixtureAccount>;
  tests?: Record<string, OnChainFixtureTestSpec>;
}

/**
 * 5IVE test command implementation
 */
export const testCommand: CommandDefinition = {
  name: 'test',
  description: 'Run test suites',
  aliases: ['t'],

  options: [
    {
      flags: '-p, --pattern <pattern>',
      description: 'Test discovery pattern (default: *)',
      defaultValue: '*'
    },
    {
      flags: '-f, --filter <filter>',
      description: 'Run tests matching filter pattern',
      required: false
    },
    {
      flags: '--timeout <ms>',
      description: 'Test timeout in milliseconds',
      defaultValue: 30000
    },
    {
      flags: '--max-cu <units>',
      description: 'Maximum compute units per test',
      defaultValue: 1000000
    },
    {
      flags: '--parallel <count>',
      description: 'Number of parallel test workers (0 = CPU count)',
      defaultValue: 0
    },
    {
      flags: '--benchmark',
      description: 'Run performance benchmarks',
      defaultValue: false
    },
    {
      flags: '--coverage',
      description: 'Generate test coverage report',
      defaultValue: false
    },
    {
      flags: '--watch',
      description: 'Watch for file changes and re-run tests',
      defaultValue: false
    },
    {
      flags: '--format <format>',
      description: 'Output format',
      choices: ['text', 'json', 'junit'],
      defaultValue: 'text'
    },
    {
      flags: '--verbose',
      description: 'Show detailed test output',
      defaultValue: false
    },
    {
      flags: '--sdk-runner',
      description: 'Use modern SDK-based test runner (recommended)',
      defaultValue: false
    },
    {
      flags: '--on-chain',
      description: 'Execute tests on-chain (deploy + execute)',
      defaultValue: false
    },
    {
      flags: '--batch',
      description: 'Run all .bin files in batch mode',
      defaultValue: false
    },
    {
      flags: '-t, --target <target>',
      description: 'Override target network (devnet, testnet, mainnet, local)',
      required: false
    },
    {
      flags: '-n, --network <url>',
      description: 'Override network RPC URL',
      required: false
    },
    {
      flags: '-k, --keypair <file>',
      description: 'Override keypair file path',
      required: false
    },
    {
      flags: '--retry-failed',
      description: 'Retry only previously failed tests',
      defaultValue: false
    },
    {
      flags: '--analyze-costs',
      description: 'Include detailed cost analysis in results',
      defaultValue: false
    },
    {
      flags: '--allow-mainnet-tests',
      description: 'Allow on-chain tests on mainnet (requires --max-cost-sol)',
      defaultValue: false
    },
    {
      flags: '--max-cost-sol <amount>',
      description: 'Maximum SOL budget for on-chain test runs',
      required: false
    },
    {
      flags: '--project <path>',
      description: 'Project directory or five.toml path',
      required: false
    }
  ],

  arguments: [
    {
      name: 'path',
      description: 'Test directory or file (default: ./tests)',
      required: false
    }
  ],

  examples: [
    {
      command: '5ive test',
      description: 'Run all tests in ./tests directory'
    },
    {
      command: '5ive test --filter "token*" --verbose',
      description: 'Run token tests with verbose output'
    },
    {
      command: '5ive test ./my-tests --benchmark --format json',
      description: 'Run benchmarks with JSON output'
    },
    {
      command: '5ive test --watch --parallel 4',
      description: 'Watch mode with 4 parallel workers'
    },
    {
      command: '5ive test test-scripts/ --on-chain --target devnet',
      description: 'Run on-chain tests on devnet'
    },
    {
      command: '5ive test test-scripts/ --on-chain --batch --analyze-costs',
      description: 'Batch test all .bin files with cost analysis'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;

    try {
      const projectContext = await loadProjectConfig(options.project, process.cwd());

      // Apply project defaults if not provided
      if (!options.target && projectContext?.config.cluster) {
        options.target = projectContext.config.cluster;
      }
      if (!options.network && projectContext?.config.rpcUrl) {
        options.network = projectContext.config.rpcUrl;
      }
      if (!options.keypair && projectContext?.config.keypairPath) {
        options.keypair = projectContext.config.keypairPath;
      }

      const testPath =
        args[0] ||
        (projectContext ? join(projectContext.rootDir, 'tests') : undefined) ||
        './tests';

      // Handle on-chain testing mode
      if (options.onChain) {
        await runOnChainTests(testPath, options, context);
        return;
      }

      // Use modern SDK-based test runner if requested
      if (options.sdkRunner) {
        await runWithSdkRunner(testPath, options, context);
        return;
      }

      // Legacy approach with SDK integration
      // Initialize SDK for testing
      const spinner = ora('Initializing 5IVE SDK for testing...').start();

      // No initialization needed for SDK - it's stateless
      const sdk = FiveSDK.create({ debug: options.verbose });

      spinner.succeed('5IVE SDK initialized');

      // Discover test files
      const testSuites = await discoverTestSuites(testPath, options, logger);

      if (testSuites.length === 0) {
        logger.warn('No test files found');
        return;
      }

      logger.info(`Found ${testSuites.length} test suite(s) with ${getTotalTestCount(testSuites)} test(s)`);

      // Run tests
      const results = await runTestSuites(testSuites, sdk, options, context);

      // Display results
      displayTestResults(results, options, logger);

      // Handle watch mode
      if (options.watch) {
        await watchAndRerun(testPath, options, context);
      }

      // Exit with appropriate code
      const failed = results.some(suite => suite.results.some(test => !test.passed));
      if (failed) {
        process.exit(1);
      }

    } catch (error) {
      logger.error('Test execution failed:', error);
      throw error;
    }
  }
};

/**
 * Discover test suites from files (both .test.json and .v source)
 */
async function discoverTestSuites(
  testPath: string,
  options: any,
  logger: any
): Promise<TestSuite[]> {
  const testSuites: TestSuite[] = [];
  const compiledVTests = new Map<string, { bytecode: Uint8Array; abi?: any }>();
  const loadedJsonSuites = new Set<string>();

  try {
    // Use new TestDiscovery to find both .test.json and .v files
    const discoveredTests = await TestDiscovery.discoverTests(testPath, { verbose: options.verbose });

    if (discoveredTests.length === 0 && options.verbose) {
      logger.info('No tests discovered');
    }

    // Organize discovered tests into suites
    const suiteMap = new Map<string, TestCase[]>();

    for (const test of discoveredTests) {
      if (test.type === 'v-source') {
        // Compile .v source file if not already compiled
        if (!compiledVTests.has(test.path)) {
          const spinner = ora(`Compiling ${basename(test.path)}...`).start();

          try {
            const source = await readFile(test.path, 'utf8');
            const compilation = await FiveSDK.compile(
              { filename: test.path, content: source },
              { debug: options.verbose, optimize: false }
            );

            if (compilation.success && compilation.bytecode) {
              compiledVTests.set(test.path, {
                bytecode: compilation.bytecode,
                abi: compilation.abi
              });
              spinner.succeed(`Compiled ${basename(test.path)}`);
            } else {
              spinner.fail(`Failed to compile ${basename(test.path)}`);
              logger.error(`Compilation errors: ${compilation.errors?.join(', ')}`);
              continue;
            }
          } catch (error) {
            spinner.fail(`Error compiling ${basename(test.path)}`);
            logger.error(error instanceof Error ? error.message : 'Unknown error');
            continue;
          }
        }

        // Create test case from compiled .v file
        const compiled = compiledVTests.get(test.path);
        if (compiled && test.source) {
          const sourceMeta = test.source as any;
          const suite = suiteMap.get(test.path) || [];
          suite.push({
            name: test.name,
            bytecode: test.path,
            sourceFile: test.path,
            functionRef: test.source.functionName,
            parameters: test.parameters || [],
            inlineBytecode: compiled.bytecode,
            inlineAbi: compiled.abi,
            expected: {
              success: true,
              result: sourceMeta?.expectsResult ? sourceMeta?.expectedResult : undefined
            }
          });
          suiteMap.set(test.path, suite);
        }
      } else if (test.type === 'json-suite') {
        if (loadedJsonSuites.has(test.path)) {
          continue;
        }
        // Load .test.json suite
        const suite = await loadTestSuite(test.path);
        if (suite) {
          testSuites.push(suite);
          loadedJsonSuites.add(test.path);
        }
      }
    }

    // Convert compiled V tests to TestSuite format
    for (const [filePath, testCases] of suiteMap.entries()) {
      testSuites.push({
        name: `${basename(filePath, '.v')}`,
        description: `Tests from ${filePath}`,
        testCases
      });
    }
  } catch (error) {
    logger.warn(`Failed to discover tests at ${testPath}: ${error}`);
  }

  // Apply filter if specified
  if (options.filter) {
    return testSuites.filter(suite =>
      suite.name.includes(options.filter) ||
      suite.testCases.some(test => test.name.includes(options.filter))
    );
  }

  return testSuites;
}

/**
 * Load test suite from JSON file
 */
async function loadTestSuite(filePath: string): Promise<TestSuite | null> {
  try {
    const content = await readFile(filePath, 'utf8');
    const data = JSON.parse(content);

    return {
      name: data.name || basename(filePath, '.test.json'),
      description: data.description,
      testCases: data.tests || data.testCases || []
    };
  } catch (error) {
    console.warn(`Failed to load test suite ${filePath}: ${error}`);
    return null;
  }
}

/**
 * Run all test suites
 */
async function runTestSuites(
  testSuites: TestSuite[],
  sdk: FiveSDK,
  options: any,
  context: CommandContext
): Promise<Array<{ suite: TestSuite; results: TestResult[] }>> {
  const { logger } = context;
  const results: Array<{ suite: TestSuite; results: TestResult[] }> = [];

  for (const suite of testSuites) {
    logger.info(`Running test suite: ${suite.name}`);

    const suiteResults: TestResult[] = [];

    for (const testCase of suite.testCases) {
      const result = await runSingleTest(testCase, sdk, options, context);
      suiteResults.push(result);

      if (options.verbose) {
        displaySingleTestResult(result, logger);
      }
    }

    results.push({ suite, results: suiteResults });
  }

  return results;
}

/**
 * Run a single test case
 */
async function runSingleTest(
  testCase: TestCase,
  sdk: FiveSDK,
  options: any,
  context: CommandContext
): Promise<TestResult> {
  const { logger } = context;
  const startTime = Date.now();

  try {
    let bytecode: Uint8Array;
    let abi: any = testCase.inlineAbi;
    if (testCase.inlineBytecode) {
      bytecode = testCase.inlineBytecode;
    } else {
      // Load bytecode using centralized manager
      const fileManager = FiveFileManager.getInstance();
      const loadedFile = await fileManager.loadFile(testCase.bytecode, {
        validateFormat: true
      });
      bytecode = loadedFile.bytecode;
      if (!abi) {
        abi = loadedFile.abi;
      }
    }

    // Parse input parameters if specified
    let parameters: any[] = testCase.parameters || [];
    if (parameters.length === 0 && testCase.input) {
      const inputData = await readFile(testCase.input, 'utf8');
      try {
        parameters = JSON.parse(inputData);
      } catch {
        // If not JSON, treat as raw string parameter
        parameters = [inputData];
      }
    }

    // Execute with timeout using 5IVE SDK
    const executionPromise = FiveSDK.executeLocally(
      bytecode,
      testCase.functionRef ?? 0,
      parameters,
      {
        debug: options.verbose,
        trace: options.verbose,
        computeUnitLimit: options.maxCu,
        abi // Pass ABI for function name resolution
      }
    );

    const timeoutPromise = new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Test timeout')), options.timeout)
    );

    const result = await Promise.race([executionPromise, timeoutPromise]) as any;
    const duration = Date.now() - startTime;

    // Validate result against expected
    const passed = validateTestResult(result, testCase.expected);

    return {
      name: testCase.name,
      passed,
      duration,
      computeUnits: result.computeUnitsUsed || 0,
      details: options.verbose ? result : undefined
    };

  } catch (error) {
    const duration = Date.now() - startTime;

    // Check if error was expected
    const passed = testCase.expected.success === false &&
      testCase.expected.error !== undefined &&
      error instanceof Error &&
      error.message.includes(testCase.expected.error);

    return {
      name: testCase.name,
      passed,
      duration,
      error: error instanceof Error ? error.message : 'Unknown error'
    };
  }
}

/**
 * Validate test result against expected outcome
 */
function validateTestResult(result: any, expected: any): boolean {
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

  // Check compute units limit
  if (expected.maxComputeUnits && result.computeUnitsUsed > expected.maxComputeUnits) {
    return false;
  }

  return true;
}

/**
 * Display single test result
 */
function displaySingleTestResult(result: TestResult, logger: any): void {
  const status = result.passed ? 'OK' : 'FAIL';
  const duration = `(${result.duration}ms)`;
  const cu = result.computeUnits ? `[${result.computeUnits} CU]` : '';

  console.log(`  ${status} ${result.name} ${duration} ${cu}`);

  if (!result.passed && result.error) {
    console.log(`    Error: ${result.error}`);
  }
}

/**
 * Display comprehensive test results
 */
function displayTestResults(
  results: Array<{ suite: TestSuite; results: TestResult[] }>,
  options: any,
  logger: any
): void {
  if (options.format === 'json') {
    console.log(JSON.stringify(results, null, 2));
    return;
  }

  console.log('\n' + section('Test Results'));

  let totalTests = 0;
  let totalPassed = 0;
  let totalDuration = 0;

  for (const { suite, results: suiteResults } of results) {
    const passed = suiteResults.filter(r => r.passed).length;
    const total = suiteResults.length;
    const suiteDuration = suiteResults.reduce((sum, r) => sum + r.duration, 0);

    totalTests += total;
    totalPassed += passed;
    totalDuration += suiteDuration;

    const status = passed === total ? 'OK' : 'FAIL';
    console.log(`\n${status} ${suite.name}: ${passed}/${total} passed (${suiteDuration}ms)`);

    if (options.verbose || passed !== total) {
      suiteResults.forEach(result => displaySingleTestResult(result, logger));
    }
  }

  // Summary
  console.log('\n' + section('Summary'));
  console.log(`  Total: ${totalTests} tests`);
  console.log(`  Passed: ${totalPassed}`);

  const failed = totalTests - totalPassed;
  if (failed > 0) {
    console.log(`  Failed: ${failed}`);
  }

  console.log(`  Duration: ${totalDuration}ms`);

  if (failed === 0) {
    console.log(uiSuccess('All tests passed'));
  } else {
    console.log(uiError(`${failed} test(s) failed`));
  }
}

/**
 * Watch for file changes and re-run tests
 */
async function watchAndRerun(
  testPath: string,
  options: any,
  context: CommandContext
): Promise<void> {
  const { logger } = context;

  // Dynamic import for file watching
  const chokidar = await import('chokidar');

  logger.info('Watching for file changes...');

  const watcher = chokidar.watch([testPath, '**/*.bin'], {
    persistent: true,
    ignoreInitial: true
  });

  watcher.on('change', async (filePath) => {
    logger.info(`File changed: ${filePath}`);
    logger.info('Re-running tests...');

    try {
      // Re-run the test command
      const testSuites = await discoverTestSuites(testPath, options, logger);
      const sdk = FiveSDK.create({ debug: options.verbose });

      const results = await runTestSuites(testSuites, sdk, options, context);
      displayTestResults(results, options, logger);
    } catch (error) {
      logger.error(`Test re-run failed: ${error}`);
    }
  });

  // Handle graceful shutdown
  process.on('SIGINT', () => {
    logger.info('Stopping test watcher...');
    watcher.close();
    process.exit(0);
  });
}

/**
 * Run on-chain tests with deploy + execute pipeline
 */
async function runOnChainTests(
  testPath: string,
  options: any,
  context: CommandContext
): Promise<void> {
  const { logger } = context;

  logger.info('Starting on-chain test pipeline');

  try {
    // Apply configuration with CLI overrides
    const configManager = ConfigManager.getInstance();
    const overrides: ConfigOverrides = {
      target: options.target,
      network: options.network,
      keypair: options.keypair
    };

    const config = await configManager.applyOverrides(overrides);
    const targetPrefix = ConfigManager.getTargetPrefix(config.target);

    logger.info(`${targetPrefix} Testing on ${config.target}`);
    logger.info(`Network: ${config.networks[config.target].rpcUrl}`);
    logger.info(`Keypair: ${config.keypairPath}`);

    // Discover artifact test files (legacy on-chain mode)
    const testFiles = await discoverBinFiles(testPath, options);
    if (testFiles.length > 0) {
      logger.info(`Found ${testFiles.length} artifact test script(s)`);
    }

    // Setup Solana connection and keypair
    const connection = new Connection(config.networks[config.target].rpcUrl, 'confirmed');
    const signerKeypair = await loadKeypair(config.keypairPath);

    logger.info(`Deployer: ${signerKeypair.publicKey.toString()}`);

    const maxCostLamports = parseMaxCostLamports(options.maxCostSol);
    if (config.target === 'mainnet') {
      if (!options.allowMainnetTests) {
        throw new Error('mainnet on-chain tests require --allow-mainnet-tests');
      }
      if (maxCostLamports === undefined) {
        throw new Error('mainnet on-chain tests require --max-cost-sol <amount>');
      }
    }

    await ensureOnChainBalance(connection, signerKeypair, config.target, logger);

    const discovered = await TestDiscovery.discoverTests(testPath, { verbose: options.verbose });
    const vTests = discovered.filter((t) => t.type === 'v-source' && t.source);

    let results: OnChainTestSummary;
    if (vTests.length > 0) {
      results = await runDiscoveredVOnChainTests(
        vTests,
        connection,
        signerKeypair,
        options,
        config,
        maxCostLamports
      );
    } else {
      // Fall back to artifact-driven on-chain mode
      if (testFiles.length === 0) {
        logger.warn('No on-chain tests discovered (.v test functions or .bin/.five artifacts)');
        return;
      }
      results = await runBatchOnChainTests(
        testFiles,
        connection,
        signerKeypair,
        options,
        config,
        maxCostLamports
      );
    }

    // Display comprehensive results
    displayOnChainTestResults(results, options, logger);

    // Exit with appropriate code
    if (results.failed > 0) {
      logger.error(`${results.failed}/${results.totalScripts} tests failed`);
      process.exit(1);
    } else {
      logger.info(`All ${results.passed}/${results.totalScripts} tests passed`);
    }

  } catch (error) {
    logger.error('On-chain testing failed:', error);
    throw error;
  }
}

function parseMaxCostLamports(raw: unknown): number | undefined {
  if (raw === undefined || raw === null || raw === '') return undefined;
  const parsed = Number(raw);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    throw new Error(`Invalid --max-cost-sol value: ${raw}`);
  }
  return Math.floor(parsed * 1_000_000_000);
}

async function ensureOnChainBalance(
  connection: Connection,
  signerKeypair: Keypair,
  target: string,
  logger: any
): Promise<void> {
  const minLamports = 200_000_000; // 0.2 SOL baseline
  const balance = await connection.getBalance(signerKeypair.publicKey, 'confirmed');
  if (balance >= minLamports) {
    return;
  }

  if (target === 'local' || target === 'localnet') {
    logger.info('Low localnet balance detected; requesting airdrop...');
    const sig = await connection.requestAirdrop(signerKeypair.publicKey, 2_000_000_000);
    await connection.confirmTransaction(sig, 'confirmed');
    return;
  }

  throw new Error(
    `Insufficient balance for on-chain tests on ${target}. Balance=${(balance / 1e9).toFixed(6)} SOL`
  );
}

/**
 * Run tests using modern SDK-based test runner
 */
async function runWithSdkRunner(
  testPath: string,
  options: any,
  context: CommandContext
): Promise<void> {
  const { logger } = context;

  logger.info('Using 5IVE SDK test runner');

  // Create test runner with options
  const runner = new FiveTestRunner({
    timeout: options.timeout,
    maxComputeUnits: options.maxCu,
    parallel: options.parallel || 0,
    verbose: options.verbose,
    debug: options.verbose,
    trace: options.verbose,
    pattern: options.filter || options.pattern || '*',
    failFast: false
  });

  try {
    // Discover test suites
    const testSuites = await runner.discoverTestSuites(testPath);

    if (testSuites.length === 0) {
      logger.warn('No test files found');
      return;
    }

    logger.info(`Found ${testSuites.length} test suite(s)`);

    // Run test suites
    const results = await runner.runTestSuites(testSuites);

    // Display results in requested format
    if (options.format === 'json') {
      console.log(JSON.stringify(results, null, 2));
    } else {
      displaySdkTestResults(results, logger);
    }

    // Check for failures
    const totalFailed = results.reduce((sum, r) => sum + r.failed, 0);
    if (totalFailed > 0) {
      process.exit(1);
    }

  } catch (error) {
    logger.error('SDK Test Runner failed:', error);
    process.exit(1);
  }
}

/**
 * Display SDK test results
 */
function displaySdkTestResults(results: any[], logger: any): void {
  logger.info('\nTest Results Summary:');

  let totalPassed = 0;
  let totalFailed = 0;
  let totalSkipped = 0;
  let totalDuration = 0;

  for (const result of results) {
    totalPassed += result.passed;
    totalFailed += result.failed;
    totalSkipped += result.skipped;
    totalDuration += result.duration;

    const status = result.failed === 0 ? 'OK' : 'FAIL';
    logger.info(`${status} ${result.suite.name}: ${result.passed}/${result.passed + result.failed + result.skipped} passed (${result.duration}ms)`);

    if (result.failed > 0) {
      const failedTests = result.results.filter((r: any) => !r.passed);
      for (const test of failedTests) {
        logger.error(`   FAIL ${test.name}: ${test.error || 'Test failed'}`);
      }
    }
  }

  logger.info(`\nOverall: ${totalPassed} passed, ${totalFailed} failed, ${totalSkipped} skipped (${totalDuration}ms)`);

  if (totalFailed === 0) {
    logger.info(uiSuccess('All tests passed'));
  } else {
    logger.error(uiError(`${totalFailed} test(s) failed`));
  }
}

/**
 * Get total test count across all suites
 */
function getTotalTestCount(testSuites: TestSuite[]): number {
  return testSuites.reduce((total, suite) => total + suite.testCases.length, 0);
}

/**
 * Discover .bin files for on-chain testing
 */
async function discoverBinFiles(testPath: string, options: any): Promise<string[]> {
  const binFiles: string[] = [];

  try {
    const stats = await stat(testPath);

    if (stats.isFile()) {
      // Single file - check if it's a supported artifact
      if (testPath.endsWith('.bin') || testPath.endsWith('.five')) {
        binFiles.push(testPath);
      }
    } else if (stats.isDirectory()) {
      // Directory - recursively find all artifact files
      const files = await readdir(testPath, { recursive: true });

      for (const file of files) {
        if (
          typeof file === 'string' &&
          (file.endsWith('.bin') || file.endsWith('.five'))
        ) {
          const fullPath = join(testPath, file);

          // Skip node_modules directories
          if (fullPath.includes('node_modules')) {
            continue;
          }

          try {
            // Verify it's actually a file, not a directory
            const fileStats = await stat(fullPath);
            if (fileStats.isFile()) {
              binFiles.push(fullPath);
            }
          } catch (error) {
            // Skip files that can't be accessed
            continue;
          }
        }
      }
    }
  } catch (error) {
    console.warn(`Failed to discover .bin files at ${testPath}: ${error}`);
  }

  // Apply filter if specified
  if (options.filter) {
    return binFiles.filter(file => basename(file).includes(options.filter));
  }

  return binFiles.sort(); // Sort for consistent ordering
}

/**
 * Load keypair from file path
 */
async function loadKeypair(keypairPath: string): Promise<Keypair> {
  try {
    const keypairData = await readFile(keypairPath, 'utf8');
    const secretKey = JSON.parse(keypairData);
    return Keypair.fromSecretKey(new Uint8Array(secretKey));
  } catch (error) {
    throw new Error(`Failed to load keypair from ${keypairPath}: ${error}`);
  }
}

/**
 * Run batch on-chain tests with deploy → execute → verify pipeline
 */
async function runBatchOnChainTests(
  testFiles: string[],
  connection: Connection,
  signerKeypair: Keypair,
  options: any,
  config: any,
  maxCostLamports?: number
): Promise<OnChainTestSummary> {
  const results: OnChainTestResult[] = [];
  const startTime = Date.now();
  let totalCost = 0;

  console.log('\n' + section(`Running ${testFiles.length} On-Chain Tests`));

  for (let i = 0; i < testFiles.length; i++) {
    const scriptFile = testFiles[i];
    const scriptName = basename(scriptFile, '.bin');
    const testStartTime = Date.now();

    const spinner = ora(`[${i + 1}/${testFiles.length}] Testing ${scriptName}...`).start();

    try {
      // Load bytecode using centralized manager
      const fileManager = FiveFileManager.getInstance();
      const loadedFile = await fileManager.loadFile(scriptFile, {
        validateFormat: true
      });

      const bytecode = loadedFile.bytecode;

      if (options.verbose || options.debug) {
        spinner.text = `[${i + 1}/${testFiles.length}] Deploying ${scriptName} (${bytecode.length} bytes)...`;
      }

      // Deploy script
      const deployResult = await FiveSDK.deployToSolana(
        bytecode,
        connection,
        signerKeypair,
        {
          debug: options.verbose || options.debug || false,
          network: config.target,
          computeBudget: 1000000,
          maxRetries: 3
        }
      );

      if (!deployResult.success) {
        spinner.fail(`[${i + 1}/${testFiles.length}] ${scriptName} deployment failed`);

        results.push({
          scriptFile,
          passed: false,
          deployResult: {
            success: false,
            error: deployResult.error,
            cost: deployResult.deploymentCost || 0
          },
          totalDuration: Date.now() - testStartTime,
          totalCost: deployResult.deploymentCost || 0,
          error: `Deployment failed: ${deployResult.error}`
        });

        totalCost += deployResult.deploymentCost || 0;
        continue;
      }

      if (options.verbose || options.debug) {
        spinner.text = `[${i + 1}/${testFiles.length}] Executing ${scriptName}...`;
      }

      // Execute script (function 0 with no parameters)
      const executeResult = await FiveSDK.executeOnSolana(
        deployResult.programId!,
        connection,
        signerKeypair,
        0, // Function index 0
        [], // No parameters
        [], // No additional accounts
        {
          debug: options.verbose || options.debug || false,
          network: config.target,
          computeUnitLimit: 1000000,
          maxRetries: 3
        }
      );

      const testDuration = Date.now() - testStartTime;
      const testCost = (deployResult.deploymentCost || 0) + (executeResult.cost || 0);
      totalCost += testCost;
      if (maxCostLamports !== undefined && totalCost > maxCostLamports) {
        throw new Error(
          `On-chain test cost cap exceeded: ${(totalCost / 1e9).toFixed(6)} SOL > ${(maxCostLamports / 1e9).toFixed(6)} SOL`
        );
      }

      const passed = deployResult.success && executeResult.success;

      if (passed) {
        spinner.succeed(`[${i + 1}/${testFiles.length}] ${scriptName} OK (${testDuration}ms, ${(testCost / 1e9).toFixed(4)} SOL)`);
      } else {
        spinner.fail(`[${i + 1}/${testFiles.length}] ${scriptName} FAIL (${testDuration}ms)`);
      }

      results.push({
        scriptFile,
        passed,
        deployResult: {
          success: deployResult.success,
          scriptAccount: deployResult.programId,
          transactionId: deployResult.transactionId,
          cost: deployResult.deploymentCost || 0,
          error: deployResult.error
        },
        executeResult: {
          success: executeResult.success,
          transactionId: executeResult.transactionId,
          computeUnitsUsed: executeResult.computeUnitsUsed,
          result: executeResult.result,
          error: executeResult.error
        },
        totalDuration: testDuration,
        totalCost: testCost
      });

    } catch (error) {
      const testDuration = Date.now() - testStartTime;
      spinner.fail(`[${i + 1}/${testFiles.length}] ${scriptName} FAIL (error)`);

      results.push({
        scriptFile,
        passed: false,
        totalDuration: testDuration,
        totalCost: 0,
        error: error instanceof Error ? error.message : 'Unknown error'
      });
    }
  }

  const totalDuration = Date.now() - startTime;
  const passed = results.filter(r => r.passed).length;
  const failed = results.length - passed;

  return {
    totalScripts: testFiles.length,
    passed,
    failed,
    totalCost,
    totalDuration,
    results
  };
}

async function runDiscoveredVOnChainTests(
  discoveredVTests: any[],
  connection: Connection,
  signerKeypair: Keypair,
  options: any,
  config: any,
  maxCostLamports?: number
): Promise<OnChainTestSummary> {
  const grouped = new Map<string, any[]>();
  for (const test of discoveredVTests) {
    const tests = grouped.get(test.path) || [];
    tests.push(test);
    grouped.set(test.path, tests);
  }

  const results: OnChainTestResult[] = [];
  let totalCost = 0;
  const start = Date.now();

  for (const [sourceFile, tests] of grouped.entries()) {
    const source = await readFile(sourceFile, 'utf8');
    const compilation = await FiveSDK.compile(
      { filename: sourceFile, content: source },
      { debug: options.verbose, optimize: false }
    );
    if (!compilation.success || !compilation.bytecode) {
      for (const test of tests) {
        results.push({
          scriptFile: `${sourceFile}::${test.source.functionName}`,
          passed: false,
          totalDuration: 0,
          totalCost: 0,
          error: `Compilation failed: ${compilation.errors?.join(', ')}`
        });
      }
      continue;
    }

    const fixture = await loadOnChainFixture(sourceFile);

    const deploy = await FiveSDK.deployToSolana(
      compilation.bytecode,
      connection,
      signerKeypair,
      {
        debug: options.verbose || false,
        network: config.target,
        computeBudget: 1_000_000,
        maxRetries: 3
      }
    );
    totalCost += deploy.deploymentCost || 0;
    if (!deploy.success || !deploy.programId) {
      for (const test of tests) {
        results.push({
          scriptFile: `${sourceFile}::${test.source.functionName}`,
          passed: false,
          deployResult: {
            success: false,
            error: deploy.error,
            cost: deploy.deploymentCost || 0
          },
          totalDuration: 0,
          totalCost: deploy.deploymentCost || 0,
          error: `Deployment failed: ${deploy.error || 'unknown error'}`
        });
      }
      continue;
    }

    for (const test of tests) {
      const testStart = Date.now();
      const fixtureSpec = fixture?.tests?.[test.source.functionName];
      const expectedSuccess = fixtureSpec?.expected?.success ?? true;
      const params = fixtureSpec?.parameters ?? test.parameters ?? [];
      const { accounts, error: fixtureError } = await createPerTestFixtureAccounts(
        connection,
        signerKeypair,
        fixture,
        fixtureSpec,
        options.verbose
      );
      if (fixtureError) {
        results.push({
          scriptFile: `${sourceFile}::${test.source.functionName}`,
          passed: false,
          deployResult: {
            success: true,
            scriptAccount: deploy.programId,
            transactionId: deploy.transactionId,
            cost: deploy.deploymentCost || 0
          },
          totalDuration: Date.now() - testStart,
          totalCost: 0,
          error: fixtureError
        });
        continue;
      }

      const execute = await FiveSDK.executeOnSolana(
        deploy.programId,
        connection,
        signerKeypair,
        test.source.functionName,
        params,
        accounts,
        {
          debug: options.verbose || false,
          network: config.target,
          computeUnitLimit: 1_000_000,
          maxRetries: 3,
          abi: compilation.abi
        }
      );

      const testCost = execute.cost || 0;
      totalCost += testCost;
      if (maxCostLamports !== undefined && totalCost > maxCostLamports) {
        throw new Error(
          `On-chain test cost cap exceeded: ${(totalCost / 1e9).toFixed(6)} SOL > ${(maxCostLamports / 1e9).toFixed(6)} SOL`
        );
      }

      const passed = expectedSuccess ? execute.success : !execute.success;
      const errorContains = fixtureSpec?.expected?.errorContains;
      const errorMatches = errorContains
        ? (execute.error || '').includes(errorContains)
        : true;

      results.push({
        scriptFile: `${sourceFile}::${test.source.functionName}`,
        passed: passed && errorMatches,
        deployResult: {
          success: true,
          scriptAccount: deploy.programId,
          transactionId: deploy.transactionId,
          cost: deploy.deploymentCost || 0
        },
        executeResult: {
          success: execute.success || false,
          transactionId: execute.transactionId,
          computeUnitsUsed: execute.computeUnitsUsed,
          cost: execute.cost,
          result: execute.result,
          error: execute.error
        },
        totalDuration: Date.now() - testStart,
        totalCost: testCost,
        error: passed && errorMatches ? undefined : execute.error
      });
    }
  }

  const passed = results.filter((r) => r.passed).length;
  const failed = results.length - passed;
  return {
    totalScripts: results.length,
    passed,
    failed,
    totalCost,
    totalDuration: Date.now() - start,
    results
  };
}

async function loadOnChainFixture(sourceFile: string): Promise<OnChainFixtureFile | undefined> {
  const fixturePath = sourceFile.replace(/\.v$/, '.test.json');
  try {
    const content = await readFile(fixturePath, 'utf8');
    return JSON.parse(content) as OnChainFixtureFile;
  } catch {
    return undefined;
  }
}

async function createPerTestFixtureAccounts(
  connection: Connection,
  signerKeypair: Keypair,
  fixture: OnChainFixtureFile | undefined,
  testSpec: OnChainFixtureTestSpec | undefined,
  verbose: boolean
): Promise<{ accounts: string[]; error?: string }> {
  const required = testSpec?.accounts || [];
  if (required.length === 0) {
    return { accounts: [] };
  }
  if (!fixture?.accounts) {
    return { accounts: [], error: 'Fixture accounts are required but no companion .test.json accounts block was found' };
  }

  const { PublicKey, SystemProgram, Transaction, sendAndConfirmTransaction } = await import('@solana/web3.js');
  const createdAccounts: string[] = [];

  for (const accountName of required) {
    const spec = fixture.accounts[accountName];
    if (!spec) {
      return { accounts: [], error: `Fixture account '${accountName}' not found in companion fixture file` };
    }
    if (spec.is_signer) {
      return {
        accounts: [],
        error: `Fixture account '${accountName}' requests is_signer=true; external signer fixtures are not supported yet`
      };
    }

    const owner = resolveFixtureOwner(spec.owner, signerKeypair.publicKey.toBase58());
    const dataLen = spec.data_len || 0;
    const lamports = spec.lamports ?? (await connection.getMinimumBalanceForRentExemption(dataLen));
    const keypair = Keypair.generate();
    const tx = new Transaction().add(
      SystemProgram.createAccount({
        fromPubkey: signerKeypair.publicKey,
        newAccountPubkey: keypair.publicKey,
        lamports,
        space: dataLen,
        programId: new PublicKey(owner)
      })
    );
    await sendAndConfirmTransaction(connection, tx, [signerKeypair, keypair], { commitment: 'confirmed' });
    createdAccounts.push(keypair.publicKey.toBase58());
    if (verbose) {
      console.log(`[on-chain fixture] created ${accountName}=${keypair.publicKey.toBase58()}`);
    }
  }

  return { accounts: createdAccounts };
}

function resolveFixtureOwner(owner: string | undefined, fallback: string): string {
  if (!owner || owner === 'system') {
    return '11111111111111111111111111111111';
  }
  if (owner === 'payer') {
    return fallback;
  }
  return owner;
}

/**
 * Display comprehensive on-chain test results
 */
function displayOnChainTestResults(
  summary: OnChainTestSummary,
  options: any,
  logger: any
): void {
  console.log('\n' + section('On-Chain Test Results'));

  // Overall statistics
  const successRate = ((summary.passed / summary.totalScripts) * 100).toFixed(1);
  const avgCostPerScript = summary.totalCost / summary.totalScripts;
  const totalCostSOL = summary.totalCost / 1e9;

  console.log(`Passed: ${summary.passed}/${summary.totalScripts} (${successRate}%)`);
  console.log(`Total duration: ${summary.totalDuration}ms`);
  console.log(`Total cost: ${totalCostSOL.toFixed(6)} SOL`);
  console.log(`Average cost per script: ${(avgCostPerScript / 1e9).toFixed(6)} SOL`);

  if (options.analyzeCosts) {
    console.log('\n' + section('Cost Analysis'));

    let deploymentCost = 0;
    let executionCost = 0;

    for (const result of summary.results) {
      if (result.deployResult?.cost) {
        deploymentCost += result.deployResult.cost;
      }
      if (result.executeResult?.cost) {
        executionCost += result.executeResult.cost;
      }
    }

    console.log(`Total deployment cost: ${(deploymentCost / 1e9).toFixed(6)} SOL`);
    console.log(`Total execution cost: ${(executionCost / 1e9).toFixed(6)} SOL`);
    console.log(`Deployment vs Execution: ${((deploymentCost / summary.totalCost) * 100).toFixed(1)}% : ${((executionCost / summary.totalCost) * 100).toFixed(1)}%`);
  }

  // Failed tests details
  if (summary.failed > 0) {
    console.log('\n' + section('Failed Tests'));

    const failedResults = summary.results.filter(r => !r.passed);
    for (const result of failedResults) {
      const scriptName = basename(result.scriptFile, '.bin');
      console.log(`  FAIL ${scriptName}:`);

      if (result.error) {
        console.log(`     Error: ${result.error}`);
      }

      if (result.deployResult && !result.deployResult.success) {
        console.log(`     Deployment: Failed - ${result.deployResult.error}`);
      }

      if (result.executeResult && !result.executeResult.success) {
        console.log(`     Execution: Failed - ${result.executeResult.error}`);
      }

      console.log(`     Duration: ${result.totalDuration}ms`);
      console.log(`     Cost: ${(result.totalCost / 1e9).toFixed(6)} SOL\n`);
    }
  }

  // Successful tests (if verbose)
  if (options.verbose && summary.passed > 0) {
    console.log('\n' + section('Successful Tests'));

    const passedResults = summary.results.filter(r => r.passed);
    for (const result of passedResults) {
      const scriptName = basename(result.scriptFile, '.bin');
      const deployTx = result.deployResult?.transactionId?.substring(0, 8) || 'N/A';
      const executeTx = result.executeResult?.transactionId?.substring(0, 8) || 'N/A';
      const computeUnits = result.executeResult?.computeUnitsUsed || 0;

      console.log(`  OK ${scriptName}:`);
      console.log(`     Deploy: ${deployTx}... | Execute: ${executeTx}...`);
      console.log(`     Compute Units: ${computeUnits.toLocaleString()}`);
      console.log(`     Duration: ${result.totalDuration}ms | Cost: ${(result.totalCost / 1e9).toFixed(6)} SOL\n`);
    }
  }

  // Summary message
  if (summary.failed === 0) {
    console.log(uiSuccess('All on-chain tests passed'));
  } else {
    console.log(uiError(`${summary.failed} test(s) failed. Check logs above for details.`));
  }
}
