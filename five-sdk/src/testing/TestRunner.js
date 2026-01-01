/**
 * Five SDK Test Runner
 *
 * SDK-based test utilities that replace shell script approaches with programmatic
 * Five SDK usage. Provides comprehensive testing capabilities for Five VM scripts.
 */
import { readFile, readdir, stat } from 'fs/promises';
import { join, extname, basename } from 'path';
import { FiveSDK } from '../FiveSDK.js';
import { FiveBytecode, FiveScriptSource } from '../types.js';
/**
 * Five SDK-based test runner
 */
export class FiveTestRunner {
    options;
    constructor(options = {}) {
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
    async runTestCase(testCase) {
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
            let bytecode;
            // Get bytecode (compile source or load existing)
            if (testCase.source) {
                const result = await this.compileToBytecode(testCase.source);
                if (!result.success || !result.bytecode) {
                    throw new Error(`Compilation failed: ${result.errors?.join(', ')}`);
                }
                bytecode = result.bytecode;
            }
            else if (testCase.bytecode) {
                const data = await readFile(testCase.bytecode);
                bytecode = new Uint8Array(data);
            }
            else {
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
            const executionPromise = FiveSDK.executeLocally(bytecode, testCase.function || 0, testCase.parameters || [], {
                debug: this.options.debug,
                trace: this.options.trace,
                computeUnitLimit: this.options.maxComputeUnits
            });
            const timeoutPromise = new Promise((_, reject) => setTimeout(() => reject(new Error('Test timeout')), executionTimeout));
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
        }
        catch (error) {
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
    async runTestSuite(suite) {
        const startTime = Date.now();
        const results = [];
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
                    const chunkResults = await Promise.all(chunk.map(testCase => this.runTestCase(testCase)));
                    results.push(...chunkResults);
                    // Fail fast check
                    if (this.options.failFast && chunkResults.some(r => !r.passed)) {
                        break;
                    }
                }
            }
            else {
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
        }
        catch (error) {
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
    async runTestSuites(suites) {
        const results = [];
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
    async discoverTestSuites(directory, pattern = '**/*.test.json') {
        const testFiles = await this.findTestFiles(directory, pattern);
        const suites = [];
        for (const file of testFiles) {
            try {
                const content = await readFile(file, 'utf8');
                const data = JSON.parse(content);
                const suite = {
                    name: data.name || basename(file, '.test.json'),
                    description: data.description,
                    testCases: data.tests || data.testCases || []
                };
                suites.push(suite);
            }
            catch (error) {
                console.warn(`Failed to load test suite ${file}: ${error}`);
            }
        }
        return suites;
    }
    /**
     * Compile and execute test - convenience method
     */
    static async compileAndTest(source, functionName, parameters = [], options = {}) {
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
            const execution = await FiveSDK.executeLocally(compilation.bytecode, functionName || 0, parameters, options);
            return {
                compilation,
                execution,
                success: execution.success || false,
                error: execution.error
            };
        }
        catch (error) {
            return {
                compilation: null,
                execution: null,
                success: false,
                error: error instanceof Error ? error.message : 'Unknown error'
            };
        }
    }
    // Private helper methods
    async compileToBytecode(sourcePath) {
        const source = await readFile(sourcePath, 'utf8');
        return FiveSDK.compile(source, { debug: this.options.debug });
    }
    validateResult(result, expected) {
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
    getValidationError(result, expected) {
        const errors = [];
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
    matchesPattern(name, pattern) {
        if (pattern === '*')
            return true;
        const regex = new RegExp(pattern.replace(/\*/g, '.*').replace(/\?/g, '.'), 'i');
        return regex.test(name);
    }
    chunkArray(array, chunkSize) {
        const chunks = [];
        for (let i = 0; i < array.length; i += chunkSize) {
            chunks.push(array.slice(i, i + chunkSize));
        }
        return chunks;
    }
    async findTestFiles(directory, pattern) {
        const files = [];
        try {
            const entries = await readdir(directory, { withFileTypes: true });
            for (const entry of entries) {
                const fullPath = join(directory, entry.name);
                if (entry.isDirectory()) {
                    const subFiles = await this.findTestFiles(fullPath, pattern);
                    files.push(...subFiles);
                }
                else if (entry.isFile() && entry.name.endsWith('.test.json')) {
                    files.push(fullPath);
                }
            }
        }
        catch (error) {
            // Directory doesn't exist or not accessible
        }
        return files;
    }
    formatTestResult(result) {
        const icon = result.passed ? '✅' : '❌';
        const duration = `${result.duration}ms`;
        const cu = result.computeUnitsUsed ? `${result.computeUnitsUsed} CU` : '';
        let line = `${icon} ${result.name} (${duration}`;
        if (cu)
            line += `, ${cu}`;
        line += ')';
        if (!result.passed && result.error) {
            line += `\n   Error: ${result.error}`;
        }
        return line;
    }
    formatSuiteResult(result) {
        const { passed, failed, skipped } = result;
        const total = passed + failed + skipped;
        const duration = `${result.duration}ms`;
        let line = `📊 Results: ${passed}/${total} passed`;
        if (failed > 0)
            line += `, ${failed} failed`;
        if (skipped > 0)
            line += `, ${skipped} skipped`;
        line += ` (${duration})`;
        return line;
    }
}
//# sourceMappingURL=TestRunner.js.map