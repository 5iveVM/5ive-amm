const DEFAULT_TIMEOUT_MS = 120000;
const POLL_INTERVAL_MS = 1000;

function normalizeCommitment(commitment) {
  if (commitment === 'finalized') return 'finalized';
  if (commitment === 'processed') return 'processed';
  return 'confirmed';
}

function meetsCommitment(status, confirmations, target) {
  if (target === 'processed') {
    return Boolean(status) || (confirmations ?? 0) >= 0;
  }
  if (target === 'confirmed') {
    return status === 'confirmed' || status === 'finalized' || (confirmations ?? 0) >= 1;
  }
  return status === 'finalized';
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

export async function confirmSignature(connection, {
  signature,
  commitment = 'confirmed',
  timeoutMs = DEFAULT_TIMEOUT_MS,
  blockhash,
  lastValidBlockHeight,
  debug = false,
}) {
  const targetCommitment = normalizeCommitment(commitment);

  if (blockhash && typeof lastValidBlockHeight === 'number') {
    try {
      const confirmation = await connection.confirmTransaction(
        { signature, blockhash, lastValidBlockHeight },
        commitment,
      );
      const err = confirmation?.value?.err;
      if (err) {
        return { success: false, err, error: JSON.stringify(err) };
      }
      if (confirmation?.value) {
        return { success: true };
      }
    } catch (error) {
      if (debug) {
        console.log(
          `[solana-confirm] confirmTransaction fallback for ${signature}: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }
  }

  const startedAt = Date.now();
  while (Date.now() - startedAt < timeoutMs) {
    try {
      const statuses = await connection.getSignatureStatuses([signature], { searchTransactionHistory: true });
      const status = statuses?.value?.[0];
      if (status) {
        if (status.err) {
          return { success: false, err: status.err, error: JSON.stringify(status.err) };
        }
        if (meetsCommitment(status.confirmationStatus, status.confirmations, targetCommitment)) {
          return { success: true };
        }
      }
    } catch (error) {
      if (debug) {
        console.log(
          `[solana-confirm] polling error for ${signature}: ${error instanceof Error ? error.message : String(error)}`,
        );
      }
    }
    await sleep(POLL_INTERVAL_MS);
  }

  return {
    success: false,
    error: `Transaction confirmation timeout after ${timeoutMs}ms. Signature: ${signature}`,
  };
}
