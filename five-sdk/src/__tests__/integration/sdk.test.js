/**
 * Five SDK Integration Tests
 *
 * End-to-end integration tests for the complete Five SDK workflow including
 * compilation, deployment, execution, and metadata operations.
 */
import { describe, it, expect, beforeEach, jest } from '@jest/globals';
import { FiveSDK } from '../../FiveSDK.js';
import { TestUtils, TestConstants } from '../setup.js';
const mockConnection = {
    getAccountInfo: jest.fn(),
    getMultipleAccountsInfo: jest.fn()
};
describe.skip('Five SDK Integration Tests', () => {
    // Skip integration tests until WASM modules are properly initialized
    beforeEach(() => {
        jest.clearAllMocks();
    });
    describe('Script Compilation Workflow', () => {
        it.skip('should compile Five script source to bytecode', async () => {
            // TODO: Test real WASM compilation once compiler is initialized
            const sourceCode = `
        function initialize() -> void {
          // Initialize the contract
        }
        
        function transfer(amount: u64, recipient: pubkey) -> bool {
          // Transfer logic
          return true;
        }
      `;
            const result = await FiveSDK.compile(sourceCode, { debug: true });
            expect(result.success).toBe(true);
            expect(result.bytecode).toBeDefined();
        });
        it('should handle compilation errors gracefully', async () => {
            const invalidSource = 'invalid Five syntax';
            const expectedResult = {
                success: false,
                bytecode: null,
                errors: ['Syntax error on line 1', 'Unexpected token'],
                error_count: 2,
                warning_count: 0
            };
            mockCompiler.compile.mockResolvedValue(expectedResult);
            const result = await FiveSDK.compile(invalidSource);
            expect(result.success).toBe(false);
            expect(result.errors).toHaveLength(2);
            expect(result.bytecode).toBeNull();
        });
        it('should compile from file path', async () => {
            const filePath = '/path/to/script.v';
            const expectedResult = {
                success: true,
                bytecode: TestConstants.SAMPLE_BYTECODE,
                bytecode_size: TestConstants.SAMPLE_BYTECODE.length
            };
            mockCompiler.compileFile.mockResolvedValue(expectedResult);
            const result = await FiveSDK.compileFile(filePath, { optimize: true });
            expect(result.success).toBe(true);
            expect(mockCompiler.compileFile).toHaveBeenCalledWith(filePath, { optimize: true });
        });
    });
    describe('Local WASM Execution Workflow', () => {
        it('should execute bytecode locally using WASM VM', async () => {
            const bytecode = TestConstants.SAMPLE_BYTECODE;
            const functionName = 'transfer';
            const parameters = [1000, TestConstants.TEST_USER_PUBKEY];
            const expectedResult = {
                success: true,
                result: true,
                logs: ['Transfer completed'],
                computeUnitsUsed: 5000,
                trace: [
                    { instruction: 'PUSH', value: 1000 },
                    { instruction: 'PUSH', value: TestConstants.TEST_USER_PUBKEY },
                    { instruction: 'CALL', function: 'transfer' }
                ]
            };
            mockWasmVM.execute.mockResolvedValue(expectedResult);
            const result = await FiveSDK.executeLocally(bytecode, functionName, parameters, { debug: true, trace: true });
            expect(result.success).toBe(true);
            expect(result.result).toBe(true);
            expect(result.computeUnitsUsed).toBe(5000);
            expect(result.logs).toContain('Transfer completed');
            expect(result.trace).toHaveLength(3);
            expect(typeof result.executionTime).toBe('number');
        });
        it('should handle execution errors', async () => {
            const bytecode = new Uint8Array([1, 2, 3]); // Invalid bytecode
            const functionName = 'nonexistent';
            mockWasmVM.execute.mockResolvedValue({
                success: false,
                error: 'Function not found: nonexistent',
                computeUnitsUsed: 100
            });
            const result = await FiveSDK.executeLocally(bytecode, functionName, []);
            expect(result.success).toBe(false);
            expect(result.error).toContain('Function not found');
            expect(result.computeUnitsUsed).toBe(100);
        });
        it('should compile and execute in one step', async () => {
            const sourceCode = `
        function add(a: u64, b: u64) -> u64 {
          return a + b;
        }
      `;
            // Mock successful compilation
            mockCompiler.compile.mockResolvedValue({
                success: true,
                bytecode: TestConstants.SAMPLE_BYTECODE,
                metadata: { functions: [{ name: 'add', index: 0 }] }
            });
            // Mock successful execution
            mockWasmVM.execute.mockResolvedValue({
                success: true,
                result: 42,
                computeUnitsUsed: 3000
            });
            const result = await FiveSDK.compileAndExecuteLocally(sourceCode, 'add', [20, 22], { debug: true, optimize: true });
            expect(result.success).toBe(true);
            expect(result.result).toBe(42);
            expect(result.compilation).toBeDefined();
            expect(result.compilation.success).toBe(true);
            expect(result.bytecodeSize).toBe(TestConstants.SAMPLE_BYTECODE.length);
            expect(result.functions).toHaveLength(1);
        });
        it('should handle compilation failure in compile-and-execute', async () => {
            const invalidSource = 'syntax error';
            mockCompiler.compile.mockResolvedValue({
                success: false,
                errors: ['Parse error'],
                bytecode: null
            });
            const result = await FiveSDK.compileAndExecuteLocally(invalidSource, 'test', []);
            expect(result.success).toBe(false);
            expect(result.error).toBe('Compilation failed');
            expect(result.compilationErrors).toContain('Parse error');
        });
    });
    describe('Bytecode Validation Workflow', () => {
        it('should validate correct bytecode', async () => {
            const bytecode = TestConstants.SAMPLE_BYTECODE;
            mockWasmVM.validateBytecode.mockResolvedValue({
                valid: true,
                metadata: {
                    version: 1,
                    functions: 2,
                    size: bytecode.length
                },
                functions: TestConstants.SAMPLE_ABI.functions
            });
            const result = await FiveSDK.validateBytecode(bytecode, { debug: true });
            expect(result.valid).toBe(true);
            expect(result.functions).toHaveLength(2);
            expect(result.metadata?.size).toBe(bytecode.length);
        });
        it('should detect invalid bytecode', async () => {
            const invalidBytecode = new Uint8Array([0xFF, 0xFF, 0xFF]); // Invalid magic bytes
            mockWasmVM.validateBytecode.mockResolvedValue({
                valid: false,
                errors: ['Invalid magic bytes', 'Unsupported version']
            });
            const result = await FiveSDK.validateBytecode(invalidBytecode);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('Invalid magic bytes');
            expect(result.errors).toContain('Unsupported version');
        });
        it('should handle validation exceptions', async () => {
            const bytecode = new Uint8Array([1, 2, 3]);
            mockWasmVM.validateBytecode.mockRejectedValue(new Error('WASM validation failed'));
            const result = await FiveSDK.validateBytecode(bytecode);
            expect(result.valid).toBe(false);
            expect(result.errors).toContain('WASM validation failed');
        });
    });
    describe('Deployment Instruction Generation', () => {
        it('should generate deployment instruction with PDA and rent calculation', async () => {
            const bytecode = TestConstants.SAMPLE_BYTECODE;
            const deployer = TestConstants.TEST_USER_PUBKEY;
            const expectedScriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
            const result = await FiveSDK.generateDeployInstruction(bytecode, deployer, {
                scriptAccount: expectedScriptAccount,
                debug: true
            });
            expect(result.instruction.programId).toBe('FiveProgramID11111111111111111111111111111');
            expect(result.scriptAccount).toBe(expectedScriptAccount);
            expect(result.requiredSigners).toContain(deployer);
            expect(result.bytecodeSize).toBe(bytecode.length);
            expect(result.estimatedCost).toBeGreaterThan(0);
            // Verify account structure
            expect(result.instruction.accounts).toHaveLength(4);
            expect(result.instruction.accounts[0].pubkey).toBe(expectedScriptAccount);
            expect(result.instruction.accounts[0].isWritable).toBe(true);
            expect(result.instruction.accounts[1].pubkey).toBe(deployer);
            expect(result.instruction.accounts[1].isSigner).toBe(true);
            // Verify instruction data is base64 encoded
            expect(typeof result.instruction.data).toBe('string');
            const decodedData = Buffer.from(result.instruction.data, 'base64');
            expect(decodedData.length).toBeGreaterThan(0);
            expect(decodedData[0]).toBe(1); // Deploy discriminator
        });
        it('should derive script account automatically when not provided', async () => {
            const bytecode = new Uint8Array([1, 2, 3, 4, 5]);
            const deployer = TestConstants.TEST_USER_PUBKEY;
            const result = await FiveSDK.generateDeployInstruction(bytecode, deployer);
            expect(typeof result.scriptAccount).toBe('string');
            expect(result.scriptAccount.length).toBeGreaterThan(40); // Valid base58 pubkey
            expect(result.instruction.accounts[0].pubkey).toBe(result.scriptAccount);
        });
    });
    describe('Execution Instruction Generation', () => {
        it('should generate execution instruction with parameter encoding', async () => {
            const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
            const functionName = 'transfer';
            const parameters = [1000, TestConstants.TEST_USER_PUBKEY];
            const accounts = [TestConstants.TEST_USER_PUBKEY];
            // Mock script metadata
            const mockMetadata = {
                functions: [
                    {
                        name: 'transfer',
                        index: 1,
                        parameters: [
                            { name: 'amount', type: 'u64' },
                            { name: 'recipient', type: 'pubkey' }
                        ],
                        returnType: 'bool',
                        visibility: 'public'
                    }
                ]
            };
            // Mock metadata retrieval
            jest.doMock('../../metadata/index.js', () => ({
                ScriptMetadataParser: {
                    getScriptMetadata: jest.fn().mockResolvedValue({
                        address: scriptAccount,
                        abi: { functions: mockMetadata.functions }
                    })
                }
            }));
            const result = await FiveSDK.generateExecuteInstruction(scriptAccount, functionName, parameters, accounts, { debug: true, computeUnitLimit: 100000 });
            expect(result.instruction.programId).toBe('FiveProgramID11111111111111111111111111111');
            expect(result.scriptAccount).toBe(scriptAccount);
            expect(result.parameters.function).toBe(functionName);
            expect(result.parameters.count).toBe(parameters.length);
            expect(result.estimatedComputeUnits).toBe(100000);
            // Verify account structure
            expect(result.instruction.accounts).toHaveLength(2); // Script + user account
            expect(result.instruction.accounts[0].pubkey).toBe(scriptAccount);
            expect(result.instruction.accounts[0].isWritable).toBe(false);
            expect(result.instruction.accounts[1].pubkey).toBe(accounts[0]);
            expect(result.instruction.accounts[1].isWritable).toBe(true);
            // Verify instruction data
            const decodedData = Buffer.from(result.instruction.data, 'base64');
            expect(decodedData[0]).toBe(2); // Execute discriminator
            expect(decodedData[1]).toBe(1); // Function index (varint encoded)
        });
        it('should resolve function name to index', async () => {
            const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
            const functionIndex = 0;
            const parameters = [];
            const accounts = [];
            const mockMetadata = {
                functions: [
                    { name: 'initialize', index: 0 },
                    { name: 'transfer', index: 1 }
                ]
            };
            jest.doMock('../../metadata/index.js', () => ({
                ScriptMetadataParser: {
                    getScriptMetadata: jest.fn().mockResolvedValue({
                        address: scriptAccount,
                        abi: { functions: mockMetadata.functions }
                    })
                }
            }));
            const result = await FiveSDK.generateExecuteInstruction(scriptAccount, functionIndex, // Use numeric index
            parameters, accounts);
            expect(result.parameters.function).toBe(functionIndex);
        });
        it('should handle missing function error', async () => {
            const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
            const functionName = 'nonexistent_function';
            const mockMetadata = {
                functions: [
                    { name: 'initialize', index: 0 },
                    { name: 'transfer', index: 1 }
                ]
            };
            jest.doMock('../../metadata/index.js', () => ({
                ScriptMetadataParser: {
                    getScriptMetadata: jest.fn().mockResolvedValue({
                        address: scriptAccount,
                        abi: { functions: mockMetadata.functions }
                    })
                }
            }));
            await expect(FiveSDK.generateExecuteInstruction(scriptAccount, functionName, [], [])).rejects.toThrow(`Function '${functionName}' not found`);
        });
    });
    describe('Script Metadata Operations', () => {
        it('should parse script metadata from account data', () => {
            const bytecode = new Uint8Array([1, 2, 3, 4]);
            const abi = TestConstants.SAMPLE_ABI;
            const address = TestConstants.TEST_SCRIPT_ACCOUNT;
            const accountData = TestUtils.createTestScriptAccountData(bytecode);
            const result = FiveSDK.parseScriptMetadata(accountData, address);
            expect(result.address).toBe(address);
            expect(result.bytecode).toEqual(expect.any(Uint8Array));
            expect(result.abi).toBeDefined();
            expect(typeof result.deployedAt).toBe('number');
            expect(typeof result.authority).toBe('string');
        });
        it('should get script metadata with connection', async () => {
            const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
            const bytecode = TestConstants.SAMPLE_BYTECODE;
            const accountData = TestUtils.createTestScriptAccountData(bytecode);
            mockConnection.getAccountInfo.mockResolvedValue({
                data: accountData,
                executable: false,
                lamports: 1000000,
                owner: { toBase58: () => 'FiveProgramID11111111111111111111111111111' },
                rentEpoch: 200
            });
            const result = await FiveSDK.getScriptMetadataWithConnection(scriptAccount, mockConnection);
            expect(result.address).toBe(scriptAccount);
            expect(result.bytecode).toEqual(expect.any(Uint8Array));
            expect(result.abi).toBeDefined();
        });
        it('should use cached metadata for performance', async () => {
            const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
            const mockMetadata = {
                address: scriptAccount,
                bytecode: TestConstants.SAMPLE_BYTECODE,
                abi: TestConstants.SAMPLE_ABI,
                deployedAt: Date.now(),
                version: '1',
                authority: TestConstants.TEST_USER_PUBKEY
            };
            mockConnection.getAccountInfo.mockResolvedValue({
                data: TestUtils.createTestScriptAccountData(TestConstants.SAMPLE_BYTECODE),
                executable: false,
                lamports: 1000000,
                owner: { toBase58: () => 'FiveProgramID11111111111111111111111111111' },
                rentEpoch: 200
            });
            // First call should fetch from blockchain
            const result1 = await FiveSDK.getCachedScriptMetadata(scriptAccount, mockConnection, 60000 // 1 minute TTL
            );
            // Second call should use cache
            const result2 = await FiveSDK.getCachedScriptMetadata(scriptAccount, mockConnection, 60000);
            expect(result1.address).toBe(scriptAccount);
            expect(result2.address).toBe(scriptAccount);
            expect(mockConnection.getAccountInfo).toHaveBeenCalledTimes(1); // Only called once
        });
        it('should invalidate metadata cache', async () => {
            const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
            // This should not throw
            FiveSDK.invalidateMetadataCache(scriptAccount);
            const stats = FiveSDK.getMetadataCacheStats();
            expect(typeof stats).toBe('object');
        });
    });
    describe('Error Handling and Edge Cases', () => {
        it('should handle WASM loading failure gracefully', async () => {
            const bytecode = TestConstants.SAMPLE_BYTECODE;
            // Mock WASM import failure
            jest.doMock('../../wasm/vm.js', () => {
                throw new Error('Failed to load WASM module');
            });
            await expect(FiveSDK.executeLocally(bytecode, 'test', []))
                .rejects.toThrow('Failed to load WASM VM');
        });
        it('should handle empty parameters correctly', async () => {
            const result = await FiveSDK.generateExecuteInstruction(TestConstants.TEST_SCRIPT_ACCOUNT, 0, // Function index
            [], // No parameters
            [] // No additional accounts
            );
            expect(result.parameters.count).toBe(0);
            expect(result.instruction.accounts).toHaveLength(1); // Only script account
        });
        it('should handle large bytecode deployment', async () => {
            const largeBytecode = new Uint8Array(10000); // 10KB bytecode
            largeBytecode.fill(42);
            const result = await FiveSDK.generateDeployInstruction(largeBytecode, TestConstants.TEST_USER_PUBKEY);
            expect(result.bytecodeSize).toBe(10000);
            expect(result.estimatedCost).toBeGreaterThan(0);
        });
        it('should estimate compute units based on function complexity', async () => {
            const scriptAccount = TestConstants.TEST_SCRIPT_ACCOUNT;
            const complexParameters = [
                1000, 'complex_string_parameter', true,
                new Uint8Array([1, 2, 3, 4, 5]),
                TestConstants.TEST_USER_PUBKEY
            ];
            const mockMetadata = {
                functions: [
                    {
                        name: 'complex_function',
                        index: 5,
                        parameters: [
                            { name: 'amount', type: 'u64' },
                            { name: 'name', type: 'string' },
                            { name: 'active', type: 'bool' },
                            { name: 'data', type: 'bytes' },
                            { name: 'authority', type: 'pubkey' }
                        ]
                    }
                ]
            };
            jest.doMock('../../metadata/index.js', () => ({
                ScriptMetadataParser: {
                    getScriptMetadata: jest.fn().mockResolvedValue({
                        address: scriptAccount,
                        abi: { functions: mockMetadata.functions }
                    })
                }
            }));
            const result = await FiveSDK.generateExecuteInstruction(scriptAccount, 'complex_function', complexParameters, []);
            // Should estimate higher compute units for complex function with many parameters
            expect(result.estimatedComputeUnits).toBeGreaterThan(5000);
        });
    });
});
//# sourceMappingURL=sdk.test.js.map