#!/usr/bin/env node
/**
 * Token Template E2E Test - Using Five SDK
 */

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    TransactionInstruction, LAMPORTS_PER_SOL
} from '@solana/web3.js';
import { FiveSDK } from '../../five-sdk/dist/index.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

const FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
const VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');
const TOKEN_SCRIPT_ACCOUNT = new PublicKey('GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ');

function loadKeypair(kpPath) {
    const secretKey = JSON.parse(fs.readFileSync(kpPath, 'utf-8'));
    return Keypair.fromSecretKey(Uint8Array.from(secretKey));
}

function loadABI() {
    const fiveFile = JSON.parse(fs.readFileSync(
        path.join(__dirname, 'build', 'five-token-template.five'), 'utf-8'
    ));

    // Transform ABI: filter out account parameters and rename param_type to type
    const transformedAbi = {
        ...fiveFile.abi,
        functions: fiveFile.abi.functions.map(fn => ({
            ...fn,
            parameters: fn.parameters
                .filter(p => !p.is_account)  // Filter out account parameters
                .map(p => ({
                    ...p,
                    type: p.param_type  // SDK expects 'type' not 'param_type'
                }))
        }))
    };

    return transformedAbi;
}

async function main() {
    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = loadKeypair(PAYER_KEYPAIR_PATH);
    const abi = loadABI();

    console.log('Payer:', payer.publicKey.toBase58());
    console.log('ABI functions:', abi.functions.length);

    // Generate fresh keypairs
    const mintAccount = Keypair.generate();
    const user1 = Keypair.generate();

    console.log('Mint account:', mintAccount.publicKey.toBase58());
    console.log('User1 (authority):', user1.publicKey.toBase58());

    // Fund the mint account and user1 with lamports (creates System-owned accounts with 0 data)
    // All accounts passed to the VM must have lamports to pass lazy validation
    console.log('Funding accounts...');
    const airdrop1 = await connection.requestAirdrop(mintAccount.publicKey, 0.1 * LAMPORTS_PER_SOL);
    const airdrop2 = await connection.requestAirdrop(user1.publicKey, 0.1 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(airdrop1, 'confirmed');
    await connection.confirmTransaction(airdrop2, 'confirmed');
    console.log('Accounts funded');

    // Use Five SDK to generate execute instruction
    console.log('\nGenerating execute instruction via Five SDK...');

    // For init_mint:
    // Account params: mint_account (@init), authority (@signer)
    // Data params: freeze_authority (pubkey), decimals (u8), name (string), symbol (string), uri (string)

    const executionData = await FiveSDK.generateExecuteInstruction(
        TOKEN_SCRIPT_ACCOUNT.toBase58(),
        0,  // init_mint function index
        [
            user1.publicKey.toBase58(),  // freeze_authority
            6,                           // decimals
            "TestToken",                 // name
            "TEST",                      // symbol
            "https://example.com"        // uri
        ],
        [
            mintAccount.publicKey.toBase58(),  // mint_account (@init)
            user1.publicKey.toBase58(),        // authority (@signer)
            payer.publicKey.toBase58()         // payer for rent
        ],
        null,  // no connection needed, we provide ABI
        {
            debug: true,
            vmStateAccount: VM_STATE_PDA.toBase58(),
            fiveVMProgramId: FIVE_PROGRAM_ID.toBase58(),
            abi: abi
        }
    );

    console.log('\nSDK returned:');
    console.log('  Instruction data (base64):', executionData.instruction.data);
    console.log('  Accounts:', executionData.instruction.accounts.length);

    // Build the transaction instruction
    const instructionData = Buffer.from(executionData.instruction.data, 'base64');
    console.log('  Instruction data (hex):', instructionData.toString('hex'));
    console.log('  Instruction data length:', instructionData.length);

    // Convert SDK accounts to web3.js format
    const keys = executionData.instruction.accounts.map(acc => ({
        pubkey: new PublicKey(acc.pubkey),
        isSigner: acc.isSigner,
        isWritable: acc.isWritable
    }));

    // Override signer/writable flags based on ABI
    // Account 0 = script (readonly)
    // Account 1 = vm_state (writable)
    // Account 2 = mint_account (@init, @mut, signer for creation)
    // Account 3 = authority (@signer)
    // Account 4 = payer
    if (keys.length >= 5) {
        keys[2].isSigner = true;  // mint_account needs to sign for creation
        keys[2].isWritable = true;
        keys[3].isSigner = true;  // authority
        keys[4].isSigner = true;  // payer
        keys[4].isWritable = true;
    }

    console.log('\nAccount order:');
    keys.forEach((k, i) => console.log('  ' + i + ': ' + k.pubkey.toBase58() + ' (signer=' + k.isSigner + ', writable=' + k.isWritable + ')'));

    const ix = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys,
        data: instructionData
    });

    const tx = new Transaction().add(ix);

    console.log('\nSending transaction...');
    try {
        const txSig = await connection.sendTransaction(tx, [payer, mintAccount, user1], { skipPreflight: true });
        await connection.confirmTransaction(txSig, 'confirmed');

        const txDetails = await connection.getTransaction(txSig, {
            maxSupportedTransactionVersion: 0,
            commitment: 'confirmed'
        });

        console.log('Transaction succeeded!');
        console.log('Compute units:', txDetails?.meta?.computeUnitsConsumed);
        if (txDetails?.meta?.logMessages) {
            console.log('\nProgram logs:');
            txDetails.meta.logMessages.forEach(log => console.log('  ', log));
        }

        // Check if mint account was created
        const mintInfo = await connection.getAccountInfo(mintAccount.publicKey);
        if (mintInfo) {
            console.log('\nMint account created!');
            console.log('  Owner:', mintInfo.owner.toBase58());
            console.log('  Data length:', mintInfo.data.length);
            console.log('  Lamports:', mintInfo.lamports);
        }

    } catch (e) {
        console.error('Transaction failed:', e.message);
        if (e.logs) {
            console.log('\nProgram logs:');
            e.logs.forEach(log => console.log('  ', log));
        }
    }
}

main().catch(console.error);
