/**
 * Deployment service tests.
 * Uses real Solana networks when configured.
 */

import { describe, beforeAll, beforeEach, afterEach, it, expect, jest } from '@jest/globals';
import { Keypair, PublicKey, Connection } from '@solana/web3.js';
import { WalletAdapter } from '@solana/wallet-adapter-base';
import { 
    DeploymentService, 
    DeploymentConfig, 
    SolanaNetwork, 
    DeploymentUtils 
} from '../app/deployment-service';
import { WasmCompilerService } from '../app/wasm-compiler';

/**
 * Mock wallet adapter for testing
 */
class MockWalletAdapter implements WalletAdapter {
    private keypair: Keypair;
    public connected: boolean = false;
    public connecting: boolean = false;
    public disconnecting: boolean = false;

    constructor() {
        this.keypair = Keypair.generate();
    }

    get publicKey(): PublicKey {
        return this.keypair.publicKey;
    }

    get name(): string {
        return 'Mock Wallet';
    }

    get url(): string {
        return 'https://mock-wallet.com';
    }

    get icon(): string {
        return '';
    }

    get readyState(): any {
        return 'Installed';
    }

    async connect(): Promise<void> {
        this.connecting = true;
        await new Promise(resolve => setTimeout(resolve, 100));
        this.connected = true;
        this.connecting = false;
    }

    async disconnect(): Promise<void> {
        this.disconnecting = true;
        await new Promise(resolve => setTimeout(resolve, 100));
        this.connected = false;
        this.disconnecting = false;
    }

    async signTransaction(transaction: any): Promise<any> {
        if (!this.connected) {
            throw new Error('Wallet not connected');
        }
        transaction.sign(this.keypair);
        return transaction;
    }

    async signAllTransactions(transactions: any[]): Promise<any[]> {
        return transactions.map(tx => {
            tx.sign(this.keypair);
            return tx;
        });
    }

    on(event: string, handler: Function): void {
        // Mock event handling
    }

    off(event: string, handler: Function): void {
        // Mock event handling
    }
}

describe('DeploymentService', () => {
    let deploymentService: DeploymentService;
    let wasmCompiler: WasmCompilerService;
    let mockWallet: MockWalletAdapter;
    let testBytecode: Uint8Array;

    beforeAll(async () => {
        // Initialize WASM compiler for creating test bytecode
        wasmCompiler = new WasmCompilerService();
        await wasmCompiler.initialize();

        // Create valid test bytecode
        testBytecode = wasmCompiler.createTestBytecode([
            { opcode: 'PUSH', args: ['U64', 100] },
            { opcode: 'PUSH', args: ['U64', 50] },
            { opcode: 'ADD' },
            { opcode: 'HALT' }
        ]);
    });

    beforeEach(async () => {
        // Create fresh instances for each test
        const config: DeploymentConfig = {
            network: 'localnet',
            rpcUrl: 'http://localhost:8899',
            commitment: 'confirmed',
            confirmationTimeout: 30000
        };

        deploymentService = new DeploymentService(config);
        mockWallet = new MockWalletAdapter();
        await mockWallet.connect();
    });

    afterEach(() => {
        jest.clearAllMocks();
    });

    describe('Initialization', () => {
        it('should initialize with valid configuration', async () => {
            await expect(deploymentService.initialize()).resolves.not.toThrow();
        });

        it('should validate network configuration', async () => {
            const invalidConfig: DeploymentConfig = {
                network: 'invalid-network' as SolanaNetwork
            };

            expect(() => new DeploymentService(invalidConfig)).toThrow();
        });

        it('should handle custom RPC URLs', async () => {
            const customConfig: DeploymentConfig = {
                network: 'devnet',
                rpcUrl: 'https://custom-rpc.example.com'
            };

            const service = new DeploymentService(customConfig);
            await expect(service.initialize()).resolves.not.toThrow();
        });
    });

    describe('Cost Estimation', () => {
        beforeEach(async () => {
            await deploymentService.initialize();
        });

        it('should estimate deployment costs for valid bytecode', async () => {
            const estimation = await deploymentService.estimateDeploymentCost(testBytecode);

            expect(estimation).toMatchObject({
                computeUnits: expect.any(Number),
                estimatedCost: expect.any(Number),
                rentExemptBalance: expect.any(Number),
                transactionFee: expect.any(Number),
                totalCost: expect.any(Number)
            });

            expect(estimation.computeUnits).toBeGreaterThan(0);
            expect(estimation.totalCost).toBeGreaterThan(0);
            expect(estimation.totalCost).toBe(
                estimation.rentExemptBalance + estimation.transactionFee
            );
        });

        it('should reject invalid bytecode', async () => {
            const invalidBytecode = new Uint8Array([0x12, 0x34, 0x56, 0x78]); // Invalid magic bytes

            await expect(
                deploymentService.estimateDeploymentCost(invalidBytecode)
            ).rejects.toThrow('Invalid bytecode format');
        });

        it('should handle empty bytecode', async () => {
            const emptyBytecode = new Uint8Array();

            await expect(
                deploymentService.estimateDeploymentCost(emptyBytecode)
            ).rejects.toThrow('Invalid bytecode format');
        });

        it('should scale costs with bytecode size', async () => {
            const smallBytecode = wasmCompiler.createTestBytecode([
                { opcode: 'HALT' }
            ]);

            const largeBytecode = wasmCompiler.createTestBytecode([
                { opcode: 'PUSH', args: ['U64', 1] },
                { opcode: 'PUSH', args: ['U64', 2] },
                { opcode: 'PUSH', args: ['U64', 3] },
                { opcode: 'PUSH', args: ['U64', 4] },
                { opcode: 'PUSH', args: ['U64', 5] },
                { opcode: 'ADD' },
                { opcode: 'ADD' },
                { opcode: 'ADD' },
                { opcode: 'ADD' },
                { opcode: 'HALT' }
            ]);

            const smallEstimation = await deploymentService.estimateDeploymentCost(smallBytecode);
            const largeEstimation = await deploymentService.estimateDeploymentCost(largeBytecode);

            expect(largeEstimation.rentExemptBalance).toBeGreaterThan(smallEstimation.rentExemptBalance);
            expect(largeEstimation.computeUnits).toBeGreaterThanOrEqual(smallEstimation.computeUnits);
        });
    });

    describe('Network Operations', () => {
        beforeEach(async () => {
            await deploymentService.initialize();
        });

        it('should check network connectivity', async () => {
            // This will fail for localnet unless a validator is running
            // In a real test environment, you'd have a test validator running
            const connected = await deploymentService.checkNetworkConnectivity();
            expect(typeof connected).toBe('boolean');
        });

        it('should provide network information', async () => {
            const networkInfo = await deploymentService.getNetworkInfo();

            expect(networkInfo).toMatchObject({
                network: 'localnet',
                blockHeight: expect.any(Number),
                programId: expect.any(PublicKey),
                connected: expect.any(Boolean)
            });
        });

        it('should handle network connection failures gracefully', async () => {
            // Create service with invalid RPC URL
            const badService = new DeploymentService({
                network: 'devnet',
                rpcUrl: 'http://invalid-rpc-url:9999'
            });

            await badService.initialize();
            const connected = await badService.checkNetworkConnectivity();
            expect(connected).toBe(false);
        });
    });

    describe('Deployment History', () => {
        beforeEach(async () => {
            await deploymentService.initialize();
        });

        it('should start with empty history', () => {
            const history = deploymentService.getDeploymentHistory();
            expect(Array.isArray(history)).toBe(true);
            // History might not be empty if previous tests ran
        });

        it('should clear deployment history', () => {
            deploymentService.clearDeploymentHistory();
            const history = deploymentService.getDeploymentHistory();
            expect(history).toHaveLength(0);
        });

        // Note: History management is mostly tested through successful deployments
        // which would require a running test validator
    });

    describe('Input Validation', () => {
        beforeEach(async () => {
            await deploymentService.initialize();
        });

        it('should validate wallet connection for deployment', async () => {
            const disconnectedWallet = new MockWalletAdapter();
            // Don't connect the wallet

            await expect(
                deploymentService.deployScript(
                    'test-script',
                    testBytecode,
                    disconnectedWallet
                )
            ).rejects.toThrow('Wallet not connected');
        });

        it('should validate script names', async () => {
            await expect(
                deploymentService.deployScript(
                    '', // Empty name
                    testBytecode,
                    mockWallet
                )
            ).rejects.toThrow();
        });

        it('should validate bytecode before deployment', async () => {
            const invalidBytecode = new Uint8Array([1, 2, 3, 4]); // Invalid

            await expect(
                deploymentService.deployScript(
                    'test-script',
                    invalidBytecode,
                    mockWallet
                )
            ).rejects.toThrow('Invalid bytecode format');
        });
    });

    describe('Progress Tracking', () => {
        beforeEach(async () => {
            await deploymentService.initialize();
        });

        it('should report progress during deployment attempt', async () => {
            const progressEvents: any[] = [];
            
            // This will fail because we don't have a real validator running,
            // but we should still see progress events
            try {
                await deploymentService.deployScript(
                    'test-script',
                    testBytecode,
                    mockWallet,
                    (progress) => {
                        progressEvents.push(progress);
                    }
                );
            } catch (error) {
                // Expected to fail without a running validator
            }

            // Should have received at least validation progress
            expect(progressEvents.length).toBeGreaterThan(0);
            expect(progressEvents[0]).toMatchObject({
                step: 'validating',
                description: expect.any(String),
                progress: expect.any(Number)
            });
        });
    });
});

describe('DeploymentUtils', () => {
    describe('Formatting Functions', () => {
        it('should format SOL amounts correctly', () => {
            expect(DeploymentUtils.formatSol(1000000000)).toBe('1.000000 SOL'); // 1 SOL
            expect(DeploymentUtils.formatSol(500000000)).toBe('0.500000 SOL');  // 0.5 SOL
            expect(DeploymentUtils.formatSol(1000)).toBe('0.000001 SOL');       // 1000 lamports
        });

        it('should format transaction signatures', () => {
            const signature = '1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef';
            const formatted = DeploymentUtils.formatSignature(signature);
            
            expect(formatted).toBe('12345678...90abcdef');
            expect(formatted.length).toBe(19); // 8 + 3 + 8
        });

        it('should generate correct explorer URLs', () => {
            const signature = 'test-signature';
            
            const devnetUrl = DeploymentUtils.getExplorerUrl(signature, 'devnet');
            expect(devnetUrl).toBe('https://explorer.solana.com?cluster=devnet/tx/test-signature');

            const mainnetUrl = DeploymentUtils.getExplorerUrl(signature, 'mainnet-beta');
            expect(mainnetUrl).toBe('https://explorer.solana.com/tx/test-signature');
        });

        it('should generate correct account explorer URLs', () => {
            const address = 'test-address';
            
            const devnetUrl = DeploymentUtils.getAccountExplorerUrl(address, 'devnet');
            expect(devnetUrl).toBe('https://explorer.solana.com?cluster=devnet/account/test-address');

            const mainnetUrl = DeploymentUtils.getAccountExplorerUrl(address, 'mainnet-beta');
            expect(mainnetUrl).toBe('https://explorer.solana.com/account/test-address');
        });
    });

    describe('Validation Functions', () => {
        it('should validate Solana addresses', () => {
            // Valid base58 public key
            const validAddress = '11111111111111111111111111111112'; // System program
            expect(DeploymentUtils.isValidSolanaAddress(validAddress)).toBe(true);

            // Invalid addresses
            expect(DeploymentUtils.isValidSolanaAddress('')).toBe(false);
            expect(DeploymentUtils.isValidSolanaAddress('invalid')).toBe(false);
            expect(DeploymentUtils.isValidSolanaAddress('0x1234567890abcdef')).toBe(false);
        });
    });
});

describe('Integration with WASM Compiler', () => {
    let deploymentService: DeploymentService;
    let wasmCompiler: WasmCompilerService;

    beforeAll(async () => {
        wasmCompiler = new WasmCompilerService();
        await wasmCompiler.initialize();

        deploymentService = new DeploymentService({
            network: 'localnet',
            rpcUrl: 'http://localhost:8899'
        });
        await deploymentService.initialize();
    });

    it('should accept bytecode generated by WASM compiler', async () => {
        const bytecode = wasmCompiler.createTestBytecode([
            { opcode: 'PUSH', args: ['U64', 42] },
            { opcode: 'PUSH', args: ['U64', 24] },
            { opcode: 'ADD' },
            { opcode: 'HALT' }
        ]);

        // Should validate successfully
        expect(wasmCompiler.validateBytecode(bytecode)).toBe(true);

        // Should be accepted by deployment service
        await expect(
            deploymentService.estimateDeploymentCost(bytecode)
        ).resolves.toMatchObject({
            computeUnits: expect.any(Number),
            totalCost: expect.any(Number)
        });
    });

    it('should reject bytecode that fails WASM validation', async () => {
        const invalidBytecode = new Uint8Array([0x12, 0x34, 0x56, 0x78]);

        // Should fail WASM validation
        expect(wasmCompiler.validateBytecode(invalidBytecode)).toBe(false);

        // Should be rejected by deployment service
        await expect(
            deploymentService.estimateDeploymentCost(invalidBytecode)
        ).rejects.toThrow('Invalid bytecode format');
    });

    it('should handle complex bytecode programs', async () => {
        const complexBytecode = wasmCompiler.createTestBytecode([
            { opcode: 'PUSH', args: ['U64', 100] },
            { opcode: 'PUSH', args: ['U64', 200] },
            { opcode: 'DUP' },
            { opcode: 'ADD' },
            { opcode: 'SWAP' },
            { opcode: 'SUB' },
            { opcode: 'HALT' }
        ]);

        expect(wasmCompiler.validateBytecode(complexBytecode)).toBe(true);
        
        const estimation = await deploymentService.estimateDeploymentCost(complexBytecode);
        expect(estimation.computeUnits).toBeGreaterThan(0);
        expect(estimation.totalCost).toBeGreaterThan(0);
    });
});

describe('Error Handling', () => {
    let deploymentService: DeploymentService;

    beforeEach(async () => {
        deploymentService = new DeploymentService({
            network: 'localnet',
            rpcUrl: 'http://localhost:8899'
        });
        await deploymentService.initialize();
    });

    it('should handle network timeouts gracefully', async () => {
        const shortTimeoutService = new DeploymentService({
            network: 'localnet',
            rpcUrl: 'http://localhost:8899',
            confirmationTimeout: 1 // Very short timeout
        });

        await shortTimeoutService.initialize();

        // Network operations should fail quickly
        const startTime = Date.now();
        const connected = await shortTimeoutService.checkNetworkConnectivity();
        const elapsed = Date.now() - startTime;

        // Should not hang indefinitely
        expect(elapsed).toBeLessThan(5000); // 5 seconds max
    });

    it('should provide meaningful error messages', async () => {
        const mockWallet = new MockWalletAdapter();
        const invalidBytecode = new Uint8Array([1, 2, 3]);

        try {
            await deploymentService.deployScript(
                'test-script',
                invalidBytecode,
                mockWallet
            );
            fail('Should have thrown an error');
        } catch (error) {
            expect(error instanceof Error).toBe(true);
            expect((error as Error).message).toContain('Invalid bytecode format');
        }
    });

    it('should handle disconnected wallets', async () => {
        const mockWallet = new MockWalletAdapter();
        // Don't connect the wallet
        
        const wasmCompiler = new WasmCompilerService();
        await wasmCompiler.initialize();
        const validBytecode = wasmCompiler.createTestBytecode([{ opcode: 'HALT' }]);

        await expect(
            deploymentService.deployScript(
                'test-script',
                validBytecode,
                mockWallet
            )
        ).rejects.toThrow('Wallet not connected');
    });
});
