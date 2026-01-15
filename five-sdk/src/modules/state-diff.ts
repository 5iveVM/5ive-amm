import { ScriptMetadata } from "../metadata/index.js";
import { executeOnSolana } from "./execute.js";
import { fetchMultipleAccountsAndDeserializeVLE } from "./accounts.js";
import { PDAUtils } from "../crypto/index.js";
import { FIVE_VM_PROGRAM_ID } from "../types.js";

export async function executeWithStateDiff(
  scriptAccount: string,
  connection: any,
  signerKeypair: any,
  functionName: string | number,
  parameters: any[] = [],
  options: {
    debug?: boolean;
    network?: string;
    computeUnitLimit?: number;
    trackGlobalFields?: boolean;
    additionalAccounts?: string[];
    includeVMState?: boolean;
  } = {},
): Promise<{
  success: boolean;
  execution?: {
    transactionId?: string;
    result?: any;
    computeUnitsUsed?: number;
    logs?: string[];
  };
  stateDiff?: {
    beforeState: Map<string, any>;
    afterState: Map<string, any>;
    changes: Array<{
      account: string;
      fieldName?: string;
      oldValue: any;
      newValue: any;
      changeType: "created" | "modified" | "deleted";
    }>;
    globalFieldChanges?: Array<{
      fieldName: string;
      oldValue: any;
      newValue: any;
    }>;
  };
  error?: string;
  logs?: string[];
}> {
  const logs: string[] = [];

  try {
    if (options.debug) {
      console.log(`[FiveSDK] Starting execution with state diff tracking`);
      console.log(`  Script Account: ${scriptAccount}`);
      console.log(`  Function: ${functionName}`);
      console.log(`  Parameters: ${JSON.stringify(parameters)}`);
      console.log(`  Track Global Fields: ${options.trackGlobalFields}`);
    }

    const accountsToTrack = [scriptAccount];

    if (options.includeVMState) {
      const vmStatePDAResult = await PDAUtils.deriveVMStatePDA(FIVE_VM_PROGRAM_ID);
      const vmStatePDA = vmStatePDAResult.address;
      accountsToTrack.push(vmStatePDA);
      if (options.debug) {
        console.log(`  Added VM State PDA to tracking: ${vmStatePDA}`);
      }
    }

    if (options.additionalAccounts) {
      accountsToTrack.push(...options.additionalAccounts);
      if (options.debug) {
        console.log(
          `  Added ${options.additionalAccounts.length} additional accounts to tracking`,
        );
      }
    }

    logs.push(
      `Tracking ${accountsToTrack.length} accounts for state changes`,
    );

    if (options.debug) {
      console.log(
        `[FiveSDK] Fetching BEFORE state for ${accountsToTrack.length} accounts...`,
      );
    }

    const beforeState = await fetchMultipleAccountsAndDeserializeVLE(
      accountsToTrack,
      connection,
      {
        debug: false,
        parseMetadata: true,
        validateVLE: false,
      },
    );

    let successfulBeforeFetches = 0;
    for (const [address, result] of beforeState.entries()) {
      if (result.success) {
        successfulBeforeFetches++;
      } else if (options.debug) {
        console.warn(
          `[FiveSDK] Warning: Failed to fetch BEFORE state for ${address}: ${result.error}`,
        );
      }
    }

    logs.push(
      `BEFORE state: ${successfulBeforeFetches}/${accountsToTrack.length} accounts fetched`,
    );

    let beforeGlobalFields: Record<string, any> = {};
    if (options.trackGlobalFields) {
      const scriptBefore = beforeState.get(scriptAccount);
      if (scriptBefore?.success && scriptBefore.scriptMetadata) {
        beforeGlobalFields = extractGlobalFields(
          scriptBefore.scriptMetadata,
          "before",
        );
        if (options.debug) {
          console.log(
            `[FiveSDK] Extracted ${Object.keys(beforeGlobalFields).length} global fields from BEFORE state`,
          );
        }
      }
    }

    if (options.debug) {
      console.log(`[FiveSDK] Executing script...`);
    }

    const executionResult = await executeOnSolana(
      scriptAccount,
      connection,
      signerKeypair,
      functionName,
      parameters,
      options.additionalAccounts || [],
      {
        debug: options.debug,
        network: options.network,
        computeUnitLimit: options.computeUnitLimit,
      },
    );

    if (!executionResult.success) {
      logs.push(`Execution failed: ${executionResult.error}`);
      return {
        success: false,
        error: `Script execution failed: ${executionResult.error}`,
        logs,
      };
    }

    logs.push(`Execution successful: ${executionResult.transactionId}`);

    await new Promise((resolve) => setTimeout(resolve, 1000));

    if (options.debug) {
      console.log(`[FiveSDK] Fetching AFTER state...`);
    }

    const afterState = await fetchMultipleAccountsAndDeserializeVLE(
      accountsToTrack,
      connection,
      {
        debug: false,
        parseMetadata: true,
        validateVLE: false,
      },
    );

    let successfulAfterFetches = 0;
    for (const [address, result] of afterState.entries()) {
      if (result.success) {
        successfulAfterFetches++;
      } else if (options.debug) {
        console.warn(
          `[FiveSDK] Warning: Failed to fetch AFTER state for ${address}: ${result.error}`,
        );
      }
    }

    logs.push(
      `AFTER state: ${successfulAfterFetches}/${accountsToTrack.length} accounts fetched`,
    );

    let afterGlobalFields: Record<string, any> = {};
    if (options.trackGlobalFields) {
      const scriptAfter = afterState.get(scriptAccount);
      if (scriptAfter?.success && scriptAfter.scriptMetadata) {
        afterGlobalFields = extractGlobalFields(
          scriptAfter.scriptMetadata,
          "after",
        );
        if (options.debug) {
          console.log(
            `[FiveSDK] Extracted ${Object.keys(afterGlobalFields).length} global fields from AFTER state`,
          );
        }
      }
    }

    if (options.debug) {
      console.log(`[FiveSDK] Computing state differences...`);
    }

    const changes = computeStateDifferences(
      beforeState,
      afterState,
      options.debug,
    );
    let globalFieldChanges: Array<{
      fieldName: string;
      oldValue: any;
      newValue: any;
    }> = [];

    if (options.trackGlobalFields) {
      globalFieldChanges = computeGlobalFieldChanges(
        beforeGlobalFields,
        afterGlobalFields,
      );
      if (options.debug) {
        console.log(
          `[FiveSDK] Found ${globalFieldChanges.length} global field changes`,
        );
      }
    }

    logs.push(
      `State analysis: ${changes.length} account changes, ${globalFieldChanges.length} global field changes`,
    );

    return {
      success: true,
      execution: {
        transactionId: executionResult.transactionId,
        result: executionResult.result,
        computeUnitsUsed: executionResult.computeUnitsUsed,
        logs: executionResult.logs,
      },
      stateDiff: {
        beforeState,
        afterState,
        changes,
        globalFieldChanges,
      },
      logs,
    };
  } catch (error) {
    const errorMessage =
      error instanceof Error ? error.message : "Unknown state tracking error";

    if (options.debug) {
      console.error(`[FiveSDK] State diff execution failed: ${errorMessage}`);
    }

    return {
      success: false,
      error: errorMessage,
      logs,
    };
  }
}

function computeStateDifferences(
  beforeState: Map<string, any>,
  afterState: Map<string, any>,
  debug: boolean = false,
): Array<{
  account: string;
  fieldName?: string;
  oldValue: any;
  newValue: any;
  changeType: "created" | "modified" | "deleted";
}> {
  const changes: Array<{
    account: string;
    fieldName?: string;
    oldValue: any;
    newValue: any;
    changeType: "created" | "modified" | "deleted";
  }> = [];

  const allAccounts = new Set([...beforeState.keys(), ...afterState.keys()]);

  for (const account of allAccounts) {
    const before = beforeState.get(account);
    const after = afterState.get(account);

    if (debug) {
      console.log(
        `[FiveSDK] Analyzing account ${account.substring(0, 8)}...`,
      );
    }

    if (!before?.success && after?.success) {
      changes.push({
        account,
        oldValue: null,
        newValue: {
          lamports: after.accountInfo?.lamports,
          dataLength: after.accountInfo?.dataLength,
          owner: after.accountInfo?.owner,
        },
        changeType: "created",
      });
      continue;
    }

    if (before?.success && !after?.success) {
      changes.push({
        account,
        oldValue: {
          lamports: before.accountInfo?.lamports,
          dataLength: before.accountInfo?.dataLength,
          owner: before.accountInfo?.owner,
        },
        newValue: null,
        changeType: "deleted",
      });
      continue;
    }

    if (before?.success && after?.success) {
      if (before.accountInfo?.lamports !== after.accountInfo?.lamports) {
        changes.push({
          account,
          fieldName: "lamports",
          oldValue: before.accountInfo?.lamports,
          newValue: after.accountInfo?.lamports,
          changeType: "modified",
        });
      }

      if (before.accountInfo?.dataLength !== after.accountInfo?.dataLength) {
        changes.push({
          account,
          fieldName: "dataLength",
          oldValue: before.accountInfo?.dataLength,
          newValue: after.accountInfo?.dataLength,
          changeType: "modified",
        });
      }

      if (before.rawBytecode && after.rawBytecode) {
        if (!bytecodeEqual(before.rawBytecode, after.rawBytecode)) {
          changes.push({
            account,
            fieldName: "bytecode",
            oldValue: `${before.rawBytecode.length} bytes (hash: ${hashBytecode(before.rawBytecode)})`,
            newValue: `${after.rawBytecode.length} bytes (hash: ${hashBytecode(after.rawBytecode)})`,
            changeType: "modified",
          });
        }
      }

      if (before.scriptMetadata && after.scriptMetadata) {
        compareScriptMetadata(
          before.scriptMetadata,
          after.scriptMetadata,
          account,
          changes,
        );
      }
    }
  }

  if (debug) {
    console.log(`[FiveSDK] Found ${changes.length} total state changes`);
  }

  return changes;
}

function extractGlobalFields(
  scriptMetadata: ScriptMetadata,
  phase: "before" | "after",
): Record<string, any> {
  const globalFields: Record<string, any> = {};

  try {
    if (scriptMetadata.bytecode && scriptMetadata.bytecode.length > 6) {
      const stateSection = extractStateSection(scriptMetadata.bytecode);
      if (stateSection) {
        Object.assign(globalFields, stateSection);
      }
    }
  } catch (error) {
    console.warn(
      `[FiveSDK] Failed to extract global fields (${phase}):`,
      error,
    );
  }

  return globalFields;
}

function computeGlobalFieldChanges(
  beforeFields: Record<string, any>,
  afterFields: Record<string, any>,
): Array<{ fieldName: string; oldValue: any; newValue: any }> {
  const changes: Array<{ fieldName: string; oldValue: any; newValue: any }> =
    [];

  const allFields = new Set([
    ...Object.keys(beforeFields),
    ...Object.keys(afterFields),
  ]);

  for (const fieldName of allFields) {
    const oldValue = beforeFields[fieldName];
    const newValue = afterFields[fieldName];

    if (!deepEqual(oldValue, newValue)) {
      changes.push({
        fieldName,
        oldValue,
        newValue,
      });
    }
  }

  return changes;
}

function compareScriptMetadata(
  beforeMetadata: ScriptMetadata,
  afterMetadata: ScriptMetadata,
  account: string,
  changes: Array<{
    account: string;
    fieldName?: string;
    oldValue: any;
    newValue: any;
    changeType: "created" | "modified" | "deleted";
  }>,
): void {
  if (
    beforeMetadata.abi.functions.length !== afterMetadata.abi.functions.length
  ) {
    changes.push({
      account,
      fieldName: "function_count",
      oldValue: beforeMetadata.abi.functions.length,
      newValue: afterMetadata.abi.functions.length,
      changeType: "modified",
    });
  }

  if (beforeMetadata.abi.name !== afterMetadata.abi.name) {
    changes.push({
      account,
      fieldName: "script_name",
      oldValue: beforeMetadata.abi.name,
      newValue: afterMetadata.abi.name,
      changeType: "modified",
    });
  }

  if (beforeMetadata.authority !== afterMetadata.authority) {
    changes.push({
      account,
      fieldName: "authority",
      oldValue: beforeMetadata.authority,
      newValue: afterMetadata.authority,
      changeType: "modified",
    });
  }
}

function extractStateSection(
  bytecode: Uint8Array,
): Record<string, any> | null {
  try {
    if (bytecode.length < 6) return null;

    const stateMarker = new Uint8Array([0xff, 0xfe]);

    for (let i = 6; i < bytecode.length - 1; i++) {
      if (
        bytecode[i] === stateMarker[0] &&
        bytecode[i + 1] === stateMarker[1]
      ) {
        const stateData: Record<string, any> = {};
        return stateData;
      }
    }
  } catch (error) {
    console.warn("[FiveSDK] State section extraction failed:", error);
  }

  return null;
}

function bytecodeEqual(a: Uint8Array, b: Uint8Array): boolean {
  if (a.length !== b.length) return false;
  for (let i = 0; i < a.length; i++) {
    if (a[i] !== b[i]) return false;
  }
  return true;
}

function hashBytecode(bytecode: Uint8Array): string {
  let hash = 0;
  for (let i = 0; i < bytecode.length; i++) {
    hash = ((hash << 5) - hash + bytecode[i]) & 0xffffffff;
  }
  return hash.toString(16);
}

function deepEqual(a: any, b: any): boolean {
  if (a === b) return true;
  if (a == null || b == null) return false;
  if (typeof a !== typeof b) return false;

  if (typeof a === "object") {
    if (Array.isArray(a) !== Array.isArray(b)) return false;

    const keysA = Object.keys(a);
    const keysB = Object.keys(b);

    if (keysA.length !== keysB.length) return false;

    for (const key of keysA) {
      if (!keysB.includes(key)) return false;
      if (!deepEqual(a[key], b[key])) return false;
    }

    return true;
  }

  return false;
}
