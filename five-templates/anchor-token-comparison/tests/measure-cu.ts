// Simple CU measurement using raw Solana transactions
// Outputs TX signatures for verification on local validator
import {
    Connection,
    Keypair,
    LAMPORTS_PER_SOL,
    PublicKey,
    SystemProgram,
    Transaction,
    TransactionInstruction,
    sendAndConfirmTransaction,
} from "@solana/web3.js";

const PROGRAM_ID = new PublicKey("EXYTTMwHkRziMdQ1guGGrThxzX6dJDvhJBzz57JGKmsw");

// Anchor discriminators (first 8 bytes of sha256("global:<snake_case_instruction_name>"))
const DISCRIMINATORS = {
    initMint: Buffer.from([126, 176, 233, 16, 66, 117, 209, 125]),
    initTokenAccount: Buffer.from([17, 16, 88, 108, 240, 140, 102, 248]),
    mintTo: Buffer.from([241, 34, 48, 186, 37, 179, 123, 192]),
    transfer: Buffer.from([163, 52, 200, 231, 140, 3, 69, 186]),
    approve: Buffer.from([69, 74, 217, 36, 115, 117, 97, 76]),
    burn: Buffer.from([116, 110, 29, 56, 107, 219, 42, 93]),
};

async function main() {
    const connection = new Connection("http://localhost:8899", "confirmed");
    const payer = Keypair.generate();

    console.log("Airdropping SOL...");
    const sig = await connection.requestAirdrop(payer.publicKey, 10 * LAMPORTS_PER_SOL);
    await connection.confirmTransaction(sig);
    console.log(`Payer: ${payer.publicKey.toBase58()}`);
    await new Promise(r => setTimeout(r, 500));

    const results: { instruction: string; cu: number; sig: string }[] = [];

    // Test 1: init_mint
    console.log("\n=== Testing init_mint ===");
    const mintKeypair = Keypair.generate();
    console.log(`Mint Account: ${mintKeypair.publicKey.toBase58()}`);

    try {
        const freezeAuthority = payer.publicKey.toBytes();
        const decimals = 9;
        const name = "Test Token";
        const symbol = "TEST";
        const uri = "https://test.com";

        const nameBytes = Buffer.from(name, "utf8");
        const symbolBytes = Buffer.from(symbol, "utf8");
        const uriBytes = Buffer.from(uri, "utf8");

        const dataLen = 8 + 32 + 1 + 4 + nameBytes.length + 4 + symbolBytes.length + 4 + uriBytes.length;
        const data = Buffer.alloc(dataLen);
        let offset = 0;
        DISCRIMINATORS.initMint.copy(data, offset); offset += 8;
        data.set(freezeAuthority, offset); offset += 32;
        data.writeUInt8(decimals, offset); offset += 1;
        data.writeUInt32LE(nameBytes.length, offset); offset += 4;
        nameBytes.copy(data, offset); offset += nameBytes.length;
        data.writeUInt32LE(symbolBytes.length, offset); offset += 4;
        symbolBytes.copy(data, offset); offset += symbolBytes.length;
        data.writeUInt32LE(uriBytes.length, offset); offset += 4;
        uriBytes.copy(data, offset);

        const tx = new Transaction().add(
            new TransactionInstruction({
                keys: [
                    { pubkey: mintKeypair.publicKey, isSigner: true, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
                    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
                ],
                programId: PROGRAM_ID,
                data,
            })
        );

        const txSig = await sendAndConfirmTransaction(connection, tx, [payer, mintKeypair]);
        console.log(`TX: ${txSig}`);
        await new Promise(r => setTimeout(r, 500));
        const txDetails = await connection.getTransaction(txSig, { maxSupportedTransactionVersion: 0 });
        const cu = txDetails?.meta?.computeUnitsConsumed || 0;
        console.log(`CU: ${cu}`);
        results.push({ instruction: "init_mint", cu: Number(cu), sig: txSig });
    } catch (e: any) {
        console.error("init_mint failed:", e.message || e);
    }

    // Test 2: init_token_account
    console.log("\n=== Testing init_token_account ===");
    const tokenAccount1 = Keypair.generate();
    console.log(`Token Account 1: ${tokenAccount1.publicKey.toBase58()}`);

    try {
        const data = Buffer.alloc(8 + 32);
        DISCRIMINATORS.initTokenAccount.copy(data, 0);
        data.set(mintKeypair.publicKey.toBytes(), 8);

        const tx = new Transaction().add(
            new TransactionInstruction({
                keys: [
                    { pubkey: tokenAccount1.publicKey, isSigner: true, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
                    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
                ],
                programId: PROGRAM_ID,
                data,
            })
        );

        const txSig = await sendAndConfirmTransaction(connection, tx, [payer, tokenAccount1]);
        console.log(`TX: ${txSig}`);
        await new Promise(r => setTimeout(r, 500));
        const txDetails = await connection.getTransaction(txSig, { maxSupportedTransactionVersion: 0 });
        const cu = txDetails?.meta?.computeUnitsConsumed || 0;
        console.log(`CU: ${cu}`);
        results.push({ instruction: "init_token_account", cu: Number(cu), sig: txSig });
    } catch (e: any) {
        console.error("init_token_account failed:", e.message || e);
    }

    // Test 3: mint_to
    console.log("\n=== Testing mint_to ===");
    try {
        const data = Buffer.alloc(8 + 8);
        DISCRIMINATORS.mintTo.copy(data, 0);
        data.writeBigUInt64LE(BigInt(1000000000), 8);

        const tx = new Transaction().add(
            new TransactionInstruction({
                keys: [
                    { pubkey: mintKeypair.publicKey, isSigner: false, isWritable: true },
                    { pubkey: tokenAccount1.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: false },
                ],
                programId: PROGRAM_ID,
                data,
            })
        );

        const txSig = await sendAndConfirmTransaction(connection, tx, [payer]);
        console.log(`TX: ${txSig}`);
        await new Promise(r => setTimeout(r, 500));
        const txDetails = await connection.getTransaction(txSig, { maxSupportedTransactionVersion: 0 });
        const cu = txDetails?.meta?.computeUnitsConsumed || 0;
        console.log(`CU: ${cu}`);
        results.push({ instruction: "mint_to", cu: Number(cu), sig: txSig });
    } catch (e: any) {
        console.error("mint_to failed:", e.message || e);
    }

    // Create second token account for transfer
    console.log("\n=== Creating second token account ===");
    const tokenAccount2 = Keypair.generate();
    const recipient = Keypair.generate();
    console.log(`Token Account 2: ${tokenAccount2.publicKey.toBase58()}`);
    console.log(`Recipient: ${recipient.publicKey.toBase58()}`);

    await connection.requestAirdrop(recipient.publicKey, LAMPORTS_PER_SOL);
    await new Promise(r => setTimeout(r, 1000));

    try {
        const data = Buffer.alloc(8 + 32);
        DISCRIMINATORS.initTokenAccount.copy(data, 0);
        data.set(mintKeypair.publicKey.toBytes(), 8);

        const tx = new Transaction().add(
            new TransactionInstruction({
                keys: [
                    { pubkey: tokenAccount2.publicKey, isSigner: true, isWritable: true },
                    { pubkey: recipient.publicKey, isSigner: true, isWritable: true },
                    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
                ],
                programId: PROGRAM_ID,
                data,
            })
        );
        const txSig = await sendAndConfirmTransaction(connection, tx, [recipient, tokenAccount2]);
        console.log(`TX: ${txSig}`);
    } catch (e: any) {
        console.error("Creating second token account failed:", e.message || e);
    }

    // Test 4: transfer
    console.log("\n=== Testing transfer ===");
    try {
        const data = Buffer.alloc(8 + 8);
        DISCRIMINATORS.transfer.copy(data, 0);
        data.writeBigUInt64LE(BigInt(100000000), 8);

        const tx = new Transaction().add(
            new TransactionInstruction({
                keys: [
                    { pubkey: tokenAccount1.publicKey, isSigner: false, isWritable: true },
                    { pubkey: tokenAccount2.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: false },
                ],
                programId: PROGRAM_ID,
                data,
            })
        );

        const txSig = await sendAndConfirmTransaction(connection, tx, [payer]);
        console.log(`TX: ${txSig}`);
        await new Promise(r => setTimeout(r, 500));
        const txDetails = await connection.getTransaction(txSig, { maxSupportedTransactionVersion: 0 });
        const cu = txDetails?.meta?.computeUnitsConsumed || 0;
        console.log(`CU: ${cu}`);
        results.push({ instruction: "transfer", cu: Number(cu), sig: txSig });
    } catch (e: any) {
        console.error("transfer failed:", e.message || e);
    }

    // Test 5: approve
    console.log("\n=== Testing approve ===");
    try {
        const data = Buffer.alloc(8 + 32 + 8);
        DISCRIMINATORS.approve.copy(data, 0);
        data.set(recipient.publicKey.toBytes(), 8);
        data.writeBigUInt64LE(BigInt(50000000), 40);

        const tx = new Transaction().add(
            new TransactionInstruction({
                keys: [
                    { pubkey: tokenAccount1.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: false },
                ],
                programId: PROGRAM_ID,
                data,
            })
        );

        const txSig = await sendAndConfirmTransaction(connection, tx, [payer]);
        console.log(`TX: ${txSig}`);
        await new Promise(r => setTimeout(r, 500));
        const txDetails = await connection.getTransaction(txSig, { maxSupportedTransactionVersion: 0 });
        const cu = txDetails?.meta?.computeUnitsConsumed || 0;
        console.log(`CU: ${cu}`);
        results.push({ instruction: "approve", cu: Number(cu), sig: txSig });
    } catch (e: any) {
        console.error("approve failed:", e.message || e);
    }

    // Test 6: burn
    console.log("\n=== Testing burn ===");
    try {
        const data = Buffer.alloc(8 + 8);
        DISCRIMINATORS.burn.copy(data, 0);
        data.writeBigUInt64LE(BigInt(10000000), 8);

        const tx = new Transaction().add(
            new TransactionInstruction({
                keys: [
                    { pubkey: mintKeypair.publicKey, isSigner: false, isWritable: true },
                    { pubkey: tokenAccount1.publicKey, isSigner: false, isWritable: true },
                    { pubkey: payer.publicKey, isSigner: true, isWritable: false },
                ],
                programId: PROGRAM_ID,
                data,
            })
        );

        const txSig = await sendAndConfirmTransaction(connection, tx, [payer]);
        console.log(`TX: ${txSig}`);
        await new Promise(r => setTimeout(r, 500));
        const txDetails = await connection.getTransaction(txSig, { maxSupportedTransactionVersion: 0 });
        const cu = txDetails?.meta?.computeUnitsConsumed || 0;
        console.log(`CU: ${cu}`);
        results.push({ instruction: "burn", cu: Number(cu), sig: txSig });
    } catch (e: any) {
        console.error("burn failed:", e.message || e);
    }

    // Print summary
    console.log("\n" + "=".repeat(120));
    console.log("ANCHOR TOKEN CU USAGE SUMMARY");
    console.log("=".repeat(120));
    console.log("| Instruction         | CU      | TX Signature                                                       |");
    console.log("|---------------------|---------|-----------------------------------------------------------------------|");
    for (const r of results) {
        console.log(`| ${r.instruction.padEnd(19)} | ${r.cu.toString().padStart(7)} | ${r.sig} |`);
    }
    console.log("=".repeat(120));
}

main().catch(console.error);
