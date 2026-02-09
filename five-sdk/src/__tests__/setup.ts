/**
 * Test Setup for Five SDK
 * 
 * Configures the testing environment for comprehensive SDK testing including
 * WASM module mocking, Solana connection mocking, and test utilities.
 */

import { jest } from '@jest/globals';

// Global test configuration
global.console = {
  ...console,
  // Reduce console noise during tests but keep errors
  log: jest.fn(),
  debug: jest.fn(),
  info: jest.fn(),
  warn: console.warn,
  error: console.error
};

// Mock WebAssembly for tests that don't need real WASM
const mockWasmModule = {
  FiveVMWasm: jest.fn(() => ({
    execute_partial: jest.fn(() => 'Ok(Some(U64(42)))'),
    validate_bytecode: jest.fn(() => true),
    get_state: jest.fn(() => '{}')
  })),
  WasmFiveCompiler: jest.fn(() => ({
    compile: jest.fn(() => ({
      success: true,
      bytecode: new Uint8Array([1, 2, 3, 4]),
      bytecode_size: 4,
      compilation_time: 100,
      compiler_errors: [],
      error_count: 0,
      warning_count: 0
    })),
    validate_syntax: jest.fn(() => '{"valid": true, "errors": [], "warnings": []}'),
    generate_abi: jest.fn(() => '{"name": "test", "functions": []}')
  })),
  ParameterEncoder: {
    encode_execute_params: jest.fn((functionIndex: number, params: any[]) => {
      // Mock fixed execute param payload shape used by tests.
      return new Uint8Array([0, functionIndex, params.length, ...params.flatMap(p => [p.type, p.value])]);
    }),
    encode_execute_vle: jest.fn((functionIndex: number, params: any[]) => {
        // Legacy alias retained for older tests/helpers.
        return new Uint8Array([functionIndex, ...params.map(p => typeof p === 'number' ? p : 0)]);
    })
  },
  BytecodeAnalyzer: {
    analyze_semantic: jest.fn(() => '{"summary": {"total_size": 100}}'),
    get_bytecode_summary: jest.fn(() => '{"instructions": 10}')
  }
};

// Mock WASM module imports
jest.unstable_mockModule('../../assets/vm/five_vm_wasm.js', () => mockWasmModule);

// Test utilities
export const TestUtils = {
  /**
   * Create mock Solana connection
   */
  createMockConnection: () => ({
    getAccountInfo: jest.fn(),
    getMultipleAccountsInfo: jest.fn(),
    getRecentBlockhash: jest.fn(() => ({
      blockhash: 'test-blockhash',
      feeCalculator: { lamportsPerSignature: 5000 }
    })),
    sendTransaction: jest.fn(),
    confirmTransaction: jest.fn()
  }),

  /**
   * Create test bytecode
   */
  createTestBytecode: (size: number = 100): Uint8Array => {
    const bytecode = new Uint8Array(size);
    for (let i = 0; i < size; i++) {
      bytecode[i] = i % 256;
    }
    return bytecode;
  },

  /**
   * Create test script account data
   */
  createTestScriptAccountData: (bytecode: Uint8Array): Uint8Array => {
    const magic = new Uint8Array([0x46, 0x49, 0x56, 0x45, 0x5F, 0x53, 0x43, 0x52]); // "FIVE_SCR"
    const version = new Uint8Array([1, 0, 0, 0]); // Version 1
    const timestamp = new Uint8Array(8); // Timestamp (placeholder)
    const authority = new Uint8Array(32); // Authority pubkey (placeholder)
    const bytecodeLength = new Uint8Array(4);
    const abiData = Buffer.from(JSON.stringify({
      name: 'test_script',
      functions: [
        {
          name: 'test',
          index: 0,
          parameters: [],
          returnType: 'u64',
          visibility: 'public'
        },
        {
          name: 'add',
          index: 1,
          parameters: [
            { name: 'a', type: 'u64' },
            { name: 'b', type: 'u64' }
          ],
          returnType: 'u64',
          visibility: 'public'
        }
      ]
    }));
    const abiLength = new Uint8Array(4);
    const reserved = new Uint8Array(8);

    // Encode lengths
    const view = new DataView(bytecodeLength.buffer);
    view.setUint32(0, bytecode.length, true); // Little endian
    const abiView = new DataView(abiLength.buffer);
    abiView.setUint32(0, abiData.length, true);

    return new Uint8Array([
      ...magic,
      ...version,
      ...timestamp,
      ...authority,
      ...bytecodeLength,
      ...abiLength,
      ...reserved,
      ...bytecode,
      ...abiData
    ]);
  },

  /**
   * Create test Solana account info
   */
  createTestAccountInfo: (data: Uint8Array, owner: string = 'FiveProgramID11111111111111111111111111111') => ({
    data: Buffer.from(data),
    executable: false,
    lamports: 1000000,
    owner: { toBase58: () => owner },
    rentEpoch: 200
  }),

  /**
   * Wait for async operations to complete
   */
  waitForAsync: () => new Promise(resolve => setImmediate(resolve)),

  /**
   * Generate test public key
   */
  generateTestPubkey: (): string => {
    const chars = '123456789ABCDEFGHJKLMNPQRSTUVWXYZabcdefghijkmnopqrstuvwxyz';
    let result = '';
    for (let i = 0; i < 44; i++) {
      result += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return result;
  },

  /**
   * Create deterministic test data
   */
  createDeterministicTestData: (seed: string): {
    bytecode: Uint8Array;
    pubkey: string;
    accountData: Uint8Array;
  } => {
    // Simple deterministic data generation for reproducible tests
    const hash = seed.split('').reduce((a, b) => {
      a = ((a << 5) - a) + b.charCodeAt(0);
      return a & a;
    }, 0);
    
    const bytecode = new Uint8Array(50);
    for (let i = 0; i < 50; i++) {
      bytecode[i] = (hash + i) % 256;
    }

    return {
      bytecode,
      pubkey: TestUtils.generateTestPubkey(),
      accountData: TestUtils.createTestScriptAccountData(bytecode)
    };
  },

  /**
   * Mock console methods for specific tests
   */
  mockConsole: () => {
    const originalConsole = global.console;
    const mockMethods = {
      log: jest.fn(),
      debug: jest.fn(),
      info: jest.fn(),
      warn: jest.fn(),
      error: jest.fn()
    };

    Object.assign(global.console, mockMethods);

    return {
      restore: () => {
        Object.assign(global.console, originalConsole);
      },
      mocks: mockMethods
    };
  },

  /**
   * Expect async function to throw
   */
  expectAsyncThrow: async (asyncFn: () => Promise<any>, expectedError?: string) => {
    try {
      await asyncFn();
      throw new Error('Expected function to throw');
    } catch (error) {
      if (expectedError && error instanceof Error) {
        expect(error.message).toContain(expectedError);
      }
      return error;
    }
  }
};

// Test constants
export const TestConstants = {
  FIVE_VM_PROGRAM_ID: 'FiveProgramID11111111111111111111111111111',
  SYSTEM_PROGRAM_ID: '11111111111111111111111111111112',
  RENT_SYSVAR_ID: 'SysvarRent111111111111111111111111111111111',
  CLOCK_SYSVAR_ID: 'SysvarC1ock11111111111111111111111111111111',
  SPL_TOKEN_PROGRAM_ID: 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA',

  TEST_USER_PUBKEY: '11111111111111111111111111111114',
  TEST_SCRIPT_ACCOUNT: '22222222222222222222222222222224',
  TEST_METADATA_ACCOUNT: '33333333333333333333333333333334',

  SAMPLE_BYTECODE: new Uint8Array([
    // Five VM bytecode header
    0x46, 0x49, 0x56, 0x45, // "FIVE"
    0x01, 0x00, 0x00, 0x00, // Version 1
    // Sample instructions
    0x01, 0x00, // PUSH 0
    0x02, 0x05, // PUSH 5
    0x10,       // ADD
    0x30        // RETURN
  ]),

  SAMPLE_ABI: {
    name: 'test_contract',
    functions: [
      {
        name: 'test',
        index: 0,
        parameters: [],
        returnType: 'u64',
        visibility: 'public' as const
      },
      {
        name: 'add',
        index: 1,
        parameters: [
          { name: 'a', type: 'u64' },
          { name: 'b', type: 'u64' }
        ],
        returnType: 'u64',
        visibility: 'public' as const
      }
    ]
  }
};

// Test data generators
export const TestData = {
  /**
   * Generate various parameter types for testing
   */
  parameters: {
    u64: { name: 'test_u64', type: 'u64', value: 42 },
    string: { name: 'test_string', type: 'string', value: 'hello world' },
    bool: { name: 'test_bool', type: 'bool', value: true },
    bytes: { name: 'test_bytes', type: 'bytes', value: new Uint8Array([1, 2, 3, 4]) },
    pubkey: { name: 'test_pubkey', type: 'pubkey', value: TestConstants.TEST_USER_PUBKEY },
    array: { name: 'test_array', type: 'array', value: [1, 2, 3, 4, 5] }
  }
};

export { mockWasmModule };
