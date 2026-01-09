#!/usr/bin/env node
import fs from 'fs';
import { Connection, Keypair, PublicKey, Transaction, TransactionInstruction, SystemProgram, SYSVAR_RENT_PUBKEY, LAMPORTS_PER_SOL } from '@solana/web3.js';
import { FiveSDK } from '../../five-sdk/dist/index.js';

// Load config from deployment-config.json
const config = JSON.parse(fs.readFileSync("deployment-config.json"));
console.log("Loaded config:", config);

const FIVE_ID = new PublicKey(config.fiveProgramId);
const VM_STATE = new PublicKey(config.vmStatePda);
const SCRIPT = new PublicKey(config.tokenScriptAccount);

console.log("Starting test with:");
console.log("  Five Program:", FIVE_ID.toBase58());
console.log("  VM State:", VM_STATE.toBase58());
console.log("  Script:", SCRIPT.toBase58());

const abi = JSON.parse(fs.readFileSync("build/five-token-template.abi.json"));
console.log("ABI loaded, functions:", abi.functions?.length);

const conn = new Connection(config.rpcUrl || "http://127.0.0.1:8899");
const payer = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(process.env.HOME + "/.config/solana/id.json"))));

const user1 = Keypair.generate();
const mintAccount = Keypair.generate();

console.log("Requesting airdrop...");
const airdropSig = await conn.requestAirdrop(user1.publicKey, 10 * LAMPORTS_PER_SOL);
await conn.confirmTransaction(airdropSig, "confirmed");
console.log("User1 funded:", user1.publicKey.toBase58());

console.log("Calling FiveSDK.generateExecuteInstruction...");
const executeData = await FiveSDK.generateExecuteInstruction(
    SCRIPT.toBase58(),
    "init_mint",
    [mintAccount.publicKey, user1.publicKey, user1.publicKey, 6, "Test", "TST", "https://x.com"],
    [
        mintAccount.publicKey.toBase58(),
        user1.publicKey.toBase58(),
        payer.publicKey.toBase58(),
        SystemProgram.programId.toBase58(),
        SYSVAR_RENT_PUBKEY.toBase58()
    ],
    conn,
    {
        debug: false,
        vmStateAccount: VM_STATE.toBase58(),
        fiveVMProgramId: FIVE_ID.toBase58(),
        abi: abi
    }
);

console.log("Building transaction...");
const ixKeys = executeData.instruction.accounts.map(acc => ({
    pubkey: new PublicKey(acc.pubkey),
    isSigner: acc.isSigner,
    isWritable: acc.isWritable
}));

const ix = new TransactionInstruction({
    programId: FIVE_ID,
    keys: ixKeys,
    data: Buffer.from(executeData.instruction.data, "base64")
});

const tx = new Transaction().add(ix);

try {
    console.log("Sending transaction with preflight...");
    const sig = await conn.sendTransaction(tx, [payer, user1, mintAccount], {
        skipPreflight: false,
        maxRetries: 3
    });
    console.log("TX Sent:", sig);
    await conn.confirmTransaction(sig, "confirmed");
    console.log("SUCCESS!");
} catch (e) {
    console.error("Error:", e.message);
    if (e.logs) {
        console.log("Logs:");
        e.logs.forEach(l => console.log("  " + l));
    }
}
