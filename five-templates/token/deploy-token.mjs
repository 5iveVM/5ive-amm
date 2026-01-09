import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram, SYSVAR_RENT_PUBKEY, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { FiveSDK } from '../../five-sdk/dist/index.js';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

async function main() {
    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8'))));
    const FIVE_PROGRAM_ID = new PublicKey('13MYBMJQdvXBuhE396tGn24xYzDZL5bjiGZRAszDvBJx');

    console.log('=== Token Template Full Deployment ===');
    console.log('Payer:', payer.publicKey.toBase58());
    console.log('FIVE Program ID:', FIVE_PROGRAM_ID.toBase58());

    // Step 1: Create VM state account
    console.log('\n=== Step 1: Creating VM state account ===');
    const vmStateKeypair = Keypair.generate();
    const vmStatePubkey = vmStateKeypair.publicKey;
    const vmStateSpace = 256; // Space for VM state
    const vmStateRent = await connection.getMinimumBalanceForRentExemption(vmStateSpace);

    const createVMStateIx = SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: vmStatePubkey,
        lamports: vmStateRent,
        space: vmStateSpace,
        programId: FIVE_PROGRAM_ID
    });

    // Step 2: Initialize VM state
    console.log('Step 2: Initializing VM state');
    const initializeIx = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys: [
            { pubkey: vmStatePubkey, isSigner: false, isWritable: true },
            { pubkey: payer.publicKey, isSigner: true, isWritable: false }
        ],
        data: Buffer.from([0x00]) // Initialize discriminator
    });

    const initTx = new Transaction().add(createVMStateIx).add(initializeIx);
    const initSig = await connection.sendTransaction(initTx, [payer, vmStateKeypair], { skipPreflight: false });
    await connection.confirmTransaction(initSig, 'confirmed');
    console.log('✅ VM state initialized:', vmStatePubkey.toBase58());

    // Step 3: Load pre-compiled token template
    console.log('\n=== Step 3: Loading pre-compiled token template ===');
    const tokenBuildPath = path.join(__dirname, 'build/five-token-template.five');
    const tokenBuildData = JSON.parse(fs.readFileSync(tokenBuildPath, 'utf-8'));

    // Extract bytecode from the Five file format (base64-encoded)
    let bytecode;
    if (tokenBuildData.bytecode && typeof tokenBuildData.bytecode === 'string') {
        // Bytecode is base64-encoded
        bytecode = Buffer.from(tokenBuildData.bytecode, 'base64');
    } else {
        console.error('❌ Unable to extract bytecode from token build file');
        process.exit(1);
    }

    console.log('✅ Token template loaded');
    console.log(`  Bytecode: ${bytecode.length} bytes`);

    // Step 3.5: Set deployment fees to 0 to avoid large fee charges
    console.log('\n=== Step 3.5: Setting deployment fees to 0 ===');
    const setFeesIx = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys: [
            { pubkey: vmStatePubkey, isSigner: false, isWritable: true },
            { pubkey: payer.publicKey, isSigner: true, isWritable: false }
        ],
        data: Buffer.concat([
            Buffer.from([0x06]), // SetFees discriminator (6)
            Buffer.from([0, 0, 0, 0]), // deploy_fee_bps = 0 (u32LE)
            Buffer.from([0, 0, 0, 0])  // execute_fee_bps = 0 (u32LE)
        ])
    });

    const setFeesTx = new Transaction().add(setFeesIx);
    const setFeesSig = await connection.sendTransaction(setFeesTx, [payer], { skipPreflight: false });
    await connection.confirmTransaction(setFeesSig, 'confirmed');
    console.log('✅ Deployment fees set to 0');

    // Step 4: Create script account
    console.log('\n=== Step 4: Creating script account ===');
    const scriptKeypair = Keypair.generate();
    const scriptPubkey = scriptKeypair.publicKey;
    const SCRIPT_HEADER_SIZE = 64; // ScriptAccountHeader::LEN
    const bytecodeLength = bytecode.length;
    const scriptAccountSize = SCRIPT_HEADER_SIZE + bytecodeLength; // Header + bytecode
    const scriptRent = await connection.getMinimumBalanceForRentExemption(scriptAccountSize);

    const createScriptIx = SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: scriptPubkey,
        lamports: scriptRent,
        space: scriptAccountSize, // Include space for header + bytecode
        programId: FIVE_PROGRAM_ID
    });

    const createScriptTx = new Transaction().add(createScriptIx);
    const createScriptSig = await connection.sendTransaction(createScriptTx, [payer, scriptKeypair], { skipPreflight: false });
    await connection.confirmTransaction(createScriptSig, 'confirmed');
    console.log('✅ Script account created:', scriptPubkey.toBase58());

    // Step 5: Deploy bytecode using InitLargeProgram + AppendBytecode + FinalizeScript
    console.log('\n=== Step 5: Deploying bytecode (large program flow) ===');

    // Step 5a: InitLargeProgram
    console.log('Step 5a: Initializing large program upload');
    const initLargeIx = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys: [
            { pubkey: scriptPubkey, isSigner: false, isWritable: true },
            { pubkey: payer.publicKey, isSigner: true, isWritable: false },
            { pubkey: vmStatePubkey, isSigner: false, isWritable: true }
        ],
        data: Buffer.concat([
            Buffer.from([0x04]), // InitLargeProgram discriminator (4)
            Buffer.from([bytecode.length & 0xFF, (bytecode.length >> 8) & 0xFF, (bytecode.length >> 16) & 0xFF, (bytecode.length >> 24) & 0xFF]) // expected_size as u32LE
        ])
    });

    const initLargeTx = new Transaction().add(initLargeIx);
    const initLargeSig = await connection.sendTransaction(initLargeTx, [payer], { skipPreflight: false });
    await connection.confirmTransaction(initLargeSig, 'confirmed');
    console.log('✅ Large program initialized');

    // Step 5b: AppendBytecode (upload bytecode in chunks to fit within transaction size limits)
    console.log('Step 5b: Uploading bytecode in chunks');
    const MAX_CHUNK_SIZE = 700; // Leave room for Solana transaction overhead (~532 bytes)
    let offset = 0;

    while (offset < bytecode.length) {
        const chunkEnd = Math.min(offset + MAX_CHUNK_SIZE, bytecode.length);
        const chunk = bytecode.slice(offset, chunkEnd);

        const appendIx = new TransactionInstruction({
            programId: FIVE_PROGRAM_ID,
            keys: [
                { pubkey: scriptPubkey, isSigner: false, isWritable: true },
                { pubkey: payer.publicKey, isSigner: true, isWritable: false },
                { pubkey: vmStatePubkey, isSigner: false, isWritable: false }
            ],
            data: Buffer.concat([
                Buffer.from([0x05]), // AppendBytecode discriminator (5)
                chunk
            ])
        });

        const appendTx = new Transaction().add(appendIx);
        const appendSig = await connection.sendTransaction(appendTx, [payer], { skipPreflight: false });
        await connection.confirmTransaction(appendSig, 'confirmed');
        console.log(`✅ Bytecode chunk ${Math.floor(offset / MAX_CHUNK_SIZE) + 1}: ${offset}-${chunkEnd} bytes`);

        offset = chunkEnd;
    }

    console.log('✅ Bytecode fully uploaded');

    // Step 5c: FinalizeScript
    console.log('Step 5c: Finalizing script');
    const finalizeIx = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys: [
            { pubkey: scriptPubkey, isSigner: false, isWritable: true },
            { pubkey: payer.publicKey, isSigner: true, isWritable: false }
        ],
        data: Buffer.from([0x07]) // FinalizeScript discriminator (7)
    });

    const finalizeTx = new Transaction().add(finalizeIx);
    const finalizeSig = await connection.sendTransaction(finalizeTx, [payer], { skipPreflight: false });
    await connection.confirmTransaction(finalizeSig, 'confirmed');
    console.log('✅ Script finalized and ready for execution');

    console.log('✅ Bytecode fully deployed');

    // Step 6: Update deployment config
    console.log('\n=== Step 6: Updating deployment config ===');
    const configPath = path.join(__dirname, 'deployment-config.json');
    const config = {
        tokenScriptAccount: scriptPubkey.toBase58(),
        fiveProgramId: FIVE_PROGRAM_ID.toBase58(),
        vmStatePda: vmStatePubkey.toBase58(),
        rpcUrl: RPC_URL,
        timestamp: new Date().toISOString()
    };
    fs.writeFileSync(configPath, JSON.stringify(config, null, 2));
    console.log('✅ Deployment config updated');

    console.log('\n=== Deployment Complete ===');
    console.log('Configuration:');
    console.log(`  Script Account: ${scriptPubkey.toBase58()}`);
    console.log(`  VM State: ${vmStatePubkey.toBase58()}`);
    console.log(`  Program ID: ${FIVE_PROGRAM_ID.toBase58()}`);
}

main().catch(e => {
    console.error('❌ Deployment failed:', e.message);
    process.exit(1);
});
