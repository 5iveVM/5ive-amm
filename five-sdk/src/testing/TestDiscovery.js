/**
 * Five CLI Test Discovery
 *
 * Enhanced test discovery for Five VM scripts supporting both .v source files
 * and compiled .test.json test suites. Automatically detects test functions,
 * extracts parameters from @test-params comments, and compiles source files.
 */
import { readFile, readdir, stat } from 'fs/promises';
import { basename, join } from 'path';
import { FiveSDK } from '../FiveSDK.js';
/**
 * Discover tests from directory
 */
export class TestDiscovery {
    static normalizeJsonTestCases(data) {
        const testCases = data.tests || data.testCases || [];
        if (Array.isArray(testCases)) {
            return testCases;
        }
        if (testCases && typeof testCases === 'object') {
            return null;
        }
        return [];
    }
    /**
     * Discover all tests in a directory
     */
    static async discoverTests(testDir, options = {}) {
        const tests = [];
        try {
            const stats = await stat(testDir);
            if (stats.isFile()) {
                // Single test file
                const tests = await this.discoverFromFile(testDir);
                return tests;
            }
            else if (stats.isDirectory()) {
                // Discover all test files
                const vSourceTests = await this.discoverVTests(testDir);
                const jsonTests = await this.discoverJsonTests(testDir);
                tests.push(...vSourceTests, ...jsonTests);
            }
        }
        catch (error) {
            if (options.verbose) {
                console.warn(`Failed to discover tests in ${testDir}: ${error}`);
            }
        }
        return tests;
    }
    /**
     * Discover .v source test files
     */
    static async discoverVTests(testDir) {
        const tests = [];
        try {
            const files = await this.findFilesRecursive(testDir, (f) => f.endsWith('.v'));
            for (const file of files) {
                const sourceTests = await this.parseVFile(file);
                tests.push(...sourceTests);
            }
        }
        catch (error) {
            // Silently skip if directory doesn't exist
        }
        return tests;
    }
    /**
     * Discover .test.json test suites
     */
    static async discoverJsonTests(testDir) {
        const tests = [];
        try {
            const files = await this.findFilesRecursive(testDir, (f) => f.endsWith('.test.json'));
            for (const file of files) {
                try {
                    const content = await readFile(file, 'utf8');
                    const data = JSON.parse(content);
                    const testCases = this.normalizeJsonTestCases(data);
                    if (testCases === null) {
                        continue;
                    }
                    for (const testCase of testCases) {
                        tests.push({
                            name: testCase.name,
                            path: file,
                            type: 'json-suite',
                            description: testCase.description,
                            parameters: testCase.parameters
                        });
                    }
                }
                catch (error) {
                    console.warn(`Failed to parse test file ${file}: ${error}`);
                }
            }
        }
        catch (error) {
            // Silently skip if directory doesn't exist
        }
        return tests;
    }
    /**
     * Discover from single file
     */
    static async discoverFromFile(file) {
        if (file.endsWith('.v')) {
            return this.parseVFile(file);
        }
        else if (file.endsWith('.test.json')) {
            try {
                const content = await readFile(file, 'utf8');
                const data = JSON.parse(content);
                const testCases = this.normalizeJsonTestCases(data);
                if (testCases === null) {
                    return [];
                }
                return testCases.map((testCase) => ({
                    name: testCase.name,
                    path: file,
                    type: 'json-suite',
                    description: testCase.description,
                    parameters: testCase.parameters
                }));
            }
            catch (error) {
                console.warn(`Failed to parse test file ${file}: ${error}`);
                return [];
            }
        }
        return [];
    }
    /**
     * Parse .v source file for test functions and parameters
     */
    static async parseVFile(file) {
        const tests = [];
        try {
            const content = await readFile(file, 'utf8');
            const lines = content.split('\n');
            let pendingParams;
            for (let i = 0; i < lines.length; i++) {
                const line = lines[i].trim();
                // Check for @test-params comment
                const paramsMatch = line.match(/@test-params(?:\s+(.*))?$/);
                if (paramsMatch) {
                    try {
                        const paramsStr = (paramsMatch[1] || '').trim();
                        if (paramsStr.length === 0) {
                            pendingParams = [];
                        }
                        else if (paramsStr.startsWith('[')) {
                            const parsed = JSON.parse(paramsStr);
                            pendingParams = Array.isArray(parsed) ? parsed : [];
                        }
                        else {
                            pendingParams = paramsStr
                                .split(/\s+/)
                                .filter(Boolean)
                                .map((token) => this.parseTokenValue(token));
                        }
                    }
                    catch (error) {
                        console.warn(`Failed to parse @test-params in ${file}:${i + 1}: ${line}`);
                        pendingParams = undefined;
                    }
                    continue;
                }
                const funcMatch = line.match(/^pub\s+(?:fn\s+)?(test_[A-Za-z0-9_]*|[A-Za-z0-9_]*_test)\s*\([^)]*\)\s*(?:->\s*([A-Za-z0-9_<>\[\]]+))?/);
                if (funcMatch) {
                    const functionName = funcMatch[1];
                    const returnType = funcMatch[2];
                    const hasReturnValue = !!returnType;
                    const [parameters, expectedResult, expectsResult] = this.splitParamsAndExpectation(pendingParams, hasReturnValue);
                    const name = `${basename(file, '.v')}::${functionName}`;
                    tests.push({
                        name,
                        path: file,
                        type: 'v-source',
                        source: {
                            type: 'v-source',
                            file,
                            functionName,
                            parameters: parameters.length > 0 ? parameters : undefined,
                            expectedResult,
                            expectsResult
                        },
                        parameters: parameters.length > 0 ? parameters : undefined,
                        expectedResult,
                        expectsResult
                    });
                    pendingParams = undefined;
                }
            }
        }
        catch (error) {
            console.warn(`Failed to parse V file ${file}: ${error}`);
        }
        return tests;
    }
    static splitParamsAndExpectation(values, hasReturnValue) {
        const parsed = Array.isArray(values) ? values : [];
        if (!hasReturnValue || parsed.length === 0) {
            return [parsed, undefined, false];
        }
        const params = parsed.slice(0, parsed.length - 1);
        return [params, parsed[parsed.length - 1], true];
    }
    static parseTokenValue(token) {
        if ((token.startsWith('"') && token.endsWith('"')) ||
            (token.startsWith("'") && token.endsWith("'"))) {
            return token.slice(1, -1);
        }
        if (token === 'true')
            return true;
        if (token === 'false')
            return false;
        const asNumber = Number(token);
        if (!Number.isNaN(asNumber)) {
            return asNumber;
        }
        return token;
    }
    /**
     * Compile a .v test source file
     */
    static async compileVTest(file) {
        try {
            const source = await readFile(file, 'utf8');
            // Check if file has use statements for multi-file compilation
            const hasUseStatements = /^\s*use\s+/m.test(source) || /^\s*import\s+/m.test(source);
            let compilation;
            if (hasUseStatements) {
                // Use multi-file compilation with auto-discovery
                // This would require access to WasmCompilerWrapper
                // Use single-file compilation
                compilation = await FiveSDK.compile(source, {
                    debug: false,
                    optimize: false
                });
            }
            else {
                compilation = await FiveSDK.compile(source, {
                    debug: false,
                    optimize: false
                });
            }
            if (!compilation.success) {
                const errors = compilation.errors || [];
                const errorMessages = errors.map(e => typeof e === 'string' ? e : e.message || 'Unknown error');
                return {
                    success: false,
                    errors: errorMessages
                };
            }
            return {
                success: true,
                bytecode: compilation.bytecode
            };
        }
        catch (error) {
            return {
                success: false,
                errors: [error instanceof Error ? error.message : 'Compilation failed']
            };
        }
    }
    /**
     * Find files recursively matching predicate
     */
    static async findFilesRecursive(dir, predicate) {
        const files = [];
        try {
            const entries = await readdir(dir, { withFileTypes: true });
            for (const entry of entries) {
                const fullPath = join(dir, entry.name);
                if (entry.isDirectory() && !entry.name.startsWith('.')) {
                    // Recursively search subdirectories
                    const subFiles = await this.findFilesRecursive(fullPath, predicate);
                    files.push(...subFiles);
                }
                else if (entry.isFile() && predicate(entry.name)) {
                    files.push(fullPath);
                }
            }
        }
        catch (error) {
            // Silently skip inaccessible directories
        }
        return files;
    }
}
//# sourceMappingURL=TestDiscovery.js.map
