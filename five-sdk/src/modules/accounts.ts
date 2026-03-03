import { ScriptMetadata } from "../metadata/index.js";
import { ScriptMetadataParser } from "../metadata/index.js";
import { normalizeAbiFunctions } from "../utils/abi.js";
import { loadWasmVM } from "../wasm/instance.js";
import { SolanaPublicKeyUtils } from "../crypto/index.js";
import { getAccountInfoWithRetry } from "../utils/transaction.js";

export async function fetchAccountAndDeserialize(
  accountAddress: string,
  connection: any, // Solana Connection object
  options: {
    debug?: boolean;
    parseMetadata?: boolean; // Parse full script metadata or just raw data
    validateEncoding?: boolean; // Validate encoding format
  } = {},
): Promise<{
  success: boolean;
  accountInfo?: {
    address: string;
    owner: string;
    lamports: number;
    dataLength: number;
  };
  scriptMetadata?: ScriptMetadata;
  rawBytecode?: Uint8Array;
  decodedData?: {
    header: any;
    bytecode: Uint8Array;
    abi?: any;
    functions?: Array<{ name: string; index: number; parameters: any[] }>;
  };
  error?: string;
  logs?: string[];
}> {
  try {
    if (options.debug) {
      console.log(
        `[FiveSDK] Fetching account and deserializing data: ${accountAddress}`,
      );
    }

    if (!SolanaPublicKeyUtils.isValid(accountAddress)) {
      return {
        success: false,
        error: `Invalid account address format: ${accountAddress}`,
        logs: [],
      };
    }

    const { PublicKey } = await import("@solana/web3.js");

    let accountPubkey: any;
    try {
      accountPubkey = new PublicKey(accountAddress);
    } catch (addressError) {
      return {
        success: false,
        error: `Invalid account address format: ${accountAddress}`,
        logs: [],
      };
    }

    const accountInfo = await getAccountInfoWithRetry(connection, accountPubkey, {
      commitment: "finalized",
      retries: 2,
      delayMs: 1000,
      debug: options.debug,
    });

    if (!accountInfo) {
      return {
        success: false,
        error: `Account not found: ${accountAddress}`,
        logs: [],
      };
    }

    if (!accountInfo.data || accountInfo.data.length === 0) {
      return {
        success: false,
        error: `Account has no data: ${accountAddress}`,
        logs: [],
      };
    }

    const logs: string[] = [];

    if (options.debug) {
      console.log(`[FiveSDK] Account fetched successfully:`);
      console.log(`  - Address: ${accountAddress}`);
      console.log(`  - Owner: ${accountInfo.owner.toString()}`);
      console.log(`  - Lamports: ${accountInfo.lamports}`);
      console.log(`  - Data length: ${accountInfo.data.length} bytes`);

      logs.push(`Account fetched: ${accountInfo.data.length} bytes`);
      logs.push(`Owner: ${accountInfo.owner.toString()}`);
      logs.push(`Balance: ${accountInfo.lamports / 1e9} SOL`);
    }

    const result: any = {
      success: true,
      accountInfo: {
        address: accountAddress,
        owner: accountInfo.owner.toString(),
        lamports: accountInfo.lamports,
        dataLength: accountInfo.data.length,
      },
      logs,
    };

    if (options.parseMetadata) {
      try {
        const scriptMetadata = ScriptMetadataParser.parseMetadata(
          accountInfo.data,
          accountAddress,
        );
        result.scriptMetadata = scriptMetadata;
        result.rawBytecode = scriptMetadata.bytecode;

        result.decodedData = {
          header: {
            version: scriptMetadata.version,
            deployedAt: scriptMetadata.deployedAt,
            authority: scriptMetadata.authority,
          },
          bytecode: scriptMetadata.bytecode,
          abi: scriptMetadata.abi,
          functions: normalizeAbiFunctions(
            scriptMetadata.abi?.functions ?? scriptMetadata.abi,
          ).map((func: any) => ({
            name: func.name,
            index: func.index,
            parameters: func.parameters || [],
          })),
        };
        const parsedFunctions = result.decodedData.functions;

        if (options.debug) {
          console.log(`[FiveSDK] Script metadata parsed successfully:`);
          console.log(`  - Script name: ${scriptMetadata.abi.name}`);
          console.log(
            `  - Functions: ${parsedFunctions.length}`,
          );
          console.log(
            `  - Bytecode size: ${scriptMetadata.bytecode.length} bytes`,
          );
          console.log(`  - Authority: ${scriptMetadata.authority}`);

          logs.push(
            `Script metadata parsed: ${parsedFunctions.length} functions`,
          );
          logs.push(`Bytecode: ${scriptMetadata.bytecode.length} bytes`);
        }
      } catch (metadataError) {
        if (options.debug) {
          console.warn(
            `[FiveSDK] Failed to parse script metadata:`,
            metadataError,
          );
        }

        result.rawBytecode = accountInfo.data;
        logs.push(
          "Warning: Failed to parse script metadata, treating as raw data",
        );
      }
    } else {
      result.rawBytecode = accountInfo.data;
      logs.push("Raw account data returned (metadata parsing disabled)");
    }

    if (options.validateEncoding && result.rawBytecode) {
      try {
        const validation = await validateBytecodeEncoding(
          result.rawBytecode,
          options.debug,
        );
        if (validation.valid) {
          logs.push("Encoding validation: PASSED");
          if (options.debug) {
            console.log(
              `[FiveSDK] Validation passed: ${validation.info}`,
            );
          }
        } else {
          logs.push(`Encoding validation: FAILED - ${validation.error}`);
          if (options.debug) {
            console.warn(
              `[FiveSDK] Validation failed: ${validation.error}`,
            );
          }
        }
      } catch (validationError) {
        logs.push(
          `Validation error: ${validationError instanceof Error ? validationError.message : "Unknown error"}`,
        );
      }
    }

    return result;
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : "Unknown account fetch error";

    if (options.debug) {
      console.error(
        `[FiveSDK] Account fetch and deserialization failed: ${errorMessage}`,
      );
    }

    return {
      success: false,
      error: errorMessage,
      logs: [],
    };
  }
}

export async function fetchMultipleAccountsAndDeserialize(
  accountAddresses: string[],
  connection: any,
  options: {
    debug?: boolean;
    parseMetadata?: boolean; // Parse full script metadata or just raw data
    validateEncoding?: boolean; // Validate encoding format
    batchSize?: number; // Solana RPC batch limit
  } = {},
): Promise<
  Map<
    string,
    {
      success: boolean;
      accountInfo?: any;
      scriptMetadata?: ScriptMetadata;
      rawBytecode?: Uint8Array;
      decodedData?: any;
      error?: string;
      logs?: string[];
    }
  >
> {
  const batchSize = options.batchSize || 100;
  const results = new Map();

  if (options.debug) {
    console.log(
      `[FiveSDK] Batch fetching ${accountAddresses.length} accounts (batch size: ${batchSize})`,
    );
  }

  for (let i = 0; i < accountAddresses.length; i += batchSize) {
    const batch = accountAddresses.slice(i, i + batchSize);

    if (options.debug) {
      console.log(
        `[FiveSDK] Processing batch ${Math.floor(i / batchSize) + 1}/${Math.ceil(accountAddresses.length / batchSize)}`,
      );
    }

    const batchPromises = batch.map((address) =>
      fetchAccountAndDeserialize(address, connection, {
        debug: false,
        parseMetadata: options.parseMetadata,
        validateEncoding: options.validateEncoding,
      }),
    );

    const batchResults = await Promise.allSettled(batchPromises);

    batch.forEach((address, index) => {
      const batchResult = batchResults[index];
      if (batchResult.status === "fulfilled") {
        results.set(address, batchResult.value);
      } else {
        results.set(address, {
          success: false,
          error: `Batch processing failed: ${batchResult.reason}`,
          logs: [],
        });
      }
    });
  }

  if (options.debug) {
    const successful = Array.from(results.values()).filter(
      (r) => r.success,
    ).length;
    console.log(
      `[FiveSDK] Batch processing completed: ${successful}/${accountAddresses.length} successful`,
    );
  }

  return results;
}

export async function deserializeParameters(
  instructionData: Uint8Array,
  expectedTypes: string[] = [],
  options: { debug?: boolean } = {},
): Promise<{
  success: boolean;
  parameters?: Array<{ type: string; value: any }>;
  functionIndex?: number;
  discriminator?: number;
  error?: string;
}> {
  try {
    if (options.debug) {
      console.log(
        `[FiveSDK] Deserializing parameters from ${instructionData.length} bytes:`,
      );
      console.log(
        `[FiveSDK] Instruction data (hex):`,
        Buffer.from(instructionData).toString("hex"),
      );
      console.log(`[FiveSDK] Expected parameter types:`, expectedTypes);
    }

    const wasmVM = await loadWasmVM();

    try {
      const wasmModule = await import(
        "../assets/vm/five_vm_wasm.js" as string
      );

      if (options.debug) {
        console.log(`[FiveSDK] Using WASM ParameterEncoder for decoding`);
      }

      const decodedResult =
        wasmModule.ParameterEncoder.decode_instruction_data(instructionData);

      if (options.debug) {
        console.log(`[FiveSDK] Decoding result:`, decodedResult);
      }

      const parameters: Array<{ type: string; value: any }> = [];

      if (decodedResult && decodedResult.parameters) {
        decodedResult.parameters.forEach((param: any, index: number) => {
          parameters.push({
            type: expectedTypes[index] || "unknown",
            value: param,
          });
        });
      }

      return {
        success: true,
        parameters,
        functionIndex: decodedResult.function_index,
        discriminator: decodedResult.discriminator,
      };
    } catch (wasmError) {
      if (options.debug) {
        console.warn(
          `[FiveSDK] WASM decoding failed:`,
          wasmError,
        );
      }
      throw wasmError;
    }
  } catch (error) {
    const errorMessage =
      error instanceof Error
        ? error.message
        : "Unknown deserialization error";

    if (options.debug) {
      console.error(
        `[FiveSDK] Parameter deserialization failed: ${errorMessage}`,
      );
    }

    return {
      success: false,
      error: errorMessage,
    };
  }
}

export async function validateBytecodeEncoding(
  bytecode: Uint8Array,
  debug: boolean = false,
): Promise<{ valid: boolean; error?: string; info?: string }> {
  try {
    if (bytecode.length < 10) {
      return { valid: false, error: "Bytecode too short for Five VM format (min 10 bytes)" };
    }

    const magicBytes = bytecode.slice(0, 4);
    const expectedMagic = new Uint8Array([0x35, 0x49, 0x56, 0x45]); // "5IVE"

    let isValidHeader = true;
    for (let i = 0; i < 4; i++) {
      if (magicBytes[i] !== expectedMagic[i]) {
        isValidHeader = false;
        break;
      }
    }

    if (!isValidHeader) {
      return {
        valid: false,
        error: 'Invalid Five VM magic bytes (expected "5IVE")',
      };
    }

    // ScriptBytecodeHeaderV1:
    // 0-3: Magic
    // 4-7: Features (u32 LE)
    // 8: Public Function Count (u8)
    // 9: Total Function Count (u8)

    // Read features as u32 LE
    const features = bytecode[4] | (bytecode[5] << 8) | (bytecode[6] << 16) | (bytecode[7] << 24);

    const publicFunctionCount = bytecode[8];
    const totalFunctionCount = bytecode[9];

    if (debug) {
      console.log(
        `[FiveSDK] Validation - Magic: "5IVE", Features: ${features}, PublicFuncs: ${publicFunctionCount}, TotalFuncs: ${totalFunctionCount}`,
      );
    }

    return {
      valid: true,
      info: `Valid Five VM bytecode with ${publicFunctionCount}/${totalFunctionCount} functions (features: ${features})`,
    };
  } catch (error) {
    return {
      valid: false,
      error: error instanceof Error ? error.message : "Validation error",
    };
  }
}
