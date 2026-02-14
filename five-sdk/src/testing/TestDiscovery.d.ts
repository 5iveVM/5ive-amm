/**
 * Five CLI Test Discovery
 *
 * Enhanced test discovery for Five VM scripts supporting both .v source files
 * and compiled .test.json test suites. Automatically detects test functions,
 * extracts parameters from @test-params comments, and compiles source files.
 */
/**
 * Discovered test source from .v file
 */
export interface VSourceTest {
    type: 'v-source';
    file: string;
    functionName: string;
    parameters?: any[];
    expectedResult?: any;
    expectsResult?: boolean;
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
    expectedResult?: any;
    expectsResult?: boolean;
}
/**
 * Discover tests from directory
 */
export declare class TestDiscovery {
    /**
     * Discover all tests in a directory
     */
    static discoverTests(testDir: string, options?: {
        pattern?: string;
        verbose?: boolean;
    }): Promise<DiscoveredTest[]>;
    /**
     * Discover .v source test files
     */
    private static discoverVTests;
    /**
     * Discover .test.json test suites
     */
    private static discoverJsonTests;
    /**
     * Discover from single file
     */
    private static discoverFromFile;
    /**
     * Parse .v source file for test functions and parameters
     */
    private static parseVFile;
    /**
     * Compile a .v test source file
     */
    static compileVTest(file: string): Promise<{
        success: boolean;
        bytecode?: Uint8Array;
        errors?: string[];
    }>;
    /**
     * Find files recursively matching predicate
     */
    private static findFilesRecursive;
}
//# sourceMappingURL=TestDiscovery.d.ts.map
