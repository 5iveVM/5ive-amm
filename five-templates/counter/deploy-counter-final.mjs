import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram, ComputeBudgetProgram } from '@solana/web3.js';
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const RPC_URL = 'http://127.0.0.1:8899';
const FIVE_PROGRAM_ID = new PublicKey('HvXw1h2ndbBRyBccW8UtYa1XVoFh2M5rWgUQTkoJWtEq');
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

async function main() {
    const connection = new Connection(RPC_URL, 'confirmed');
    const secretKey = JSON.parse(fs.readFileSync(PAYER_KEYPAIR_PATH, 'utf-8'));
    const payer = Keypair.fromSecretKey(Uint8Array.from(secretKey));

    console.log('=== Counter Template Deployment ===');
    console.log('Five Program:', FIVE_PROGRAM_ID.toBase58());

    // 1. Create VM State
    const vmStateKeypair = Keypair.generate();
    const vmStateSpace = 256;
    const vmStateRent = await connection.getMinimumBalanceForRentExemption(vmStateSpace);

    const createVMStateIx = SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: vmStateKeypair.publicKey,
        lamports: vmStateRent,
        space: vmStateSpace,
        programId: FIVE_PROGRAM_ID
    });

    const initVMStateIx = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys: [
            { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
            { pubkey: payer.publicKey, isSigner: true, isWritable: false }
        ],
        data: Buffer.from([0x00]) // Initialize discriminator
    });

    const tx1 = new Transaction().add(createVMStateIx).add(initVMStateIx);
    await connection.sendTransaction(tx1, [payer, vmStateKeypair]);
    console.log('✅ VM State created:', vmStateKeypair.publicKey.toBase58());

    // 2. Create Script Account
    const scriptKeypair = Keypair.generate();
    const bytecode = fs.readFileSync(path.join(__dirname, 'src/counter.bin'));
    const scriptSpace = bytecode.length + 128;
    const scriptRent = await connection.getMinimumBalanceForRentExemption(scriptSpace);

    const createScriptIx = SystemProgram.createAccount({
        fromPubkey: payer.publicKey,
        newAccountPubkey: scriptKeypair.publicKey,
        lamports: scriptRent,
        space: scriptSpace,
        programId: FIVE_PROGRAM_ID
    });

    const deployIx = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys: [
            { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
            { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
            { pubkey: payer.publicKey, isSigner: true, isWritable: false }
        ],
        data: Buffer.concat([
            Buffer.from([0x08]), // Deploy discriminator
            Buffer.from(new Uint32Array([bytecode.length]).buffer),
            Buffer.from([0x00]), // permissions
            bytecode
        ])
    });

    const tx2 = new Transaction()
        .add(ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }))
        .add(createScriptIx)
        .add(deployIx);
    
    await connection.sendTransaction(tx2, [payer, scriptKeypair]);
    console.log('✅ Script deployed:', scriptKeypair.publicKey.toBase58());

    // Save config
    const config = {
        rpcUrl: RPC_URL,
        fiveProgramId: FIVE_PROGRAM_ID.toBase58(),
        vmStatePda: vmStateKeypair.publicKey.toBase58(),
        counterScriptAccount: scriptKeypair.publicKey.toBase58()
    };
    fs.writeFileSync(path.join(__dirname, 'deployment-config.json'), JSON.stringify(config, null, 2));
    console.log('✅ Config saved');
}

main().catch(console.error);
