/**
 * Account Test Fixture Framework for Five VM
 *
 * Provides a reusable, composable pattern for testing Five account-system scripts.
 * Builders use this to:
 * 1. Define test accounts with specific constraints
 * 2. Initialize account state before test execution
 * 3. Validate constraints locally before on-chain testing
 * 4. Reuse common account setups across multiple tests
 *
 * Example:
 * ```
 * const fixture = new AccountTestFixture()
 *   .addSignerAccount('payer')
 *   .addStateAccount('counter', { value: 0 })
 *   .addMutableAccount('target')
 *   .build();
 * ```
 */
import { GeneratedAccountMeta, AccountMetaGenerator } from './AccountMetaGenerator.js';
import { SolanaPublicKeyUtils } from '../crypto/index.js';
import { Connection, Keypair, PublicKey, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { OnChainAccountManager } from './OnChainAccountManager.js';
import { StateSerializer } from './StateSerializer.js';
/**
 * Account Test Fixture Builder
 * Fluent API for constructing test account setups
 */
export class AccountTestFixture {
    specs = [];
    stateTemplates = new Map();
    /**
     * Add a signer account (@signer constraint)
     * Used for transaction signers, authority checks
     */
    addSignerAccount(name, options = {}) {
        this.specs.push({
            name,
            type: 'signer',
            description: options.description || `Signer account: ${name}`
        });
        return this;
    }
    /**
     * Add a mutable account (@mut constraint)
     * Can be modified by the script
     */
    addMutableAccount(name, state, options = {}) {
        this.specs.push({
            name,
            type: 'mutable',
            state,
            description: options.description || `Mutable account: ${name}`
        });
        if (state) {
            this.stateTemplates.set(name, state);
        }
        return this;
    }
    /**
     * Add a read-only account
     * Cannot be modified by the script
     */
    addReadOnlyAccount(name, state, options = {}) {
        this.specs.push({
            name,
            type: 'readonly',
            state,
            description: options.description || `Read-only account: ${name}`
        });
        if (state) {
            this.stateTemplates.set(name, state);
        }
        return this;
    }
    /**
     * Add a state account (@mut state: StateType)
     * Typically for program state storage
     */
    addStateAccount(name, state = {}, options = {}) {
        this.specs.push({
            name,
            type: 'state',
            state,
            description: options.description || `State account: ${name}`
        });
        this.stateTemplates.set(name, state);
        return this;
    }
    /**
     * Add an initialization account (@init constraint)
     * For account creation patterns
     */
    addInitAccount(name, options = {}) {
        this.specs.push({
            name,
            type: 'init',
            description: options.description || `Init account: ${name}`
        });
        return this;
    }
    /**
     * Add multiple accounts in one call
     * Useful for standard patterns
     */
    addPattern(pattern) {
        switch (pattern) {
            case 'authorization':
                this.addSignerAccount('authority');
                this.addStateAccount('state', { admin: '', authorized_users: 0 });
                break;
            case 'state-mutation':
                this.addStateAccount('state', { count: 0, modification_count: 0 });
                break;
            case 'batch-operation':
                this.addSignerAccount('authority');
                this.addMutableAccount('account1');
                this.addMutableAccount('account2');
                this.addStateAccount('state', { operation_count: 0 });
                break;
        }
        return this;
    }
    /**
     * Compile fixture into accounts ready for execution
     * Supports both local (synthetic) and on-chain (real) account modes
     */
    async build(options = {}) {
        const mode = options.mode || 'local';
        if (mode === 'local') {
            return this.buildLocal(options);
        }
        else if (mode === 'onchain') {
            if (!options.connection || !options.payer) {
                throw new Error('connection and payer required for onchain mode');
            }
            return this.buildOnChain(options);
        }
        else {
            throw new Error(`Unknown mode: ${mode}`);
        }
    }
    /**
     * Build fixture with local synthetic accounts (existing behavior)
     */
    async buildLocal(options = {}) {
        const accounts = [];
        const accountsByName = new Map();
        const signerCount = this.specs.filter(s => s.type === 'signer').length;
        const mutableCount = this.specs.filter(s => s.type === 'mutable' || s.type === 'state').length;
        const readonlyCount = this.specs.filter(s => s.type === 'readonly').length;
        const stateCount = this.specs.filter(s => s.type === 'state').length;
        if (options.debug) {
            console.log(`[AccountTestFixture] Building local fixture with ${this.specs.length} accounts:`);
        }
        for (const spec of this.specs) {
            const account = await this.createAccount(spec, options);
            accounts.push(account);
            accountsByName.set(spec.name, account);
            if (options.debug) {
                console.log(`  ${spec.name} (${spec.type}): ${account.pubkey.substring(0, 8)}...`);
            }
        }
        return {
            accounts,
            accountsByName,
            stateData: this.stateTemplates,
            specs: this.specs,
            metadata: {
                signerCount,
                mutableCount,
                readonlyCount,
                stateCount
            }
        };
    }
    /**
     * Build fixture with real on-chain accounts
     */
    async buildOnChain(options) {
        const manager = new OnChainAccountManager(options.connection, options.payer, {
            debug: options.debug,
            cleanup: options.cleanup
        });
        const accounts = [];
        const accountsByName = new Map();
        const createdAccounts = [];
        const signerCount = this.specs.filter(s => s.type === 'signer').length;
        const mutableCount = this.specs.filter(s => s.type === 'mutable' || s.type === 'state').length;
        const readonlyCount = this.specs.filter(s => s.type === 'readonly').length;
        const stateCount = this.specs.filter(s => s.type === 'state').length;
        if (options.debug) {
            console.log(`[AccountTestFixture] Building on-chain fixture with ${this.specs.length} accounts`);
        }
        // Create each account on-chain
        for (const spec of this.specs) {
            let publicKey;
            let keypair;
            try {
                if (spec.type === 'signer') {
                    // Create signer account with keypair
                    const result = await manager.createSignerAccount(LAMPORTS_PER_SOL);
                    publicKey = result.publicKey;
                    keypair = result.keypair;
                    if (options.debug) {
                        console.log(`  ${spec.name} (signer): ${publicKey.toString()}`);
                    }
                    accounts.push({
                        pubkey: publicKey.toString(),
                        isSigner: true,
                        isWritable: true,
                        keypair: {
                            publicKey: publicKey.toString(),
                            secretKey: keypair.secretKey
                        }
                    });
                }
                else if (spec.type === 'state' || spec.type === 'mutable') {
                    // Create state/mutable account with initial data
                    const space = 1024; // Default space
                    const owner = options.fiveVMProgramId || new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
                    // Serialize state data if provided
                    let initialData;
                    if (spec.state && spec.type === 'state') {
                        const stateDefinition = {
                            name: spec.name,
                            fields: Object.keys(spec.state).map(name => ({ name, type: 'u64' })) // Simple default types
                        };
                        initialData = StateSerializer.serialize(stateDefinition, spec.state, { debug: options.debug });
                    }
                    publicKey = await manager.createStateAccount(space, owner, initialData);
                    if (options.debug) {
                        console.log(`  ${spec.name} (${spec.type}): ${publicKey.toString()}`);
                    }
                    accounts.push({
                        pubkey: publicKey.toString(),
                        isSigner: false,
                        isWritable: true
                    });
                }
                else if (spec.type === 'init') {
                    // Create init account (will be initialized by script)
                    const space = 1024;
                    const owner = options.fiveVMProgramId || new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
                    publicKey = await manager.createAccount(space, owner);
                    if (options.debug) {
                        console.log(`  ${spec.name} (init): ${publicKey.toString()}`);
                    }
                    accounts.push({
                        pubkey: publicKey.toString(),
                        isSigner: false,
                        isWritable: true
                    });
                }
                else {
                    // Create readonly account
                    const space = 0;
                    const owner = options.fiveVMProgramId || new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
                    publicKey = await manager.createAccount(space, owner);
                    if (options.debug) {
                        console.log(`  ${spec.name} (readonly): ${publicKey.toString()}`);
                    }
                    accounts.push({
                        pubkey: publicKey.toString(),
                        isSigner: false,
                        isWritable: false
                    });
                }
                accountsByName.set(spec.name, accounts[accounts.length - 1]);
                createdAccounts.push(publicKey);
            }
            catch (error) {
                const errorMessage = error instanceof Error ? error.message : String(error);
                throw new Error(`Failed to create account '${spec.name}': ${errorMessage}`);
            }
        }
        // Create cleanup function if enabled
        const cleanup = options.cleanup
            ? async () => manager.cleanup()
            : undefined;
        return {
            accounts,
            accountsByName,
            stateData: this.stateTemplates,
            specs: this.specs,
            metadata: {
                signerCount,
                mutableCount,
                readonlyCount,
                stateCount
            },
            cleanup
        };
    }
    /**
     * Create individual account based on spec
     */
    async createAccount(spec, options) {
        // Determine writable and signer flags based on type
        const isWritable = spec.type === 'mutable' || spec.type === 'state' || spec.type === 'init';
        const isSigner = spec.type === 'signer';
        // Use AccountMetaGenerator for consistent account creation
        const constraints = {
            name: spec.name,
            writable: isWritable,
            signer: isSigner
        };
        if (isSigner) {
            return await AccountMetaGenerator['generateSignerAccount'](constraints, { debug: options.debug });
        }
        else {
            return await AccountMetaGenerator['generateRegularAccount'](constraints, { debug: options.debug });
        }
    }
    /**
     * Validate fixture against ABI constraints
     */
    validateAgainstABI(abiFunction) {
        const errors = [];
        const warnings = [];
        const requiredAccounts = abiFunction.accounts || [];
        if (this.specs.length !== requiredAccounts.length) {
            errors.push(`Account count mismatch: fixture has ${this.specs.length} accounts, ` +
                `ABI requires ${requiredAccounts.length}`);
        }
        for (let i = 0; i < Math.min(this.specs.length, requiredAccounts.length); i++) {
            const spec = this.specs[i];
            const abiAccount = requiredAccounts[i];
            // Check signer constraint
            if (spec.type === 'signer' && !abiAccount.signer) {
                warnings.push(`Account ${i} (${spec.name}): marked as signer but ABI doesn't require it`);
            }
            if (spec.type !== 'signer' && abiAccount.signer) {
                errors.push(`Account ${i} (${spec.name}): ABI requires signer, but fixture provides ${spec.type}`);
            }
            // Check mutability constraint
            const isWritable = spec.type === 'mutable' || spec.type === 'state' || spec.type === 'init';
            if (isWritable && !abiAccount.writable) {
                warnings.push(`Account ${i} (${spec.name}): marked as writable but ABI doesn't require it`);
            }
            if (!isWritable && abiAccount.writable) {
                errors.push(`Account ${i} (${spec.name}): ABI requires writable, but fixture provides ${spec.type}`);
            }
        }
        return {
            valid: errors.length === 0,
            errors,
            warnings
        };
    }
    /**
     * Get summary of fixture specs for debugging
     */
    getSummary() {
        const lines = [
            `Account Test Fixture (${this.specs.length} accounts)`,
            '─'.repeat(40)
        ];
        for (const spec of this.specs) {
            const stateStr = spec.state ? ` with state: ${JSON.stringify(spec.state)}` : '';
            lines.push(`  • ${spec.name} (${spec.type})${stateStr}`);
        }
        lines.push('─'.repeat(40));
        lines.push(`Signers: ${this.specs.filter(s => s.type === 'signer').length}, ` +
            `Writable: ${this.specs.filter(s => s.type !== 'readonly' && s.type !== 'signer').length}`);
        return lines.join('\n');
    }
}
/**
 * Predefined fixture templates for common patterns
 * Builders can extend these for their specific needs
 */
export class FixtureTemplates {
    /**
     * Simple state mutation pattern
     * For scripts that increment counters, track modifications, etc.
     */
    static stateCounter() {
        return new AccountTestFixture()
            .addStateAccount('state', {
            count: 0,
            modification_count: 0
        });
    }
    /**
     * Authorization pattern
     * For scripts that check permissions via @signer
     */
    static authorization() {
        return new AccountTestFixture()
            .addSignerAccount('authority', {
            description: 'Signer that has authorization'
        })
            .addStateAccount('state', {
            admin: '11111111111111111111111111111111', // Placeholder public key
            authorized_users: 0
        });
    }
    /**
     * Account creation pattern
     * For scripts using @init constraint
     */
    static accountCreation() {
        return new AccountTestFixture()
            .addSignerAccount('payer')
            .addInitAccount('new_account')
            .addStateAccount('state', {
            total_created: 0,
            last_created: '11111111111111111111111111111111'
        });
    }
    /**
     * Multi-account transaction pattern
     * For scripts that operate on multiple mutable accounts
     */
    static batchOperation() {
        return new AccountTestFixture()
            .addSignerAccount('authority')
            .addMutableAccount('account1')
            .addMutableAccount('account2')
            .addStateAccount('state', {
            operation_count: 0,
            last_operator: '11111111111111111111111111111111'
        });
    }
    /**
     * Complex authorization pattern
     * For multi-signature or advanced permission schemes
     */
    static multiSigPattern() {
        return new AccountTestFixture()
            .addSignerAccount('primary')
            .addSignerAccount('secondary')
            .addStateAccount('state', {
            owner: '11111111111111111111111111111111',
            authorized_signers: 2,
            transaction_count: 0
        });
    }
    /**
     * PDA pattern
     * For scripts that use Program Derived Addresses
     */
    static pdaPattern() {
        return new AccountTestFixture()
            .addSignerAccount('payer')
            .addInitAccount('pda_vault')
            .addStateAccount('state', {
            vault_bump: 255,
            token_bump: 255
        });
    }
}
/**
 * Test execution builder
 * Combines fixture with test parameters for execution
 */
export class AccountTestExecutor {
    /**
     * Bind fixture accounts to a test execution
     */
    static bindFixture(fixture, functionName, parameters = []) {
        const keypairs = new Map();
        // Collect keypairs from signer accounts for transaction signing
        for (const [name, account] of fixture.accountsByName) {
            if (account.keypair) {
                keypairs.set(name, account.keypair.secretKey);
            }
        }
        return {
            fixture,
            functionName,
            parameters,
            accountAddresses: fixture.accounts.map(a => a.pubkey),
            keypairs
        };
    }
    /**
     * Validate execution context before running
     */
    static validateContext(context) {
        const errors = [];
        const warnings = [];
        // Verify all accounts are properly initialized
        if (context.fixture.accounts.length === 0) {
            errors.push('No accounts in fixture');
        }
        // Verify signer accounts have keypairs
        for (const account of context.fixture.accounts) {
            const spec = context.fixture.specs.find(s => s.name ===
                Array.from(context.fixture.accountsByName.entries())
                    .find(([_, acc]) => acc === account)?.[0]);
            if (spec?.type === 'signer' && !account.keypair) {
                errors.push(`Signer account '${spec.name}' missing keypair`);
            }
        }
        // Verify account addresses are valid
        if (context.accountAddresses.length !== context.fixture.accounts.length) {
            errors.push('Account address count mismatch');
        }
        return {
            valid: errors.length === 0,
            errors,
            warnings
        };
    }
    /**
     * Get human-readable execution summary
     */
    static getSummary(context) {
        const lines = [
            `Test Execution Context`,
            '─'.repeat(50),
            `Function: ${context.functionName}`,
            `Parameters: ${JSON.stringify(context.parameters)}`,
            `Accounts: ${context.fixture.accounts.length}`,
            '─'.repeat(50)
        ];
        for (let i = 0; i < context.fixture.accounts.length; i++) {
            const account = context.fixture.accounts[i];
            const spec = context.fixture.specs[i];
            const signer = account.isSigner ? ' [SIGNER]' : '';
            const writable = account.isWritable ? ' [WRITABLE]' : ' [READONLY]';
            lines.push(`  ${i}: ${spec.name}${signer}${writable}`);
            lines.push(`     Address: ${account.pubkey}`);
        }
        return lines.join('\n');
    }
}
//# sourceMappingURL=AccountTestFixture.js.map