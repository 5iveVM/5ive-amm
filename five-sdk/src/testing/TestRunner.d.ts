/**
 * Five SDK Test Runner
 *
 * SDK-based test utilities that replace shell script approaches with programmatic
 * Five SDK usage. Provides comprehensive testing capabilities for Five VM scripts.
 */
import { FiveScriptSource } from '../types.js';
/**
 * Test case definition
 */
export interface TestCase {
    name: string;
    description?: string;
    bytecode?: string;
    source?: string;
    function?: string | number;
    parameters?: any[];
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
    timeout?: number;
    maxComputeUnits?: number;
    parallel?: number;
    verbose?: boolean;
    debug?: boolean;
    trace?: boolean;
    pattern?: string;
    failFast?: boolean;
}
/**
 * Five SDK-based test runner
 */
export declare class FiveTestRunner {
    private options;
    constructor(options?: TestRunnerOptions);
    /**
     * Run a single test case
     */
    runTestCase(testCase: TestCase): Promise<TestResult>;
    /**
     * Run a complete test suite
     */
    runTestSuite(suite: TestSuite): Promise<TestSuiteResult>;
    /**
     * Run multiple test suites
     */
    runTestSuites(suites: TestSuite[]): Promise<TestSuiteResult[]>;
    /**
     * Discover and load test suites from directory
     */
    discoverTestSuites(directory: string, pattern?: string): Promise<TestSuite[]>;
    /**
     * Compile and execute test - convenience method
     */
    static compileAndTest(source: FiveScriptSource, functionName?: string | number, parameters?: any[], options?: {
        debug?: boolean;
        trace?: boolean;
        computeUnitLimit?: number;
    }): Promise<{
        compilation: any;
        execution: any;
        success: boolean;
        error?: string;
    }>;
    private compileToBytecode;
    private validateResult;
    private getValidationError;
    private matchesPattern;
    private chunkArray;
    private findTestFiles;
    private formatTestResult;
    private formatSuiteResult;
}
//# sourceMappingURL=TestRunner.d.ts.map