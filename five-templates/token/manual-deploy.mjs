import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { Connection, Keypair, PublicKey } from '@solana/web3.js';
import { FiveSDK } from '../../five-sdk/dist/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Config
const RPC_URL = process.env.RPC_URL || 'http://127.0.0.1:8899';
const BYTECODE_PATH = path.join(__dirname, 'simple_test.five');
const PAYER_PATH = process.env.HOME + '/.config/solana/id.json';

// Existing VM State from deploy-and-init.sh
const VM_STATE_PDA_STR = 'DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys';
const FIVE_PROGRAM_ID_STR = '9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH';

async function main() {
    console.log(`Connecting to ${RPC_URL}...`);
    const connection = new Connection(RPC_URL, 'confirmed');

    // Load Payer
    const payerKeypair = Keypair.fromSecretKey(
        new Uint8Array(JSON.parse(fs.readFileSync(PAYER_PATH, 'utf-8')))
    );
    console.log(`Payer: ${payerKeypair.publicKey.toBase58()}`);

    // Load Bytecode
    if (!fs.existsSync(BYTECODE_PATH)) {
        throw new Error(`Bytecode not found at ${BYTECODE_PATH}`);
    }
    const fileContent = fs.readFileSync(BYTECODE_PATH, 'utf-8');
    let bytecode;
    try {
        const json = JSON.parse(fileContent);
        if (json.bytecode) {
            bytecode = Buffer.from(json.bytecode, 'base64');
            console.log(`Loaded .five file, extracted bytecode (${bytecode.length} bytes)`);
        } else {
            bytecode = Buffer.from(fileContent); // Assume raw if no bytecode field
        }
    } catch (e) {
        bytecode = fs.readFileSync(BYTECODE_PATH); // Fallback to raw binary
        console.log(`Loaded raw bytecode (${bytecode.length} bytes)`);
    }

    // Deploy
    console.log('Deploying via FiveSDK...');

    // Note: deployLargeProgramToSolana arguments based on usage patterns
    // (connection, payer, bytecode, options)
    // Options often include: vmStateAccount, fiveVMProgramId

    try {
        const result = await FiveSDK.deployLargeProgramToSolana(
            bytecode, // 1st arg
            connection, // 2nd arg
            payerKeypair, // 3rd arg
            {
                vmStateAccount: VM_STATE_PDA_STR, // Pass as string, SDK will parse
                fiveVMProgramId: FIVE_PROGRAM_ID_STR,
                chunkSize: 500,
                debug: true
            }
        );

        if (result.success) {
            console.log(`\nDeployment Successful!`);
            console.log(`Script Account: ${result.scriptAccount}`);
            console.log(`Program ID: ${result.scriptAccount}`); // For compatibility

            // Output explicit JSON for shell script to parse easily (if I were using shell parsing)
            console.log(`JSON_OUTPUT: ${JSON.stringify({
                scriptAccount: result.scriptAccount,
                vmStateAccount: result.vmStateAccount
            })}`);
        } else {
            console.error('Deployment Failed:', result.error);
            process.exit(1);
        }
    } catch (err) {
        console.error('Deployment Failed:', err);
        process.exit(1);
    }
}

main();
