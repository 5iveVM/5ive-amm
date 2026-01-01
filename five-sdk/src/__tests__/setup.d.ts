/**
 * Test Setup for Five SDK
 *
 * Configures the testing environment for comprehensive SDK testing including
 * WASM module mocking, Solana connection mocking, and test utilities.
 */
declare const mockWasmModule: {
    FiveVMWasm: any;
    WasmFiveCompiler: any;
    ParameterEncoder: {
        encode_execute_params: any;
    };
    BytecodeAnalyzer: {
        analyze_semantic: any;
        get_bytecode_summary: any;
    };
};
export declare const TestUtils: {
    /**
     * Create mock Solana connection
     */
    createMockConnection: () => {
        getAccountInfo: any;
        getMultipleAccountsInfo: any;
        getRecentBlockhash: any;
        sendTransaction: any;
        confirmTransaction: any;
    };
    /**
     * Create test bytecode
     */
    createTestBytecode: (size?: number) => Uint8Array;
    /**
     * Create test script account data
     */
    createTestScriptAccountData: (bytecode: Uint8Array) => Uint8Array;
    /**
     * Create test Solana account info
     */
    createTestAccountInfo: (data: Uint8Array, owner?: string) => {
        data: any;
        executable: boolean;
        lamports: number;
        owner: {
            toBase58: () => string;
        };
        rentEpoch: number;
    };
    /**
     * Wait for async operations to complete
     */
    waitForAsync: () => Promise<unknown>;
    /**
     * Generate test public key
     */
    generateTestPubkey: () => string;
    /**
     * Create deterministic test data
     */
    createDeterministicTestData: (seed: string) => {
        bytecode: Uint8Array;
        pubkey: string;
        accountData: Uint8Array;
    };
    /**
     * Mock console methods for specific tests
     */
    mockConsole: () => {
        restore: () => void;
        mocks: {
            log: any;
            debug: any;
            info: any;
            warn: any;
            error: any;
        };
    };
    /**
     * Expect async function to throw
     */
    expectAsyncThrow: (asyncFn: () => Promise<any>, expectedError?: string) => Promise<unknown>;
};
export declare const TestConstants: {
    FIVE_VM_PROGRAM_ID: string;
    SYSTEM_PROGRAM_ID: string;
    RENT_SYSVAR_ID: string;
    CLOCK_SYSVAR_ID: string;
    SPL_TOKEN_PROGRAM_ID: string;
    TEST_USER_PUBKEY: string;
    TEST_SCRIPT_ACCOUNT: string;
    TEST_METADATA_ACCOUNT: string;
    SAMPLE_BYTECODE: Uint8Array<ArrayBuffer>;
    SAMPLE_ABI: {
        name: string;
        functions: {
            name: string;
            index: number;
            parameters: {
                name: string;
                type: string;
            }[];
            returnType: string;
            visibility: "public";
        }[];
    };
};
export declare const TestData: {
    /**
     * Generate various parameter types for testing
     */
    parameters: {
        u64: {
            name: string;
            type: string;
            value: number;
        };
        string: {
            name: string;
            type: string;
            value: string;
        };
        bool: {
            name: string;
            type: string;
            value: boolean;
        };
        bytes: {
            name: string;
            type: string;
            value: Uint8Array<ArrayBuffer>;
        };
        pubkey: {
            name: string;
            type: string;
            value: string;
        };
        array: {
            name: string;
            type: string;
            value: number[];
        };
    };
};
export { mockWasmModule };
//# sourceMappingURL=setup.d.ts.map