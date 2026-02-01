#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection,
    Keypair,
    PublicKey,
    Transaction,
    TransactionInstruction,
    SystemProgram,
    LAMPORTS_PER_SOL,
    ComputeBudgetProgram
} from '@solana/web3.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const RPC_URL = process.env.RPC_URL || 'http://127.0.0.1:8899';
const FIVE_PROGRAM_ID = new PublicKey(process.env.FIVE_PROGRAM_ID || '6ndNfSrrGoFfTbS1sdJFybuJJyA6YQHtNgRdoXFREi8k');

const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const CYAN = '\x1b[36m';
const RED = '\x1b[31m';
const NC = '\x1b[0m';

async function deployRegisterOptimized() {
    console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}`);
    console.log(`${CYAN}Token Template - Register-Optimized Deployment (Smaller Chunks)${NC}`);
    console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}\n`);

    try {
        const connection = new Connection(RPC_URL, 'confirmed');
        const payerKeyPath = path.join(process.env.HOME, '.config/solana/id.json');
        const payer = Keypair.fromSecretKey(
            Uint8Array.from(JSON.parse(fs.readFileSync(payerKeyPath, 'utf-8')))
        );

        console.log(`${CYAN}▶ Configuration${NC}`);
        console.log(`  RPC URL: ${RPC_URL}`);
        console.log(`  Payer: ${payer.publicKey.toBase58()}`);
        console.log(`  Five Program: ${FIVE_PROGRAM_ID.toBase58()}\n`);

        const balance = await connection.getBalance(payer.publicKey);
        if (balance < 0.1 * LAMPORTS_PER_SOL) {
            console.log(`${RED}✗ Insufficient balance (need at least 0.1 SOL).${NC}`);
            process.exit(1);
        }

        const bytecodeFile = path.join(__dirname, 'build/five-token-registers.five');
        if (!fs.existsSync(bytecodeFile)) {
            console.log(`${RED}✗ File not found: ${bytecodeFile}${NC}`);
            process.exit(1);
        }

        const fiveFileContent = fs.readFileSync(bytecodeFile);
        let bytecode;
        try {
            const fiveFile = JSON.parse(fiveFileContent.toString('utf-8'));
            bytecode = new Uint8Array(Buffer.from(fiveFile.bytecode, 'base64'));
        } catch (e) {
            console.log(`${YELLOW}⚠ JSON parse failed, assuming raw binary...${NC}`);
            bytecode = new Uint8Array(fiveFileContent);
        }

        console.log(`  Bytecode size: ${bytecode.length} bytes`);

        // Helper for robust confirmation
        const confirmTx = async (signature, description) => {
            const latestBlockhash = await connection.getLatestBlockhash();
            const confirmation = await connection.confirmTransaction(
                { signature, ...latestBlockhash },
                'confirmed'
            );
            if (confirmation.value.err) {
                throw new Error(`${description} failed: ${JSON.stringify(confirmation.value.err)}`);
            }
            return signature;
        };

        // --- Deployment Logic ---

        // 1. VM State Account
        let vmStatePda;
        const globalVmStateKeypairPath = path.join(__dirname, '../../five-solana/target/deploy/vm-state-keypair.json');

        let vmStateKeypair;
        if (process.env.VM_STATE_PDA) {
            vmStatePda = new PublicKey(process.env.VM_STATE_PDA);
            console.log(`${CYAN}▶ Using provided VM State Account: ${vmStatePda.toBase58()}${NC}`);
        } else {
            if (fs.existsSync(globalVmStateKeypairPath)) {
                vmStateKeypair = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(globalVmStateKeypairPath, 'utf-8'))));
                console.log(`${CYAN}▶ Loaded global VM State Keypair: ${vmStateKeypair.publicKey.toBase58()}${NC}`);
            } else {
                vmStateKeypair = Keypair.generate();
                console.log(`${CYAN}▶ Generated new VM State Keypair: ${vmStateKeypair.publicKey.toBase58()}${NC}`);
            }
            vmStatePda = vmStateKeypair.publicKey;
        }

        // Check if VM State exists
        let vmStateInfo = await connection.getAccountInfo(vmStatePda);

        if (!vmStateInfo && vmStateKeypair) {
            console.log(`${CYAN}▶ VM State account not found on-chain. Creating/Initializing...${NC}`);
            const VM_STATE_SIZE = 56;
            const vmStateRent = await connection.getMinimumBalanceForRentExemption(VM_STATE_SIZE);

            const vmStateTx = new Transaction().add(
                ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }),
                SystemProgram.createAccount({
                    fromPubkey: payer.publicKey,
                    newAccountPubkey: vmStateKeypair.publicKey,
                    lamports: vmStateRent,
                    space: VM_STATE_SIZE,
                    programId: FIVE_PROGRAM_ID,
                }),
                new TransactionInstruction({
                    keys: [
                        { pubkey: vmStateKeypair.publicKey, isSigner: false, isWritable: true },
                        { pubkey: payer.publicKey, isSigner: true, isWritable: false },
                    ],
                    programId: FIVE_PROGRAM_ID,
                    data: Buffer.from([0, 255]),
                })
            );

            const vmSig = await connection.sendTransaction(vmStateTx, [payer, vmStateKeypair], { skipPreflight: true });
            await confirmTx(vmSig, 'VM State Creation');
            console.log(`  VM State initialized: ${vmStateKeypair.publicKey.toBase58()}`);

            vmStateInfo = await connection.getAccountInfo(vmStatePda);
        }

        if (!vmStateInfo) {
            console.error(`${RED}Error: VM State account created but not found!${NC}`);
            process.exit(1);
        }
        if (!vmStateInfo.owner.equals(FIVE_PROGRAM_ID)) {
            console.error(`${RED}Error: VM State owned by ${vmStateInfo.owner.toBase58()}, expected ${FIVE_PROGRAM_ID.toBase58()}${NC}`);
            process.exit(1);
        }
        console.log(`  VM State Owner Verified: ${vmStateInfo.owner.toBase58()}`);

        // 2. Create Script Account
        const scriptKeypair = Keypair.generate();
        const SCRIPT_HEADER_SIZE = 64;

        const finalScriptSize = SCRIPT_HEADER_SIZE + bytecode.length;
        const rentRequired = await connection.getMinimumBalanceForRentExemption(finalScriptSize);
        const REALLOCATION_BUFFER = 0.01 * LAMPORTS_PER_SOL;
        const initialLamports = rentRequired + REALLOCATION_BUFFER;

        console.log(`${CYAN}▶ Creating Script Account...${NC}`);
        console.log(`  Final size: ${finalScriptSize} bytes`);
        console.log(`  Rent required: ${rentRequired} lamports`);
        console.log(`  Initial funding: ${initialLamports} lamports (${(initialLamports / LAMPORTS_PER_SOL).toFixed(8)} SOL)`);

        const initTx = new Transaction().add(
            ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }),
            SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: scriptKeypair.publicKey,
                lamports: initialLamports,
                space: SCRIPT_HEADER_SIZE,
                programId: FIVE_PROGRAM_ID,
            }),
            new TransactionInstruction({
                keys: [
                    { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
                    { pubkey: vmStatePda, isSigner: false, isWritable: true },
                ],
                programId: FIVE_PROGRAM_ID,
                data: Buffer.concat([
                    Buffer.from([4]), // InitLargeProgram
                    Buffer.from(new Uint32Array([bytecode.length]).buffer)
                ]),
            })
        );

        const initSig = await connection.sendTransaction(initTx, [payer, scriptKeypair], { skipPreflight: true });
        await confirmTx(initSig, 'Script Account Init');
        console.log(`  Script Account: ${scriptKeypair.publicKey.toBase58()}`);

        await new Promise(r => setTimeout(r, 1000));

        // 3. Append Chunks (SMALLER SIZE: 200 bytes instead of 400)
        const CHUNK_SIZE = 200;
        const chunks = [];
        for (let i = 0; i < bytecode.length; i += CHUNK_SIZE) {
            chunks.push(bytecode.slice(i, Math.min(i + CHUNK_SIZE, bytecode.length)));
        }

        console.log(`${CYAN}▶ Appending ${chunks.length} chunks (200 bytes each)...${NC}`);

        for (let i = 0; i < chunks.length; i++) {
            const chunk = chunks[i];

            const appendTx = new Transaction();
            appendTx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }));

            appendTx.add(new TransactionInstruction({
                keys: [
                    { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
                    { pubkey: vmStatePda, isSigner: false, isWritable: true },
                    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
                ],
                programId: FIVE_PROGRAM_ID,
                data: Buffer.concat([
                    Buffer.from([5]), // AppendBytecode
                    chunk
                ]),
            }));

            appendTx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
            appendTx.feePayer = payer.publicKey;

            const appendSig = await connection.sendTransaction(appendTx, [payer], { skipPreflight: true });
            await confirmTx(appendSig, `Chunk ${i} append`);
            process.stdout.write('.');
        }

        console.log('\n');
        console.log(`${GREEN}✓ Register-optimized bytecode deployed successfully!${NC}`);
        console.log(`${CYAN}▶ Deployment Details${NC}`);
        console.log(`  Script Account: ${scriptKeypair.publicKey.toBase58()}`);
        console.log(`  VM State PDA: ${vmStatePda.toBase58()}`);
        console.log(`  Five Program ID: ${FIVE_PROGRAM_ID.toBase58()}`);
        console.log(`  Bytecode Size: ${bytecode.length} bytes`);

        // Save deployment config
        const deploymentConfig = {
            tokenScriptAccount: scriptKeypair.publicKey.toBase58(),
            fiveProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStatePda: vmStatePda.toBase58(),
            rpcUrl: RPC_URL,
            timestamp: new Date().toISOString(),
            bytecodeSize: bytecode.length,
            type: "register-optimized"
        };

        fs.writeFileSync('deployment-config.json', JSON.stringify(deploymentConfig, null, 2));
        console.log(`  Config saved to deployment-config.json`);

    } catch (error) {
        console.error(`${RED}✗ Deployment failed:${NC}`, error.message);
        process.exit(1);
    }
}

deployRegisterOptimized();
