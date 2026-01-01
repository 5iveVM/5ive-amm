#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import {
    Connection, Keypair, PublicKey, Transaction, SystemProgram, LAMPORTS_PER_SOL
} from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
const VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');

async function main() {
    try {
        console.log('\n═══════════════════════════════════════════════════════════');
        console.log('Creating Token Program Account on Five VM');
        console.log('═══════════════════════════════════════════════════════════\n');

        const connection = new Connection(RPC_URL, 'confirmed');
        const payer = Keypair.fromSecretKey(
            Uint8Array.from(JSON.parse(fs.readFileSync(
                path.join(process.env.HOME, '.config/solana/id.json'), 'utf-8'
            )))
        );

        const bytecode = fs.readFileSync('./build/five-token-template.five');
        const tokenAccount = Keypair.generate();
        const space = Math.max(8192, bytecode.length + 2048);
        const lamports = await connection.getMinimumBalanceForRentExemption(space);

        console.log(`Payer: ${payer.publicKey.toBase58()}`);
        console.log(`Account Size: ${space} bytes`);
        console.log(`Bytecode Size: ${bytecode.length} bytes`);
        console.log(`Rent: ${(lamports / LAMPORTS_PER_SOL).toFixed(6)} SOL\n`);

        const ix = SystemProgram.createAccount({
            fromPubkey: payer.publicKey,
            newAccountPubkey: tokenAccount.publicKey,
            lamports,
            space,
            programId: FIVE_PROGRAM_ID,
        });

        const tx = new Transaction().add(ix);
        const sig = await connection.sendTransaction(tx, [payer, tokenAccount], { skipPreflight: true });

        console.log('Confirming transaction...');
        await connection.confirmTransaction(sig, 'confirmed');

        console.log('\n✅ SUCCESS\n');
        console.log('Token Script Account: ' + tokenAccount.publicKey.toBase58());
        console.log('Signature: ' + sig);
        console.log('\n═══════════════════════════════════════════════════════════\n');
        console.log('Update e2e-token-test.mjs with:\n');
        console.log('const TOKEN_SCRIPT_ACCOUNT = new PublicKey(\'' + tokenAccount.publicKey.toBase58() + '\');\n');

        // Save config
        const config = {
            tokenScriptAccount: tokenAccount.publicKey.toBase58(),
            fiveProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStatePda: VM_STATE_PDA.toBase58(),
            timestamp: new Date().toISOString()
        };
        fs.writeFileSync('deployment-config.json', JSON.stringify(config, null, 2));
        console.log('Config saved to: deployment-config.json\n');

    } catch (error) {
        console.error('\n❌ ERROR:', error.message, '\n');
        process.exit(1);
    }
}

main();
