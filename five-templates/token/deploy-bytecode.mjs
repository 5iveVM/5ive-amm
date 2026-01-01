#!/usr/bin/env node
/**
 * Deploy bytecode to existing Five VM script account
 */

import fs from 'fs';
import path from 'path';
import {
    Connection, Keypair, PublicKey, Transaction,
    TransactionInstruction, LAMPORTS_PER_SOL
} from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
const VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');
const TOKEN_SCRIPT_ACCOUNT = new PublicKey('GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ');

async function main() {
    console.log('═══════════════════════════════════════════════════════════');
    console.log('Deploy Token Bytecode to Five VM');
    console.log('═══════════════════════════════════════════════════════════\n');

    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = Keypair.fromSecretKey(
        Uint8Array.from(JSON.parse(fs.readFileSync(
            path.join(process.env.HOME, '.config/solana/id.json'), 'utf-8'
        )))
    );

    console.log(`Payer: ${payer.publicKey.toBase58()}`);
    console.log(`Five Program: ${FIVE_PROGRAM_ID.toBase58()}`);
    console.log(`VM State PDA: ${VM_STATE_PDA.toBase58()}`);
    console.log(`Script Account: ${TOKEN_SCRIPT_ACCOUNT.toBase58()}\n`);

    // Load compiled bytecode
    const fiveFile = JSON.parse(fs.readFileSync('./build/five-token-template.five', 'utf-8'));
    const bytecodeBase64 = fiveFile.bytecode;
    const bytecode = Buffer.from(bytecodeBase64, 'base64');

    console.log(`Bytecode size: ${bytecode.length} bytes`);
    console.log(`Bytecode starts with: ${bytecode.slice(0, 8).toString('hex')}`);

    // Check if it's valid Five bytecode (should start with "5IVE" magic)
    const magic = bytecode.slice(0, 4).toString('ascii');
    console.log(`Magic bytes: "${magic}"`);

    // Build deploy instruction
    // Format: [discriminator=8, bytecode_length (u32 LE), permissions (u8), bytecode...]
    const discriminator = 8; // Deploy instruction
    const permissions = 0;   // No special permissions

    const instructionData = Buffer.alloc(1 + 4 + 1 + bytecode.length);
    instructionData.writeUInt8(discriminator, 0);
    instructionData.writeUInt32LE(bytecode.length, 1);
    instructionData.writeUInt8(permissions, 5);
    bytecode.copy(instructionData, 6);

    console.log(`\nInstruction data size: ${instructionData.length} bytes`);

    const deployIx = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys: [
            { pubkey: TOKEN_SCRIPT_ACCOUNT, isSigner: false, isWritable: true },
            { pubkey: VM_STATE_PDA, isSigner: false, isWritable: true },
            { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        ],
        data: instructionData,
    });

    const tx = new Transaction().add(deployIx);

    console.log('\nSending deploy transaction...');

    try {
        const sig = await connection.sendTransaction(tx, [payer], {
            skipPreflight: false,
            maxRetries: 3
        });

        console.log(`Signature: ${sig}`);
        console.log('Confirming...');

        await connection.confirmTransaction(sig, 'confirmed');

        console.log('\n✅ Bytecode deployed successfully!');

        // Verify the account now has data
        const accountInfo = await connection.getAccountInfo(TOKEN_SCRIPT_ACCOUNT);
        if (accountInfo) {
            const nonZeroBytes = accountInfo.data.filter(b => b !== 0).length;
            console.log(`\nScript account now has ${nonZeroBytes} non-zero bytes`);
        }

    } catch (error) {
        console.error('\n❌ Deploy failed:', error.message);
        if (error.logs) {
            console.error('\nProgram logs:');
            error.logs.forEach(log => console.error('  ', log));
        }
        process.exit(1);
    }
}

main();
