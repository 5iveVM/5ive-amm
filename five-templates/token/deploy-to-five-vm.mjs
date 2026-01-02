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
    LAMPORTS_PER_SOL
} from '@solana/web3.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const RPC_URL = process.env.RPC_URL || 'http://127.0.0.1:8899';
const FIVE_PROGRAM_ID = new PublicKey(process.env.FIVE_PROGRAM_ID || '9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');

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
        if (balance < 1 * LAMPORTS_PER_SOL) {
            console.log(`${RED}✗ Insufficient balance.${NC}`);
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

        // --- Deployment Logic ---

        // 1. Setup VM State Account
        let vmStatePda;

        if (process.env.VM_STATE_PDA) {
            vmStatePda = new PublicKey(process.env.VM_STATE_PDA);
            console.log(`${CYAN}▶ Using provided VM State Account: ${vmStatePda.toBase58()}${NC}`);
        } else {
            const vmStateKeypair = Keypair.generate();
            const VM_STATE_SIZE = 56;
            const vmStateRent = await connection.getMinimumBalanceForRentExemption(VM_STATE_SIZE);

            console.log(`${CYAN}▶ Creating VM State Account...${NC}`);
            const vmStateTx = new Transaction().add(
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
                    data: Buffer.from([0]), // Initialize discriminator
                })
            );

            const vmSig = await connection.sendTransaction(vmStateTx, [payer, vmStateKeypair], { skipPreflight: false });
            await connection.confirmTransaction(vmSig, 'confirmed');
            console.log(`  VM State: ${vmStateKeypair.publicKey.toBase58()} (${vmSig})`);
            vmStatePda = vmStateKeypair.publicKey;
        }

        // Check VM State Owner
        const vmStateInfo = await connection.getAccountInfo(vmStatePda);
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
        const rentLamports = await connection.getMinimumBalanceForRentExemption(SCRIPT_HEADER_SIZE); // Initially just header

        console.log(`${CYAN}▶ Creating Script Account...${NC}`);
        const initTx = new Transaction().add(
            SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: scriptKeypair.publicKey,
                lamports: rentLamports,
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
                    Buffer.from(new Uint32Array([bytecode.length]).buffer) // expected_size
                ]),
            })
        );

        const initSig = await connection.sendTransaction(initTx, [payer, scriptKeypair], { skipPreflight: true });
        await connection.confirmTransaction(initSig, 'confirmed');
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

        for (let i = 0; i < chunks.length; i++) {
            const chunk = chunks[i];

            // Retry getAccountInfo
            let currentInfo = null;
            let retries = 5;
            while (!currentInfo && retries > 0) {
                currentInfo = await connection.getAccountInfo(scriptKeypair.publicKey, 'confirmed');
                if (!currentInfo) {
                    console.log(`  Retrying getAccountInfo... (${retries})`);
                    await new Promise(r => setTimeout(r, 1000));
                    retries--;
                }
            }
            if (!currentInfo) throw new Error("Could not fetch script account info");

            const newSize = currentInfo.data.length + chunk.length;
            const newRentRequired = await connection.getMinimumBalanceForRentExemption(newSize);
            const additionalRent = Math.max(0, newRentRequired - currentInfo.lamports);

            const appendTx = new Transaction();
            if (additionalRent > 0) {
                appendTx.add(SystemProgram.transfer({
                    fromPubkey: payer.publicKey,
                    toPubkey: scriptKeypair.publicKey,
                    lamports: additionalRent,
                }));
            }

            appendTx.add(new TransactionInstruction({
                keys: [
                    { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
                    { pubkey: vmStatePda, isSigner: false, isWritable: true },
                ],
                programId: FIVE_PROGRAM_ID,
                data: Buffer.concat([
                    Buffer.from([5]), // AppendBytecode
                    chunk
                ]),
            }));

            const appendSig = await connection.sendTransaction(appendTx, [payer], { skipPreflight: true });
            await connection.confirmTransaction(appendSig, 'confirmed');
            process.stdout.write('.');
        }
        console.log(`\n${GREEN}✓ All chunks appended.${NC}\n`);

        // 4. Finalize the script upload (discriminator 7)
        // This marks upload_complete = true, allowing Execute calls to succeed
        console.log(`${CYAN}▶ Finalizing script upload...${NC}`);
        const finalizeTx = new Transaction().add(
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
        await connection.confirmTransaction(finalizeSig, 'confirmed');
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
