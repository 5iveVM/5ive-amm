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
  const pollIntervalMs = 1000;

  if (debug) {
    console.log(`[FiveSDK] Starting confirmation poll with ${timeoutMs}ms timeout`);
  }

  while (Date.now() - startTime < timeoutMs) {
    try {
      const confirmationStatus = await connection.getSignatureStatus(signature);

      if (debug && (Date.now() - startTime) % 10000 < 1000) {
        console.log(`[FiveSDK] Confirmation status: ${JSON.stringify(confirmationStatus.value)}`);
      }

      if (confirmationStatus.value) {
        if (confirmationStatus.value.confirmationStatus === commitment ||
          confirmationStatus.value.confirmations >= 1) {
          const transactionError = confirmationStatus.value.err;
          const succeeded = !transactionError;

          if (debug) {
            console.log(
              `[FiveSDK] Transaction confirmed after ${Date.now() - startTime}ms${succeeded ? '' : ' (with error)'}`
            );
            if (transactionError) {
              console.log(`[FiveSDK] Transaction error: ${JSON.stringify(transactionError)}`);
            }
          }

          return {
            success: succeeded,
            err: transactionError,
            error: transactionError ? JSON.stringify(transactionError) : undefined,
          };
        }
      }

      await new Promise(resolve => setTimeout(resolve, pollIntervalMs));
    } catch (error) {
      if (debug) {
        console.log(`[FiveSDK] Polling error: ${error instanceof Error ? error.message : String(error)}`);
      }
      await new Promise(resolve => setTimeout(resolve, pollIntervalMs));
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
