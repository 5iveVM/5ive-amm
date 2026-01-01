/**
 * Five SDK Metadata System Unit Tests
 *
 * Tests for script metadata parsing, ABI extraction, and caching functionality
 * with real Solana account data parsing instead of mock implementations.
 */
import { describe, it, expect, beforeEach, jest } from '@jest/globals';
import { ScriptMetadataParser, MetadataCache, ScriptMetadata, ScriptABI, FunctionDefinition } from '../../metadata/index.js';
// Mock @solana/web3.js
const mockAccountInfo = {
    data: Buffer.alloc(0),
    executable: false,
    lamports: 1000000,
    owner: { toBase58: () => 'FiveProgramID11111111111111111111111111111' },
    rentEpoch: 200
};
const mockConnection = {
    getAccountInfo: jest.fn(),
    getMultipleAccountsInfo: jest.fn()
};
jest.unstable_mockModule('@solana/web3.js', () => ({
    PublicKey: jest.fn().mockImplementation((key) => ({
        toBase58: () => key
    }))
}));
describe('Five SDK Metadata System', () => {
    beforeEach(() => {
        jest.clearAllMocks();
    });
    describe('ScriptMetadataParser', () => {
        describe('parseMetadata', () => {
            it('should parse valid script account data', () => {
                const bytecode = new Uint8Array([1, 2, 3, 4, 5, 6, 7, 8]);
                const abi = {
                    name: 'TestScript',
                    functions: [
                        {
                            name: 'initialize',
                            index: 0,
                            parameters: [],
                            returnType: 'void',
                            visibility: 'public'
                        },
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
                const accountData = createTestScriptAccountData(bytecode, abi);
                const address = '11111111111111111111111111111114';
                const result = ScriptMetadataParser.parseMetadata(accountData, address);
                expect(result.address).toBe(address);
                expect(result.bytecode).toEqual(bytecode);
                expect(result.abi).toEqual(abi);
                expect(result.version).toBe('1');
                expect(typeof result.deployedAt).toBe('number');
                expect(typeof result.authority).toBe('string');
            });
            it('should throw error for data too small', () => {
                const smallData = new Uint8Array(32); // Smaller than minimum header size
                const address = '11111111111111111111111111111114';
                expect(() => ScriptMetadataParser.parseMetadata(smallData, address))
                    .toThrow('Invalid script account: data too small');
            });
            it('should throw error for invalid magic bytes', () => {
                const invalidData = new Uint8Array(128);
                // Don't set correct magic bytes
                const address = '11111111111111111111111111111114';
                expect(() => ScriptMetadataParser.parseMetadata(invalidData, address))
                    .toThrow('Invalid script account: magic bytes mismatch');
            });
            it('should throw error for unsupported version', () => {
                const accountData = new Uint8Array(128);
                // Set magic bytes
                accountData.set([0x46, 0x49, 0x56, 0x45, 0x5F, 0x53, 0x43, 0x52], 0);
                // Set unsupported version (version 999)
                const view = new DataView(accountData.buffer);
                view.setUint32(8, 999, true); // Little endian
                const address = '11111111111111111111111111111114';
                expect(() => ScriptMetadataParser.parseMetadata(accountData, address))
                    .toThrow('Unsupported script version: 999');
            });
            it('should throw error for invalid account size', () => {
                const bytecode = new Uint8Array([1, 2, 3, 4]);
                const abi = { name: 'test', functions: [] };
                // Create account data but truncate it
                const fullData = createTestScriptAccountData(bytecode, abi);
                const truncatedData = fullData.slice(0, fullData.length - 10); // Remove last 10 bytes
                const address = '11111111111111111111111111111114';
                expect(() => ScriptMetadataParser.parseMetadata(truncatedData, address))
                    .toThrow('Invalid script account: expected');
            });
            it('should throw error for invalid ABI JSON', () => {
                const bytecode = new Uint8Array([1, 2, 3, 4]);
                const accountData = createTestScriptAccountDataWithInvalidABI(bytecode);
                const address = '11111111111111111111111111111114';
                expect(() => ScriptMetadataParser.parseMetadata(accountData, address))
                    .toThrow('Invalid ABI JSON');
            });
            it('should parse complex script with multiple functions', () => {
                const bytecode = new Uint8Array(100).fill(42); // Large bytecode
                const complexABI = {
                    name: 'ComplexScript',
                    functions: [
                        {
                            name: 'initialize',
                            index: 0,
                            parameters: [
                                { name: 'admin', type: 'pubkey' },
                                { name: 'config', type: 'bytes' }
                            ],
                            returnType: 'void',
                            visibility: 'public',
                            docs: 'Initialize the script with admin and configuration'
                        },
                        {
                            name: 'transfer',
                            index: 1,
                            parameters: [
                                { name: 'from', type: 'pubkey' },
                                { name: 'to', type: 'pubkey' },
                                { name: 'amount', type: 'u64' }
                            ],
                            returnType: 'bool',
                            visibility: 'public'
                        },
                        {
                            name: 'internal_helper',
                            index: 2,
                            parameters: [],
                            returnType: 'u64',
                            visibility: 'private'
                        }
                    ],
                    types: [
                        {
                            name: 'Config',
                            structure: 'struct',
                            fields: [
                                { name: 'fee_rate', type: 'u64' },
                                { name: 'max_supply', type: 'u64' }
                            ]
                        }
                    ]
                };
                const accountData = createTestScriptAccountData(bytecode, complexABI);
                const address = '22222222222222222222222222222224';
                const result = ScriptMetadataParser.parseMetadata(accountData, address);
                expect(result.address).toBe(address);
                expect(result.bytecode).toEqual(bytecode);
                expect(result.abi.functions).toHaveLength(3);
                expect(result.abi.types).toHaveLength(1);
                expect(result.abi.functions[0].docs).toBeDefined();
            });
        });
        describe('getScriptMetadata', () => {
            it('should fetch and parse metadata from blockchain', async () => {
                const bytecode = new Uint8Array([1, 2, 3, 4]);
                const abi = { name: 'TestScript', functions: [] };
                const accountData = createTestScriptAccountData(bytecode, abi);
                const scriptAddress = '11111111111111111111111111111114';
                mockConnection.getAccountInfo.mockResolvedValue({
                    ...mockAccountInfo,
                    data: accountData
                });
                const result = await ScriptMetadataParser.getScriptMetadata(mockConnection, scriptAddress);
                expect(result.address).toBe(scriptAddress);
                expect(result.bytecode).toEqual(bytecode);
                expect(result.abi).toEqual(abi);
                expect(mockConnection.getAccountInfo).toHaveBeenCalledWith(expect.anything(), // PublicKey instance
                'confirmed');
            });
            it('should throw error for invalid address', async () => {
                const invalidAddress = 'invalid-address';
                await expect(ScriptMetadataParser.getScriptMetadata(mockConnection, invalidAddress))
                    .rejects.toThrow(`Invalid script address: ${invalidAddress}`);
            });
            it('should throw error when account not found', async () => {
                const scriptAddress = '11111111111111111111111111111114';
                mockConnection.getAccountInfo.mockResolvedValue(null);
                await expect(ScriptMetadataParser.getScriptMetadata(mockConnection, scriptAddress))
                    .rejects.toThrow(`Script account not found: ${scriptAddress}`);
            });
            it('should throw error when account has no data', async () => {
                const scriptAddress = '11111111111111111111111111111114';
                mockConnection.getAccountInfo.mockResolvedValue({
                    ...mockAccountInfo,
                    data: Buffer.alloc(0)
                });
                await expect(ScriptMetadataParser.getScriptMetadata(mockConnection, scriptAddress))
                    .rejects.toThrow(`Script account has no data: ${scriptAddress}`);
            });
        });
        describe('getMultipleScriptMetadata', () => {
            it('should fetch multiple script metadata entries', async () => {
                const addresses = [
                    '11111111111111111111111111111114', // System program variant
                    'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA', // SPL Token Program
                    'SysvarRent111111111111111111111111111111111' // Sysvar Rent
                ];
                const bytecode1 = new Uint8Array([1, 2, 3]);
                const bytecode2 = new Uint8Array([4, 5, 6]);
                const bytecode3 = new Uint8Array([7, 8, 9]);
                const abi1 = { name: 'Script1', functions: [] };
                const abi2 = { name: 'Script2', functions: [] };
                const abi3 = { name: 'Script3', functions: [] };
                const accountInfos = [
                    { ...mockAccountInfo, data: createTestScriptAccountData(bytecode1, abi1) },
                    { ...mockAccountInfo, data: createTestScriptAccountData(bytecode2, abi2) },
                    { ...mockAccountInfo, data: createTestScriptAccountData(bytecode3, abi3) }
                ];
                mockConnection.getMultipleAccountsInfo.mockResolvedValue(accountInfos);
                const results = await ScriptMetadataParser.getMultipleScriptMetadata(mockConnection, addresses);
                expect(results.size).toBe(3);
                expect(results.get(addresses[0])?.abi.name).toBe('Script1');
                expect(results.get(addresses[1])?.abi.name).toBe('Script2');
                expect(results.get(addresses[2])?.abi.name).toBe('Script3');
            });
            it('should handle mixed valid/invalid addresses', async () => {
                const addresses = [
                    '11111111111111111111111111111114', // valid (system program with different trailing digit)
                    'invalid-address', // invalid
                    'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA' // valid (SPL Token Program)
                ];
                const bytecode = new Uint8Array([1, 2, 3]);
                const abi = { name: 'TestScript', functions: [] };
                const validAccountData = createTestScriptAccountData(bytecode, abi);
                mockConnection.getMultipleAccountsInfo.mockResolvedValue([
                    { ...mockAccountInfo, data: validAccountData },
                    { ...mockAccountInfo, data: validAccountData }
                ]);
                const results = await ScriptMetadataParser.getMultipleScriptMetadata(mockConnection, addresses);
                expect(results.size).toBe(3);
                expect(results.get(addresses[0])).not.toBeNull();
                expect(results.get(addresses[1])).toBeNull(); // Invalid address
                expect(results.get(addresses[2])).not.toBeNull();
            });
            it('should fail when batch request fails (no fallback)', async () => {
                const addresses = ['11111111111111111111111111111114'];
                // Batch request fails
                mockConnection.getMultipleAccountsInfo.mockRejectedValue(new Error('Batch request failed'));
                await expect(ScriptMetadataParser.getMultipleScriptMetadata(mockConnection, addresses))
                    .rejects.toThrow('Batch metadata fetch failed: Batch request failed');
            });
        });
        describe('extractFunctionSignatures', () => {
            it('should extract function signatures from ABI', () => {
                const abi = {
                    name: 'TestScript',
                    functions: [
                        {
                            name: 'initialize',
                            index: 0,
                            parameters: [],
                            returnType: 'void',
                            visibility: 'public'
                        },
                        {
                            name: 'transfer',
                            index: 1,
                            parameters: [
                                { name: 'amount', type: 'u64' },
                                { name: 'recipient', type: 'pubkey' }
                            ],
                            returnType: 'bool',
                            visibility: 'public'
                        },
                        {
                            name: 'optional_param',
                            index: 2,
                            parameters: [
                                { name: 'required', type: 'u64' },
                                { name: 'optional', type: 'string', optional: true }
                            ],
                            visibility: 'public'
                        }
                    ]
                };
                const signatures = ScriptMetadataParser.extractFunctionSignatures(abi);
                expect(signatures).toHaveLength(3);
                expect(signatures[0].signature).toBe('initialize() -> void');
                expect(signatures[1].signature).toBe('transfer(amount: u64, recipient: pubkey) -> bool');
                expect(signatures[2].signature).toBe('optional_param(required: u64, optional: string?)');
            });
        });
        describe('validateABI', () => {
            it('should validate correct ABI structure', () => {
                const validABI = {
                    name: 'TestScript',
                    functions: [
                        {
                            name: 'test_function',
                            index: 0,
                            parameters: [
                                { name: 'param1', type: 'u64' }
                            ],
                            returnType: 'bool',
                            visibility: 'public'
                        }
                    ]
                };
                const result = ScriptMetadataParser.validateABI(validABI);
                expect(result.valid).toBe(true);
                expect(result.errors).toHaveLength(0);
            });
            it('should reject ABI with missing name', () => {
                const invalidABI = {
                    // name missing
                    functions: []
                };
                const result = ScriptMetadataParser.validateABI(invalidABI);
                expect(result.valid).toBe(false);
                expect(result.errors).toContain('ABI must have a non-empty name');
            });
            it('should reject ABI with invalid functions array', () => {
                const invalidABI = {
                    name: 'TestScript',
                    functions: 'not an array'
                };
                const result = ScriptMetadataParser.validateABI(invalidABI);
                expect(result.valid).toBe(false);
                expect(result.errors).toContain('ABI must have a functions array');
            });
            it('should reject ABI with invalid function definitions', () => {
                const invalidABI = {
                    name: 'TestScript',
                    functions: [
                        {
                            // name missing
                            index: 'not a number',
                            parameters: 'not an array',
                            visibility: 'invalid'
                        }
                    ]
                };
                const result = ScriptMetadataParser.validateABI(invalidABI);
                expect(result.valid).toBe(false);
                expect(result.errors.length).toBeGreaterThan(0);
                expect(result.errors.some(err => err.includes('must have a non-empty name'))).toBe(true);
                expect(result.errors.some(err => err.includes('must have a non-negative index'))).toBe(true);
                expect(result.errors.some(err => err.includes('must have a parameters array'))).toBe(true);
                expect(result.errors.some(err => err.includes('visibility must be'))).toBe(true);
            });
        });
    });
    describe('MetadataCache', () => {
        let cache;
        beforeEach(() => {
            cache = new MetadataCache();
        });
        it('should cache and retrieve metadata', async () => {
            const scriptAddress = '11111111111111111111111111111114';
            const mockMetadata = {
                address: scriptAddress,
                bytecode: new Uint8Array([1, 2, 3]),
                abi: { name: 'TestScript', functions: [] },
                deployedAt: Date.now(),
                version: '1',
                authority: '22222222222222222222222222222224'
            };
            const fetcher = jest.fn().mockResolvedValue(mockMetadata);
            // First call should fetch
            const result1 = await cache.getMetadata(scriptAddress, fetcher, 60000); // 1 minute TTL
            expect(result1).toEqual(mockMetadata);
            expect(fetcher).toHaveBeenCalledTimes(1);
            // Second call should use cache
            const result2 = await cache.getMetadata(scriptAddress, fetcher, 60000);
            expect(result2).toEqual(mockMetadata);
            expect(fetcher).toHaveBeenCalledTimes(1); // Still called only once
        });
        it('should refetch expired metadata', async () => {
            const scriptAddress = '11111111111111111111111111111114';
            const mockMetadata = {
                address: scriptAddress,
                bytecode: new Uint8Array([1, 2, 3]),
                abi: { name: 'TestScript', functions: [] },
                deployedAt: Date.now(),
                version: '1',
                authority: '22222222222222222222222222222224'
            };
            const fetcher = jest.fn().mockResolvedValue(mockMetadata);
            // First call with very short TTL
            await cache.getMetadata(scriptAddress, fetcher, 1); // 1ms TTL
            // Wait for expiration
            await new Promise(resolve => setTimeout(resolve, 5));
            // Second call should refetch
            await cache.getMetadata(scriptAddress, fetcher, 60000);
            expect(fetcher).toHaveBeenCalledTimes(2);
        });
        it('should invalidate cache entries', async () => {
            const scriptAddress = '11111111111111111111111111111114';
            const mockMetadata = {
                address: scriptAddress,
                bytecode: new Uint8Array([1, 2, 3]),
                abi: { name: 'TestScript', functions: [] },
                deployedAt: Date.now(),
                version: '1',
                authority: '22222222222222222222222222222224'
            };
            const fetcher = jest.fn().mockResolvedValue(mockMetadata);
            // Cache metadata
            await cache.getMetadata(scriptAddress, fetcher, 60000);
            expect(fetcher).toHaveBeenCalledTimes(1);
            // Invalidate
            cache.invalidate(scriptAddress);
            // Next call should refetch
            await cache.getMetadata(scriptAddress, fetcher, 60000);
            expect(fetcher).toHaveBeenCalledTimes(2);
        });
        it('should provide cache statistics', async () => {
            const scriptAddress = '11111111111111111111111111111114';
            const mockMetadata = {
                address: scriptAddress,
                bytecode: new Uint8Array([1, 2, 3]),
                abi: { name: 'TestScript', functions: [] },
                deployedAt: Date.now(),
                version: '1',
                authority: '22222222222222222222222222222224'
            };
            const fetcher = jest.fn().mockResolvedValue(mockMetadata);
            await cache.getMetadata(scriptAddress, fetcher, 60000);
            const stats = cache.getStats();
            expect(stats.size).toBe(1);
            expect(stats.entries).toHaveLength(1);
            expect(stats.entries[0].address).toBe(scriptAddress);
            expect(typeof stats.entries[0].age).toBe('number');
            expect(stats.entries[0].ttl).toBe(60000);
        });
        it('should cleanup expired entries', async () => {
            const scriptAddress = '11111111111111111111111111111114';
            const mockMetadata = {
                address: scriptAddress,
                bytecode: new Uint8Array([1, 2, 3]),
                abi: { name: 'TestScript', functions: [] },
                deployedAt: Date.now(),
                version: '1',
                authority: '22222222222222222222222222222224'
            };
            const fetcher = jest.fn().mockResolvedValue(mockMetadata);
            // Add entry with short TTL
            await cache.getMetadata(scriptAddress, fetcher, 1); // 1ms TTL
            expect(cache.getStats().size).toBe(1);
            // Wait for expiration
            await new Promise(resolve => setTimeout(resolve, 5));
            // Cleanup should remove expired entries
            cache.cleanup();
            expect(cache.getStats().size).toBe(0);
        });
    });
});
// Helper functions for creating test data
function createTestScriptAccountData(bytecode, abi) {
    const magic = new Uint8Array([0x46, 0x49, 0x56, 0x45, 0x5F, 0x53, 0x43, 0x52]); // "FIVE_SCR"
    const version = new Uint8Array([1, 0, 0, 0]); // Version 1
    const timestamp = new Uint8Array(8); // Timestamp (placeholder)
    const authority = new Uint8Array(32); // Authority pubkey (placeholder)
    const abiData = Buffer.from(JSON.stringify(abi));
    // Create account data buffer
    // Total size = 8 (magic) + 4 (version) + 8 (timestamp) + 32 (authority) + 4 (bytecode len) + 4 (abi len) + 8 (reserved) + bytecode + abi
    const totalSize = 8 + 4 + 8 + 32 + 4 + 4 + 8 + bytecode.length + abiData.length;
    const accountData = new Uint8Array(totalSize);
    const view = new DataView(accountData.buffer);
    let offset = 0;
    // Write header
    accountData.set(magic, offset);
    offset += 8;
    accountData.set(version, offset);
    offset += 4;
    accountData.set(timestamp, offset);
    offset += 8;
    accountData.set(authority, offset);
    offset += 32;
    // Write bytecode length
    view.setUint32(offset, bytecode.length, true); // Little endian
    offset += 4;
    // Write ABI length
    view.setUint32(offset, abiData.length, true); // Little endian
    offset += 4;
    // Skip reserved space
    offset += 8;
    // Write bytecode
    accountData.set(bytecode, offset);
    offset += bytecode.length;
    // Write ABI
    accountData.set(abiData, offset);
    return accountData;
}
function createTestScriptAccountDataWithInvalidABI(bytecode) {
    const magic = new Uint8Array([0x46, 0x49, 0x56, 0x45, 0x5F, 0x53, 0x43, 0x52]);
    const version = new Uint8Array([1, 0, 0, 0]);
    const timestamp = new Uint8Array(8);
    const authority = new Uint8Array(32);
    const invalidAbiData = Buffer.from('{ invalid json'); // Invalid JSON
    // Total size = 8 (magic) + 4 (version) + 8 (timestamp) + 32 (authority) + 4 (bytecode len) + 4 (abi len) + 8 (reserved) + bytecode + abi
    const totalSize = 8 + 4 + 8 + 32 + 4 + 4 + 8 + bytecode.length + invalidAbiData.length;
    const accountData = new Uint8Array(totalSize);
    const view = new DataView(accountData.buffer);
    let offset = 0;
    accountData.set(magic, offset);
    offset += 8;
    accountData.set(version, offset);
    offset += 4;
    accountData.set(timestamp, offset);
    offset += 8;
    accountData.set(authority, offset);
    offset += 32;
    view.setUint32(offset, bytecode.length, true);
    offset += 4;
    view.setUint32(offset, invalidAbiData.length, true);
    offset += 4;
    offset += 8; // Reserved
    accountData.set(bytecode, offset);
    offset += bytecode.length;
    accountData.set(invalidAbiData, offset);
    return accountData;
}
//# sourceMappingURL=metadata.test.js.map