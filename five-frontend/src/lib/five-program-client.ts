/**
 * FiveProgram Client - High-level execution using FiveProgram API
 * 
 * Provides a clean interface for executing functions on deployed Five scripts
 * using the FiveProgram fluent API from five-sdk.
 */

import { Connection, PublicKey, Transaction, TransactionInstruction } from '@solana/web3.js';
import { FiveProgram } from 'five-sdk';
import { NETWORKS, type NetworkType } from './network-config';

// Define ScriptABI inline to avoid import issues
export interface ScriptABI {
    name: string;
    functions: Array<{
        name: string;
        index: number;
        parameters: Array<{
            name: string;
            param_type: any;
            is_account: boolean;
            attributes?: string[];
        }>;
    }>;
}

export interface ExecutionOptions {
    /** Network to execute on */
    network: NetworkType;
    /** Script account address */
    scriptAccount: string;
    /** Script ABI */
    abi: ScriptABI;
    /** Function name to call */
    functionName: string;
    /** Account parameters (name -> address mapping) */
    accounts: Record<string, string>;
    /** Data parameters (name -> value mapping) */
    args: Record<string, any>;
    /** Optional debug mode */
    debug?: boolean;
}

export interface ExecutionInstructionResult {
    /** Serialized instruction ready for transaction */
    instruction: TransactionInstruction;
    /** Program ID used */
    programId: PublicKey;
    /** All accounts included */
    accounts: Array<{ pubkey: PublicKey; isSigner: boolean; isWritable: boolean }>;
}

/**
 * Create a FiveProgram instance for a deployed script
 */
export function createFiveProgram(
    scriptAccount: string,
    abi: ScriptABI,
    network: NetworkType
): FiveProgram {
    const networkConfig = NETWORKS[network];

    return FiveProgram.fromABI(scriptAccount, abi as any, {
        fiveVMProgramId: networkConfig.programId,
        debug: false
    });
}

/**
 * Build an execution instruction using FiveProgram fluent API
 * 
 * This uses the same proven encoding as the SDK, ensuring
 * compatibility with the on-chain Five VM.
 */
export async function buildExecuteInstruction(
    options: ExecutionOptions
): Promise<ExecutionInstructionResult> {
    const { network, scriptAccount, abi, functionName, accounts, args, debug } = options;
    const networkConfig = NETWORKS[network];

    // Create FiveProgram instance
    const program = FiveProgram.fromABI(scriptAccount, abi as any, {
        fiveVMProgramId: networkConfig.programId,
        debug: debug || false
    });

    // Build instruction using fluent API
    const serializedIx = await program
        .function(functionName)
        .accounts(accounts)
        .args(args)
        .instruction();

    // Convert to Solana TransactionInstruction
    const instruction = new TransactionInstruction({
        programId: new PublicKey(serializedIx.programId),
        keys: serializedIx.keys.map(key => ({
            pubkey: new PublicKey(key.pubkey),
            isSigner: key.isSigner,
            isWritable: key.isWritable
        })),
        data: Buffer.from(serializedIx.data, 'base64')
    });

    return {
        instruction,
        programId: new PublicKey(serializedIx.programId),
        accounts: instruction.keys
    };
}

/**
 * Execute a function on a deployed Five script
 * 
 * Full execution flow:
 * 1. Build instruction using FiveProgram
 * 2. Create transaction
 * 3. Sign with wallet
 * 4. Send to network
 */
export async function executeFunction(
    options: ExecutionOptions & {
        wallet: {
            publicKey: PublicKey;
            signTransaction?: (tx: Transaction) => Promise<Transaction>;
            sendTransaction?: (tx: Transaction, connection: Connection) => Promise<string>;
        };
    }
): Promise<{ signature: string; success: boolean }> {
    const { wallet, network } = options;
    const networkConfig = NETWORKS[network];

    // Build the instruction
    const { instruction } = await buildExecuteInstruction(options);

    // Create connection
    const connection = new Connection(networkConfig.rpcUrl, 'confirmed');

    // Create transaction
    const transaction = new Transaction();
    transaction.add(instruction);
    transaction.feePayer = wallet.publicKey;

    const latestBlockhash = await connection.getLatestBlockhash();
    transaction.recentBlockhash = latestBlockhash.blockhash;

    let signature: string;

    // Sign and send with fallback support
    if (wallet.signTransaction) {
        try {
            const signedTx = await wallet.signTransaction(transaction);
            signature = await connection.sendRawTransaction(signedTx.serialize(), {
                skipPreflight: true
            });
        } catch (err: any) {
            // Fallback for Standard Wallets if signTransaction fails with specific error
            const errorStr = err.message || err.toString();
            if (errorStr.includes("signTransaction is not a function") && wallet.sendTransaction) {
                console.warn("signTransaction failed, falling back to sendTransaction");
                signature = await wallet.sendTransaction(transaction, connection);
            } else {
                throw err;
            }
        }
    } else if (wallet.sendTransaction) {
        signature = await wallet.sendTransaction(transaction, connection);
    } else {
        throw new Error("Wallet adapter does not support signTransaction or sendTransaction");
    }

    // Confirm transaction
    await connection.confirmTransaction(signature, 'confirmed');

    return { signature, success: true };
}

/**
 * Get available functions from ABI
 */
export function getAvailableFunctions(abi: ScriptABI): Array<{
    name: string;
    index: number;
    parameters: any[];
}> {
    return abi.functions.map((fn: ScriptABI['functions'][number]) => ({
        name: fn.name,
        index: fn.index,
        parameters: fn.parameters
    }));
}
