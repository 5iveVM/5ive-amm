type CommitmentLevel = "processed" | "confirmed" | "finalized";

export const SDK_COMMITMENTS = {
  WRITE: "confirmed",
  READ: "finalized",
  CONFIRM: "finalized",
} as const;

const DEFAULT_POLL_INTERVAL_MS = 1000;
const DEFAULT_RETRY_DELAY_MS = 700;

async function sleep(ms: number): Promise<void> {
  await new Promise(resolve => setTimeout(resolve, ms));
}

function backoffDelayMs(baseMs: number, attempt: number): number {
  const exp = Math.min(attempt, 6);
  const raw = baseMs * (2 ** exp);
  const jitter = Math.floor(raw * (0.15 * Math.random()));
  return raw + jitter;
}

function normalizeCommitment(commitment: string): CommitmentLevel {
  if (commitment === "finalized") return "finalized";
  if (commitment === "processed") return "processed";
  return "confirmed";
}

function statusMeetsCommitment(
  status: string | null | undefined,
  confirmations: number | null | undefined,
  target: CommitmentLevel,
): boolean {
  if (target === "processed") {
    return !!status || (confirmations ?? 0) >= 0;
  }
  if (target === "confirmed") {
    return status === "confirmed" || status === "finalized" || (confirmations ?? 0) >= 1;
  }
  // finalized must be explicitly finalized; confirmations count is insufficient.
  return status === "finalized";
}

export async function pollForConfirmation(
  connection: any,
  signature: string,
  commitment: string = "confirmed",
  timeoutMs: number = 120000,
  debug: boolean = false
): Promise<{
  success: boolean;
  err?: any;
  error?: string;
}> {
  const startTime = Date.now();
  const pollIntervalMs = DEFAULT_POLL_INTERVAL_MS;
  const targetCommitment = normalizeCommitment(commitment);

  if (debug) {
    console.log(`[FiveSDK] Starting confirmation poll with ${timeoutMs}ms timeout`);
  }

  while (Date.now() - startTime < timeoutMs) {
    try {
      const confirmationStatus = await connection.getSignatureStatuses(
        [signature],
        { searchTransactionHistory: true },
      );
      const statusValue = confirmationStatus?.value?.[0];

      if (debug && (Date.now() - startTime) % 10000 < 1000) {
        console.log(`[FiveSDK] Confirmation status: ${JSON.stringify(statusValue)}`);
      }

      if (statusValue) {
        const transactionError = statusValue.err;
        if (transactionError) {
          if (debug) {
            console.log(`[FiveSDK] Transaction error: ${JSON.stringify(transactionError)}`);
          }
          return {
            success: false,
            err: transactionError,
            error: JSON.stringify(transactionError),
          };
        }

        if (
          statusMeetsCommitment(
            statusValue.confirmationStatus,
            statusValue.confirmations,
            targetCommitment,
          )
        ) {
          const succeeded = true;

          if (debug) {
            console.log(
              `[FiveSDK] Transaction confirmed after ${Date.now() - startTime}ms${succeeded ? '' : ' (with error)'}`
            );
          }

          return {
            success: succeeded,
            err: undefined,
            error: undefined,
          };
        }
      }

      await sleep(pollIntervalMs);
    } catch (error) {
      if (debug) {
        console.log(`[FiveSDK] Polling error: ${error instanceof Error ? error.message : String(error)}`);
      }
      await sleep(pollIntervalMs);
    }
  }

  const elapsed = Date.now() - startTime;
  if (debug) {
    console.log(`[FiveSDK] Confirmation polling timeout after ${elapsed}ms`);
  }

  return {
    success: false,
    error: `Transaction confirmation timeout after ${elapsed}ms. Signature: ${signature}`,
  };
}

export async function confirmTransactionRobust(
  connection: any,
  signature: string,
  options: {
    commitment?: string;
    timeoutMs?: number;
    debug?: boolean;
    blockhash?: string;
    lastValidBlockHeight?: number;
  } = {},
): Promise<{ success: boolean; err?: any; error?: string }> {
  const commitment = options.commitment || SDK_COMMITMENTS.CONFIRM;
  const timeoutMs = options.timeoutMs || 120000;
  const debug = options.debug || false;

  try {
    const confirmArg =
      options.blockhash && typeof options.lastValidBlockHeight === "number"
        ? {
            signature,
            blockhash: options.blockhash,
            lastValidBlockHeight: options.lastValidBlockHeight,
          }
        : signature;

    const confirmation = await connection.confirmTransaction(confirmArg, commitment);
    const err = confirmation?.value?.err;
    if (!err) {
      return { success: true };
    }
    return { success: false, err, error: JSON.stringify(err) };
  } catch (error) {
    if (debug) {
      console.log(
        `[FiveSDK] confirmTransaction threw, falling back to polling: ${
          error instanceof Error ? error.message : String(error)
        }`,
      );
    }
  }

  return pollForConfirmation(connection, signature, commitment, timeoutMs, debug);
}

export async function sendAndConfirmRawTransactionRobust(
  connection: any,
  serializedTx: Uint8Array | Buffer,
  options: {
    commitment?: string;
    timeoutMs?: number;
    debug?: boolean;
    maxRetries?: number;
    skipPreflight?: boolean;
    preflightCommitment?: string;
    resendIntervalMs?: number;
    blockhash?: string;
    lastValidBlockHeight?: number;
  } = {},
): Promise<{ success: boolean; signature?: string; err?: any; error?: string }> {
  const commitment = options.commitment || SDK_COMMITMENTS.CONFIRM;
  const timeoutMs = options.timeoutMs || 120000;
  const debug = options.debug || false;
  const resendIntervalMs = options.resendIntervalMs || 1500;
  const targetCommitment = normalizeCommitment(commitment);
  const skipPreflight = options.skipPreflight ?? true;

  const signature = await connection.sendRawTransaction(serializedTx, {
    skipPreflight,
    preflightCommitment: options.preflightCommitment || "confirmed",
    maxRetries: options.maxRetries || 3,
  });

  const startTime = Date.now();
  let lastResendAt = startTime;

  while (Date.now() - startTime < timeoutMs) {
    try {
      const statuses = await connection.getSignatureStatuses(
        [signature],
        { searchTransactionHistory: true },
      );
      const status = statuses?.value?.[0];

      if (status?.err) {
        return { success: false, signature, err: status.err, error: JSON.stringify(status.err) };
      }

      if (
        status &&
        statusMeetsCommitment(
          status.confirmationStatus,
          status.confirmations,
          targetCommitment,
        )
      ) {
        return { success: true, signature };
      }

      if (Date.now() - lastResendAt >= resendIntervalMs) {
        try {
          await connection.sendRawTransaction(serializedTx, {
            skipPreflight,
            preflightCommitment: options.preflightCommitment || "confirmed",
            maxRetries: 0,
          });
          if (debug) {
            console.log(`[FiveSDK] Rebroadcasting transaction ${signature}`);
          }
        } catch (error) {
          const message = error instanceof Error ? error.message : String(error);
          if (
            debug &&
            !/already processed|already received|blockhash not found|too old|duplicate/i.test(message)
          ) {
            console.log(`[FiveSDK] Rebroadcast error: ${message}`);
          }
        }
        lastResendAt = Date.now();
      }
    } catch (error) {
      if (debug) {
        console.log(`[FiveSDK] send/confirm polling error: ${error instanceof Error ? error.message : String(error)}`);
      }
    }

    if (
      typeof options.lastValidBlockHeight === "number" &&
      typeof connection.getBlockHeight === "function"
    ) {
      try {
        const currentBlockHeight = await connection.getBlockHeight("confirmed");
        if (currentBlockHeight > options.lastValidBlockHeight) {
          return {
            success: false,
            signature,
            error: `Transaction expired before confirmation. Signature: ${signature}`,
          };
        }
      } catch {
        // Ignore block height lookup failures and keep polling.
      }
    }

    await sleep(DEFAULT_POLL_INTERVAL_MS);
  }

  return {
    success: false,
    signature,
    error: `Transaction confirmation timeout after ${Date.now() - startTime}ms. Signature: ${signature}`,
  };
}

export async function getAccountInfoWithRetry(
  connection: any,
  pubkey: any,
  options: {
    commitment?: string;
    retries?: number;
    delayMs?: number;
    debug?: boolean;
  } = {},
): Promise<any | null> {
  const commitment = options.commitment || SDK_COMMITMENTS.READ;
  const retries = options.retries ?? 2;
  const delayMs = options.delayMs ?? DEFAULT_RETRY_DELAY_MS;
  const debug = options.debug || false;

  let info = await connection.getAccountInfo(pubkey, commitment);
  for (let attempt = 0; !info && attempt < retries; attempt++) {
    const waitMs = backoffDelayMs(delayMs, attempt);
    if (debug) {
      console.log(
        `[FiveSDK] getAccountInfo retry ${attempt + 1}/${retries}, waiting ${waitMs}ms`,
      );
    }
    await sleep(waitMs);
    info = await connection.getAccountInfo(pubkey, commitment);
  }
  return info;
}
