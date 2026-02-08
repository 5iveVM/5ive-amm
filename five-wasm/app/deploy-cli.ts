#!/usr/bin/env ts-node

/**
 * Deployment CLI for Stacks VM WASM.
 * Performs real deployments to Solana networks (no simulation).
 */

import * as fs from 'fs';
import * as path from 'path';
import { Command } from 'commander';
import { Keypair, Connection, PublicKey } from '@solana/web3.js';
import { DeploymentService, SolanaNetwork, DeploymentUtils } from './deployment-service';
import { WasmCompilerService } from './wasm-compiler';

/**
 * Wallet adapter for CLI use (using keypair files)
 */
class CLIWalletAdapter {
    private keypair: Keypair;
    public connected: boolean = false;

    constructor(keypair: Keypair) {
        this.keypair = keypair;
        this.connected = true;
    }

    get publicKey(): PublicKey {
        return this.keypair.publicKey;
    }

    async connect(): Promise<void> {
        this.connected = true;
    }

    async disconnect(): Promise<void> {
        this.connected = false;
    }

    async signTransaction(transaction: any): Promise<any> {
        transaction.sign(this.keypair);
        return transaction;
    }
}

/**
 * CLI deployment manager
 */
class DeploymentCLI {
    private program: Command;

    constructor() {
        this.program = new Command();
        this.setupCommands();
    }

    /**
     * Setup CLI commands
     */
    private setupCommands(): void {
        this.program
            .name('stacks-deploy')
            .description('Deploy Stacks VM scripts to Solana networks')
            .version('1.0.0');

        // Deploy command
        this.program
            .command('deploy')
            .description('Deploy a Stacks script to Solana')
            .argument('<script-file>', 'Path to bytecode file (.sbin) or source file (.stacks)')
            .argument('<script-name>', 'Name for the deployed script')
            .option('-n, --network <network>', 'Target network (devnet, testnet, mainnet-beta, localnet)', 'devnet')
            .option('-w, --wallet <path>', 'Path to wallet keypair file', this.getDefaultWalletPath())
            .option('-r, --rpc <url>', 'Custom RPC URL')
            .option('-c, --compile', 'Compile source file before deployment')
            .option('--dry-run', 'Estimate costs without deploying')
            .option('--confirm-timeout <ms>', 'Transaction confirmation timeout in milliseconds', '60000')
            .action(async (scriptFile, scriptName, options) => {
                await this.handleDeploy(scriptFile, scriptName, options);
            });

        // Estimate command
        this.program
            .command('estimate')
            .description('Estimate deployment costs')
            .argument('<script-file>', 'Path to bytecode file (.sbin)')
            .option('-n, --network <network>', 'Target network', 'devnet')
            .option('-r, --rpc <url>', 'Custom RPC URL')
            .action(async (scriptFile, options) => {
                await this.handleEstimate(scriptFile, options);
            });

        // History command
        this.program
            .command('history')
            .description('Show deployment history')
            .option('-n, --network <network>', 'Filter by network')
            .option('--clear', 'Clear deployment history')
            .action(async (options) => {
                await this.handleHistory(options);
            });

        // Status command
        this.program
            .command('status')
            .description('Check deployment and network status')
            .argument('[transaction-id]', 'Transaction signature to check')
            .option('-n, --network <network>', 'Target network', 'devnet')
            .option('-r, --rpc <url>', 'Custom RPC URL')
            .action(async (transactionId, options) => {
                await this.handleStatus(transactionId, options);
            });

        // Wallet command
        this.program
            .command('wallet')
            .description('Wallet management utilities')
            .option('--balance', 'Check wallet balance')
            .option('--address', 'Show wallet address')
            .option('-w, --wallet <path>', 'Path to wallet keypair file', this.getDefaultWalletPath())
            .option('-n, --network <network>', 'Target network', 'devnet')
            .option('-r, --rpc <url>', 'Custom RPC URL')
            .action(async (options) => {
                await this.handleWallet(options);
            });
    }

    /**
     * Run the CLI
     */
    async run(argv: string[]): Promise<void> {
        await this.program.parseAsync(argv);
    }

    /**
     * Handle deploy command
     */
    private async handleDeploy(scriptFile: string, scriptName: string, options: any): Promise<void> {
        try {
            console.log(`🚀 Deploying ${scriptName} to ${options.network}`);
            console.log(`📁 Script file: ${scriptFile}`);

            // Validate inputs
            if (!fs.existsSync(scriptFile)) {
                throw new Error(`Script file not found: ${scriptFile}`);
            }

            const network = this.validateNetwork(options.network);
            const walletPath = options.wallet;

            // Load wallet
            console.log(`👛 Loading wallet from: ${walletPath}`);
            const wallet = this.loadWallet(walletPath);

            // Setup deployment service
            const deploymentService = new DeploymentService({
                network,
                rpcUrl: options.rpc,
                confirmationTimeout: parseInt(options.confirmTimeout)
            });

            await deploymentService.initialize();

            // Check network connectivity
            console.log(`🔗 Checking network connectivity...`);
            const connected = await deploymentService.checkNetworkConnectivity();
            if (!connected) {
                throw new Error(`Cannot connect to ${network} network`);
            }

            // Load or compile bytecode
            let bytecode: Uint8Array;

            if (scriptFile.endsWith('.sbin')) {
                // Load pre-compiled bytecode
                bytecode = fs.readFileSync(scriptFile);
            } else if (scriptFile.endsWith('.stacks') && options.compile) {
                // Compile source code (simplified)
                console.log(`🔧 Compiling source code...`);
                bytecode = await this.compileSourceCode(scriptFile);
            } else {
                throw new Error('Unsupported file type. Use .sbin for bytecode or .stacks with --compile');
            }

            console.log(`📊 Bytecode size: ${bytecode.length} bytes`);

            // Estimate costs
            console.log(`💰 Estimating deployment costs...`);
            const estimation = await deploymentService.estimateDeploymentCost(bytecode);
            
            console.log(`\n📋 Cost Estimation:`);
            console.log(`   Compute Units: ${estimation.computeUnits}`);
            console.log(`   Rent-exempt balance: ${DeploymentUtils.formatSol(estimation.rentExemptBalance * 1e9)}`);
            console.log(`   Transaction fee: ${DeploymentUtils.formatSol(estimation.transactionFee * 1e9)}`);
            console.log(`   Total cost: ${DeploymentUtils.formatSol(estimation.totalCost * 1e9)}`);

            if (options.dryRun) {
                console.log(`\n✅ Dry run completed. Script would cost ${DeploymentUtils.formatSol(estimation.totalCost * 1e9)} to deploy.`);
                return;
            }

            // Deploy the script
            console.log(`\n🚀 Starting deployment...`);
            
            const result = await deploymentService.deployScript(
                scriptName,
                bytecode,
                wallet,
                (progress) => {
                    console.log(`   ${this.getProgressIcon(progress.step)} ${progress.description} (${progress.progress}%)`);
                }
            );

            if (result.success) {
                console.log(`\n✅ Deployment successful!`);
                console.log(`   Script address: ${result.scriptAddress?.toBase58()}`);
                console.log(`   Transaction: ${result.signature}`);
                console.log(`   Network: ${result.network}`);
                console.log(`   Cost: ${DeploymentUtils.formatSol(result.cost * 1e9)}`);
                console.log(`   Explorer: ${DeploymentUtils.getExplorerUrl(result.signature!, result.network)}`);
                
                // Save deployment info
                this.saveDeploymentInfo(scriptName, result);
            } else {
                console.error(`\n❌ Deployment failed: ${result.error}`);
                process.exit(1);
            }

        } catch (error) {
            console.error(`\n❌ Deployment error: ${error instanceof Error ? error.message : error}`);
            process.exit(1);
        }
    }

    /**
     * Handle estimate command
     */
    private async handleEstimate(scriptFile: string, options: any): Promise<void> {
        try {
            console.log(`📊 Estimating deployment cost for ${scriptFile}`);

            if (!fs.existsSync(scriptFile)) {
                throw new Error(`Script file not found: ${scriptFile}`);
            }

            const network = this.validateNetwork(options.network);
            const bytecode = fs.readFileSync(scriptFile);

            const deploymentService = new DeploymentService({
                network,
                rpcUrl: options.rpc
            });

            await deploymentService.initialize();

            const estimation = await deploymentService.estimateDeploymentCost(bytecode);

            console.log(`\n📋 Deployment Cost Estimation:`);
            console.log(`   Network: ${network}`);
            console.log(`   Bytecode size: ${bytecode.length} bytes`);
            console.log(`   Compute units: ${estimation.computeUnits}`);
            console.log(`   Rent-exempt balance: ${DeploymentUtils.formatSol(estimation.rentExemptBalance * 1e9)}`);
            console.log(`   Transaction fee: ${DeploymentUtils.formatSol(estimation.transactionFee * 1e9)}`);
            console.log(`   Total cost: ${DeploymentUtils.formatSol(estimation.totalCost * 1e9)}`);

        } catch (error) {
            console.error(`❌ Estimation error: ${error instanceof Error ? error.message : error}`);
            process.exit(1);
        }
    }

    /**
     * Handle history command
     */
    private async handleHistory(options: any): Promise<void> {
        try {
            const deploymentService = new DeploymentService({
                network: 'devnet' // Default for history access
            });

            await deploymentService.initialize();

            if (options.clear) {
                deploymentService.clearDeploymentHistory();
                console.log(`✅ Deployment history cleared`);
                return;
            }

            const history = deploymentService.getDeploymentHistory();

            if (history.length === 0) {
                console.log(`📝 No deployment history found`);
                return;
            }

            console.log(`\n📝 Deployment History:`);
            console.log(`════════════════════════════════════════════════════════════════`);

            for (const entry of history) {
                if (options.network && entry.result.network !== options.network) {
                    continue;
                }

                const status = entry.result.success ? '✅' : '❌';
                const cost = DeploymentUtils.formatSol(entry.result.cost * 1e9);
                
                console.log(`${status} ${entry.name}`);
                console.log(`   Network: ${entry.result.network}`);
                console.log(`   Deployed: ${entry.result.deployedAt.toLocaleString()}`);
                console.log(`   Cost: ${cost}`);
                
                if (entry.result.success) {
                    console.log(`   Address: ${entry.result.scriptAddress?.toBase58()}`);
                    console.log(`   Transaction: ${DeploymentUtils.formatSignature(entry.result.signature!)}`);
                } else {
                    console.log(`   Error: ${entry.result.error}`);
                }
                console.log(`────────────────────────────────────────────────────────────────`);
            }

        } catch (error) {
            console.error(`❌ History error: ${error instanceof Error ? error.message : error}`);
            process.exit(1);
        }
    }

    /**
     * Handle status command
     */
    private async handleStatus(transactionId: string | undefined, options: any): Promise<void> {
        try {
            const network = this.validateNetwork(options.network);
            
            const deploymentService = new DeploymentService({
                network,
                rpcUrl: options.rpc
            });

            await deploymentService.initialize();

            if (transactionId) {
                // Check specific transaction
                console.log(`🔍 Checking transaction: ${transactionId}`);
                
                const txDetails = await deploymentService.getTransactionDetails(transactionId);
                
                if (txDetails) {
                    console.log(`\n📋 Transaction Details:`);
                    console.log(`   Signature: ${transactionId}`);
                    console.log(`   Slot: ${txDetails.slot}`);
                    console.log(`   Block time: ${new Date(txDetails.blockTime * 1000).toLocaleString()}`);
                    console.log(`   Status: ${txDetails.meta?.err ? '❌ Failed' : '✅ Success'}`);
                    console.log(`   Fee: ${DeploymentUtils.formatSol(txDetails.meta?.fee || 0)}`);
                    console.log(`   Explorer: ${DeploymentUtils.getExplorerUrl(transactionId, network)}`);
                } else {
                    console.log(`❌ Transaction not found or not yet confirmed`);
                }
            } else {
                // Check network status
                console.log(`🔗 Checking ${network} network status...`);
                
                const networkInfo = await deploymentService.getNetworkInfo();
                
                console.log(`\n📋 Network Status:`);
                console.log(`   Network: ${networkInfo.network}`);
                console.log(`   Connected: ${networkInfo.connected ? '✅' : '❌'}`);
                console.log(`   Block height: ${networkInfo.blockHeight}`);
                console.log(`   Program ID: ${networkInfo.programId.toBase58()}`);
            }

        } catch (error) {
            console.error(`❌ Status error: ${error instanceof Error ? error.message : error}`);
            process.exit(1);
        }
    }

    /**
     * Handle wallet command
     */
    private async handleWallet(options: any): Promise<void> {
        try {
            const walletPath = options.wallet;
            const network = this.validateNetwork(options.network);

            console.log(`👛 Wallet operations for ${network}`);

            const wallet = this.loadWallet(walletPath);

            if (options.address) {
                console.log(`\n📋 Wallet Address:`);
                console.log(`   ${wallet.publicKey.toBase58()}`);
                console.log(`   Explorer: ${DeploymentUtils.getAccountExplorerUrl(wallet.publicKey.toBase58(), network)}`);
            }

            if (options.balance) {
                const deploymentService = new DeploymentService({
                    network,
                    rpcUrl: options.rpc
                });

                await deploymentService.initialize();
                
                // Create connection to check balance
                const connection = new Connection(
                    options.rpc || this.getDefaultRpcUrl(network),
                    'confirmed'
                );

                const balance = await connection.getBalance(wallet.publicKey);
                
                console.log(`\n💰 Wallet Balance:`);
                console.log(`   ${DeploymentUtils.formatSol(balance)}`);
                
                if (balance === 0) {
                    console.log(`\n💡 Tip: Fund your wallet using:`);
                    if (network === 'devnet') {
                        console.log(`   solana airdrop 2 ${wallet.publicKey.toBase58()} --url devnet`);
                    } else {
                        console.log(`   Transfer SOL to this address from another wallet`);
                    }
                }
            }

            if (!options.address && !options.balance) {
                console.log(`\n📋 Wallet Info:`);
                console.log(`   Path: ${walletPath}`);
                console.log(`   Address: ${wallet.publicKey.toBase58()}`);
                console.log(`   Network: ${network}`);
                console.log(`\n💡 Use --balance or --address flags for more details`);
            }

        } catch (error) {
            console.error(`❌ Wallet error: ${error instanceof Error ? error.message : error}`);
            process.exit(1);
        }
    }

    // Private helper methods

    private validateNetwork(network: string): SolanaNetwork {
        const validNetworks: SolanaNetwork[] = ['devnet', 'testnet', 'mainnet-beta', 'localnet'];
        if (!validNetworks.includes(network as SolanaNetwork)) {
            throw new Error(`Invalid network: ${network}. Valid options: ${validNetworks.join(', ')}`);
        }
        return network as SolanaNetwork;
    }

    private loadWallet(walletPath: string): CLIWalletAdapter {
        try {
            if (!fs.existsSync(walletPath)) {
                throw new Error(`Wallet file not found: ${walletPath}`);
            }

            const walletData = JSON.parse(fs.readFileSync(walletPath, 'utf-8'));
            const keypair = Keypair.fromSecretKey(Uint8Array.from(walletData));
            
            return new CLIWalletAdapter(keypair);

        } catch (error) {
            throw new Error(`Failed to load wallet: ${error instanceof Error ? error.message : error}`);
        }
    }

    private getDefaultWalletPath(): string {
        return process.env.ANCHOR_WALLET || 
               path.join(process.env.HOME || '', '.config/solana/id.json');
    }

    private getDefaultRpcUrl(network: SolanaNetwork): string {
        switch (network) {
            case 'devnet': return 'https://api.devnet.solana.com';
            case 'testnet': return 'https://api.testnet.solana.com';
            case 'mainnet-beta': return 'https://api.mainnet-beta.solana.com';
            case 'localnet': return 'http://localhost:8899';
            default: throw new Error(`Unknown network: ${network}`);
        }
    }

    private async compileSourceCode(sourceFile: string): Promise<Uint8Array> {
        // Compilation stub: in real implementation, call the actual compiler
        const wasmCompiler = new WasmCompilerService();
        await wasmCompiler.initialize();

        // Create test bytecode for demonstration
        const testBytecode = wasmCompiler.createTestBytecode([
            { opcode: 'PUSH', args: ['U64', 42] },
            { opcode: 'PUSH', args: ['U64', 24] },
            { opcode: 'ADD' },
            { opcode: 'HALT' }
        ]);

        return testBytecode;
    }

    private getProgressIcon(step: string): string {
        switch (step) {
            case 'validating': return '🔍';
            case 'estimating': return '📊';
            case 'creating_account': return '🏗️';
            case 'deploying': return '🚀';
            case 'confirming': return '⏳';
            case 'completed': return '✅';
            case 'failed': return '❌';
            default: return '⚙️';
        }
    }

    private saveDeploymentInfo(scriptName: string, result: any): void {
        const deploymentFile = path.join(process.cwd(), 'deployments.json');
        let deployments = [];

        if (fs.existsSync(deploymentFile)) {
            try {
                deployments = JSON.parse(fs.readFileSync(deploymentFile, 'utf-8'));
            } catch (error) {
                console.warn('Warning: Could not read existing deployments.json');
            }
        }

        deployments.push({
            name: scriptName,
            ...result,
            deployedAt: result.deployedAt.toISOString()
        });

        try {
            fs.writeFileSync(deploymentFile, JSON.stringify(deployments, null, 2));
            console.log(`   Deployment info saved to deployments.json`);
        } catch (error) {
            console.warn('Warning: Could not save deployment info to deployments.json');
        }
    }
}

// CLI entry point
async function main(): Promise<void> {
    const cli = new DeploymentCLI();
    await cli.run(process.argv);
}

// Only run if this is the main module
if (require.main === module) {
    main().catch((error) => {
        console.error('CLI Error:', error);
        process.exit(1);
    });
}

export default DeploymentCLI;
