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
const FIVE_PROGRAM_ID = new PublicKey(process.env.FIVE_PROGRAM_ID || 'DmBJLjdfSidk5SYMscpRZJeiyMqeBZvir1nHAVZZvAX8');

const GREEN = '\x1b[32m';
const YELLOW = '\x1b[33m';
const CYAN = '\x1b[36m';
const RED = '\x1b[31m';
const NC = '\x1b[0m';

async function deployTokenProgram() {
    console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}`);
    console.log(`${CYAN}Token Template - Five VM Deployment (Robust)${NC}`);
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

        const bytecodeFile = path.join(__dirname, 'build/five-token-template.five');
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

        // 1. Setup VM State Account
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
                    data: Buffer.from([0, 255]), // Initialize discriminator + bump byte
                })
            );

            const vmSig = await connection.sendTransaction(vmStateTx, [payer, vmStateKeypair], { skipPreflight: true });
            await confirmTx(vmSig, 'VM State Creation');
            console.log(`  VM State initialized: ${vmStateKeypair.publicKey.toBase58()} (${vmSig})`);

            // Refresh info
            vmStateInfo = await connection.getAccountInfo(vmStatePda);
        }

        // Check VM State Owner
        // Check VM State Owner
        if (!vmStateInfo) {
            console.error(`${RED}Error: VM State account created but not found!${NC}`);
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

        // Calculate actual rent needed for final script size
        const finalScriptSize = SCRIPT_HEADER_SIZE + bytecode.length;
        const rentRequired = await connection.getMinimumBalanceForRentExemption(finalScriptSize);
        // Add small buffer to handle reallocation overhead (bytecode is small, so buffer is small)
        const REALLOCATION_BUFFER = 0.01 * LAMPORTS_PER_SOL;  // 0.01 SOL buffer
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
                space: SCRIPT_HEADER_SIZE,  // Start with header size as expected
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
                    Buffer.from(new Uint32Array([bytecode.length]).buffer) // expected_size
                ]),
            })
        );

        const initSig = await connection.sendTransaction(initTx, [payer, scriptKeypair], { skipPreflight: true });
        await confirmTx(initSig, 'Script Account Init');
        console.log(`  Script Account: ${scriptKeypair.publicKey.toBase58()} (${initSig})`);

        // Wait for account to be visible
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

            // Calculate size based on LOCAL tracking
            const newSize = currentSize + chunk.length;

            // Pre-funded, so no need to transfer additional rent
            // const oldRent = ...

            const appendTx = new Transaction();
            appendTx.add(ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }));
            // appendTx.add(SystemProgram.transfer({...}));

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
            const msg = appendTx.compileMessage();
            console.log(`DEBUG: Chunk ${i} keys:`, msg.accountKeys.map(k => k.toBase58()));

            const appendSig = await connection.sendTransaction(appendTx, [payer], { skipPreflight: true });
            await confirmTx(appendSig, `Chunk ${i} append`);
            process.stdout.write('.');

            // Update current size for next iteration for ACCURATE rent calculation
            currentSize = newSize;
        }
        console.log(`\n${GREEN}✓ All chunks appended.${NC}\n`);

        // 4. Finalize the script upload (discriminator 7)
        // This marks upload_complete = true, allowing Execute calls to succeed
        console.log(`${CYAN}▶ Finalizing script upload...${NC}`);
        const finalizeTx = new Transaction().add(
            ComputeBudgetProgram.setComputeUnitLimit({ units: 1400000 }),
            new TransactionInstruction({
                keys: [
                    { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: false },
                    { pubkey: vmStatePda, isSigner: false, isWritable: true },
                ],
                programId: FIVE_PROGRAM_ID,
                data: Buffer.from([7]), // FinalizeScript discriminator
            })
        );

        const finalizeSig = await connection.sendTransaction(finalizeTx, [payer], { skipPreflight: true });
        await confirmTx(finalizeSig, 'Finalize Script');
        console.log(`${GREEN}✓ Script finalized: ${finalizeSig}${NC}\n`);

        const tokenScriptAccount = scriptKeypair.publicKey.toBase58();
        const vmStatePdaString = vmStatePda.toBase58();

        console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}`);
        console.log(`${GREEN}✓ Deployment Complete${NC}\n`);
        console.log(`  Script Account: ${tokenScriptAccount}`);
        console.log(`  VM State: ${vmStatePdaString}\n`);

        // Save config
        const config = {
            tokenScriptAccount: tokenScriptAccount,
            fiveProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStatePda: vmStatePdaString,
            rpcUrl: RPC_URL,
            timestamp: new Date().toISOString(),
        };

        fs.writeFileSync('deployment-config.json', JSON.stringify(config, null, 2));
        console.log(`${GREEN}✓ Config saved to deployment-config.json${NC}\n`);

        console.log(`${YELLOW}Next steps:${NC}`);
        console.log(`  1. Update constants in verify-state.mjs and e2e-token-test.mjs`);

    } catch (error) {
        console.error(`\n${RED}Error: ${error.message}${NC}`);
        console.error(error);
        process.exit(1);
    }
}

deployTokenProgram();
