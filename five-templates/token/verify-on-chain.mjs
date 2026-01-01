import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { Connection, PublicKey } from '@solana/web3.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Helper for colored logs
const colors = {
    green: '\x1b[32m',
    red: '\x1b[31m',
    yellow: '\x1b[33m',
    reset: '\x1b[0m',
    blue: '\x1b[34m'
};
const success = (msg) => console.log(`${colors.green}✅ ${msg}${colors.reset}`);
const error = (msg) => console.log(`${colors.red}❌ ${msg}${colors.reset}`);
const info = (msg) => console.log(`${colors.blue}ℹ️  ${msg}${colors.reset}`);

async function main() {
    console.log(`\n${colors.blue}========================================${colors.reset}`);
    console.log(`${colors.blue}VERIFYING ON-CHAIN STATE${colors.reset}`);
    console.log(`${colors.blue}========================================${colors.reset}\n`);

    // 1. Load test state
    const statePath = path.join(__dirname, 'test-state.json');
    if (!fs.existsSync(statePath)) {
        error("test-state.json not found. Run e2e-token-test.mjs first.");
        process.exit(1);
    }

    const state = JSON.parse(fs.readFileSync(statePath, 'utf8'));
    info(`Loaded test state for Mint: ${state.accounts.mint}`);

    const connection = new Connection(state.config.rpcUrl, 'confirmed');

    // 2. Helper to fetch account
    async function checkAccount(label, pubkeyStr, expectedOwner) {
        info(`Checking ${label}: ${pubkeyStr}`);
        const account = await connection.getAccountInfo(new PublicKey(pubkeyStr));

        if (!account) {
            error(`${label} NOT FOUND on-chain`);
            return null;
        }

        if (account.owner.toBase58() !== expectedOwner) {
            error(`${label} owner mismatch. Expected ${expectedOwner}, got ${account.owner.toBase58()}`);
            return null;
        }

        success(`${label} exists and has correct owner`);
        return account;
    }

    // 3. Verify Accounts Exist
    const programId = state.config.programId;

    // Check Mint
    const mintAccount = await checkAccount('Mint', state.accounts.mint, programId);

    // Check Token Accounts
    await checkAccount('User 1 Token Account', state.accounts.user1TokenAccount, programId);
    await checkAccount('User 2 Token Account', state.accounts.user2TokenAccount, programId);
    await checkAccount('User 3 Token Account', state.accounts.user3TokenAccount, programId);

    // 4. Data Inspection (Heuristic)
    if (mintAccount) {
        // Mint supply check (heuristic)
        // Supply is a u64 in the mint struct. We don't know the exact offset without deserializer, 
        // but we can check if data is non-empty.
        if (mintAccount.data.length > 0) {
            success(`Mint data initialized (${mintAccount.data.length} bytes)`);

            // Basic pattern check: name "TestToken" should be visible
            const nameBuf = Buffer.from("TestToken");
            if (mintAccount.data.includes(nameBuf)) {
                success(`Found token name "TestToken" in mint data`);
            } else {
                error(`Token name "TestToken" NOT found in mint data`);
            }
        } else {
            error("Mint data is empty!");
        }
    }

    console.log("\nBasic on-chain verification complete.");
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
