#!/usr/bin/env node
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import {
    Connection, Keypair, PublicKey, Transaction,
    TransactionInstruction, LAMPORTS_PER_SOL
} from '@solana/web3.js';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const RPC_URL = 'http://127.0.0.1:8899';
const PAYER_KEYPAIR_PATH = process.env.HOME + '/.config/solana/id.json';

const FIVE_PROGRAM_ID = new PublicKey('9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH');
const VM_STATE_PDA = new PublicKey('DRsZtpCF8Np1MsQixQPH4iQYTKhEkZMzNCTv15RCYys');
const TOKEN_SCRIPT_ACCOUNT = new PublicKey('GvB7xAifdP5uBkSuDReuqQo3UoyMBPnNb45VD7CobrbZ');

function encodeVLE(value) {
    const buffer = [];
    let val = BigInt(value);
    if (val === 0n) return Buffer.from([0]);
    while (val > 0n) {
        let byte = Number(val & 0x7Fn);
        val >>= 7n;
        if (val > 0n) byte |= 0x80;
        buffer.push(byte);
    }
    return Buffer.from(buffer);
}

function loadKeypair(kpPath) {
    const secretKey = JSON.parse(fs.readFileSync(kpPath, 'utf-8'));
    return Keypair.fromSecretKey(Uint8Array.from(secretKey));
}

async function main() {
    const connection = new Connection(RPC_URL, 'confirmed');
    const payer = loadKeypair(PAYER_KEYPAIR_PATH);

    console.log('Payer:', payer.publicKey.toBase58());

    // Generate fresh keypairs
    const mintAccount = Keypair.generate();
    const user1 = Keypair.generate();  // authority

    console.log('Mint account:', mintAccount.publicKey.toBase58());
    console.log('User1 (authority):', user1.publicKey.toBase58());

    // Fund the mint account with lamports (creates a System-owned account with 0 data)
    // This is needed because @init accounts must have lamports but no data
    console.log('Funding mint account...');
    const airdropSig = await connection.requestAirdrop(mintAccount.publicKey, 0.1 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(airdropSig, 'confirmed');
    console.log('Mint account funded');

    // Build init_mint instruction
    // Function index 0, 5 non-account params
    const discriminator = Buffer.from([9]); // ExecuteFunction
    const functionIndex = encodeVLE(0);
    const paramCount = encodeVLE(5);

    // freeze_authority (pubkey)
    const freezeAuth = Buffer.from(user1.publicKey.toBytes());
    // decimals (u8)
    const decimals = Buffer.from([6]);
    // name (string)
    const name = Buffer.from([9, ...Buffer.from('TestToken', 'utf-8')]);
    // symbol (string)
    const symbol = Buffer.from([4, ...Buffer.from('TEST', 'utf-8')]);
    // uri (string)
    const uri = Buffer.from([19, ...Buffer.from('https://example.com', 'utf-8')]);

    const instructionData = Buffer.concat([
        discriminator,
        functionIndex,
        paramCount,
        freezeAuth,
        decimals,
        name,
        symbol,
        uri
    ]);

    console.log('Instruction data:', instructionData.toString('hex'));
    console.log('Instruction data length:', instructionData.length);

    // Account order:
    // 0: script account
    // 1: vm state pda
    // 2: mint_account (@init, @mut) - fresh keypair, must be signer
    // 3: authority (@signer)
    // 4: payer (for rent) - need to include payer for the CPI
    const keys = [
        { pubkey: TOKEN_SCRIPT_ACCOUNT, isSigner: false, isWritable: true },
        { pubkey: VM_STATE_PDA, isSigner: false, isWritable: true },
        { pubkey: mintAccount.publicKey, isSigner: true, isWritable: true },  // @init needs signer
        { pubkey: user1.publicKey, isSigner: true, isWritable: false },
        { pubkey: payer.publicKey, isSigner: true, isWritable: true },  // payer for rent
    ];

    console.log('Account order:');
    for (let i = 0; i < keys.length; i++) {
        console.log('  ' + i + ': ' + keys[i].pubkey.toBase58() + ' (signer=' + keys[i].isSigner + ', writable=' + keys[i].isWritable + ')');
    }

    const ix = new TransactionInstruction({
        programId: FIVE_PROGRAM_ID,
        keys,
        data: instructionData
    });

    const tx = new Transaction().add(ix);

    console.log('\nSending transaction...');
    try {
        // Include mintAccount and user1 as signers since they need to sign
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
