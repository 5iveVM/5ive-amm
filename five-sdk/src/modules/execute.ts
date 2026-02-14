import {
  FiveScriptSource,
  FiveBytecode,
  SerializedExecution,
  FIVE_VM_PROGRAM_ID,
  FiveSDKError,
  ExecutionSDKError,
  CompilationOptions,
} from "../types.js";
import { BytecodeEncoder } from "../lib/bytecode-encoder.js";
import { PDAUtils, Base58Utils } from "../crypto/index.js";
import { ScriptMetadataParser } from "../metadata/index.js";
import { resolveFunctionIndex, normalizeAbiFunctions } from "../utils/abi.js";
import { validator, Validators } from "../validation/index.js";
import { calculateExecuteFee } from "./fees.js";
import { loadWasmVM } from "../wasm/instance.js";
import { BytecodeCompiler } from "../compiler/BytecodeCompiler.js";
import { ProgramIdResolver } from "../config/ProgramIdResolver.js";

const DEFAULT_FEE_VAULT_SHARD_COUNT = 10;
const FEE_VAULT_NAMESPACE_SEED = Buffer.from([
  0xff, 0x66, 0x69, 0x76, 0x65, 0x5f, 0x76, 0x6d, 0x5f, 0x66, 0x65, 0x65,
  0x5f, 0x76, 0x61, 0x75, 0x6c, 0x74, 0x5f, 0x76, 0x31,
]);
const EXECUTE_FEE_HEADER_A = 0xff;
const EXECUTE_FEE_HEADER_B = 0x53;

async function deriveProgramFeeVault(
  programId: string,
  shardIndex: number,
): Promise<{ address: string; bump: number }> {
  const { PublicKey } = await import("@solana/web3.js");
  const [pda, bump] = PublicKey.findProgramAddressSync(
    [FEE_VAULT_NAMESPACE_SEED, Buffer.from([shardIndex])],
    new PublicKey(programId),
  );
  return { address: pda.toBase58(), bump };
}

async function readVMStateShardCount(
  connection: any,
  vmStateAddress: string,
): Promise<number> {
  if (!connection) return DEFAULT_FEE_VAULT_SHARD_COUNT;
  try {
    const { PublicKey } = await import("@solana/web3.js");
    const info = await connection.getAccountInfo(new PublicKey(vmStateAddress), "confirmed");
    if (!info) return DEFAULT_FEE_VAULT_SHARD_COUNT;
    const data = new Uint8Array(info.data);
    if (data.length <= 82) return DEFAULT_FEE_VAULT_SHARD_COUNT;
    const shardCount = data[82];
    return shardCount > 0 ? shardCount : DEFAULT_FEE_VAULT_SHARD_COUNT;
  } catch {
    return DEFAULT_FEE_VAULT_SHARD_COUNT;
  }
}

function selectFeeShard(shardCount: number): number {
  const totalShards = Math.max(1, shardCount | 0);
  if (typeof crypto !== "undefined" && typeof crypto.getRandomValues === "function") {
    const bytes = new Uint32Array(1);
    crypto.getRandomValues(bytes);
    return bytes[0] % totalShards;
  }
  return Math.floor(Math.random() * totalShards);
}

// Helper function to initialize ParameterEncoder if needed (though BytecodeEncoder is preferred)
// Assume BytecodeEncoder handles it or call it if needed.
// BytecodeEncoder uses WASM module directly via loader.

export async function execute(
  compiler: BytecodeCompiler,
  source: FiveScriptSource | string,
  functionName: string | number,
  parameters: any[] = [],
  options: {
    debug?: boolean;
    trace?: boolean;
    optimize?: boolean;
    computeUnitLimit?: number;
    vmStateAccount?: string;
    accounts?: string[];
  } = {},
) {
  const sourceContent = typeof source === 'string' ? source : source.content;

  Validators.sourceCode(sourceContent);
  Validators.functionRef(functionName);
  Validators.parameters(parameters);
  Validators.options(options);

  if (options.debug) {
    console.log(`[FiveSDK] Compile and execute locally: ${functionName}`);
  }

  // Compile the script
  const compilation = await compiler.compile(source, {
    optimize: options.optimize,
    debug: options.debug,
  });

  if (!compilation.success || !compilation.bytecode) {
    return {
      success: false,
      compilationErrors: compilation.errors,
      error: "Compilation failed",
    };
  }

  if (options.debug) {
    console.log(`[FiveSDK] Compilation successful, executing bytecode...`);
  }

  // Execute the compiled bytecode
  const execution = await executeLocally(
    compilation.bytecode,
    functionName,
    parameters,
    {
      debug: options.debug,
      trace: options.trace,
      computeUnitLimit: options.computeUnitLimit,
      accounts: options.accounts,
      abi: compilation.abi, // Pass ABI from compilation for function name resolution
    },
  );

  return {
    ...execution,
    compilation,
    bytecodeSize: compilation.bytecode.length,
    functions: compilation.metadata?.functions,
  };
}

export async function executeLocally(
  bytecode: FiveBytecode,
  functionName: string | number,
  parameters: any[] = [],
  options: {
    debug?: boolean;
    trace?: boolean;
    computeUnitLimit?: number;
    abi?: any; // Optional ABI for function name resolution
    accounts?: string[]; // Account addresses for execution context
  } = {},
): Promise<{
  success: boolean;
  result?: any;
  logs?: string[];
  computeUnitsUsed?: number;
  executionTime?: number;
  error?: string;
  trace?: any[];
}> {
  Validators.bytecode(bytecode);
  Validators.functionRef(functionName);
  Validators.parameters(parameters);
  Validators.options(options);

  const startTime = Date.now();

  if (options.debug) {
    console.log(
      `[FiveSDK] Executing locally: function=${functionName}, params=${parameters.length}`,
    );
    console.log(`[FiveSDK] Parameters:`, parameters);
  }

  try {
    const wasmVM = await loadWasmVM();

    let resolvedFunctionIndex: number;
    if (typeof functionName === "number") {
      resolvedFunctionIndex = functionName;
    } else if (options.abi) {
      try {
        resolvedFunctionIndex = resolveFunctionIndex(
          options.abi,
          functionName,
        );
      } catch (resolutionError) {
        throw new FiveSDKError(
          `Function name resolution failed: ${resolutionError instanceof Error ? resolutionError.message : "Unknown error"}`,
          "FUNCTION_RESOLUTION_ERROR",
        );
      }
    } else {
      throw new FiveSDKError(
        `Cannot resolve function name '${functionName}' without ABI information. Please provide function index or use compileAndExecuteLocally() instead.`,
        "MISSING_ABI_ERROR",
      );
    }

    const transformedParams = parameters.map((param, index) => ({
      type: inferParameterType(param),
      value: param,
    }));

    if (options.debug) {
      console.log(
        `[FiveSDK] Resolved function index: ${resolvedFunctionIndex}`,
      );
      console.log(`[FiveSDK] Transformed parameters:`, transformedParams);
    }

    let accountInfos: any[] = [];
    if (options.accounts && options.accounts.length > 0) {
      accountInfos = options.accounts.map((address, index) => ({
        key: address,
        lamports: 0,
        data: new Uint8Array(0),
        owner: 'TokenkegQfeZyiNwAJsyFbPVwwQQforre5PJNYbToN', // System program default
        isExecutable: false,
        isSigner: index === 0, // First account is signer by default
        isWritable: index === 1, // Second account is mutable by default
      }));

      if (options.debug) {
        console.log(
          `[FiveSDK] Passing ${accountInfos.length} accounts to WASM VM execution`
        );
        accountInfos.forEach((acc, i) => {
          console.log(
            `  Account ${i}: ${acc.key.substring(0, 8)}... (signer=${acc.isSigner}, writable=${acc.isWritable})`
          );
        });
      }
    }

    const result = await wasmVM.executeFunction(
      bytecode,
      resolvedFunctionIndex,
      transformedParams,
      accountInfos.length > 0 ? accountInfos : undefined
    );

    const executionTime = Date.now() - startTime;

    if (options.debug) {
      console.log(
        `[FiveSDK] Local execution ${result.success ? "completed" : "failed"} in ${executionTime}ms`,
      );
      if (result.computeUnitsUsed) {
        console.log(
          `[FiveSDK] Compute units used: ${result.computeUnitsUsed}`,
        );
      }
    }

    return {
      success: result.success,
      result: result.result,
      logs: result.logs,
      computeUnitsUsed: result.computeUnitsUsed,
      executionTime,
      error: result.error,
      trace: result.trace,
    };
  } catch (error) {
    const executionTime = Date.now() - startTime;
    const errorMessage =
      error instanceof Error ? error.message : "Unknown execution error";

    if (options.debug) {
      console.log(
        `[FiveSDK] Local execution failed after ${executionTime}ms: ${errorMessage}`,
      );
    }

    return {
      success: false,
      executionTime,
      error: errorMessage,
    };
  }
}

export async function generateExecuteInstruction(
  scriptAccount: string,
  functionName: string | number,
  parameters: any[] = [],
  accounts: string[] = [],
  connection?: any,
  options: {
    debug?: boolean;
    computeUnitLimit?: number;
    vmStateAccount?: string;
    fiveVMProgramId?: string;
    abi?: any;
    adminAccount?: string;
    estimateFees?: boolean;
    accountMetadata?: Map<string, { isSigner: boolean; isWritable: boolean; isSystemAccount?: boolean }>;
    feeShardIndex?: number;
  } = {},
): Promise<SerializedExecution> {
  validator.validateBase58Address(scriptAccount, "scriptAccount");
  Validators.functionRef(functionName);
  Validators.parameters(parameters);
  Validators.accounts(accounts);
  Validators.options(options);

  if (options.debug) {
    console.log(`[FiveSDK] Generating execution instruction:`, {
      scriptAccount,
      function: functionName,
      parameterCount: parameters.length,
      accountCount: accounts.length,
    });
  }

  let functionIndex: number;
  let encodedParams: Uint8Array;
  let actualParamCount: number = 0;
  let funcDef: any = null;

  try {
    let scriptMetadata = options.abi;

    if (!scriptMetadata) {
      // Need to fetch script metadata
      if (connection) {
        const metadata = await ScriptMetadataParser.getScriptMetadata(
          connection,
          scriptAccount,
        );
        const normalizedFunctions = normalizeAbiFunctions(
          metadata.abi?.functions ?? metadata.abi,
        );
        scriptMetadata = {
          functions: normalizedFunctions.map((func) => ({
            name: func.name,
            index: func.index,
            parameters: func.parameters,
            returnType: func.returnType,
            visibility: func.visibility,
          })),
        };
      } else {
        throw new Error(
          "No connection provided for metadata retrieval. " +
          "In client-agnostic mode, provide script metadata directly or use getScriptMetadataWithConnection().",
        );
      }
    }

    if (Array.isArray(scriptMetadata.functions)) {
    } else if (typeof scriptMetadata.functions === 'object' && scriptMetadata.functions !== null) {
      scriptMetadata.functions = Object.entries(scriptMetadata.functions).map(([name, func]: [string, any]) => ({
        name,
        ...(func || {}),
      }));
    }

    functionIndex =
      typeof functionName === "number"
        ? functionName
        : resolveFunctionIndex(scriptMetadata, functionName);

    funcDef = Array.isArray(scriptMetadata.functions)
      ? scriptMetadata.functions.find((f: any) => f.index === functionIndex)
      : scriptMetadata.functions[functionIndex];

    const encoded = await encodeParametersWithABI(
      parameters,
      funcDef,
      functionIndex,
      accounts,
      options,
    );
    actualParamCount = encoded.paramCount;
    encodedParams = encoded.encoded;
  } catch (metadataError) {
    if (options.debug) {
      console.log(
        `[FiveSDK] Metadata not available, using fixed encoding with assumed parameter types`,
      );
      console.log(`[FiveSDK] ABI processing error:`, metadataError);
    }

    functionIndex = typeof functionName === "number" ? functionName : 0;

    const paramDefs = parameters.map((_, index) => ({
      name: `param${index}`,
      type: "u64",
    }));

    const paramValues: Record<string, any> = {};
    paramDefs.forEach((param, index) => {
      paramValues[param.name] = parameters[index];
    });

    actualParamCount = paramDefs.length;
    encodedParams = await BytecodeEncoder.encodeExecute(
      functionIndex,
      paramDefs,
      paramValues,
      true,
      options,
    );
  }

  // Resolve program ID with consistent precedence
  const programId = ProgramIdResolver.resolve(options.fiveVMProgramId);

  const vmStatePDA = await PDAUtils.deriveVMStatePDA(programId);
  const vmState = options.vmStateAccount || vmStatePDA.address;
  const shardCount = await readVMStateShardCount(connection, vmState);
  const feeShardIndex =
    options.feeShardIndex !== undefined
      ? ((options.feeShardIndex % shardCount) + shardCount) % shardCount
      : selectFeeShard(shardCount);
  const feeVault = await deriveProgramFeeVault(programId, feeShardIndex);

  const instructionAccounts = [
    { pubkey: scriptAccount, isSigner: false, isWritable: false },
    { pubkey: vmState, isSigner: false, isWritable: false },
  ];

  const abiAccountMetadata = new Map<string, { isSigner: boolean; isWritable: boolean }>();

  if (funcDef && funcDef.parameters) {
    // First pass: detect if there's an @init constraint and find the payer
    let hasInit = false;
    let payerPubkey: string | undefined;
    for (let i = 0; i < funcDef.parameters.length; i++) {
      const param = funcDef.parameters[i];
      if (param.is_account || param.isAccount) {
        const attributes = param.attributes || [];
        if (attributes.includes('init')) {
          hasInit = true;
          for (let j = 0; j < funcDef.parameters.length; j++) {
            const payerParam = funcDef.parameters[j];
            if (
              i !== j &&
              (payerParam.is_account || payerParam.isAccount) &&
              (payerParam.attributes || []).includes('signer')
            ) {
              const payerValue = parameters[j];
              payerPubkey = payerValue?.toString();
              break;
            }
          }
          break;
        }
      }
    }

    funcDef.parameters.forEach((param: any, paramIndex: number) => {
      if (param.is_account || param.isAccount) {
        const value = parameters[paramIndex];
        const pubkey = value?.toString();
        if (pubkey) {
          const attributes = param.attributes || [];
          const isSigner = attributes.includes('signer');
          const isWritable = attributes.includes('mut') ||
            attributes.includes('init') ||
            (hasInit && pubkey === payerPubkey);

          const existing = abiAccountMetadata.get(pubkey) || { isSigner: false, isWritable: false };
          abiAccountMetadata.set(pubkey, {
            isSigner: existing.isSigner || isSigner,
            isWritable: existing.isWritable || isWritable
          });
        }
      }
    });
  }

  const userInstructionAccounts = accounts.map((acc) => {
    // Check both derived ABI metadata and passed-in metadata (from FunctionBuilder)
    const abiMetadata = abiAccountMetadata.get(acc);
    const passedMetadata = options.accountMetadata?.get(acc);
    const metadata = abiMetadata || passedMetadata;
    const isSigner = metadata ? metadata.isSigner : false;
    const isWritable = metadata ? metadata.isWritable : true;

    return {
      pubkey: acc,
      isSigner,
      isWritable
    };
  });

  instructionAccounts.push(...userInstructionAccounts);

  const instructionData = encodeExecuteInstruction(
    functionIndex,
    encodedParams,
    actualParamCount,
    feeShardIndex,
    feeVault.bump,
  );

  const result: SerializedExecution = {
    instruction: {
      programId: programId,
      accounts: instructionAccounts,
      data: Buffer.from(instructionData).toString("base64"),
    },
    scriptAccount,
    parameters: {
      function: functionName,
      data: encodedParams,
      count: parameters.length,
    },
    requiredSigners: [],
    estimatedComputeUnits:
      options.computeUnitLimit ||
      estimateComputeUnits(functionIndex, parameters.length),
    adminAccount: feeVault.address,
    feeRecipientAccount: feeVault.address,
  };

  const shouldEstimateFees = options.estimateFees !== false && connection;

  if (shouldEstimateFees) {
    try {
      const executeFee = await calculateExecuteFee(
        connection,
        programId,
      );
      result.feeInformation = executeFee;
    } catch (error) {
      if (options.debug) {
        console.warn(
          `[FiveSDK] Could not estimate execute fees:`,
          error instanceof Error ? error.message : "Unknown error",
        );
      }
    }
  }

  return result;
}

export async function executeOnSolana(
  scriptAccount: string,
  connection: any,
  signerKeypair: any,
  functionName: string | number,
  parameters: any[] = [],
  accounts: string[] = [],
  options: {
    debug?: boolean;
    network?: string;
    computeUnitLimit?: number;
    computeUnitPrice?: number;
    maxRetries?: number;
    skipPreflight?: boolean;
    vmStateAccount?: string;
    fiveVMProgramId?: string;
    abi?: any;
    feeShardIndex?: number;
  } = {},
): Promise<{
  success: boolean;
  result?: any;
  transactionId?: string;
  computeUnitsUsed?: number;
  cost?: number;
  error?: string;
  logs?: string[];
}> {
  let lastSignature: string | undefined;

  try {
    const {
      PublicKey,
      Transaction,
      TransactionInstruction,
      ComputeBudgetProgram,
    } = await import("@solana/web3.js");

    let executionData;
    try {
      executionData = await generateExecuteInstruction(
        scriptAccount,
        functionName,
        parameters,
        accounts,
        connection,
        {
          debug: options.debug,
          computeUnitLimit: options.computeUnitLimit,
          vmStateAccount: options.vmStateAccount,
          fiveVMProgramId: options.fiveVMProgramId,
          abi: options.abi,
          feeShardIndex: options.feeShardIndex,
        },
      );
    } catch (metadataError) {
      throw new Error(`Execution instruction generation failed: ${metadataError instanceof Error ? metadataError.message : "Unknown metadata error"}`);
    }

    const transaction = new Transaction();

    if (options.computeUnitLimit && options.computeUnitLimit > 200000) {
      const computeBudgetIx = ComputeBudgetProgram.setComputeUnitLimit({
        units: options.computeUnitLimit,
      });
      transaction.add(computeBudgetIx);
    }

    if (options.computeUnitPrice && options.computeUnitPrice > 0) {
      const computePriceIx = ComputeBudgetProgram.setComputeUnitPrice({
        microLamports: options.computeUnitPrice,
      });
      transaction.add(computePriceIx);
    }

    const accountKeys = [...executionData.instruction.accounts];
    if (options.vmStateAccount && accountKeys.length >= 2) {
      for (let i = 0; i < accountKeys.length; i++) {
        if (i === 1) {
          accountKeys[i].pubkey = options.vmStateAccount;
          break;
        }
      }
    }

    const signerPubkey = signerKeypair.publicKey.toString();
    const systemProgramId = "11111111111111111111111111111111";
    let signerFound = false;
    for (const meta of accountKeys) {
      if (meta.pubkey === signerPubkey) {
        meta.isSigner = true;
        meta.isWritable = true;
        signerFound = true;
      }
    }

    if (!signerFound) {
      accountKeys.push({
        pubkey: signerPubkey,
        isSigner: true,
        isWritable: true,
      });
    }
    // Strict runtime fee account contract tail: [payer, fee_vault, system_program]
    accountKeys.push({
      pubkey: signerPubkey,
      isSigner: true,
      isWritable: true,
    });
    accountKeys.push({
      pubkey: executionData.adminAccount!,
      isSigner: false,
      isWritable: true,
    });
    accountKeys.push({
      pubkey: systemProgramId,
      isSigner: false,
      isWritable: false,
    });

    const executeInstruction = new TransactionInstruction({
      keys: accountKeys.map((acc) => ({
        pubkey: new PublicKey(acc.pubkey),
        isSigner: acc.isSigner,
        isWritable: acc.isWritable,
      })),
      programId: new PublicKey(executionData.instruction.programId),
      data: Buffer.from(executionData.instruction.data, "base64"),
    });

    transaction.add(executeInstruction);

    transaction.feePayer = signerKeypair.publicKey;
    const { blockhash } = await connection.getLatestBlockhash("confirmed");
    transaction.recentBlockhash = blockhash;

    transaction.partialSign(signerKeypair);
    const firstSig = transaction.signatures[0]?.signature;
    if (firstSig) {
      lastSignature = Base58Utils.encode(firstSig);
    }

    const signature = await connection.sendRawTransaction(
      transaction.serialize(),
      {
        skipPreflight: options.skipPreflight ?? false,
        preflightCommitment: "confirmed",
        maxRetries: options.maxRetries || 3,
      },
    );
    lastSignature = signature;

    let confirmation;
    try {
      confirmation = await connection.confirmTransaction(
        {
          signature,
          blockhash,
          lastValidBlockHeight: (
            await connection.getLatestBlockhash("confirmed")
          ).lastValidBlockHeight,
        },
        "confirmed",
      );
    } catch (confirmError) {
      try {
        const txDetails = await connection.getTransaction(signature, {
          commitment: "confirmed",
          maxSupportedTransactionVersion: 0,
        });

        if (txDetails) {
          if (txDetails.meta?.err) {
            return {
              success: false,
              error: `Transaction failed: ${JSON.stringify(txDetails.meta.err)}`,
              logs: txDetails.meta.logMessages || [],
              transactionId: signature,
            };
          } else {
            return {
              success: true,
              transactionId: signature,
              computeUnitsUsed: txDetails.meta?.computeUnitsConsumed,
              logs: txDetails.meta?.logMessages || [],
              result:
                "Execution completed successfully (confirmation timeout but transaction succeeded)",
            };
          }
        }
      } catch (getTransactionError) {}
      throw confirmError;
    }

    if (confirmation.value.err) {
      let logs: string[] = [];
      let computeUnitsUsed: number | undefined;
      try {
        const txDetails = await connection.getTransaction(signature, {
          commitment: "confirmed",
          maxSupportedTransactionVersion: 0,
        });
        if (txDetails?.meta) {
          logs = txDetails.meta.logMessages || [];
          computeUnitsUsed = txDetails.meta.computeUnitsConsumed || undefined;
        }
      } catch { }

      const errorMessage = `Execution transaction failed: ${JSON.stringify(
        confirmation.value.err,
      )}`;
      return {
        success: false,
        error: errorMessage,
        transactionId: signature,
        logs,
        computeUnitsUsed,
      };
    }

    let computeUnitsUsed: number | undefined;
    let logs: string[] = [];

    try {
      const txDetails = await connection.getTransaction(signature, {
        commitment: "confirmed",
        maxSupportedTransactionVersion: 0,
      });

      if (txDetails?.meta) {
        computeUnitsUsed = txDetails.meta.computeUnitsConsumed || undefined;
        logs = txDetails.meta.logMessages || [];
      }
    } catch (logError) {}

    return {
      success: true,
      transactionId: signature,
      computeUnitsUsed,
      logs,
      result: "Execution completed successfully",
    };
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : "Unknown execution error";

    if (!lastSignature && (error as any)?.signature) {
      lastSignature = (error as any).signature;
    }

    let logs: string[] = (error as any)?.transactionLogs || [];
    if (typeof (error as any)?.getLogs === "function") {
      try {
        const extracted = await (error as any).getLogs();
        if (Array.isArray(extracted)) {
          logs = extracted;
        }
      } catch {}
    }

    return {
      success: false,
      error: errorMessage,
      transactionId: lastSignature,
      logs,
    };
  }
}

export async function executeScriptAccount(
  scriptAccount: string,
  functionIndex: number = 0,
  parameters: any[] = [],
  connection: any,
  signerKeypair: any,
  options: {
    debug?: boolean;
    network?: string;
    computeBudget?: number;
    maxRetries?: number;
    vmStateAccount?: string;
    fiveVMProgramId?: string;
  } = {},
): Promise<{
  success: boolean;
  result?: any;
  transactionId?: string;
  computeUnitsUsed?: number;
  cost?: number;
  error?: string;
  logs?: string[];
}> {
  return executeOnSolana(
    scriptAccount,
    connection,
    signerKeypair,
    functionIndex,
    parameters,
    [],
    {
      debug: options.debug,
      network: options.network,
      computeUnitLimit: options.computeBudget || 1400000,
      maxRetries: options.maxRetries || 3,
      vmStateAccount: options.vmStateAccount,
      fiveVMProgramId: options.fiveVMProgramId,
    },
  );
}

// Helpers

function encodeExecuteInstruction(
  functionIndex: number,
  encodedParams: Uint8Array,
  paramCount: number,
  feeShardIndex: number,
  feeVaultBump: number,
): Uint8Array {
  const parts = [];
  parts.push(new Uint8Array([9]));
  parts.push(
    new Uint8Array([
      EXECUTE_FEE_HEADER_A,
      EXECUTE_FEE_HEADER_B,
      feeShardIndex & 0xff,
      feeVaultBump & 0xff,
    ]),
  );
  // Function index as fixed u32
  parts.push(encodeU32(functionIndex));

  // Param count as fixed u32
  parts.push(encodeU32(paramCount));
  parts.push(encodedParams);

  const totalLength = parts.reduce((sum, part) => sum + part.length, 0);
  const result = new Uint8Array(totalLength);
  let resultOffset = 0;

  for (const part of parts) {
    result.set(part, resultOffset);
    resultOffset += part.length;
  }
  return result;
}

function encodeU32(value: number): Uint8Array {
  const buffer = new ArrayBuffer(4);
  const view = new DataView(buffer);
  view.setUint32(0, value, true); // Little Endian
  return new Uint8Array(buffer);
}

function inferParameterType(value: any): string {
  if (typeof value === "boolean") {
    return "bool";
  } else if (typeof value === "number") {
    if (Number.isInteger(value)) {
      return value >= 0 ? "u64" : "i64";
    } else {
      return "f64";
    }
  } else if (typeof value === "string") {
    return "string";
  } else if (value instanceof Uint8Array) {
    return "bytes";
  } else {
    return "string";
  }
}

async function encodeParametersWithABI(
  parameters: any[],
  functionDef: any,
  functionIndex: number,
  _accounts: string[] = [],
  options: any = {},
): Promise<{ encoded: Uint8Array; paramCount: number }> {
  const isAccountParam = (param: any): boolean => {
    if (!param) return false;
    if (param.isAccount || param.is_account) return true;
    const type = (param.type || param.param_type || '').toString().trim().toLowerCase();
    return type === 'account' || type === 'mint' || type === 'tokenaccount';
  };

  const isPubkeyParam = (param: any): boolean => {
    if (!param) return false;
    const type = (param.type || param.param_type || '').toString().trim().toLowerCase();
    return type === 'pubkey';
  };

  const paramDefs = (functionDef.parameters || []);
  const nonAccountParamDefs = paramDefs.filter((param: any) => !isAccountParam(param));
  const fullParameterListProvided = parameters.length >= paramDefs.length;

  if (fullParameterListProvided && parameters.length !== paramDefs.length) {
    console.warn(
      `[FiveSDK] Parameter validation warning: Function '${functionDef.name}' expects ${paramDefs.length} parameters, but received ${parameters.length}.`
    );
  }

  const paramValues: Record<string, any> = {};
  let argCursor = 0;
  for (let index = 0; index < paramDefs.length; index++) {
    const param = paramDefs[index];
    if (isAccountParam(param)) {
      continue;
    }

    const sourceIndex = fullParameterListProvided ? index : argCursor;
    if (sourceIndex >= parameters.length) {
      throw new Error(`Missing value for parameter: ${param.name}`);
    }

    let value = parameters[sourceIndex];
    if (isPubkeyParam(param)) {
      if (value && typeof value === 'object' && typeof value.toBase58 === 'function') {
        value = value.toBase58();
      }
    }
    paramValues[param.name] = value;
    argCursor += 1;
  }

  const encoded = await BytecodeEncoder.encodeExecute(
    functionIndex,
    nonAccountParamDefs,
    paramValues,
    true,
    options,
  );
  return { encoded, paramCount: nonAccountParamDefs.length };
}

function estimateComputeUnits(
  functionIndex: number,
  parameterCount: number,
): number {
  return Math.max(5000, 1000 + parameterCount * 500 + functionIndex * 100);
}
