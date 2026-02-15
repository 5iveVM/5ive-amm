#!/usr/bin/env node

/**
 * Deploy AMM template to Five VM on localnet
 * Uses chunked deployment via InitLargeProgram + AppendBytecode
 */

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
const FIVE_PROGRAM_ID = new PublicKey(process.env.FIVE_PROGRAM_ID || '3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1');
const VM_STATE_PDA = process.env.VM_STATE_PDA || 'AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit';

const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const CYAN = '\x1b[36m';
const RED = '\x1b[31m';
const NC = '\x1b[0m';

async function deployAMM() {
    console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}`);
    console.log(`${CYAN}AMM Template - Five VM Deployment${NC}`);
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

        const artifactName = process.env.FIVE_ARTIFACT || 'five-amm-baseline.five';
        const bytecodeFile = path.join(__dirname, 'build', artifactName);
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

        // 1. Use hardcoded VM State Account
        const vmStatePda = new PublicKey(VM_STATE_PDA);
        console.log(`${CYAN}▶ Using hardcoded VM State Account: ${vmStatePda.toBase58()}${NC}`);

        let vmStateInfo = await connection.getAccountInfo(vmStatePda);

        if (!vmStateInfo) {
            console.error(`${RED}Error: VM State account not found on-chain!${NC}`);
            console.error(`  Expected: ${vmStatePda.toBase58()}`);
            console.error(`  Initialize with: node scripts/init-localnet-vm-state.mjs${NC}`);
            process.exit(1);
        }

        if (!vmStateInfo.owner.equals(FIVE_PROGRAM_ID)) {
            console.error(`${RED}Error: VM State owned by ${vmStateInfo.owner.toBase58()}, expected ${FIVE_PROGRAM_ID.toBase58()}${NC}`);
            process.exit(1);
        }
        console.log(`  VM State Owner Verified: ${vmStateInfo.owner.toBase58()}`);

        // 2. Create Script Account & Init
        const scriptKeypair = Keypair.generate();
        const SCRIPT_HEADER_SIZE = 64;

        const finalScriptSize = SCRIPT_HEADER_SIZE + bytecode.length;
        const rentRequired = await connection.getMinimumBalanceForRentExemption(finalScriptSize);
        const REALLOCATION_BUFFER = 0.01 * LAMPORTS_PER_SOL;
        const initialLamports = rentRequired + REALLOCATION_BUFFER;

        if (!FIVE_PROGRAM_ID) {
            console.log(`${RED}✗ Five Program ID not set!${NC}`);
            process.exit(1);
        }
        console.log(`  Script will be owned by: ${FIVE_PROGRAM_ID.toBase58()}`);

        console.log(`${CYAN}▶ Creating Script Account...${NC}`);
        console.log(`  Final size: ${finalScriptSize} bytes`);
        console.log(`  Rent required: ${rentRequired} lamports`);
        console.log(`  Initial funding: ${initialLamports} lamports (${(initialLamports / LAMPORTS_PER_SOL).toFixed(8)} SOL)`);

        // Fee vault account (hardcoded shard 0)
        const FEE_VAULT_0 = new PublicKey('HXW6bZsdJW6Be5c51NNpNb9NcVxmHbUrF9oKkt4C1tEH');

        const initTx = new Transaction().add(
            ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }),
            SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: scriptKeypair.publicKey,
                lamports: initialLamports,
                space: finalScriptSize,
                programId: FIVE_PROGRAM_ID,
            }),
            new TransactionInstruction({
                keys: [
                    { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
                    { pubkey: vmStatePda, isSigner: false, isWritable: true },
                    { pubkey: FEE_VAULT_0, isSigner: false, isWritable: true },
                    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
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
        console.log(`  Script Account: ${scriptKeypair.publicKey.toBase58()} (${initSig})`);

        await new Promise(r => setTimeout(r, 1000));

        // 3. Append Chunks
        const CHUNK_SIZE = 400;
        const chunks = [];
        for (let i = 0; i < bytecode.length; i += CHUNK_SIZE) {
            chunks.push(bytecode.slice(i, Math.min(i + CHUNK_SIZE, bytecode.length)));
        }

        console.log(`${CYAN}▶ Appending ${chunks.length} chunks...${NC}`);

        let currentSize = SCRIPT_HEADER_SIZE;

        for (let i = 0; i < chunks.length; i++) {
            const chunk = chunks[i];
            const newSize = currentSize + chunk.length;

            const appendTx = new Transaction();
            appendTx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }));

            appendTx.add(new TransactionInstruction({
                keys: [
                    { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
                    { pubkey: vmStatePda, isSigner: false, isWritable: true },
                    { pubkey: FEE_VAULT_0, isSigner: false, isWritable: true },
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

            currentSize = newSize;
        }
        console.log(`\n${GREEN}✓ All chunks appended.${NC}\n`);

        // 4. Finalize the script upload
        console.log(`${CYAN}▶ Finalizing script upload...${NC}`);
        const finalizeTx = new Transaction().add(
            ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }),
            new TransactionInstruction({
                keys: [
                    { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: false },
                    { pubkey: vmStatePda, isSigner: false, isWritable: true },
                    { pubkey: FEE_VAULT_0, isSigner: false, isWritable: true },
                    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
                ],
                programId: FIVE_PROGRAM_ID,
                data: Buffer.from([7]), // FinalizeScript discriminator
            })
        );

        const finalizeSig = await connection.sendTransaction(finalizeTx, [payer], { skipPreflight: true });
        await confirmTx(finalizeSig, 'Finalize Script');
        console.log(`${GREEN}✓ Script finalized: ${finalizeSig}${NC}\n`);

        const ammScriptAccount = scriptKeypair.publicKey.toBase58();
        const vmStatePdaString = vmStatePda.toBase58();

        console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}`);
        console.log(`${GREEN}✓ Deployment Complete${NC}\n`);
        console.log(`  Script Account: ${ammScriptAccount}`);
        console.log(`  VM State: ${vmStatePdaString}\n`);

        // Save config
        const config = {
            ammScriptAccount: ammScriptAccount,
            fiveProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStatePda: vmStatePdaString,
            rpcUrl: RPC_URL,
            timestamp: new Date().toISOString(),
        };

        fs.writeFileSync(path.join(__dirname, 'deployment-config.json'), JSON.stringify(config, null, 2));
        console.log(`${GREEN}✓ Config saved to deployment-config.json${NC}\n`);

        // Verify account ownership
        console.log(`${CYAN}▶ Verifying account ownership...${NC}`);
        const finalScriptInfo = await connection.getAccountInfo(new PublicKey(ammScriptAccount));
        const finalVmStateInfo = await connection.getAccountInfo(new PublicKey(vmStatePdaString));

        if (finalScriptInfo && finalScriptInfo.owner.equals(FIVE_PROGRAM_ID)) {
            console.log(`  ${GREEN}✓ Script account owner correct${NC}`);
        } else {
            console.log(`  ${RED}✗ Script account owner WRONG!${NC}`);
            if (finalScriptInfo) {
                console.log(`    Current: ${finalScriptInfo.owner.toBase58()}`);
                console.log(`    Expected: ${FIVE_PROGRAM_ID.toBase58()}`);
            }
        }

        if (finalVmStateInfo && finalVmStateInfo.owner.equals(FIVE_PROGRAM_ID)) {
            console.log(`  ${GREEN}✓ VM state owner correct${NC}\n`);
        } else {
            console.log(`  ${RED}✗ VM state owner WRONG!${NC}`);
        }

    } catch (error) {
        console.error(`\n${RED}Error: ${error.message}${NC}`);
        console.error(error);
        process.exit(1);
    }
}

deployAMM();
