#!/usr/bin/env ts-node

/**
 * Stacks Playground Demo
 * 
 * Demonstrates the complete development workflow using the Stacks Playground:
 * - Creating and managing projects
 * - Compiling Stacks code to bytecode
 * - Testing with WASM VM
 * - Deploying to Solana networks
 * 
 * This demo shows REAL operations - compilation, testing, and deployment
 * to actual Solana networks (when configured).
 */

import StacksPlayground from '../app/playground';
import { Keypair } from '@solana/web3.js';
import { WalletAdapter } from '@solana/wallet-adapter-base';

/**
 * Demo wallet adapter using a generated keypair
 */
class DemoWalletAdapter implements WalletAdapter {
    private keypair: Keypair;
    public connected: boolean = false;
    public connecting: boolean = false;
    public disconnecting: boolean = false;

    constructor() {
        this.keypair = Keypair.generate();
    }

    get publicKey() {
        return this.keypair.publicKey;
    }

    get name(): string {
        return 'Demo Wallet';
    }

    get url(): string {
        return 'https://demo-wallet.example.com';
    }

    get icon(): string {
        return '';
    }

    get readyState(): any {
        return 'Installed';
    }

    async connect(): Promise<void> {
        console.log('🔗 Connecting demo wallet...');
        this.connecting = true;
        await new Promise(resolve => setTimeout(resolve, 1000));
        this.connected = true;
        this.connecting = false;
        console.log(`✅ Wallet connected: ${this.publicKey.toBase58()}`);
    }

    async disconnect(): Promise<void> {
        console.log('🔌 Disconnecting wallet...');
        this.disconnecting = true;
        await new Promise(resolve => setTimeout(resolve, 500));
        this.connected = false;
        this.disconnecting = false;
        console.log('✅ Wallet disconnected');
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

    on(event: string, handler: Function): void {}
    off(event: string, handler: Function): void {}
}

/**
 * Demo script with various playground features
 */
async function playgroundDemo() {
    console.log('🚀 Starting Stacks Playground Demo\n');

    // Initialize playground
    console.log('📋 Initializing playground...');
    const playground = new StacksPlayground({
        defaultNetwork: 'devnet',
        autoCompile: false,
        autoTest: false,
        maxBytecodeSize: 1024 * 1024,
        theme: 'light'
    });

    // Setup event listeners for demo
    playground.on('projectCreated', (project) => {
        console.log(`✅ Project created: ${project.name}`);
    });

    playground.on('compilationSuccess', (data) => {
        console.log(`✅ Compilation successful: ${data.bytecode.length} bytes`);
    });

    playground.on('testCompleted', (result) => {
        console.log(`✅ Test completed: ${result.outcome} (${result.operations_tested.length} operations tested)`);
    });

    playground.on('deploymentCompleted', (result) => {
        if (result.success) {
            console.log(`✅ Deployment successful: ${result.scriptAddress?.toBase58()}`);
        } else {
            console.log(`❌ Deployment failed: ${result.error}`);
        }
    });

    await playground.initialize();
    console.log('✅ Playground initialized\n');

    // Demo 1: Project Management
    console.log('📁 Demo 1: Project Management');
    console.log('═══════════════════════════════');

    const vaultCode = `
// Advanced Vault Contract with Access Control
function init() {
    // Initialize vault state
    push_u64(0);           // Initial balance
    store_field(0, 0);     // Store at offset 0
    
    push_u64(0);           // Admin count
    store_field(0, 8);     // Store at offset 8
    
    push_u64(1);           // Vault active flag
    store_field(0, 16);    // Store at offset 16
}

function deposit(amount: u64) {
    // Check vault is active
    load_field(0, 16);
    push_u64(1);
    eq();
    require("Vault is not active");
    
    // Validate amount
    push_u64(amount);
    push_u64(0);
    gt();
    require("Deposit amount must be greater than zero");
    
    // Load current balance
    load_field(0, 0);
    
    // Add deposit amount
    push_u64(amount);
    add();
    
    // Check for overflow
    dup();
    push_u64(amount);
    gt();
    require("Deposit would cause overflow");
    
    // Store new balance
    store_field(0, 0);
}

function withdraw(amount: u64) {
    // Check vault is active
    load_field(0, 16);
    push_u64(1);
    eq();
    require("Vault is not active");
    
    // Load current balance
    load_field(0, 0);
    
    // Check sufficient balance
    push_u64(amount);
    dup();
    gt();
    require("Insufficient balance for withdrawal");
    
    // Subtract withdrawal amount
    sub();
    
    // Store new balance
    store_field(0, 0);
}

function get_balance(): u64 {
    load_field(0, 0);
}

function emergency_pause() {
    // Only admin can pause (simplified for demo)
    push_u64(0);
    store_field(0, 16);    // Set active flag to 0
}

function emergency_unpause() {
    // Only admin can unpause (simplified for demo)
    push_u64(1);
    store_field(0, 16);    // Set active flag to 1
}
`;

    const advancedVault = await playground.createProject(
        'Advanced Vault',
        vaultCode,
        'stacks'
    );

    console.log(`Project created with ${advancedVault.sourceCode.split('\n').length} lines of code\n`);

    // Demo 2: Compilation
    console.log('⚙️  Demo 2: Compilation');
    console.log('══════════════════════');

    console.log('Compiling Advanced Vault...');
    const bytecode = await playground.compileCurrentProject();
    console.log(`Generated bytecode: ${bytecode.length} bytes`);
    console.log(`Magic bytes: ${Array.from(bytecode.slice(0, 4)).map(b => `0x${b.toString(16).padStart(2, '0')}`).join(' ')}`);
    console.log('');

    // Demo 3: Testing with WASM VM
    console.log('🧪 Demo 3: WASM VM Testing');
    console.log('═══════════════════════════');

    console.log('Testing compiled bytecode...');
    const testResult = await playground.testCurrentProject();
    
    console.log(`Test outcome: ${testResult.outcome}`);
    console.log(`Description: ${testResult.description}`);
    console.log(`Operations tested: ${testResult.operations_tested.join(', ')}`);
    console.log(`Compute units used: ${testResult.final_state.compute_units_used}`);
    console.log(`Test success: ${testResult.test_success ? '✅' : '❌'}`);
    console.log('');

    // Demo 4: Advanced Testing with Custom Accounts
    console.log('🔬 Demo 4: Advanced Testing');
    console.log('═══════════════════════════');

    // Create a test account that simulates a vault account
    const vaultAccount = {
        key: new Uint8Array(32).fill(1), // Fake vault account key
        data: new Uint8Array(100),       // 100 bytes for vault data
        lamports: BigInt(1000000),       // 0.001 SOL
        isWritable: true,
        isSigner: false,
        owner: new Uint8Array(32).fill(0) // System program owner
    };

    console.log('Testing with custom vault account...');
    const advancedTestResult = await playground.testCurrentProject([vaultAccount]);
    
    console.log(`Advanced test outcome: ${advancedTestResult.outcome}`);
    console.log(`Stack size after execution: ${advancedTestResult.final_state.stack_size}`);
    console.log(`Has result value: ${advancedTestResult.final_state.has_result}`);
    console.log('');

    // Demo 5: Create Multiple Projects
    console.log('📚 Demo 5: Multiple Projects');
    console.log('════════════════════════════');

    const simpleToken = await playground.createProject(
        'Simple Token',
        `
function init_token(supply: u64) {
    // Initialize total supply
    push_u64(supply);
    store_field(0, 0);
    
    // Initialize owner balance to total supply
    push_u64(supply);
    store_field(0, 8);
}

function transfer(amount: u64) {
    // Load sender balance (simplified)
    load_field(0, 8);
    
    // Check sufficient balance
    push_u64(amount);
    dup();
    gt();
    require("Insufficient token balance");
    
    // Deduct from sender
    sub();
    store_field(0, 8);
}

function get_supply(): u64 {
    load_field(0, 0);
}
`,
        'stacks'
    );

    await playground.compileCurrentProject();
    const tokenTestResult = await playground.testCurrentProject();
    console.log(`Token project compiled and tested: ${tokenTestResult.test_success ? '✅' : '❌'}`);

    // Show all projects
    const allProjects = playground.getProjects();
    console.log(`\nTotal projects in playground: ${allProjects.length}`);
    allProjects.forEach((project, index) => {
        const status = project.bytecode ? '✅ Compiled' : '⚠️  Not compiled';
        const lastTest = project.lastTestResult?.test_success ? '✅ Tested' : '❌ Not tested';
        console.log(`  ${index + 1}. ${project.name} - ${status}, ${lastTest}`);
    });
    console.log('');

    // Demo 6: Wallet Integration
    console.log('👛 Demo 6: Wallet Integration');
    console.log('═════════════════════════════');

    const demoWallet = new DemoWalletAdapter();
    await playground.connectWallet(demoWallet);
    
    const state = playground.getState();
    console.log(`Wallet status: ${state.wallet ? 'Connected' : 'Disconnected'}`);
    console.log(`Wallet address: ${demoWallet.publicKey.toBase58()}`);
    console.log('');

    // Demo 7: Deployment Simulation (will fail without running validator)
    console.log('🚀 Demo 7: Deployment Process');
    console.log('═════════════════════════════');

    console.log('Note: This will demonstrate the deployment process but will fail without a running Solana validator');
    
    try {
        // Switch back to Advanced Vault for deployment
        await playground.loadProject('Advanced Vault');
        
        console.log('Attempting deployment to localnet...');
        const deploymentResult = await playground.deployCurrentProject(
            'demo-advanced-vault',
            'localnet',
            demoWallet
        );
        
        if (deploymentResult.success) {
            console.log(`✅ Deployment successful!`);
            console.log(`   Script address: ${deploymentResult.scriptAddress?.toBase58()}`);
            console.log(`   Transaction: ${deploymentResult.signature}`);
            console.log(`   Cost: ${deploymentResult.cost} SOL`);
        } else {
            console.log(`❌ Deployment failed: ${deploymentResult.error}`);
        }
    } catch (error) {
        console.log(`❌ Deployment error (expected without running validator): ${error instanceof Error ? error.message : error}`);
    }
    console.log('');

    // Demo 8: Project Statistics
    console.log('📊 Demo 8: Project Statistics');
    console.log('═════════════════════════════');

    const finalState = playground.getState();
    console.log('Playground Statistics:');
    console.log(`  Total projects: ${finalState.projects.length}`);
    console.log(`  Current project: ${finalState.currentProject?.name || 'None'}`);
    console.log(`  Compiled projects: ${finalState.projects.filter(p => p.bytecode).length}`);
    console.log(`  Tested projects: ${finalState.projects.filter(p => p.lastTestResult).length}`);
    console.log(`  Successful tests: ${finalState.projects.filter(p => p.lastTestResult?.test_success).length}`);
    
    // Calculate total lines of code
    const totalLines = finalState.projects.reduce((total, project) => {
        return total + project.sourceCode.split('\n').length;
    }, 0);
    console.log(`  Total lines of code: ${totalLines}`);
    
    // Calculate total bytecode size
    const totalBytecodeSize = finalState.projects.reduce((total, project) => {
        return total + (project.bytecode?.length || 0);
    }, 0);
    console.log(`  Total bytecode size: ${totalBytecodeSize} bytes`);
    console.log('');

    // Demo 9: Cleanup and Export
    console.log('🧹 Demo 9: Cleanup and Export');
    console.log('═════════════════════════════');

    // Show deployment history
    const deploymentHistory = playground.getState().deployment.lastResult;
    if (deploymentHistory) {
        console.log('Last deployment attempt:', deploymentHistory.success ? 'Success' : 'Failed');
    }

    // Disconnect wallet
    await playground.disconnectWallet();
    console.log('Wallet disconnected');

    console.log('\n🎉 Playground Demo Completed!');
    console.log('════════════════════════════');
    console.log('The demo showcased:');
    console.log('✅ Project creation and management');
    console.log('✅ Code compilation to bytecode');
    console.log('✅ WASM VM testing with real execution');
    console.log('✅ Advanced testing with custom accounts');
    console.log('✅ Wallet integration');
    console.log('✅ Deployment process (requires running validator)');
    console.log('✅ Project statistics and management');
    console.log('');
    console.log('Next steps:');
    console.log('1. Start a Solana test validator: `solana-test-validator`');
    console.log('2. Fund your wallet: `solana airdrop 2 <wallet-address> --url devnet`');
    console.log('3. Try real deployments with the CLI: `npm run deploy`');
    console.log('4. Build web UI components using the playground classes');
}

// Mock required globals for Node.js environment
Object.defineProperty(global, 'localStorage', {
    value: {
        data: {} as Record<string, string>,
        getItem(key: string) {
            return this.data[key] || null;
        },
        setItem(key: string, value: string) {
            this.data[key] = value;
        },
        removeItem(key: string) {
            delete this.data[key];
        },
        clear() {
            this.data = {};
        }
    },
    writable: true
});

Object.defineProperty(global, 'document', {
    value: {
        getElementById: () => ({
            appendChild: () => {}
        }),
        createElement: () => ({
            style: {},
            appendChild: () => {},
            addEventListener: () => {}
        }),
        body: {
            appendChild: () => {}
        }
    },
    writable: true
});

// Run the demo
if (require.main === module) {
    playgroundDemo().catch((error) => {
        console.error('Demo error:', error);
        process.exit(1);
    });
}

export { playgroundDemo };