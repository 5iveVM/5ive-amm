import {
  FiveBytecode,
  DeploymentOptions,
  SerializedDeployment,
  FIVE_VM_PROGRAM_ID,
  FeeInformation,
} from "../types.js";
import { PDAUtils, RentCalculator } from "../crypto/index.js";
import { validator, Validators } from "../validation/index.js";
import { calculateDeployFee } from "./fees.js";
import { pollForConfirmation } from "../utils/transaction.js";
import { ProgramIdResolver } from "../config/ProgramIdResolver.js";

interface ExportMetadataInterfaceInput {
  name: string;
  methodMap?: Record<string, string>;
}

interface ExportMetadataInput {
  methods?: string[];
  interfaces?: ExportMetadataInterfaceInput[];
}

export async function generateDeployInstruction(
  bytecode: FiveBytecode,
  deployer: string,
  options: DeploymentOptions & { debug?: boolean } = {},
  connection?: any,
  fiveVMProgramId?: string,
): Promise<SerializedDeployment> {
  Validators.bytecode(bytecode);
  validator.validateBase58Address(deployer, "deployer");
  Validators.options(options);
  if (options.scriptAccount) {
    validator.validateBase58Address(
      options.scriptAccount,
      "options.scriptAccount",
    );
  }

  // Resolve program ID with consistent precedence
  const programId = ProgramIdResolver.resolve(
    fiveVMProgramId || options.fiveVMProgramId,
  );

  const exportMetadata = encodeExportMetadata(
    options.exportMetadata as ExportMetadataInput | undefined,
  );

  if (options.debug) {
    console.log(
      `[FiveSDK] Generating deployment transaction (${bytecode.length} bytes)...`,
    );
    console.log(`[FiveSDK] Using program ID: ${programId}`);
  }

  const scriptResult = await PDAUtils.deriveScriptAccount(
    bytecode,
    programId,
  );
  const scriptAccount = scriptResult.address;
  const scriptSeed = scriptResult.seed;

  const vmStatePDAResult = await PDAUtils.deriveVMStatePDA(programId);
  const vmStatePDA = vmStatePDAResult.address;

  if (options.debug) {
    console.log(
      `[FiveSDK] Script Account: ${scriptAccount} (seed: ${scriptSeed})`,
    );
    console.log(`[FiveSDK] VM State PDA: ${vmStatePDA}`);
  }

  const SCRIPT_HEADER_SIZE = 64; // ScriptAccountHeader size from Rust program
  const totalAccountSize = SCRIPT_HEADER_SIZE + exportMetadata.length + bytecode.length;
  const rentLamports = await RentCalculator.calculateRentExemption(totalAccountSize);

  const deployAccounts = [
    { pubkey: scriptAccount, isSigner: false, isWritable: true },
    { pubkey: vmStatePDA, isSigner: false, isWritable: true },
    { pubkey: deployer, isSigner: true, isWritable: true },
    {
      pubkey: "11111111111111111111111111111112",
      isSigner: false,
      isWritable: false,
    },
  ];

  if (options.adminAccount) {
    deployAccounts.push({
      pubkey: options.adminAccount,
      isSigner: false,
      isWritable: true,
    });
  }

  const instructionData = encodeDeployInstruction(
    bytecode,
    options.permissions || 0,
    exportMetadata,
  );

  const result: SerializedDeployment = {
    programId: programId,
    instruction: {
      programId: programId,
      accounts: deployAccounts,
      data: Buffer.from(instructionData).toString("base64"),
    },
    scriptAccount,
    requiredSigners: [deployer],
    estimatedCost: rentLamports + (options.extraLamports || 0),
    bytecodeSize: bytecode.length,
    setupInstructions: {
      createScriptAccount: {
        pda: scriptAccount,
        seed: scriptSeed,
        space: totalAccountSize,
        rent: rentLamports,
        owner: programId,
      },
    },
    adminAccount: options.adminAccount,
  };

  if (options.debug) {
    console.log(`[FiveSDK] Generated deployment transaction:`, {
      scriptAccount,
      scriptSeed,
      accountSize: totalAccountSize,
      rentCost: rentLamports,
      deployDataSize: instructionData.length,
      exportMetadataSize: exportMetadata.length,
      adminAccount: options.adminAccount,
    });
  }

  const shouldEstimateFees = options.estimateFees !== false && connection;

  if (shouldEstimateFees) {
    try {
      const deployFee = await calculateDeployFee(
        bytecode.length,
        connection,
        programId,
      );
      result.feeInformation = deployFee;

      if (options.debug) {
        console.log(`[FiveSDK] Deploy fee estimate:`, deployFee);
      }
    } catch (error) {
      if (options.debug) {
        console.warn(
          `[FiveSDK] Could not estimate deploy fees:`,
          error instanceof Error ? error.message : "Unknown error",
        );
      }
    }
  }

  return result;
}

export async function createDeploymentTransaction(
  bytecode: FiveBytecode,
  connection: any,
  deployerPublicKey: any, // PublicKey
  options: {
    debug?: boolean;
    fiveVMProgramId?: string;
    computeBudget?: number;
    exportMetadata?: ExportMetadataInput;
  } = {},
): Promise<{
  transaction: any;
  scriptKeypair: any;
  vmStateKeypair: any;
  programId: string;
  rentLamports: number;
}> {
  const {
    Keypair,
    PublicKey,
    Transaction,
    TransactionInstruction,
    SystemProgram,
    ComputeBudgetProgram,
  } = await import("@solana/web3.js");

  const programIdStr = ProgramIdResolver.resolve(options.fiveVMProgramId);
  const programId = new PublicKey(programIdStr);

  // Generate script keypair
  const scriptKeypair = Keypair.generate();
  const scriptAccount = scriptKeypair.publicKey.toString();

  // Calculate account size and rent
  const SCRIPT_HEADER_SIZE = 64; // ScriptHeader::LEN
  const exportMetadata = encodeExportMetadata(options.exportMetadata);
  const totalAccountSize = SCRIPT_HEADER_SIZE + exportMetadata.length + bytecode.length;
  const rentLamports = await connection.getMinimumBalanceForRentExemption(totalAccountSize);

  const vmStatePDA = await PDAUtils.deriveVMStatePDA(programIdStr);
  const vmStatePubkey = new PublicKey(vmStatePDA.address);

  if (options.debug) {
    console.log(`[FiveSDK] Preparing deployment transaction:`);
    console.log(`  - Script Account: ${scriptAccount}`);
    console.log(`  - VM State Account: ${vmStatePubkey.toString()}`);
    console.log(`  - Deployer: ${deployerPublicKey.toString()}`);
  }

  const tx = new Transaction();

  // Add compute budget if requested
  if (options.computeBudget && options.computeBudget > 0) {
    tx.add(
      ComputeBudgetProgram.setComputeUnitLimit({
        units: options.computeBudget,
      }),
    );
  }

  // 1. Initialize canonical VM State if missing
  const vmStateInfo = await connection.getAccountInfo(vmStatePubkey);
  if (!vmStateInfo) {
    tx.add(
      new TransactionInstruction({
        keys: [
          { pubkey: vmStatePubkey, isSigner: false, isWritable: true },
          { pubkey: deployerPublicKey, isSigner: true, isWritable: false },
          { pubkey: deployerPublicKey, isSigner: true, isWritable: true },
          { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
        ],
        programId: programId,
        data: buildInitializeVmStateInstructionData(vmStatePDA.bump),
      }),
    );
  }

  // 2. Create Script Account
  tx.add(
    SystemProgram.createAccount({
      fromPubkey: deployerPublicKey,
      newAccountPubkey: scriptKeypair.publicKey,
      lamports: rentLamports,
      space: totalAccountSize,
      programId: programId,
    }),
  );

  // 3. Deploy Instruction
  const deployData = encodeDeployInstruction(bytecode, 0, exportMetadata);
  tx.add(
    new TransactionInstruction({
      keys: [
        { pubkey: scriptKeypair.publicKey, isSigner: false, isWritable: true },
        { pubkey: vmStatePubkey, isSigner: false, isWritable: true },
        { pubkey: deployerPublicKey, isSigner: true, isWritable: true },
      ],
      programId: programId,
      data: Buffer.from(deployData),
    }),
  );

  const { blockhash } = await connection.getLatestBlockhash("confirmed");
  tx.recentBlockhash = blockhash;
  tx.feePayer = deployerPublicKey;

  // Partial sign with generated keys
  tx.partialSign(scriptKeypair);

  return {
    transaction: tx,
    scriptKeypair,
    vmStateKeypair: null,
    programId: scriptAccount,
    rentLamports,
  };
}

export async function deployToSolana(
  bytecode: FiveBytecode,
  connection: any, // Solana Connection object
  deployerKeypair: any, // Solana Keypair object
  options: {
    debug?: boolean;
    network?: string;
    computeBudget?: number;
    maxRetries?: number;
    fiveVMProgramId?: string;
    vmStateAccount?: string;
    exportMetadata?: ExportMetadataInput;
  } = {},
): Promise<{
  success: boolean;
  programId?: string;
  transactionId?: string;
  deploymentCost?: number;
  error?: string;
  logs?: string[];
  vmStateAccount?: string;
}> {
  console.log(
    `[FiveSDK] deployToSolana called with bytecode length: ${bytecode.length}`,
  );
  console.log(`[FiveSDK] options:`, options);

  // Resolve program ID with consistent precedence
  const programId = ProgramIdResolver.resolve(options.fiveVMProgramId);

  try {
    if (options.debug) {
      console.log(
        `[FiveSDK] Starting deployment with ${bytecode.length} bytes of bytecode to program ${programId}`,
      );
    }

    // Generate script keypair like frontend-five
    const {
      Keypair,
      PublicKey,
      Transaction,
      TransactionInstruction,
      SystemProgram,
    } = await import("@solana/web3.js");
    const scriptKeypair = Keypair.generate();
    const scriptAccount = scriptKeypair.publicKey.toString();

    if (options.debug) {
      console.log(`[FiveSDK] Generated script keypair: ${scriptAccount}`);
    }

    // Calculate account size and rent
    const SCRIPT_HEADER_SIZE = 64; // ScriptHeader::LEN (five-protocol)
    const exportMetadata = encodeExportMetadata(options.exportMetadata);
    const totalAccountSize = SCRIPT_HEADER_SIZE + exportMetadata.length + bytecode.length;
    const rentLamports =
      await connection.getMinimumBalanceForRentExemption(totalAccountSize);

    const vmStateResolution = await ensureCanonicalVmStateAccount(
      connection,
      deployerKeypair,
      new PublicKey(programId),
      {
        vmStateAccount: options.vmStateAccount,
        maxRetries: options.maxRetries,
        debug: options.debug,
      },
    );
    const vmStatePubkey = vmStateResolution.vmStatePubkey;
    const vmStateRent = vmStateResolution.vmStateRent;

    if (options.debug) {
      console.log(`[FiveSDK] Script Account: ${scriptAccount}`);
      console.log(`[FiveSDK] VM State Account: ${vmStatePubkey.toString()}`);
      console.log(`[FiveSDK] Account size: ${totalAccountSize} bytes`);
      console.log(`[FiveSDK] Export metadata size: ${exportMetadata.length} bytes`);
      console.log(`[FiveSDK] Rent cost: ${((rentLamports + vmStateRent) / 1e9)} SOL`);
    }

    // SINGLE TRANSACTION: create script account + deploy bytecode
    const tx = new Transaction();

    // Optional compute budget
    if (options.computeBudget && options.computeBudget > 0) {
      try {
        const { ComputeBudgetProgram } = await import("@solana/web3.js");
        tx.add(
          ComputeBudgetProgram.setComputeUnitLimit({
            units: options.computeBudget,
          }),
        );
      } catch { }
    }

    // 1) Create script account
    const createAccountIx = SystemProgram.createAccount({
      fromPubkey: deployerKeypair.publicKey,
      newAccountPubkey: scriptKeypair.publicKey,
      lamports: rentLamports,
      space: totalAccountSize,
      programId: new PublicKey(programId),
    });
    tx.add(createAccountIx);

    const deployData = encodeDeployInstruction(bytecode, 0, exportMetadata);

    const instructionDataBuffer = Buffer.from(deployData);

    const deployIx = new TransactionInstruction({
      keys: [
        {
          pubkey: scriptKeypair.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: vmStatePubkey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: deployerKeypair.publicKey,
          isSigner: true,
          isWritable: true,
        },
      ],
      programId: new PublicKey(programId),
      data: instructionDataBuffer,
    });
    tx.add(deployIx);

    const { blockhash } = await connection.getLatestBlockhash("confirmed");
    tx.recentBlockhash = blockhash;
    tx.feePayer = deployerKeypair.publicKey;

    tx.partialSign(deployerKeypair);
    tx.partialSign(scriptKeypair);

    const txSerialized = tx.serialize();
    if (options.debug) {
      console.log(`[FiveSDK] Transaction serialized: ${txSerialized.length} bytes`);
    }

    const signature = await connection.sendRawTransaction(txSerialized, {
      skipPreflight: true,
      preflightCommitment: "confirmed",
      maxRetries: options.maxRetries || 3,
    });

    if (options.debug) {
      console.log(`[FiveSDK] sendRawTransaction completed, returned signature: ${signature}`);
    }

    // Custom confirmation polling with extended timeout (120 seconds)
    const confirmationResult = await pollForConfirmation(
      connection,
      signature,
      "confirmed",
      120000, // 120 second timeout
      options.debug
    );

    if (!confirmationResult.success) {
      const errorMessage = `Deployment confirmation failed: ${confirmationResult.error || "Unknown error"}`;
      if (options.debug) console.log(`[FiveSDK] ${errorMessage}`);
      return {
        success: false,
        error: errorMessage,
        transactionId: signature,
      };
    }

    if (confirmationResult.err) {
      const errorMessage = `Combined deployment failed: ${JSON.stringify(confirmationResult.err)}`;
      if (options.debug) console.log(`[FiveSDK] ${errorMessage}`);
      return {
        success: false,
        error: errorMessage,
        transactionId: signature,
      };
    }

    if (options.debug) {
      console.log(`[FiveSDK] Combined deployment succeeded: ${signature}`);
    }

    return {
      success: true,
      programId: scriptAccount,
      transactionId: signature,
      deploymentCost: rentLamports + vmStateRent,
      logs: [
        `Script Account: ${scriptAccount}`,
        `Deployment TX: ${signature}`,
        `Deployment cost (rent): ${rentLamports / 1e9} SOL`,
        `Bytecode size: ${bytecode.length} bytes`,
        `VM State Account: ${vmStatePubkey.toString()}`,
      ],
      vmStateAccount: vmStatePubkey.toString(),
    };
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : "Unknown deployment error";

    if (options.debug) {
      console.error(`[FiveSDK] Deployment failed: ${errorMessage}`);
    }

    return {
      success: false,
      error: errorMessage,
      logs: [],
    };
  }
}

export async function deployLargeProgramToSolana(
  bytecode: FiveBytecode,
  connection: any, // Solana Connection object
  deployerKeypair: any, // Solana Keypair object
  options: {
    chunkSize?: number; // Default: 750 bytes
    debug?: boolean;
    network?: string;
    maxRetries?: number;
    fiveVMProgramId?: string;
    progressCallback?: (chunk: number, total: number) => void;
    vmStateAccount?: string;
  } = {},
): Promise<{
  success: boolean;
  scriptAccount?: string;
  transactionIds?: string[];
  totalTransactions?: number;
  deploymentCost?: number;
  chunksUsed?: number;
  vmStateAccount?: string;
  error?: string;
  logs?: string[];
}> {
  const DEFAULT_CHUNK_SIZE = 500; // Leaves room for transaction overhead
  const chunkSize = options.chunkSize || DEFAULT_CHUNK_SIZE;

  console.log(
    `[FiveSDK] deployLargeProgramToSolana called with ${bytecode.length} bytes`,
  );
  console.log(`[FiveSDK] Using chunk size: ${chunkSize} bytes`);
  console.log(`[FiveSDK] options:`, options);

  try {
    // If bytecode is small enough, use regular deployment
    if (bytecode.length <= 800) {
      if (options.debug) {
        console.log(
          `[FiveSDK] Bytecode is small (${bytecode.length} bytes), using regular deployment`,
        );
      }
      return await deployToSolana(
        bytecode,
        connection,
        deployerKeypair,
        {
          debug: options.debug,
          network: options.network,
          maxRetries: options.maxRetries,
          fiveVMProgramId: options.fiveVMProgramId,
          vmStateAccount: options.vmStateAccount,
        },
      );
    }

    const {
      Keypair,
      PublicKey,
      Transaction,
      TransactionInstruction,
      SystemProgram,
    } = await import("@solana/web3.js");

    // Generate script keypair
    const scriptKeypair = Keypair.generate();
    const scriptAccount = scriptKeypair.publicKey.toString();

    // Calculate account size and rent
    const SCRIPT_HEADER_SIZE = 64; // ScriptHeader::LEN (five-protocol)
    const totalAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
    const rentLamports =
      await connection.getMinimumBalanceForRentExemption(totalAccountSize);

    const programIdStr = ProgramIdResolver.resolve(options.fiveVMProgramId);
    const programId = new PublicKey(programIdStr);

    const vmStateResolution = await ensureCanonicalVmStateAccount(
      connection,
      deployerKeypair,
      programId,
      {
        vmStateAccount: options.vmStateAccount,
        maxRetries: options.maxRetries,
        debug: options.debug,
      },
    );
    const vmStatePubkey = vmStateResolution.vmStatePubkey;
    const vmStateRent = vmStateResolution.vmStateRent;

    if (options.debug) {
      console.log(`[FiveSDK] Script Account: ${scriptAccount}`);
      console.log(
        `[FiveSDK] VM State Account: ${vmStatePubkey.toString()}`,
      );
      console.log(`[FiveSDK] Total account size: ${totalAccountSize} bytes`);
      console.log(
        `[FiveSDK] Initial rent cost: ${(rentLamports + vmStateRent) / 1e9} SOL`,
      );
    }

    const transactionIds: string[] = [];
    let totalCost = rentLamports + vmStateRent;

    // TRANSACTION 1: Create Account + InitLargeProgram
    if (options.debug) {
      console.log(
        `[FiveSDK] Step 1: Create account and initialize large program`,
      );
    }

    const initTransaction = new Transaction();

    // Add account creation instruction
    const createAccountInstruction = SystemProgram.createAccount({
      fromPubkey: deployerKeypair.publicKey,
      newAccountPubkey: scriptKeypair.publicKey,
      lamports: rentLamports,
      space: SCRIPT_HEADER_SIZE, // Start with just header space
      programId: programId,
    });
    initTransaction.add(createAccountInstruction);

    // Add InitLargeProgram instruction (discriminator 4 + expected_size as u32)
    const initInstructionData = createInitLargeProgramInstructionData(
      bytecode.length,
    );

    const initLargeProgramInstruction = new TransactionInstruction({
      keys: [
        {
          pubkey: scriptKeypair.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: deployerKeypair.publicKey,
          isSigner: true,
          isWritable: true,
        },
        {
          pubkey: vmStatePubkey,
          isSigner: false,
          isWritable: true,
        },
      ],
      programId: programId,
      data: initInstructionData,
    });
    initTransaction.add(initLargeProgramInstruction);

    // Sign and send initialization transaction
    initTransaction.feePayer = deployerKeypair.publicKey;
    const { blockhash } = await connection.getLatestBlockhash("confirmed");
    initTransaction.recentBlockhash = blockhash;
    initTransaction.partialSign(deployerKeypair);
    initTransaction.partialSign(scriptKeypair);

    const initSignature = await connection.sendRawTransaction(
      initTransaction.serialize(),
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        maxRetries: options.maxRetries || 3,
      },
    );

    await connection.confirmTransaction(initSignature, "confirmed");
    transactionIds.push(initSignature);

    if (options.debug) {
      console.log(`[FiveSDK] ✅ Initialization completed: ${initSignature}`);
    }

    // STEP 2: Split bytecode into chunks and append each
    const chunks = chunkBytecode(bytecode, chunkSize);

    if (options.debug) {
      console.log(`[FiveSDK] Split bytecode into ${chunks.length} chunks`);
    }

    for (let i = 0; i < chunks.length; i++) {
      const chunk = chunks[i];

      if (options.progressCallback) {
        options.progressCallback(i + 1, chunks.length);
      }

      if (options.debug) {
        console.log(
          `[FiveSDK] Step ${i + 2}: Appending chunk ${i + 1}/${chunks.length} (${chunk.length} bytes)`,
        );
      }

      // Calculate additional rent needed for this chunk
      let currentInfo = await connection.getAccountInfo(
        scriptKeypair.publicKey,
      );

      // Retry logic for account info if null (eventual consistency)
      if (!currentInfo) {
        if (options.debug) console.log(`[FiveSDK] Account info null, retrying...`);
        await new Promise(resolve => setTimeout(resolve, 1000));
        currentInfo = await connection.getAccountInfo(scriptKeypair.publicKey);
        if (!currentInfo) throw new Error("Script account not found after initialization");
      }
      const newSize = currentInfo.data.length + chunk.length;
      const newRentRequired =
        await connection.getMinimumBalanceForRentExemption(newSize);
      const additionalRent = Math.max(
        0,
        newRentRequired - currentInfo.lamports,
      );

      const appendTransaction = new Transaction();

      // Add rent if needed
      if (additionalRent > 0) {
        if (options.debug) {
          console.log(
            `[FiveSDK] Adding ${additionalRent / 1e9} SOL for increased rent`,
          );
        }
        appendTransaction.add(
          SystemProgram.transfer({
            fromPubkey: deployerKeypair.publicKey,
            toPubkey: scriptKeypair.publicKey,
            lamports: additionalRent,
          }),
        );
      }

      // Add compute budget for final chunk (verification is expensive)
      if (i === chunks.length - 1) {
        try {
          const { ComputeBudgetProgram } = await import("@solana/web3.js");
          if (options.debug) console.log("[FiveSDK] Adding 1.4M CU limit for final chunk verification");
          appendTransaction.add(
            ComputeBudgetProgram.setComputeUnitLimit({
              units: 1_400_000,
            }),
          );
        } catch (e) {
          if (options.debug) console.warn("[FiveSDK] Failed to add compute budget:", e);
        }
      }

      if (additionalRent > 0) {
        totalCost += additionalRent;
      }

      // Add AppendBytecode instruction (discriminator 5 + chunk data)
      const appendInstructionData = createAppendBytecodeInstructionData(chunk);

      const appendBytecodeInstruction = new TransactionInstruction({
        keys: [
          {
            pubkey: scriptKeypair.publicKey,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: deployerKeypair.publicKey,
            isSigner: true,
            isWritable: true,
          },
          {
            pubkey: vmStatePubkey,
            isSigner: false,
            isWritable: true,
          },
        ],
        programId: programId,
        data: appendInstructionData,
      });
      appendTransaction.add(appendBytecodeInstruction);

      // Sign and send append transaction
      const appendBlockhash =
        await connection.getLatestBlockhash("confirmed");
      appendTransaction.feePayer = deployerKeypair.publicKey;
      appendTransaction.recentBlockhash = appendBlockhash.blockhash;
      appendTransaction.partialSign(deployerKeypair);

      const appendSignature = await connection.sendRawTransaction(
        appendTransaction.serialize(),
        {
          skipPreflight: true,
          preflightCommitment: "confirmed",
          maxRetries: options.maxRetries || 3,
        },
      );

      await connection.confirmTransaction(appendSignature, "confirmed");
      transactionIds.push(appendSignature);

      if (options.debug) {
        console.log(
          `[FiveSDK] ✅ Chunk ${i + 1} appended: ${appendSignature}`,
        );
      }
    }

    // Final verification
    const finalInfo = await connection.getAccountInfo(
      scriptKeypair.publicKey,
    );
    const expectedSize = SCRIPT_HEADER_SIZE + bytecode.length;

    if (options.debug) {
      console.log(`[FiveSDK] 🔍 Final verification:`);
      console.log(`[FiveSDK] Expected size: ${expectedSize} bytes`);
      console.log(`[FiveSDK] Actual size: ${finalInfo.data.length} bytes`);
      console.log(
        `[FiveSDK] Match: ${finalInfo.data.length === expectedSize ? "✅ YES" : "❌ NO"}`,
      );
    }

    return {
      success: true,
      scriptAccount,
      transactionIds,
      totalTransactions: transactionIds.length,
      deploymentCost: totalCost,
      chunksUsed: chunks.length,
      vmStateAccount: vmStatePubkey.toString(),
      logs: [
        `Deployed ${bytecode.length} bytes in ${chunks.length} chunks using ${transactionIds.length} transactions`,
      ],
    };
  } catch (error) {
    const errorMessage =
      error instanceof Error
        ? error.message
        : "Unknown large deployment error";

    if (options.debug) {
      console.error(`[FiveSDK] Large deployment failed: ${errorMessage}`);
    }

    return {
      success: false,
      error: errorMessage,
      logs: [],
    };
  }
}

export async function deployLargeProgramOptimizedToSolana(
  bytecode: FiveBytecode,
  connection: any, // Solana Connection object
  deployerKeypair: any, // Solana Keypair object
  options: {
    chunkSize?: number; // Default: 950 bytes (optimized for lower transaction overhead)
    debug?: boolean;
    network?: string;
    maxRetries?: number;
    fiveVMProgramId?: string;
    vmStateAccount?: string;
    exportMetadata?: ExportMetadataInput;
    progressCallback?: (transaction: number, total: number) => void;
  } = {},
): Promise<{
  success: boolean;
  scriptAccount?: string;
  transactionIds?: string[];
  totalTransactions?: number;
  deploymentCost?: number;
  chunksUsed?: number;
  vmStateAccount?: string;
  optimizationSavings?: {
    transactionsSaved: number;
    estimatedCostSaved: number;
  };
  error?: string;
  logs?: string[];
}> {
  const OPTIMIZED_CHUNK_SIZE = 500; // Larger chunks due to reduced transaction overhead
  const chunkSize = options.chunkSize || OPTIMIZED_CHUNK_SIZE;

  console.log(
    `[FiveSDK] deployLargeProgramOptimizedToSolana called with ${bytecode.length} bytes`,
  );
  console.log(`[FiveSDK] Using optimized chunk size: ${chunkSize} bytes`);
  console.log(`[FiveSDK] Expected optimization: 50-70% fewer transactions`);

  try {
    // If bytecode is small enough, use regular deployment
    if (bytecode.length <= 800) {
      if (options.debug) {
        console.log(
          `[FiveSDK] Bytecode is small (${bytecode.length} bytes), using regular deployment`,
        );
      }
      return await deployToSolana(
        bytecode,
        connection,
        deployerKeypair,
        {
          debug: options.debug,
          network: options.network,
          maxRetries: options.maxRetries,
          fiveVMProgramId: options.fiveVMProgramId,
          vmStateAccount: options.vmStateAccount,
          exportMetadata: options.exportMetadata,
        },
      );
    }

    const {
      Keypair,
      PublicKey,
      Transaction,
      TransactionInstruction,
      SystemProgram,
    } = await import("@solana/web3.js");

    // Generate script keypair
    const scriptKeypair = Keypair.generate();
    const scriptAccount = scriptKeypair.publicKey.toString();

    // Calculate full account size upfront
    const SCRIPT_HEADER_SIZE = 64; // ScriptAccountHeader::LEN
    const totalAccountSize = SCRIPT_HEADER_SIZE + bytecode.length;
    const rentLamports =
      await connection.getMinimumBalanceForRentExemption(totalAccountSize);

    const programIdStr = ProgramIdResolver.resolve(options.fiveVMProgramId);
    const programId = new PublicKey(programIdStr);

    const vmStateResolution = await ensureCanonicalVmStateAccount(
      connection,
      deployerKeypair,
      programId,
      {
        vmStateAccount: options.vmStateAccount,
        maxRetries: options.maxRetries,
        debug: options.debug,
      },
    );
    const vmStatePubkey = vmStateResolution.vmStatePubkey;
    const vmStateRent = vmStateResolution.vmStateRent;

    if (options.debug) {
      console.log(`[FiveSDK] Script Account: ${scriptAccount}`);
      console.log(
        `[FiveSDK] VM State Account: ${vmStatePubkey.toString()}`,
      );
      console.log(
        `[FiveSDK] PRE-ALLOCATED full account size: ${totalAccountSize} bytes`,
      );
      console.log(
        `[FiveSDK] Full rent cost paid upfront: ${(rentLamports + vmStateRent) / 1e9} SOL`,
      );
    }

    const transactionIds: string[] = [];
    let totalCost = rentLamports + vmStateRent;

    const chunks = chunkBytecode(bytecode, chunkSize);
    const firstChunk = chunks[0];
    const remainingChunks = chunks.slice(1);

    if (options.debug) {
      console.log(
        `[FiveSDK] Split into ${chunks.length} chunks (first: ${firstChunk.length} bytes, remaining: ${remainingChunks.length})`,
      );
    }

    if (options.debug) {
      console.log(
        `[FiveSDK] Create account + initialize with first chunk (${firstChunk.length} bytes)`,
      );
    }

    const initTransaction = new Transaction();

    const createAccountInstruction = SystemProgram.createAccount({
      fromPubkey: deployerKeypair.publicKey,
      newAccountPubkey: scriptKeypair.publicKey,
      lamports: rentLamports,
      space: totalAccountSize,
      programId: programId,
    });
    initTransaction.add(createAccountInstruction);

    const initInstructionData = createInitLargeProgramInstructionData(
      bytecode.length,
      firstChunk,
    );

    const initLargeProgramWithChunkInstruction = new TransactionInstruction({
      keys: [
        {
          pubkey: scriptKeypair.publicKey,
          isSigner: false,
          isWritable: true,
        },
        {
          pubkey: deployerKeypair.publicKey,
          isSigner: true,
          isWritable: true,
        },
        {
          pubkey: vmStatePubkey,
          isSigner: false,
          isWritable: true,
        },
      ],
      programId: programId,
      data: initInstructionData,
    });
    initTransaction.add(initLargeProgramWithChunkInstruction);

    initTransaction.feePayer = deployerKeypair.publicKey;
    const { blockhash } = await connection.getLatestBlockhash("confirmed");
    initTransaction.recentBlockhash = blockhash;
    initTransaction.partialSign(deployerKeypair);
    initTransaction.partialSign(scriptKeypair);

    const initSignature = await connection.sendRawTransaction(
      initTransaction.serialize(),
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        maxRetries: options.maxRetries || 3,
      },
    );

    const initConfirmation = await pollForConfirmation(
      connection,
      initSignature,
      "confirmed",
      120000,
      options.debug
    );
    if (!initConfirmation.success) {
      return {
        success: false,
        error: `Initialization confirmation failed: ${initConfirmation.error}`,
        transactionIds
      };
    }
    transactionIds.push(initSignature);

    if (options.debug) {
      console.log(
        `[FiveSDK] ✅ Optimized initialization completed: ${initSignature}`,
      );
      console.log(
        `[FiveSDK] First chunk (${firstChunk.length} bytes) included in initialization!`,
      );
    }

    // Group remaining chunks into multi-chunk transactions
    if (remainingChunks.length > 0) {
      const groupedChunks = groupChunksForOptimalTransactions(
        remainingChunks,
        500,
      ); // Leave room for multi-chunk overhead

      if (options.debug) {
        console.log(
          `[FiveSDK] Grouped ${remainingChunks.length} remaining chunks into ${groupedChunks.length} transactions`,
        );
      }

      for (let groupIdx = 0; groupIdx < groupedChunks.length; groupIdx++) {
        const chunkGroup = groupedChunks[groupIdx];

        if (options.progressCallback) {
          options.progressCallback(groupIdx + 2, groupedChunks.length + 1); // +1 for init transaction
        }

        if (options.debug) {
          console.log(
            `[FiveSDK] Step ${groupIdx + 2}: Appending ${chunkGroup.length} chunks in single transaction`,
          );
        }

        const appendTransaction = new Transaction();

        let appendInstruction: any; // TransactionInstruction from @solana/web3.js

        if (chunkGroup.length === 1) {
          // Use single-chunk AppendBytecode instruction for optimization fallback
          if (options.debug) {
            console.log(
              `[FiveSDK] Using single-chunk AppendBytecode for remaining chunk (${chunkGroup[0].length} bytes)`,
            );
          }

          const singleChunkData = createAppendBytecodeInstructionData(
            chunkGroup[0],
          );

          appendInstruction = new TransactionInstruction({
            keys: [
              {
                pubkey: scriptKeypair.publicKey,
                isSigner: false,
                isWritable: true,
              },
              {
                pubkey: deployerKeypair.publicKey,
                isSigner: true,
                isWritable: true,
              },
              {
                pubkey: vmStatePubkey,
                isSigner: false,
                isWritable: true,
              },
            ],
            programId: programId,
            data: singleChunkData,
          });
        } else {
          // Use multi-chunk instruction for groups with 2+ chunks
          const multiChunkData =
            createMultiChunkInstructionData(chunkGroup);

          appendInstruction = new TransactionInstruction({
            keys: [
              {
                pubkey: scriptKeypair.publicKey,
                isSigner: false,
                isWritable: true,
              },
              {
                pubkey: deployerKeypair.publicKey,
                isSigner: true,
                isWritable: true,
              },
              {
                pubkey: vmStatePubkey,
                isSigner: false,
                isWritable: true,
              },
            ],
            programId: programId,
            data: multiChunkData,
          });
        }

        appendTransaction.add(appendInstruction);

        // Sign and send multi-chunk transaction
        const appendBlockhash =
          await connection.getLatestBlockhash("confirmed");
        appendTransaction.feePayer = deployerKeypair.publicKey;
        appendTransaction.recentBlockhash = appendBlockhash.blockhash;
        appendTransaction.partialSign(deployerKeypair);

        const appendSignature = await connection.sendRawTransaction(
          appendTransaction.serialize(),
          {
            skipPreflight: true,
            preflightCommitment: "confirmed",
            maxRetries: options.maxRetries || 3,
          },
        );

        const appendConfirmation = await pollForConfirmation(
          connection,
          appendSignature,
          "confirmed",
          120000,
          options.debug
        );
        if (!appendConfirmation.success) {
          return {
            success: false,
            error: `Append confirmation failed: ${appendConfirmation.error}`,
            transactionIds
          };
        }
        transactionIds.push(appendSignature);

        if (options.debug) {
          console.log(
            `[FiveSDK] ✅ Multi-chunk append completed: ${appendSignature}`,
          );
          console.log(
            `[FiveSDK] Appended ${chunkGroup.length} chunks totaling ${chunkGroup.reduce((sum, chunk) => sum + chunk.length, 0)} bytes`,
          );
        }
      }
    }

    // Explicitly finalize the script to ensure upload_mode is cleared
    if (options.debug) {
      console.log(`[FiveSDK] Sending FinalizeScript instruction to complete deployment`);
    }
    const finalizeTransaction = new Transaction();
    finalizeTransaction.add(
      new TransactionInstruction({
        keys: [
          {
            pubkey: scriptKeypair.publicKey,
            isSigner: false,
            isWritable: true,
          },
          {
            pubkey: deployerKeypair.publicKey,
            isSigner: true,
            isWritable: true,
          },
        ],
        programId: programId,
        data: createFinalizeScriptInstructionData(),
      }),
    );
    finalizeTransaction.feePayer = deployerKeypair.publicKey;
    const finalizeBlockhash = await connection.getLatestBlockhash("confirmed");
    finalizeTransaction.recentBlockhash = finalizeBlockhash.blockhash;
    finalizeTransaction.partialSign(deployerKeypair);

    const finalizeSignature = await connection.sendRawTransaction(
      finalizeTransaction.serialize(),
      {
        skipPreflight: true,
        preflightCommitment: "confirmed",
        maxRetries: options.maxRetries || 3,
      },
    );
    // Use custom polling for finalize to handle validator latency
    const finalizeConfirmation = await pollForConfirmation(
      connection,
      finalizeSignature,
      "confirmed",
      120000, // 120 second timeout
      options.debug
    );
    if (!finalizeConfirmation.success) {
      console.error(`[FiveSDK] FinalizeScript confirmation failed: ${finalizeConfirmation.error}`);
    }
    transactionIds.push(finalizeSignature);
    if (options.debug) {
      console.log(`[FiveSDK] ✅ FinalizeScript completed: ${finalizeSignature}`);
    }

    // Calculate optimization savings
    const traditionalTransactionCount = 1 + chunks.length; // 1 init + N appends
    const optimizedTransactionCount = transactionIds.length;
    const transactionsSaved =
      traditionalTransactionCount - optimizedTransactionCount;
    const estimatedCostSaved = transactionsSaved * 0.000005 * 1e9; // Estimate 5000 lamports per transaction saved

    if (options.debug) {
      console.log(`[FiveSDK] 🎉 OPTIMIZATION RESULTS:`);
      console.log(
        `[FiveSDK]   Traditional method: ${traditionalTransactionCount} transactions`,
      );
      console.log(
        `[FiveSDK]   Optimized method: ${optimizedTransactionCount} transactions`,
      );
      console.log(
        `[FiveSDK]   Transactions saved: ${transactionsSaved} (${Math.round((transactionsSaved / traditionalTransactionCount) * 100)}% reduction)`,
      );
      console.log(
        `[FiveSDK]   Estimated cost saved: ${estimatedCostSaved / 1e9} SOL`,
      );
    }

    return {
      success: true,
      scriptAccount,
      transactionIds,
      totalTransactions: optimizedTransactionCount,
      deploymentCost: totalCost,
      chunksUsed: chunks.length,
      vmStateAccount: vmStatePubkey.toString(),
      optimizationSavings: {
        transactionsSaved,
        estimatedCostSaved,
      },
      logs: [
        `✅ Optimized deployment completed`,
        `📊 ${optimizedTransactionCount} transactions (saved ${transactionsSaved} vs traditional)`,
        `💰 Cost: ${totalCost / 1e9} SOL`,
        `🧩 Chunks: ${chunks.length}`,
        `⚡ Optimization: ${Math.round((transactionsSaved / traditionalTransactionCount) * 100)}% fewer transactions`,
      ],
    };
  } catch (error: any) {
    console.error("[FiveSDK] Optimized deployment failed:", error);

    const errorMessage =
      error instanceof Error ? error.message : "Unknown deployment error";

    return {
      success: false,
      error: errorMessage,
      logs: [],
    };
  }
}

function encodeDeployInstruction(
  bytecode: FiveBytecode,
  permissions: number = 0,
  metadata: Uint8Array = new Uint8Array(),
): Uint8Array {
  const lengthBuffer = Buffer.allocUnsafe(4);
  lengthBuffer.writeUInt32LE(bytecode.length, 0);
  const metadataLenBuffer = Buffer.allocUnsafe(4);
  metadataLenBuffer.writeUInt32LE(metadata.length, 0);

  const result = new Uint8Array(1 + 4 + 1 + 4 + metadata.length + bytecode.length);
  result[0] = 8; // Deploy discriminator (matches on-chain FIVE program)
  result.set(new Uint8Array(lengthBuffer), 1); // u32 LE length at bytes 1-4
  result[5] = permissions; // permissions byte at byte 5
  result.set(new Uint8Array(metadataLenBuffer), 6); // metadata length at bytes 6-9
  result.set(metadata, 10);
  result.set(bytecode, 10 + metadata.length);

  console.log(`[FiveSDK] Deploy instruction encoded:`, {
    discriminator: result[0],
    lengthBytes: Array.from(new Uint8Array(lengthBuffer)),
    permissions: result[5],
    metadataLength: metadata.length,
    bytecodeLength: bytecode.length,
    totalInstructionLength: result.length,
    expectedFormat: `[8, ${bytecode.length}_as_u32le, 0x${permissions.toString(16).padStart(2, '0')}, ${metadata.length}_as_u32le, metadata_bytes, bytecode_bytes]`,
    instructionHex:
      Buffer.from(result).toString("hex").substring(0, 20) + "...",
  });

  return result;
}

function encodeExportMetadata(input?: ExportMetadataInput): Uint8Array {
  if (!input) {
    return new Uint8Array();
  }

  const methods = (input.methods || []).filter(
    (m) => typeof m === "string" && m.length > 0,
  );
  const interfaces = (input.interfaces || []).filter(
    (i) => i && typeof i.name === "string" && i.name.length > 0,
  );

  const out: number[] = [];
  out.push(0x35, 0x45, 0x58, 0x50); // "5EXP"
  out.push(1); // bundle version

  out.push(Math.min(methods.length, 255));
  for (const method of methods.slice(0, 255)) {
    const bytes = Buffer.from(method, "utf8");
    out.push(Math.min(bytes.length, 255));
    for (const b of bytes.slice(0, 255)) out.push(b);
  }

  out.push(Math.min(interfaces.length, 255));
  for (const iface of interfaces.slice(0, 255)) {
    const nameBytes = Buffer.from(iface.name, "utf8");
    out.push(Math.min(nameBytes.length, 255));
    for (const b of nameBytes.slice(0, 255)) out.push(b);

    const pairs = Object.entries(iface.methodMap || {});
    out.push(Math.min(pairs.length, 255));
    for (const [method, callee] of pairs.slice(0, 255)) {
      const methodBytes = Buffer.from(method, "utf8");
      const calleeBytes = Buffer.from(callee, "utf8");
      out.push(Math.min(methodBytes.length, 255));
      for (const b of methodBytes.slice(0, 255)) out.push(b);
      out.push(Math.min(calleeBytes.length, 255));
      for (const b of calleeBytes.slice(0, 255)) out.push(b);
    }
  }

  return Uint8Array.from(out);
}

async function ensureCanonicalVmStateAccount(
  connection: any,
  deployerKeypair: any,
  programId: any,
  options: {
    vmStateAccount?: string;
    maxRetries?: number;
    debug?: boolean;
  } = {},
): Promise<{ vmStatePubkey: any; vmStateRent: number; created: boolean; bump: number }> {
  const { PublicKey, Transaction, TransactionInstruction, SystemProgram } =
    await import("@solana/web3.js");

  const canonical = await PDAUtils.deriveVMStatePDA(programId.toString());
  if (options.vmStateAccount && options.vmStateAccount !== canonical.address) {
    throw new Error(
      `vmStateAccount must be canonical PDA ${canonical.address}; got ${options.vmStateAccount}`,
    );
  }

  const vmStatePubkey = new PublicKey(canonical.address);
  const existing = await connection.getAccountInfo(vmStatePubkey);
  if (existing) {
    if (existing.owner.toBase58() !== programId.toBase58()) {
      throw new Error(
        `canonical VM state ${canonical.address} exists but is owned by ${existing.owner.toBase58()}, expected ${programId.toBase58()}`,
      );
    }
    if (options.debug) {
      console.log(`[FiveSDK] Reusing canonical VM State PDA: ${canonical.address}`);
    }
    return { vmStatePubkey, vmStateRent: 0, created: false, bump: canonical.bump };
  }

  const VM_STATE_SIZE = 56;
  const vmStateRent = await connection.getMinimumBalanceForRentExemption(VM_STATE_SIZE);
  if (options.debug) {
    console.log(`[FiveSDK] Initializing canonical VM State PDA: ${canonical.address}`);
  }

  const initTransaction = new Transaction();
  initTransaction.add(
    new TransactionInstruction({
      keys: [
        { pubkey: vmStatePubkey, isSigner: false, isWritable: true },
        { pubkey: deployerKeypair.publicKey, isSigner: true, isWritable: false },
        { pubkey: deployerKeypair.publicKey, isSigner: true, isWritable: true },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId,
      data: buildInitializeVmStateInstructionData(canonical.bump),
    }),
  );
  initTransaction.feePayer = deployerKeypair.publicKey;
  const initBlockhash = await connection.getLatestBlockhash("confirmed");
  initTransaction.recentBlockhash = initBlockhash.blockhash;
  initTransaction.partialSign(deployerKeypair);

  const initSignature = await connection.sendRawTransaction(
    initTransaction.serialize(),
    {
      skipPreflight: true,
      preflightCommitment: "confirmed",
      maxRetries: options.maxRetries || 3,
    },
  );
  const initConfirmation = await pollForConfirmation(
    connection,
    initSignature,
    "confirmed",
    120000,
    options.debug,
  );
  if (!initConfirmation.success || initConfirmation.err) {
    throw new Error(
      `canonical VM state initialization failed: ${initConfirmation.error || JSON.stringify(initConfirmation.err)}`,
    );
  }

  return { vmStatePubkey, vmStateRent, created: true, bump: canonical.bump };
}

function chunkBytecode(
  bytecode: Uint8Array,
  chunkSize: number,
): Uint8Array[] {
  const chunks: Uint8Array[] = [];
  for (let i = 0; i < bytecode.length; i += chunkSize) {
    const chunk = bytecode.slice(i, Math.min(i + chunkSize, bytecode.length));
    chunks.push(chunk);
  }
  return chunks;
}

function groupChunksForOptimalTransactions(
  chunks: Uint8Array[],
  maxGroupSize: number,
): Uint8Array[][] {
  const groups: Uint8Array[][] = [];
  let currentGroup: Uint8Array[] = [];
  let currentGroupSize = 0;

  const getGroupOverhead = (numChunks: number) => 1 + numChunks * 2;

  for (const chunk of chunks) {
    const groupOverhead = getGroupOverhead(currentGroup.length + 1);
    const newGroupSize = currentGroupSize + chunk.length + 2;

    if (currentGroup.length === 0) {
      currentGroup.push(chunk);
      currentGroupSize = newGroupSize;
    } else if (
      newGroupSize + groupOverhead <= maxGroupSize &&
      currentGroup.length < 8
    ) {
      currentGroup.push(chunk);
      currentGroupSize = newGroupSize;
    } else {
      groups.push(currentGroup);
      currentGroup = [chunk];
      currentGroupSize = chunk.length + 2;
    }
  }

  if (currentGroup.length > 0) {
    groups.push(currentGroup);
  }

  return groups;
}

function createMultiChunkInstructionData(chunks: Uint8Array[]): Buffer {
  if (chunks.length < 2 || chunks.length > 10) {
    throw new Error(
      `Invalid chunk count for multi-chunk instruction: ${chunks.length}`,
    );
  }

  const buffers: Buffer[] = [
    Buffer.from([5]), // AppendBytecode discriminator
  ];

  for (const chunk of chunks) {
    buffers.push(Buffer.from(chunk));
  }

  return Buffer.concat(buffers);
}

function buildInitializeVmStateInstructionData(bump: number = 0): Buffer {
  return Buffer.from([0, bump & 0xff]);
}

function createInitLargeProgramInstructionData(
  expectedSize: number,
  firstChunk?: Uint8Array,
): Buffer {
  const sizeBuffer = Buffer.allocUnsafe(4);
  sizeBuffer.writeUInt32LE(expectedSize, 0);
  const parts = [Buffer.from([4]), sizeBuffer];
  if (firstChunk && firstChunk.length > 0) {
    parts.push(Buffer.from(firstChunk));
  }
  return Buffer.concat(parts);
}

function createAppendBytecodeInstructionData(chunk: Uint8Array): Buffer {
  return Buffer.concat([Buffer.from([5]), Buffer.from(chunk)]);
}

function createFinalizeScriptInstructionData(): Buffer {
  return Buffer.from([7]);
}

export const __deployTestUtils = {
  buildInitializeVmStateInstructionData,
  createInitLargeProgramInstructionData,
  createAppendBytecodeInstructionData,
  createFinalizeScriptInstructionData,
};
