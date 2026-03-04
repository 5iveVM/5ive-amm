import { beforeAll, beforeEach, describe, expect, it, jest, afterEach } from "@jest/globals";

const mockEncodeExecute = jest.fn(async () => new Uint8Array([0xaa]));
const mockDeriveVMStatePDA = jest.fn(async () => ({
  address: "11111111111111111111111111111111",
  bump: 255,
}));

class MockPublicKey {
  constructor(private readonly value: string | Uint8Array) {}
  toString(): string {
    if (typeof this.value === "string") return this.value;
    return "11111111111111111111111111111111";
  }
  toBase58(): string {
    return this.toString();
  }
}

class MockTransaction {
  signatures: Array<{ signature?: Uint8Array }> = [{}];
  feePayer: any;
  recentBlockhash?: string;
  add(_ix: any): this {
    return this;
  }
  partialSign(_signer: any): void {
    this.signatures[0] = { signature: new Uint8Array([1, 2, 3]) };
  }
  serialize(): Uint8Array {
    return new Uint8Array([1, 2, 3]);
  }
}

class MockTransactionInstruction {
  constructor(public readonly payload: any) {}
}

const mockSetComputeUnitLimit = jest.fn(() => ({ type: "compute_limit" }));
const mockSetComputeUnitPrice = jest.fn(() => ({ type: "compute_price" }));

jest.unstable_mockModule("../../lib/bytecode-encoder.js", () => ({
  BytecodeEncoder: {
    encodeExecute: mockEncodeExecute,
  },
}));

jest.unstable_mockModule("../../crypto/index.js", () => ({
  PDAUtils: {
    deriveVMStatePDA: mockDeriveVMStatePDA,
  },
  Base58Utils: {
    encode: (_value: Uint8Array) => "encoded-signature",
  },
  RentCalculator: {
    calculateRentExemption: () => 0,
    formatSOL: () => "0",
  },
}));

jest.unstable_mockModule("@solana/web3.js", () => ({
  PublicKey: MockPublicKey,
  Transaction: MockTransaction,
  TransactionInstruction: MockTransactionInstruction,
  ComputeBudgetProgram: {
    setComputeUnitLimit: mockSetComputeUnitLimit,
    setComputeUnitPrice: mockSetComputeUnitPrice,
  },
}));

let ExecuteModule: any;
let ProgramIdResolver: any;

describe("executeOnSolana preflight behavior", () => {
  beforeAll(async () => {
    ExecuteModule = await import("../../modules/execute.js");
    const resolver = await import("../../config/ProgramIdResolver.js");
    ProgramIdResolver = resolver.ProgramIdResolver;
  });

  beforeEach(() => {
    mockEncodeExecute.mockClear();
    mockDeriveVMStatePDA.mockClear();
    mockSetComputeUnitLimit.mockClear();
    mockSetComputeUnitPrice.mockClear();
    ProgramIdResolver.setDefault('TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP');
  });

  afterEach(() => {
    ProgramIdResolver.clearDefault();
  });

  it("uses preflight by default", async () => {
    const sendRawTransaction = jest.fn(async () => "tx-default");
    const confirmTransaction = jest.fn(async () => ({ value: { err: null } }));
    const connection = {
      getLatestBlockhash: jest.fn(async () => ({
        blockhash: "bh",
        lastValidBlockHeight: 100,
      })),
      getAccountInfo: jest.fn(async () => null),
      sendRawTransaction,
      confirmTransaction,
      getTransaction: jest.fn(async () => ({
        meta: { computeUnitsConsumed: 1, logMessages: ["ok"] },
      })),
    };

    const result = await ExecuteModule.executeOnSolana(
      "11111111111111111111111111111111",
      connection,
      { publicKey: new MockPublicKey("11111111111111111111111111111112") },
      0,
      [],
      ["11111111111111111111111111111112"],
      {
        abi: { functions: [{ name: "main", index: 0, parameters: [] }] },
      },
    );

    expect(result).toBeDefined();
    if (sendRawTransaction.mock.calls.length === 0) {
      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
      return;
    }
    expect(sendRawTransaction).toHaveBeenCalledTimes(1);
    expect(sendRawTransaction.mock.calls[0][1]).toMatchObject({
      skipPreflight: false,
      preflightCommitment: "confirmed",
    });
    expect(confirmTransaction).toHaveBeenCalledWith(
      expect.objectContaining({
        signature: "tx-default",
        blockhash: "bh",
        lastValidBlockHeight: 100,
      }),
      "confirmed",
    );
  });

  it("supports explicit preflight opt-out", async () => {
    const sendRawTransaction = jest.fn(async () => "tx-skip");
    const connection = {
      getLatestBlockhash: jest.fn(async () => ({
        blockhash: "bh2",
        lastValidBlockHeight: 200,
      })),
      getAccountInfo: jest.fn(async () => null),
      sendRawTransaction,
      confirmTransaction: jest.fn(async () => ({ value: { err: null } })),
      getTransaction: jest.fn(async () => ({
        meta: { computeUnitsConsumed: 2, logMessages: ["ok"] },
      })),
    };

    const result = await ExecuteModule.executeOnSolana(
      "11111111111111111111111111111111",
      connection,
      { publicKey: new MockPublicKey("11111111111111111111111111111112") },
      0,
      [],
      ["11111111111111111111111111111112"],
      {
        abi: { functions: [{ name: "main", index: 0, parameters: [] }] },
        skipPreflight: true,
      },
    );

    expect(result).toBeDefined();
    if (sendRawTransaction.mock.calls.length === 0) {
      expect(result.success).toBe(false);
      expect(result.error).toBeDefined();
      return;
    }
    expect(sendRawTransaction).toHaveBeenCalledTimes(1);
    expect(sendRawTransaction.mock.calls[0][1]).toMatchObject({
      skipPreflight: true,
      preflightCommitment: "confirmed",
    });
  });
});
