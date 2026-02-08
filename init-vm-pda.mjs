#!/usr/bin/env node

import fs from 'fs';
import {
    Connection, Keypair, PublicKey, Transaction, SystemProgram,
    sendAndConfirmTransaction
} from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

// Updated Program ID
const FIVE_PROGRAM_ID = new PublicKey('6ndNfSrrGoFfTbS1sdJFybuJJyA6YQHtNgRdoXFREi8k');

async function deriveVMStatePDA(programId) {
    const seeds = [Buffer.from('vm_state', 'utf8')];
    const [pda, bump] = PublicKey.findProgramAddressSync(seeds, programId);
    return { pda, bump };
}

async function main() {
    console.log('🚀 Bootstrapping VM State PDA...\n');

    const connection = new Connection(RPC_URL, 'confirmed');
    const secretKey = JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8'));
    const payer = Keypair.fromSecretKey(Uint8Array.from(secretKey));

    console.log(`Program ID: ${FIVE_PROGRAM_ID.toBase58()}`);
    console.log(`Payer: ${payer.publicKey.toBase58()}`);

    const { pda: vmStatePDA, bump } = await deriveVMStatePDA(FIVE_PROGRAM_ID);
    console.log(`VM State PDA: ${vmStatePDA.toBase58()} (bump: ${bump})\n`);

    // Instruction data: [0 (discriminator), bump]
    const instructionData = Buffer.from([0x00, bump]);

    const initializeIx = {
        programId: FIVE_PROGRAM_ID,
        keys: [
            { pubkey: vmStatePDA, isSigner: false, isWritable: true },
            { pubkey: payer.publicKey, isSigner: true, isWritable: false }, // authority
            { pubkey: payer.publicKey, isSigner: true, isWritable: true },  // payer
            { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
        ],
        data: instructionData
    };

    const tx = new Transaction().add(initializeIx);

    try {
        const sig = await sendAndConfirmTransaction(connection, tx, [payer], {
            skipPreflight: true,
            commitment: 'confirmed'
        });
        console.log(`\n✅ VM State PDA initialized successfully!`);
        console.log(`Signature: ${sig}`);

        // Save configuration
        const config = {
            fiveProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStatePda: vmStatePDA.toBase58(),
            rpcUrl: RPC_URL,
            timestamp: new Date().toISOString()
        };

        fs.writeFileSync(
            '/Users/amberjackson/Documents/Development/five-org/five-mono/vm-init-config.json',
            JSON.stringify(config, null, 2)
        );
        console.log('\n📄 Configuration updated in vm-init-config.json');
    } catch (error) {
        console.error('❌ Error initializing VM State PDA:', error);
        process.exit(1);
    }
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
