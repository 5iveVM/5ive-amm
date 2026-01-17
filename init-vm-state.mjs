#!/usr/bin/env node

import fs from 'fs';
import {
    Connection, Keypair, PublicKey, Transaction, SystemProgram,
    sendAndConfirmTransaction, LAMPORTS_PER_SOL
} from '@solana/web3.js';

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

// Program ID from our deployment
const FIVE_PROGRAM_ID = new PublicKey('nnYRwyiHXRmsQbcq8u8VCeC2HAmfqy43xyRJew4TRin');

// Instruction discriminator for Initialize (0x00)
const INITIALIZE_DISCRIMINATOR = Buffer.from([0x00]);

async function deriveVMStatePDA(programId) {
    const seeds = [Buffer.from('vm_state', 'utf8')];
    const [pda, bump] = PublicKey.findProgramAddressSync(seeds, programId);
    return { pda, bump };
}

async function main() {
    console.log('🚀 Initializing VM State PDA...\n');

    // Setup
    const connection = new Connection(RPC_URL, 'confirmed');
    const secretKey = JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8'));
    const payer = Keypair.fromSecretKey(Uint8Array.from(secretKey));

    console.log(`Program ID: ${FIVE_PROGRAM_ID.toBase58()}`);
    console.log(`Payer: ${payer.publicKey.toBase58()}`);

    // Derive VM State PDA
    const { pda: vmStatePDA, bump } = await deriveVMStatePDA(FIVE_PROGRAM_ID);
    console.log(`VM State PDA: ${vmStatePDA.toBase58()} (bump: ${bump})\n`);

    // Check if already initialized
    const vmStateAccount = await connection.getAccountInfo(vmStatePDA);
    if (vmStateAccount) {
        console.log('✅ VM State PDA already exists');
        if (vmStateAccount.owner.equals(FIVE_PROGRAM_ID)) {
            console.log('✅ Already owned by Five program');
            process.exit(0);
        }
    }

    // Calculate rent for VM State account (56 bytes minimum)
    const vmStateSize = 56;
    const rentLamports = await connection.getMinimumBalanceForRentExemption(vmStateSize);
    console.log(`Rent required: ${rentLamports} lamports`);

    // Create a regular keypair for the VM State account
    // (since PDAs can't sign, we'll create it and transfer ownership)
    const vmStateKeypair = Keypair.generate();
    console.log(`\n📝 Temporary VM State Keypair: ${vmStateKeypair.publicKey.toBase58()}`);

    // Step 1: Create the VM State account with payer and temporary keypair
    console.log('\n📝 Step 1: Creating VM State account...');
    const createAccountIx = SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: vmStateKeypair.publicKey,
        lamports: rentLamports,
        space: vmStateSize,
        programId: FIVE_PROGRAM_ID
    });

    // Step 2: Initialize the VM State account
    console.log('📝 Step 2: Initializing VM State...');
    const initializeIx = {
        programId: FIVE_PROGRAM_ID,
        keys: [
            { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
            { pubkey: payer.publicKey, isSigner: true, isWritable: false }
        ],
        data: INITIALIZE_DISCRIMINATOR
    };

    // Send transaction
    const tx = new Transaction().add(createAccountIx).add(initializeIx);

    try {
        const sig = await sendAndConfirmTransaction(connection, tx, [payer, vmStateKeypair], {
            skipPreflight: true,
            commitment: 'confirmed'
        });
        console.log(`\n✅ VM State initialized successfully!`);
        console.log(`Signature: ${sig}`);

        // Verify
        const initialized = await connection.getAccountInfo(vmStateKeypair.publicKey);
        if (initialized && initialized.owner.equals(FIVE_PROGRAM_ID)) {
            console.log('✅ Verified: VM State account is owned by Five program');
        }

        // Save configuration
        const config = {
            fiveProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStateAccount: vmStateKeypair.publicKey.toBase58(),
            vmStatePda: vmStatePDA.toBase58(),
            rpcUrl: RPC_URL,
            timestamp: new Date().toISOString()
        };

        fs.writeFileSync(
            '/Users/amberjackson/Documents/Development/five-org/five-mono/vm-init-config.json',
            JSON.stringify(config, null, 2)
        );
        console.log('\n📄 Configuration saved to vm-init-config.json');
    } catch (error) {
        console.error('❌ Error initializing VM State:', error);
        process.exit(1);
    }
}

main().catch(err => {
    console.error(err);
    process.exit(1);
});
