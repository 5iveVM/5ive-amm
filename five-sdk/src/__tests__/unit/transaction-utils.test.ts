import { describe, it, expect, jest, afterEach } from "@jest/globals";
import {
  SDK_COMMITMENTS,
  confirmTransactionRobust,
  getAccountInfoWithRetry,
  pollForConfirmation,
} from "../../utils/transaction.js";

describe("transaction utils reliability", () => {
  afterEach(() => {
    jest.useRealTimers();
  });

  it("pollForConfirmation should require explicit finalized for finalized target", async () => {
    jest.useFakeTimers();

    const statuses = [
      {
        value: {
          confirmationStatus: "confirmed",
          confirmations: 10,
          err: null,
        },
      },
      {
        value: {
          confirmationStatus: "finalized",
          confirmations: null,
          err: null,
        },
      },
    ];

    const connection = {
      getSignatureStatus: jest.fn(async () => statuses.shift()!),
    };

    const pending = pollForConfirmation(connection, "sig", "finalized", 10_000, false);

    await Promise.resolve();
    await jest.advanceTimersByTimeAsync(1000);

    const result = await pending;
    expect(result.success).toBe(true);
    expect(connection.getSignatureStatus).toHaveBeenCalledTimes(2);
  });

  it("confirmTransactionRobust should return direct confirmation success", async () => {
    const connection = {
      confirmTransaction: jest.fn(async () => ({ value: { err: null } })),
      getSignatureStatus: jest.fn(),
    };

    const result = await confirmTransactionRobust(connection, "sig", {
      commitment: SDK_COMMITMENTS.CONFIRM,
    });

    expect(result.success).toBe(true);
    expect(connection.confirmTransaction).toHaveBeenCalledTimes(1);
    expect(connection.getSignatureStatus).not.toHaveBeenCalled();
  });

  it("confirmTransactionRobust should fall back to polling when confirm throws", async () => {
    const connection = {
      confirmTransaction: jest.fn(async () => {
        throw new Error("rpc timeout");
      }),
      getSignatureStatus: jest.fn(async () => ({
        value: {
          confirmationStatus: "finalized",
          confirmations: null,
          err: null,
        },
      })),
    };

    const result = await confirmTransactionRobust(connection, "sig", {
      commitment: "finalized",
      timeoutMs: 5_000,
    });

    expect(result.success).toBe(true);
    expect(connection.confirmTransaction).toHaveBeenCalledTimes(1);
    expect(connection.getSignatureStatus).toHaveBeenCalledTimes(1);
  });

  it("getAccountInfoWithRetry should retry null reads and return account", async () => {
    const account = { data: Buffer.from([1, 2, 3]), lamports: 1_000 };
    const connection = {
      getAccountInfo: jest
        .fn()
        .mockResolvedValueOnce(null)
        .mockResolvedValueOnce(null)
        .mockResolvedValueOnce(account),
    };

    const result = await getAccountInfoWithRetry(connection, "pubkey", {
      commitment: "finalized",
      retries: 3,
      delayMs: 0,
    });

    expect(result).toBe(account);
    expect(connection.getAccountInfo).toHaveBeenCalledTimes(3);
    expect(connection.getAccountInfo).toHaveBeenNthCalledWith(1, "pubkey", "finalized");
  });
});

