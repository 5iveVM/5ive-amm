import {
    Connection,
    PublicKey,
    Transaction,
    TransactionInstruction,
    SystemProgram,
    Keypair,
    sendAndConfirmTransaction,
    Signer
} from '@solana/web3.js';
import * as buffer from "buffer";
// Polyfill buffer for browser environment if needed, though most bundlers handle it.
if (typeof window !== "undefined") {
    if (typeof (window as any).Buffer === "undefined") {
        (window as any).Buffer = buffer.Buffer;
    }
}

export const DEFAULT_PROGRAM_ID = '2DXiYbzfSMwkDSxc9aWEaW7XgJjkNzGdADfRN4FbxMNN'; // Default localnet program ID
export const FIVE_VM_PROGRAM_ID = new PublicKey(DEFAULT_PROGRAM_ID);

export interface DeploymentResult {
    success: boolean;
    scriptAccount?: string;
    transactionId?: string;
    error?: string;
    logs?: string[];
}

export interface ExecutionResult {
    success: boolean;
    transactionId?: string;
    computeUnitsUsed?: number;
    error?: string;
    logs?: string[];
}

export class OnChainClient {
    private connection: Connection;
    private wallet: {
        publicKey: PublicKey;
        signTransaction?: (tx: Transaction) => Promise<Transaction>;
        sendTransaction?: (tx: Transaction, connection: Connection) => Promise<string>;
    } | Keypair;
    private programId: PublicKey;

    constructor(
        connection: Connection,
        wallet: {
            publicKey: PublicKey;
            signTransaction?: (tx: Transaction) => Promise<Transaction>;
            sendTransaction?: (tx: Transaction, connection: Connection) => Promise<string>;
        } | Keypair,
        programId?: string | PublicKey
    ) {
        this.connection = connection;
        this.wallet = wallet;
        this.programId = programId
            ? (typeof programId === 'string' ? new PublicKey(programId) : programId)
            : FIVE_VM_PROGRAM_ID;
    }

    /**
     * Derives the Script Account PDA and Seed.
     * Logic mirrors `PDAUtils.deriveScriptAccount` from FiveSDK.
     */
    static async deriveScriptAccount(
        bytecode: Uint8Array,
        programId: PublicKey = FIVE_VM_PROGRAM_ID
    ): Promise<{ address: PublicKey; seed: string }> {
        // Simple seed for now to match SDK's current "simple" derivation
        // In a real production SDK, this would likely hash the bytecode or use a counter
        // For this visualizer/demo, we'll use a timestamp-based seed to ensure uniqueness for each deploy
        const seed = `script_${Date.now()}`;

        // SystemProgram.createAccountWithSeed expectations:
        // address = createWithSeed(base, seed, programId)

        // Ideally we need the deployer's public key as the base, but static methods don't have it.
        // We will defer the full derivation to the deploy method or require passing the base.
        // For consistent API with SDK, we returns a "seed".

        return {
            address: PublicKey.default, // Placeholder, actual derivation needs deployer key
            seed: seed
        };
    }

    /**
     * Encodes the 'Deploy' instruction data.
     * Format: [discriminator(8), length(u32_le), permissions(u8), ...bytecode]
     */
    static encodeDeployInstruction(bytecode: Uint8Array): Uint8Array {
        const lengthBytes = new Uint8Array(4);
        const view = new DataView(lengthBytes.buffer);
        view.setUint32(0, bytecode.length, true); // Little-endian

        const result = new Uint8Array(1 + 4 + 1 + bytecode.length);
        result[0] = 8; // Discriminator for 'Deploy'
        result.set(lengthBytes, 1);
        result[5] = 0; // Permissions (0 = default)
        result.set(bytecode, 6);

        return result;
    }

    /**
     * Encodes the 'Execute' instruction data.
     * Format: [discriminator(9), function_index(VLE), ...VLE_encoded_params]
     * Note: "VLE_encoded_params" should usually include a VLE param count first.
     */
    static encodeExecuteInstruction(
        functionIndex: number,
        encodedParams: Uint8Array
    ): Uint8Array {
        const parts: Uint8Array[] = [];
        parts.push(new Uint8Array([9])); // Discriminator for 'Execute'

        // Encode function index as VLE
        parts.push(this.encodeVLE(functionIndex));

        // Encoded params (pre-encoded by WASM module which includes count)
        parts.push(encodedParams);

        // Concatenate
        const totalLength = parts.reduce((acc, p) => acc + p.length, 0);
        const result = new Uint8Array(totalLength);
        let offset = 0;
        for (const p of parts) {
            result.set(p, offset);
            offset += p.length;
        }
        return result;
    }

    /**
     * Helper to encode a number as VLE (Variable Length Encoding)
     */
    static encodeVLE(value: number): Uint8Array {
        const bytes: number[] = [];
        do {
            let byte = value & 0x7f;
            value >>>= 7;
            if (value !== 0) {
                byte |= 0x80;
            }
            bytes.push(byte);
        } while (value !== 0);
        return new Uint8Array(bytes);
    }



    /**
     * Encodes the 'InitLargeProgram' instruction data.
     * Format: [discriminator(4), expected_size(u32_le)]
     */
    static encodeInitLargeProgramInstruction(expectedSize: number): Uint8Array {
        const result = new Uint8Array(5);
        result[0] = 4; // Discriminator for 'InitLargeProgram'
        const view = new DataView(result.buffer);
        view.setUint32(1, expectedSize, true); // Little-endian
        return result;
    }

    /**
     * Encodes the 'AppendBytecode' instruction data.
     * Format: [discriminator(5), ...data]
     */
    static encodeAppendInstruction(chunk: Uint8Array): Uint8Array {
        const result = new Uint8Array(1 + chunk.length);
        result[0] = 5; // Discriminator for 'AppendBytecode'
        result.set(chunk, 1);
        return result;
    }

    /**
     * Deploys a compiled script to the Solana network.
     * Automatically switches to chunked deployment for large scripts.
     */
    async deploy(bytecode: Uint8Array, adminAccount?: string): Promise<DeploymentResult> {
        try {
            console.log(`Starting deployment (size: ${bytecode.length} bytes)...`);
            const deployerPubkey = this.wallet instanceof Keypair ? this.wallet.publicKey : this.wallet.publicKey;

            // 1. Derive Script Account
            const seed = `script_${Date.now().toString().slice(-6)}`; // Short unique seed
            const scriptPubkey = await PublicKey.createWithSeed(
                deployerPubkey,
                seed,
                this.programId
            );

            console.log(`Derived Script Account: ${scriptPubkey.toBase58()} (seed: ${seed})`);

            // 2. Derive VM State PDA
            const [vmStatePDA] = await PublicKey.findProgramAddress(
                [Buffer.from("vm_state", "utf8")],
                this.programId
            );
            console.log(`VM State PDA: ${vmStatePDA.toBase58()}`);

            // 3. Calculate Rent
            const space = 64 + bytecode.length; // 64 byte header + bytecode
            const rentExemption = await this.connection.getMinimumBalanceForRentExemption(space);

            const MAX_CHUNK_SIZE = 800; // Conservative limit for transaction size

            if (bytecode.length <= MAX_CHUNK_SIZE) {
                // === SIMPLE DEPLOYMENT (Small Scripts) ===

                // 4. Create Transaction
                const transaction = new Transaction();

                // 4.1 Create Account Instruction
                transaction.add(
                    SystemProgram.createAccountWithSeed({
                        fromPubkey: deployerPubkey,
                        newAccountPubkey: scriptPubkey,
                        basePubkey: deployerPubkey,
                        seed: seed,
                        lamports: rentExemption,
                        space: space,
                        programId: this.programId
                    })
                );

                // 4.2 Deploy Instruction (Five VM)
                const deployData = OnChainClient.encodeDeployInstruction(bytecode);

                const keys = [
                    { pubkey: scriptPubkey, isSigner: false, isWritable: true },
                    { pubkey: vmStatePDA, isSigner: false, isWritable: true },
                    { pubkey: deployerPubkey, isSigner: true, isWritable: true }, // Signer/Payer
                    { pubkey: SystemProgram.programId, isSigner: false, isWritable: false }
                ];

                // Add admin account for fee collection if provided
                if (adminAccount) {
                    keys.push({ pubkey: new PublicKey(adminAccount), isSigner: false, isWritable: true });
                }

                const deployInstruction = new TransactionInstruction({
                    keys,
                    programId: this.programId,
                    data: Buffer.from(deployData)
                });

                transaction.add(deployInstruction);

                // 5. Sign and Send
                const signature = await this.signAndSendTransaction(transaction, deployerPubkey);
                console.log(`Deployment successful! Tx: ${signature}`);

                return {
                    success: true,
                    scriptAccount: scriptPubkey.toBase58(),
                    transactionId: signature,
                    logs: [`Deployed ${bytecode.length} bytes to ${scriptPubkey.toBase58()}`]
                };

            } else {
                // === CHUNKED DEPLOYMENT (Large Scripts) ===
                console.log(`Script too large for single transaction. Switching to chunked deployment.`);

                // Step A: Initialize Large Program
                // We intentionally create the account and Initialize it in the first transaction
                const initTransaction = new Transaction();

                // A.1 Create Account
                initTransaction.add(
                    SystemProgram.createAccountWithSeed({
                        fromPubkey: deployerPubkey,
                        newAccountPubkey: scriptPubkey,
                        basePubkey: deployerPubkey,
                        seed: seed,
                        lamports: rentExemption,
                        space: space,
                        programId: this.programId
                    })
                );

                // A.2 InitLargeProgram Instruction
                const initData = OnChainClient.encodeInitLargeProgramInstruction(bytecode.length);
                const initKeys = [
                    { pubkey: scriptPubkey, isSigner: false, isWritable: true },
                    { pubkey: deployerPubkey, isSigner: true, isWritable: true }, // Owner
                    { pubkey: vmStatePDA, isSigner: false, isWritable: true },
                ];

                // Add admin account for fee collection if provided
                if (adminAccount) {
                    initKeys.push({ pubkey: new PublicKey(adminAccount), isSigner: false, isWritable: true });
                }

                const initInstruction = new TransactionInstruction({
                    keys: initKeys,
                    programId: this.programId,
                    data: Buffer.from(initData)
                });
                initTransaction.add(initInstruction);

                console.log("Sending Init transaction...");
                const initSig = await this.signAndSendTransaction(initTransaction, deployerPubkey);
                await this.connection.confirmTransaction(initSig, 'confirmed');

                // Step B: Send Chunks
                let offset = 0;
                let chunkIndex = 0;
                const totalChunks = Math.ceil(bytecode.length / MAX_CHUNK_SIZE);

                while (offset < bytecode.length) {
                    const chunkEnd = Math.min(offset + MAX_CHUNK_SIZE, bytecode.length);
                    const chunk = bytecode.slice(offset, chunkEnd);
                    chunkIndex++;

                    console.log(`Sending Chunk ${chunkIndex}/${totalChunks} (${chunk.length} bytes)...`);

                    const chunkTransaction = new Transaction();
                    const appendData = OnChainClient.encodeAppendInstruction(chunk);

                    const appendKeys = [
                        { pubkey: scriptPubkey, isSigner: false, isWritable: true },
                        { pubkey: deployerPubkey, isSigner: true, isWritable: true }, // Owner
                        { pubkey: vmStatePDA, isSigner: false, isWritable: true },
                    ];

                    const appendInstruction = new TransactionInstruction({
                        keys: appendKeys,
                        programId: this.programId,
                        data: Buffer.from(appendData)
                    });
                    chunkTransaction.add(appendInstruction);

                    const chunkSig = await this.signAndSendTransaction(chunkTransaction, deployerPubkey);
                    await this.connection.confirmTransaction(chunkSig, 'confirmed'); // Wait for confirmation to ensure order

                    offset += MAX_CHUNK_SIZE;
                }

                console.log(`Chunked deployment successful! Final Tx: ${initSig} ...`); // Using init sig as ID

                return {
                    success: true,
                    scriptAccount: scriptPubkey.toBase58(),
                    transactionId: initSig,
                    logs: [`Deployed ${bytecode.length} bytes via ${totalChunks} chunks`]
                };
            }

        } catch (err: any) {
            console.error("Deployment failed:", err);

            let errorMsg = "";
            let logs: string[] = err.logs || [];

            if (err instanceof Error) {
                errorMsg = err.message;
            } else if (typeof err === "string") {
                errorMsg = err;
            } else {
                try {
                    const json = JSON.stringify(err);
                    errorMsg = json === "{}" ? String(err) : json;
                } catch {
                    errorMsg = String(err);
                }
            }

            return {
                success: false,
                error: errorMsg,
                logs: logs.length > 0 ? logs : [errorMsg]
            };
        }
    }

    private async signAndSendTransaction(transaction: Transaction, deployerPubkey: PublicKey): Promise<string> {
        if (this.wallet instanceof Keypair) {
            // CLI / Local Keypair
            return sendAndConfirmTransaction(
                this.connection,
                transaction,
                [this.wallet],
                { skipPreflight: true }
            );
        } else {
            // Wallet Adapter
            transaction.feePayer = deployerPubkey;
            const latestBlockhash = await this.connection.getLatestBlockhash();
            transaction.recentBlockhash = latestBlockhash.blockhash;

            // Prefer signTransaction + sendRawTransaction for more control (e.g. chunked deploy confirmation)
            if (this.wallet.signTransaction) {
                try {
                    const signedTx = await this.wallet.signTransaction(transaction);
                    const signature = await this.connection.sendRawTransaction(signedTx.serialize(), { skipPreflight: true });
                    const confirmation = await this.connection.confirmTransaction(signature, 'confirmed');
                    if (confirmation.value.err) {
                        throw new Error(`Transaction failed: ${JSON.stringify(confirmation.value.err)}`);
                    }
                    return signature;
                } catch (err: any) {
                    // If signTransaction fails specifically with "not a function" (standard wallet issue), fallback
                    if (err.toString().includes("signTransaction is not a function") && this.wallet.sendTransaction) {
                        console.warn("signTransaction failed, falling back to sendTransaction");
                        return await this.wallet.sendTransaction(transaction, this.connection);
                    }
                    throw err;
                }
            } else if (this.wallet.sendTransaction) {
                // Fallback to sendTransaction (common for Standard Wallets)
                return await this.wallet.sendTransaction(transaction, this.connection);
            } else {
                throw new Error("Wallet adapter does not support signTransaction or sendTransaction");
            }
        }
    }

    async execute(
        scriptAccountStr: string,
        functionIndex: number,
        encodedParams: Uint8Array,
        writableAccounts: string[] = [], // Extra accounts if needed
        adminAccount?: string // Optional admin account for fee collection
    ): Promise<ExecutionResult> {
        try {
            console.log(`Executing function ${functionIndex} on ${scriptAccountStr}...`);
            const deployerPubkey = this.wallet instanceof Keypair ? this.wallet.publicKey : this.wallet.publicKey;
            const scriptPubkey = new PublicKey(scriptAccountStr);

            // 1. Derive VM State PDA
            const [vmStatePDA] = await PublicKey.findProgramAddress(
                [Buffer.from("vm_state", "utf8")],
                this.programId
            );

            // 2. Build Transaction
            const transaction = new Transaction();

            // 3. Execute Instruction
            const executeData = OnChainClient.encodeExecuteInstruction(functionIndex, encodedParams);

            const keys = [
                { pubkey: scriptPubkey, isSigner: false, isWritable: false },
                { pubkey: vmStatePDA, isSigner: false, isWritable: true },
                // Add extra accounts here if the script needs them (e.g. for transfers)
                // For now, we'll just add the signer as a writable account too, often needed
                { pubkey: deployerPubkey, isSigner: true, isWritable: true }
            ];

            // Add admin account for fee collection if provided
            if (adminAccount) {
                keys.push({ pubkey: new PublicKey(adminAccount), isSigner: false, isWritable: true });
            }

            // Add any user-specified extra accounts
            for (const acc of writableAccounts) {
                keys.push({ pubkey: new PublicKey(acc), isSigner: false, isWritable: true });
            }

            const executeInstruction = new TransactionInstruction({
                keys,
                programId: this.programId,
                data: Buffer.from(executeData)
            });

            transaction.add(executeInstruction);

            // 4. Sign and Send
            let signature: string;

            if (this.wallet instanceof Keypair) {
                const latestBlockhash = await this.connection.getLatestBlockhash();
                transaction.recentBlockhash = latestBlockhash.blockhash;
                transaction.feePayer = this.wallet.publicKey;

                signature = await sendAndConfirmTransaction(
                    this.connection,
                    transaction,
                    [this.wallet],
                    { skipPreflight: true } // Skip preflight to see logs if it fails
                );
            } else {
                transaction.feePayer = deployerPubkey;
                const latestBlockhash = await this.connection.getLatestBlockhash();
                transaction.recentBlockhash = latestBlockhash.blockhash;

                if (this.wallet.signTransaction) {
                    try {
                        const signedTx = await this.wallet.signTransaction(transaction);
                        signature = await this.connection.sendRawTransaction(signedTx.serialize(), { skipPreflight: true });
                    } catch (err: any) {
                        if (err.toString().includes("signTransaction is not a function") && this.wallet.sendTransaction) {
                            signature = await this.wallet.sendTransaction(transaction, this.connection);
                        } else {
                            throw err;
                        }
                    }
                } else if (this.wallet.sendTransaction) {
                    signature = await this.wallet.sendTransaction(transaction, this.connection);
                } else {
                    throw new Error("Wallet adapter does not support signTransaction or sendTransaction");
                }

                // We confirm with 'confirmed' to wait for logs
                const confirmation = await this.connection.confirmTransaction(signature, 'confirmed');
                if (confirmation.value.err) {
                    throw new Error(`Transaction failed: ${JSON.stringify(confirmation.value.err)}`);
                }
            }

            console.log(`Execution successful! Tx: ${signature}`);

            // 5. Fetch logs (optional, but good for debugging)
            // const txDetails = await this.connection.getTransaction(signature, { commitment: 'confirmed' });
            // const logs = txDetails?.meta?.logMessages || [];

            return {
                success: true,
                transactionId: signature,
                logs: [`Executed function ${functionIndex}`, `Tx: ${signature}`]
            };

        } catch (err: any) {
            console.error("Execution failed:", err);

            // Try to extract logs if it's a simulation error
            let logs: string[] = err.logs || [];

            let errorMsg = "";
            if (err instanceof Error) {
                errorMsg = err.message;
            } else if (typeof err === "string") {
                errorMsg = err;
            } else {
                try {
                    const json = JSON.stringify(err);
                    errorMsg = json === "{}" ? String(err) : json;
                } catch {
                    errorMsg = String(err);
                }
            }

            return {
                success: false,
                error: errorMsg,
                logs
            };
        }
    }
}
