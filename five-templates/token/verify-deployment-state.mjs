
import { Connection, PublicKey } from '@solana/web3.js';

const RPC_URL = process.env.RPC_URL || 'http://127.0.0.1:8899';
const connection = new Connection(RPC_URL, 'confirmed');

const VM_STATE_PDA = 'DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys';
const SMALL_SCRIPT_ACCOUNT = 'BTCFkDR4fHWjUbunxfrU71a6af11kqK36z6zFAHncGrz';

async function verifyAccount(pubkey, label) {
    try {
        const info = await connection.getAccountInfo(new PublicKey(pubkey));
        if (info) {
            console.log(`✅ ${label} FOUND`);
            console.log(`   Owner: ${info.owner.toBase58()}`);
            console.log(`   Data Size: ${info.data.length} bytes`);
            console.log(`   Lamports: ${info.lamports}`);
            return true;
        } else {
            console.log(`❌ ${label} NOT FOUND`);
            return false;
        }
    } catch (err) {
        console.log(`❌ ${label} ERROR: ${err.message}`);
        return false;
    }
}

async function main() {
    console.log(`Verifying state on ${RPC_URL}...`);

    // Verify VM State
    const vmStateExists = await verifyAccount(VM_STATE_PDA, 'VM State Account');

    // Verify Small Script
    const scriptExists = await verifyAccount(SMALL_SCRIPT_ACCOUNT, 'Small Script Account');

    if (vmStateExists) {
        console.log('\nDeployment Verification: PARTIAL SUCCESS (VM State exists)');
        if (scriptExists) {
            console.log('Deployment Verification: FULL SUCCESS (Script deployed)');
        } else {
            console.log('Deployment Verification: Script missing (expected if recent tests failed, but manual deployment 525 succeeded?)');
        }
    } else {
        console.log('\nDeployment Verification: FAILED (VM State missing!)');
        process.exit(1);
    }
}

main();
