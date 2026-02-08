// Local WASM execution commands.

import { readFile, readdir, stat } from 'fs/promises';
import { join, extname, basename } from 'path';
import ora from 'ora';

import {
  CommandDefinition,
  CommandContext,
  VMExecutionOptions,
  CLIOptions
} from '../types.js';
import { FiveSDK } from 'five-sdk';
import { section, success as uiSuccess, error as uiError } from '../utils/cli-ui.js';

// Parent for local subcommands.
export const localCommand: CommandDefinition = {
  name: 'local',
  description: 'Local WASM execution',
  aliases: ['l'],

  options: [
    {
      flags: '-f, --function <call>',
      description: 'Function call: add(5, 3) or function name/index'
    },
    {
      flags: '-p, --params <values...>',
      description: 'Function parameters (space-separated or JSON)'
    },
    {
      flags: '--debug',
      description: 'Enable debug output'
    },
    {
      flags: '--trace',
      description: 'Enable execution trace'
    },
    {
      flags: '--max-cu <units>',
      description: 'Max compute units (default: 1000000)'
    },
    {
      flags: '--max-compute-units <units>',
      description: 'Alias for --max-cu'
    },
    {
      flags: '--format <format>',
      description: 'Output format: text, json (default: text)'
    }
  ],

  arguments: [
    {
      name: 'subcommand',
      description: 'Local subcommand (execute, test, compile)',
      required: true
    },
    {
      name: 'args',
      description: 'Arguments for the subcommand',
      required: false,
      variadic: true
    }
  ],

  examples: [
    {
      command: 'five local execute script.bin',
      description: 'Execute script locally using WASM VM'
    },
    {
      command: 'five local execute script.five add 5 3',
      description: 'Execute add function with parameters 5 and 3'
    },
    {
      command: 'five local execute script.five test',
      description: 'Execute test function with no parameters'
    },
    {
      command: 'five local execute script.five 2',
      description: 'Execute function at index 2'
    },
    {
      command: 'five local test',
      description: 'Run local test suite with WASM VM'
    }
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;

    const flatArgs: string[] = args ?? [];

    if (flatArgs.length === 0) {
      showLocalHelp(logger);
      return;
    }

    const [subcommand, ...rawSubArgs] = flatArgs as (string | string[] | undefined)[];
    const subArgs = rawSubArgs.flatMap((arg) => {
      if (Array.isArray(arg)) {
        return arg.filter((value): value is string => typeof value === 'string');
      }
      return typeof arg === 'string' ? [arg] : [];
    });

    // Local WASM execution mode
    if (context.options.verbose) {
      logger.info('Local WASM VM');
    }

    try {
      switch (subcommand) {
        case 'execute':
        case 'exec':
        case 'run':
          await executeLocalSubcommand(subArgs, options, context);
          break;

        case 'test':
        case 't':
          await testLocalSubcommand(subArgs, options, context);
          break;

        case 'compile':
        case 'c':
          await compileLocalSubcommand(subArgs, options, context);
          break;

        default:
          logger.error(`Unknown local subcommand: ${subcommand}`);
          showLocalHelp(logger);
          process.exit(1);
      }
    } catch (error) {
      logger.error(`Local ${subcommand} failed:`, error);
      throw error;
    }
  }
};

/**
 * Local execute subcommand - execute bytecode in local WASM VM
 */
async function executeLocalSubcommand(args: string[], options: any, context: CommandContext): Promise<void> {
  const { logger } = context;

  if (options.debug) {
    logger.debug(`local execute args: ${JSON.stringify(args)}`);
  }

  if (args.length === 0) {
    console.log(uiError('No file specified for execution'));
    console.log(section('Usage'));
    console.log('  five local execute <file> [function] [params...]');
    console.log(section('Arguments'));
    console.log('  <file>       .five/.bin/.v file to execute');
    console.log('  [function]   Function name or index');
    console.log('  [params...]  Function parameters (space-separated)');
    console.log(section('Options'));
    console.log('  --debug      Enable debug output');
    console.log('  --trace      Enable execution trace');
    console.log('  --max-cu <n> Max compute units (default: 1000000)');
    console.log('  --format <f> Output format: text, json (default: text)');
    console.log(section('Examples'));
    console.log('  five local execute script.five add 5 3');
    console.log('  five local execute script.five test');
    console.log('  five local execute script.five 2');
    return;
  }

  const inputFile = args[0];
  const rawFunctionName = args.length > 1 ? args[1] : undefined;

  // Convert numeric strings to numbers for function index
  let functionName: string | number | undefined = rawFunctionName;
  if (rawFunctionName && /^\d+$/.test(rawFunctionName)) {
    functionName = parseInt(rawFunctionName, 10);
  }

  const functionParams = args.length > 2 ? args.slice(2) : [];

  if (options.debug) {
    logger.debug(`local execute parsed: ${JSON.stringify({ inputFile, functionName, functionParams })}`);
  }

  // Initialize for local execution
  if (context.options.verbose) {
    const spinner = ora('Preparing local execution...').start();
    spinner.text = 'Loading Five SDK...';
    spinner.succeed('Five SDK ready for local execution');
  }

  try {
    // Use space-separated syntax: file function param1 param2 ...
    let functionRef = functionName || options.function || options.functionIndex || 0;
    const parameters = functionParams.length > 0 ?
      (Array.isArray(functionParams) ? functionParams.map(parseValue) : [parseValue(functionParams)]) :
      parseParameters(options.params);

    const debug = Boolean(options.debug || options.trace);
    const trace = Boolean(options.trace);
    const maxCU = options.maxCu || options.maxComputeUnits || 1000000;
    const format = options.format || 'text';

    // Keep function reference as-is - let the compiler handle function name resolution
    // The Five compiler can resolve function names to indices properly

    // Debug parameter parsing
    if (debug) {
      logger.debug(`local function: ${functionRef}`);
      logger.debug(`local params: ${JSON.stringify(parameters)}`);
    }

    let result;

    if (extname(inputFile) === '.five') {
      // Handle .five files with embedded ABI
      logger.info(`Executing Five file: ${inputFile}`);

      const fileContent = await readFile(inputFile, 'utf8');
      const { bytecode, abi } = await FiveSDK.loadFiveFile(fileContent);

      // FUNCTION RESOLUTION FIX: Systematic approach for .five files
      let resolvedFunctionRef = functionRef;
      if (functionRef === undefined) {
        // SYSTEMATIC APPROACH: Public functions always have the lowest indices (0, 1, 2...)
        // So we always default to function index 0, which is guaranteed to be public if any functions exist
        resolvedFunctionRef = 0;

        if (debug && abi && abi.functions) {
          let functionName = 'function_0';

          if (Array.isArray(abi.functions)) {
            // FIVEABI format: find function with index 0
            const func0 = abi.functions.find((f: any) => f.index === 0);
            if (func0 && func0.name) functionName = func0.name;
          } else {
            // SimpleABI format: find function with index 0
            const func0Entry = Object.entries(abi.functions).find(([_, f]: [string, any]) => f.index === 0);
            if (func0Entry) functionName = func0Entry[0];
          }

          logger.debug(`auto-detected public function: ${functionName} (index 0)`);
        }
      }

      result = await FiveSDK.executeLocally(
        bytecode,
        resolvedFunctionRef,
        parameters,
        {
          debug,
          trace,
          computeUnitLimit: maxCU,
          abi // Pass ABI for function name resolution
        }
      );

    } else if (extname(inputFile) === '.v') {
      // Compile and execute Five source file
      logger.info(`Compiling and executing Five source: ${inputFile}`);

      const sourceCode = await readFile(inputFile, 'utf8');

      result = await FiveSDK.execute(
        sourceCode,
        functionRef,
        parameters,
        {
          debug,
          trace,
          optimize: true,
          computeUnitLimit: maxCU
        }
      );

      // Show compilation info
      if ('compilation' in result && result.compilation) {
        console.log(`Compilation: ${result.compilation.success ? 'OK' : 'FAIL'}`);
        if ('bytecodeSize' in result && result.bytecodeSize) {
          console.log(`Bytecode size: ${result.bytecodeSize} bytes`);
        }
      }

    } else {
      // Execute existing bytecode file
      logger.info(`Executing bytecode: ${inputFile}`);

      const bytecode = await readFile(inputFile);

      result = await FiveSDK.executeLocally(
        new Uint8Array(bytecode),
        functionRef,
        parameters,
        {
          debug,
          trace,
          computeUnitLimit: maxCU
        }
      );
    }

    // Validate execution result structure
    if (typeof result !== 'object' || result === null) {
      logger.error('Invalid execution result from SDK');
      process.exit(1);
    }

    if (typeof result.success !== 'boolean') {
      logger.error('Execution result missing success field');
      process.exit(1);
    }

    // Display results
    displayLocalExecutionResult(result, { format, trace, debug }, logger);

    // CRITICAL FIX: Check execution success and exit with proper code
    if (!result.success) {
      if (context.options.verbose) {
        logger.error(`Execution failed: ${result.error || 'Unknown error'}`);
      }
      process.exit(1); // EXIT WITH FAILURE CODE
    }

  } catch (error) {
    if (context.options.verbose) {
      ora().fail('Local execution failed');
    }
    process.exit(1); // Also ensure exceptions exit with failure
  }
}

/**
 * Local test subcommand - run test suite locally
 */
async function testLocalSubcommand(args: string[], options: any, context: CommandContext): Promise<void> {
  const { logger } = context;

  const testPath = args[0] || './tests';
  const pattern = options.pattern || '**/*.test.json';
  const filter = options.filter;
  const verbose = options.verbose || options.debug;
  const format = options.format || 'text';

  logger.info('Running local test suite with WASM VM');
  logger.info(`Test path: ${testPath}`);
  logger.info(`Pattern: ${pattern}`);

  const spinner = ora('Discovering test files...').start();

  try {
    // Discover test files
    const testFiles = await discoverTestFiles(testPath, pattern);

    if (testFiles.length === 0) {
      spinner.warn('No test files found');
      logger.warn(`No test files matching pattern "${pattern}" found in ${testPath}`);
      logger.info('Expected test file format: JSON files with .test.json extension');
      return;
    }

    spinner.succeed(`Found ${testFiles.length} test file(s)`);

    // Run tests
    const results = await runLocalTests(testFiles, {
      filter,
      verbose,
      maxCU: options.maxCu || 1000000,
      timeout: options.timeout || 30000
    }, logger);

    // Display results
    displayTestResults(results, { format, verbose }, logger);

    // Exit with error if any tests failed
    const failed = results.some(r => !r.passed);
    if (failed) {
      process.exit(1);
    }

  } catch (error) {
    spinner.fail('Test discovery failed');
    throw error;
  }
}

/**
 * Local compile subcommand - alias for main compile command
 */
async function compileLocalSubcommand(args: string[], options: any, context: CommandContext): Promise<void> {
  const { logger } = context;

  if (args.length === 0) {
    logger.error('No file specified for compilation');
    logger.info('Usage: five local compile <file> [options]');
    logger.info('This is an alias for: five compile <file>');
    return;
  }

  logger.info('Compiling locally (same as five compile)');

  // Import and delegate to compile command
  const { compileCommand } = await import('./compile.js');
  await compileCommand.handler(args, options, context);
}

/**
 * Show local command help
 */
function showLocalHelp(logger: any): void {
  logger.info('Local Five VM Commands (WASM execution, no config needed):');
  logger.info('');
  logger.info('Subcommands:');
  logger.info('  execute <file>     Execute bytecode or source file locally');
  logger.info('  test [pattern]     Run local test suite');
  logger.info('  compile <file>     Compile source file (alias for main compile)');
  logger.info('');
  logger.info('Examples:');
  logger.info('  five local execute script.bin');
  logger.info('  five local execute script.v --function 1 --parameters "[10, 5]"');
  logger.info('  five local test --pattern "*.test.json" --verbose');
  logger.info('  five local compile script.v --optimize');
  logger.info('');
  logger.info('Features:');
  logger.info('  • Always uses WASM VM (no network required)');
  logger.info('  • No configuration dependencies');
  logger.info('  • Fast development iteration');
  logger.info('  • Compute unit tracking');
  logger.info('  • Debug and trace support');
}

/**
 * Parse function call syntax: add(5, 3) -> { name: 'add', params: [5, 3] }
 */
function parseFunctionCall(functionOption: string | number): { functionRef: string | number, parameters: any[] } {
  if (typeof functionOption === 'number') {
    return { functionRef: functionOption, parameters: [] };
  }

  const functionStr = functionOption.toString().trim();

  // Check if it's a function call syntax: name(params...)
  const callMatch = functionStr.match(/^([a-zA-Z_][a-zA-Z0-9_]*)\s*\(\s*(.*?)\s*\)$/);
  if (callMatch) {
    const [, functionName, paramsStr] = callMatch;
    let parameters: any[] = [];

    if (paramsStr.trim()) {
      // Parse comma-separated parameters
      try {
        // Split by comma and parse each parameter
        const paramStrs = paramsStr.split(',').map(p => p.trim());
        parameters = paramStrs.map(parseValue);
      } catch (error) {
        throw new Error(`Failed to parse function parameters in "${functionStr}": ${error}`);
      }
    }

    return { functionRef: functionName, parameters };
  }

  // Check if it's a plain number (function index)
  const numericValue = parseInt(functionStr, 10);
  if (!isNaN(numericValue) && numericValue.toString() === functionStr) {
    return { functionRef: numericValue, parameters: [] };
  }

  // Plain function name
  return { functionRef: functionStr, parameters: [] };
}

/**
 * Parse parameters from various formats: space-separated, JSON string, or array
 */
function parseParameters(paramsOption: any): any[] {
  if (!paramsOption) {
    return [];
  }

  try {
    // Handle arrays directly (from --params a b c)
    if (Array.isArray(paramsOption)) {
      return paramsOption.map(parseValue);
    }

    // Handle string input
    if (typeof paramsOption === 'string') {
      // JSON array format
      if (paramsOption.startsWith('[') || paramsOption.startsWith('{')) {
        return JSON.parse(paramsOption);
      }
      // Single parameter
      return [parseValue(paramsOption)];
    }

    // Single value
    return [parseValue(paramsOption)];
  } catch (error) {
    throw new Error(`Failed to parse parameters: ${error}. Use space-separated values like: -p 10 5, or JSON format: --parameters "[10, 5]"`);
  }
}

/**
 * Parse a single parameter value, converting strings to appropriate types
 */
function parseValue(value: string): any {
  if (typeof value !== 'string') {
    return value;
  }

  // Try to parse as number
  const numValue = Number(value);
  if (!isNaN(numValue) && isFinite(numValue)) {
    return numValue;
  }

  // Try to parse as boolean
  if (value.toLowerCase() === 'true') return true;
  if (value.toLowerCase() === 'false') return false;

  // Return as string
  return value;
}

/**
 * Display local execution results with clear formatting
 */
function displayLocalExecutionResult(result: any, options: any, logger: any): void {
  // Only show header in verbose mode
  if (options.verbose || options.debug) {
    console.log('\n' + section('Local Execution'));
  }

  if (result.success) {
    console.log(uiSuccess('Execution succeeded'));

    if (result.result !== undefined) {
      console.log(`  Result: ${JSON.stringify(result.result)}`);
    }

    if (result.executionTime) {
      console.log(`  Time: ${result.executionTime}ms`);
    }

    if (result.computeUnitsUsed !== undefined) {
      console.log(`  Compute units: ${result.computeUnitsUsed.toLocaleString()}`);
    }

    if (result.logs && result.logs.length > 0) {
      console.log('\n' + section('Logs'));
      result.logs.forEach((log: string) => {
        console.log(`  ${log}`);
      });
    }

    if (result.trace && options.trace) {
      console.log('\n' + section('Trace'));
      if (Array.isArray(result.trace)) {
        result.trace.slice(0, 10).forEach((step: any, i: number) => {
          console.log(`  ${i}: ${JSON.stringify(step)}`);
        });
        if (result.trace.length > 10) {
          console.log(`  ... and ${result.trace.length - 10} more steps`);
        }
      }
    }

  } else {
    console.log(uiError(result.error || 'Execution failed'));

    // Enhanced VM debugging information for runtime errors
    if (result.vmState || result.debug) {
      const vmInfo = result.vmState || result.debug || result;
      console.log('\n' + section('VM Debug'));

      if (vmInfo.instructionPointer !== undefined) {
        console.log(`    Instruction Pointer: 0x${vmInfo.instructionPointer.toString(16).toUpperCase().padStart(4, '0')}`);
      }

      if (vmInfo.stoppedAtOpcode !== undefined) {
        console.log(`    Stopped at Opcode: 0x${vmInfo.stoppedAtOpcode.toString(16).toUpperCase().padStart(2, '0')} (${vmInfo.stoppedAtOpcode})`);
      }

      if (vmInfo.stoppedAtOpcodeName) {
        console.log(`    Opcode Name: ${vmInfo.stoppedAtOpcodeName}`);
      }

      if (vmInfo.errorMessage) {
        console.log(`    VM Error: ${vmInfo.errorMessage}`);
      }

      if (vmInfo.finalStack && Array.isArray(vmInfo.finalStack)) {
        console.log(`    Final Stack (${vmInfo.finalStack.length} items):`);
        vmInfo.finalStack.slice(-5).forEach((item: any, i: number) => {
          const index = vmInfo.finalStack.length - 5 + i;
          console.log(`      [${index}]: ${JSON.stringify(item)}`);
        });
        if (vmInfo.finalStack.length > 5) {
          console.log(`      ... (${vmInfo.finalStack.length - 5} more items)`);
        }
      }

      if (vmInfo.executionContext) {
        console.log(`    Execution Context: ${vmInfo.executionContext}`);
      }
    }

    if (result.compilationErrors && result.compilationErrors.length > 0) {
      console.log('\n  Compilation errors:');
      result.compilationErrors.forEach((error: any) => {
        console.log(`    - ${error.message || error}`);
      });
    }
  }

  // JSON format output
  if (options.format === 'json') {
    console.log('\n' + section('JSON Output'));
    console.log(JSON.stringify(result, null, 2));
  }
}

/**
 * Discover test files based on pattern
 */
async function discoverTestFiles(testPath: string, pattern: string): Promise<string[]> {
  const testFiles: string[] = [];

  try {
    const stats = await stat(testPath);

    if (stats.isFile()) {
      // Single test file
      if (testPath.endsWith('.test.json')) {
        testFiles.push(testPath);
      }
    } else if (stats.isDirectory()) {
      // Directory - recursively find test files
      const files = await readdir(testPath, { recursive: true });

      for (const file of files) {
        if (typeof file === 'string' && file.endsWith('.test.json')) {
          testFiles.push(join(testPath, file));
        }
      }
    }
  } catch (error) {
    // Directory/file doesn't exist - that's ok
  }

  return testFiles;
}

/**
 * Run tests locally using WASM VM
 */
async function runLocalTests(
  testFiles: string[],
  options: {
    filter?: string;
    verbose: boolean;
    maxCU: number;
    timeout: number;
  },
  logger: any
): Promise<Array<{ name: string; passed: boolean; duration: number; error?: string; details?: any }>> {
  const results: Array<{ name: string; passed: boolean; duration: number; error?: string; details?: any }> = [];

  for (const testFile of testFiles) {
    logger.info(`Running tests from: ${testFile}`);

    try {
      const content = await readFile(testFile, 'utf8');
      const testSuite = JSON.parse(content);

      const suiteName = testSuite.name || basename(testFile, '.test.json');
      const testCases = testSuite.tests || testSuite.testCases || [];

      // Apply filter if specified
      let filteredTests = testCases;
      if (options.filter) {
        filteredTests = testCases.filter((test: any) =>
          test.name?.includes(options.filter) || suiteName.includes(options.filter!)
        );
      }

      for (const testCase of filteredTests) {
        const result = await runSingleLocalTest(testCase, options, logger);
        results.push(result);

        if (options.verbose) {
          displaySingleTestResult(result, logger);
        }
      }

    } catch (error) {
      results.push({
        name: `Load ${basename(testFile)}`,
        passed: false,
        duration: 0,
        error: `Failed to load test file: ${error}`
      });
    }
  }

  return results;
}

/**
 * Run a single test case locally
 */
async function runSingleLocalTest(
  testCase: any,
  options: { maxCU: number; timeout: number; verbose: boolean },
  logger: any
): Promise<{ name: string; passed: boolean; duration: number; error?: string; details?: any }> {
  const startTime = Date.now();

  try {
    // Load bytecode
    const bytecode = await readFile(testCase.bytecode);

    // Parse input parameters
    let parameters: any[] = [];
    if (testCase.input) {
      try {
        const inputData = await readFile(testCase.input, 'utf8');
        parameters = JSON.parse(inputData);
      } catch {
        parameters = [];
      }
    }

    // Execute with timeout
    const executionPromise = FiveSDK.executeLocally(
      new Uint8Array(bytecode),
      testCase.functionIndex || 0,
      parameters,
      {
        debug: options.verbose,
        trace: options.verbose,
        computeUnitLimit: options.maxCU
      }
    );

    const timeoutPromise = new Promise((_, reject) =>
      setTimeout(() => reject(new Error('Test timeout')), options.timeout)
    );

    const result = await Promise.race([executionPromise, timeoutPromise]) as any;
    const duration = Date.now() - startTime;

    // Validate result
    const passed = validateTestResult(result, testCase.expected || {});

    return {
      name: testCase.name,
      passed,
      duration,
      details: options.verbose ? result : undefined
    };

  } catch (error) {
    const duration = Date.now() - startTime;

    // Check if error was expected
    const passed = testCase.expected?.success === false &&
      testCase.expected?.error !== undefined &&
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
function displaySingleTestResult(result: any, logger: any): void {
  const status = result.passed ? 'OK' : 'FAIL';
  const duration = `(${result.duration}ms)`;

  console.log(`  ${status} ${result.name} ${duration}`);

  if (!result.passed && result.error) {
    console.log(`    Error: ${result.error}`);
  }
}

/**
 * Display comprehensive test results
 */
function displayTestResults(
  results: Array<{ name: string; passed: boolean; duration: number; error?: string }>,
  options: { format: string; verbose: boolean },
  logger: any
): void {
  if (options.format === 'json') {
    console.log(JSON.stringify(results, null, 2));
    return;
  }

  console.log('\n' + section('Local Tests'));

  const passed = results.filter(r => r.passed).length;
  const total = results.length;
  const totalDuration = results.reduce((sum, r) => sum + r.duration, 0);

  // Summary
  console.log(`\nSummary:`);
  console.log(`  Total: ${total} tests`);
  console.log(`  Passed: ${passed}`);

  const failed = total - passed;
  if (failed > 0) {
    console.log(`  Failed: ${failed}`);

    // Show failed tests
    if (!options.verbose) {
      console.log('\nFailed tests:');
      results.filter(r => !r.passed).forEach(result => {
        console.log(`  FAIL ${result.name}: ${result.error || 'Test failed'}`);
      });
    }
  }

  console.log(`  Duration: ${totalDuration}ms`);

  if (failed === 0) {
    console.log(uiSuccess('All tests passed'));
  } else {
    console.log(uiError(`${failed} test(s) failed`));
  }
}
