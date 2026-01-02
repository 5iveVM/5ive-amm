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

async function deployCounterProgram() {
    console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}`);
    console.log(`${CYAN}Counter Template - Five VM Deployment (Single-Chunk)${NC}`);
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

        const bytecodeFile = path.join(__dirname, 'build/five-counter-template.five');
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

        // --- Deployment Logic (SINGLE-CHUNK) ---

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

        // 2. Create Script Account & Deploy in Single Transaction
        const scriptKeypair = Keypair.generate();
        const SCRIPT_HEADER_SIZE = 64;
        const totalSize = SCRIPT_HEADER_SIZE + bytecode.length;
        const rentLamports = await connection.getMinimumBalanceForRentExemption(totalSize);

        console.log(`${CYAN}▶ Deploying Counter Script (Single-Chunk)...${NC}`);
        console.log(`  Total size: ${totalSize} bytes (header: ${SCRIPT_HEADER_SIZE}, bytecode: ${bytecode.length})`);
        console.log(`  Rent required: ${rentLamports} lamports`);

        const deployTx = new Transaction().add(
            SystemProgram.createAccount({
                fromPubkey: payer.publicKey,
                newAccountPubkey: scriptKeypair.publicKey,
                lamports: rentLamports,
                space: totalSize,
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
                    Buffer.from([8]), // Deploy discriminator
                    Buffer.from(new Uint32Array([bytecode.length]).buffer), // Bytecode length (u32 LE)
                    Buffer.from([0]), // Permissions (0 = no special permissions)
                    bytecode
                ]),
            })
        );

        const deploySig = await connection.sendTransaction(deployTx, [payer, scriptKeypair], { skipPreflight: false });
        await connection.confirmTransaction(deploySig, 'confirmed');
        console.log(`${GREEN}✓ Deployment successful: ${deploySig}${NC}`);
        console.log(`  Script Account: ${scriptKeypair.publicKey.toBase58()}\n`);

        const counterScriptAccount = scriptKeypair.publicKey.toBase58();
        const vmStatePdaString = vmStatePda.toBase58();

        console.log(`${CYAN}═══════════════════════════════════════════════════════════${NC}`);
        console.log(`${GREEN}✓ Deployment Complete${NC}\n`);
        console.log(`  Script Account: ${counterScriptAccount}`);
        console.log(`  VM State: ${vmStatePdaString}\n`);

        // Save config
        const config = {
            counterScriptAccount: counterScriptAccount,
            fiveProgramId: FIVE_PROGRAM_ID.toBase58(),
            vmStatePda: vmStatePdaString,
            rpcUrl: RPC_URL,
            timestamp: new Date().toISOString(),
        };

        fs.writeFileSync('deployment-config.json', JSON.stringify(config, null, 2));
        console.log(`${GREEN}✓ Config saved to deployment-config.json${NC}\n`);

        console.log(`${YELLOW}Next steps:${NC}`);
        console.log(`  1. Run e2e-counter-test.mjs to test the counter program`);

    } catch (error) {
        console.error(`\n${RED}Error: ${error.message}${NC}`);
        console.error(error);
        process.exit(1);
    }
}

deployCounterProgram();
