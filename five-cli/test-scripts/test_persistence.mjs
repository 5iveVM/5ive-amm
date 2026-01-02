import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { FiveSDK } from '../../five-sdk/dist/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Config
const RPC_URL = 'http://127.0.0.1:8899';
const BYTECODE_PATH = path.join(__dirname, 'counter_debug.bin');
const PAYER_PATH = process.env.HOME + '/.config/solana/id.json';

// NEW Program ID from recent deployment
const FIVE_PROGRAM_ID_STR = 'CDY1QWFzVYehSAYct1mqMFDxeh8dSzP9RiDDLH2eJwPS';

async function main() {
    console.log(`Connecting to ${RPC_URL}...`);
    const connection = new Connection(RPC_URL, 'confirmed');

    // Load Payer
    const payerBytes = JSON.parse(fs.readFileSync(PAYER_PATH, 'utf-8'));
    const payerKeypair = Keypair.fromSecretKey(new Uint8Array(payerBytes));
    console.log(`Payer: ${payerKeypair.publicKey.toBase58()}`);

    // Load Bytecode
    if (!fs.existsSync(BYTECODE_PATH)) {
        throw new Error(`Bytecode not found at ${BYTECODE_PATH}`);
    }
    const bytecode = fs.readFileSync(BYTECODE_PATH);
    console.log(`Loaded bytecode (${bytecode.length} bytes)`);

    // 1. Deploy
    console.log('\n--- Step 1: Deploying Script ---');
    let scriptAccount;
    let vmStateAccount;
    try {
        // We let the SDK create or find a VM state if we don't pass one
        const result = await FiveSDK.deployLargeProgramToSolana(
            bytecode,
            connection,
            payerKeypair,
            {
                fiveVMProgramId: FIVE_PROGRAM_ID_STR,
                chunkSize: 500,
                debug: true
            }
        );

        if (result.success) {
            scriptAccount = new PublicKey(result.scriptAccount);
            vmStateAccount = new PublicKey(result.vmStateAccount);
            console.log(`Successfully deployed.`);
            console.log(`Script Account: ${scriptAccount.toBase58()}`);
            console.log(`VM State Account: ${vmStateAccount.toBase58()}`);
        } else {
            console.error('Deployment Failed:', result.error);
            process.exit(1);
        }
    } catch (err) {
        console.error('Deployment Failed with error:', err);
        process.exit(1);
    }

    // 2. Initialize Counter (Function index 0)
    console.log('\n--- Step 2: Initializing Counter ---');

    // Derive Counter account: seeds=[b"Counter"], programId=FIVE_PROGRAM_ID
    const [counterPDA] = PublicKey.findProgramAddressSync(
        [Buffer.from("Counter")],
        scriptAccount
    );
    console.log(`Counter PDA: ${counterPDA.toBase58()}`);

    // Function indices: 0=initialize, 1=increment, 2=get_count
    const initInstruction = await buildExecuteInstruction(
        connection,
        payerKeypair,
        scriptAccount,
        vmStateAccount,
        0, // initialize
        [], // no extra params
        [
            { pubkey: counterPDA, isSigner: false, isWritable: true },
            { pubkey: payerKeypair.publicKey, isSigner: true, isWritable: true },
            { pubkey: new PublicKey("11111111111111111111111111111111"), isSigner: false, isWritable: false }, // System Program
        ]
    );

    await sendAndConfirm(connection, initInstruction, [payerKeypair]);
    console.log('Counter initialized (expected count = 42)');

    // 3. Increment Counter (Function index 1)
    console.log('\n--- Step 3: Incrementing Counter ---');
    const incInstruction = await buildExecuteInstruction(
        connection,
        payerKeypair,
        scriptAccount,
        vmStateAccount,
        1, // increment
        [],
        [
            { pubkey: counterPDA, isSigner: false, isWritable: true }
        ]
    );

    await sendAndConfirm(connection, incInstruction, [payerKeypair]);
    console.log('Counter incremented');

    // 4. Verify Count (Function index 2)
    console.log('\n--- Step 4: Verifying Count ---');
    const accountInfo = await connection.getAccountInfo(counterPDA);
    if (accountInfo) {
        console.log(`Counter Data Length: ${accountInfo.data.length}`);
        console.log('Raw Data (hex):', accountInfo.data.toString('hex'));

        // Counter data: discriminator (8 bytes) + count (u64, 8 bytes)
        if (accountInfo.data.length >= 16) {
            const count = accountInfo.data.readBigUInt64LE(8);
            console.log(`Count value read directly from account: ${count}`);
            if (count === 43n) {
                console.log('✅ SUCCESS: State persisted and incremented correctly!');
            } else {
                console.log(`❌ FAILURE: Expected 43, got ${count}`);
            }
        } else {
            console.log('❌ FAILURE: Account data too short!');
        }
    } else {
        console.log('❌ FAILURE: Counter account not found!');
    }
}

async function buildExecuteInstruction(connection, payer, scriptAccount, vmStateAccount, functionIndex, params, extraAccounts) {
    // Discriminator for ExecuteFunction is 9
    const discriminator = Buffer.from([9]);

    // Simple VLE encoding for function index
    let data = Buffer.concat([discriminator, Buffer.from([functionIndex])]);

    // Standard keys: script, vm_state, signer
    const keys = [
        { pubkey: scriptAccount, isSigner: false, isWritable: true },
        { pubkey: vmStateAccount, isSigner: false, isWritable: true },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    ];

    // Add extra accounts
    keys.push(...extraAccounts);

    return new TransactionInstruction({
        keys,
        programId: new PublicKey(FIVE_PROGRAM_ID_STR),
        data
    });
}


async function sendAndConfirm(connection, instruction, signers) {
    const tx = new Transaction().add(instruction);
    const sig = await connection.sendTransaction(tx, signers);
    await connection.confirmTransaction(sig);
    console.log(`Transaction confirmed: ${sig}`);
}

main().catch(console.error);
