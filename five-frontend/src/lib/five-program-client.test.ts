import { buildExecuteInstruction, createFiveProgram, executeFunction } from "./five-program-client";

const mockInstruction = {
  programId: "Program111111111111111111111111111111111",
  keys: [
    {
      pubkey: "Account111111111111111111111111111111111",
      isSigner: false,
      isWritable: true,
    },
  ],
  data: Buffer.from([1, 2, 3]).toString("base64"),
};

const mockFromABI = jest.fn(() => ({
  function: () => ({
    accounts: () => ({
      args: () => ({
        instruction: async () => mockInstruction,
      }),
    }),
  }),
}));

const connectionCalls = {
  getLatestBlockhash: jest.fn(async () => ({ blockhash: "latest-bh" })),
  sendRawTransaction: jest.fn(async () => "sig-123"),
  confirmTransaction: jest.fn(async () => ({ value: { err: null } })),
};

jest.mock("five-sdk", () => ({
  FiveProgram: {
    fromABI: (...args: any[]) => mockFromABI(...args),
  },
}));

jest.mock("@solana/web3.js", () => {
  class PublicKey {
    constructor(private readonly value: string) {}
    toString() {
      return this.value;
    }
  }

  class TransactionInstruction {
    keys: any[];
    programId: PublicKey;
    data: Buffer;
    constructor(config: any) {
      this.keys = config.keys;
      this.programId = config.programId;
      this.data = config.data;
    }
  }

  class Transaction {
    instructions: any[] = [];
    feePayer: PublicKey | undefined;
    recentBlockhash: string | undefined;
    add(ix: any) {
      this.instructions.push(ix);
      return this;
    }
    serialize() {
      return Buffer.from([9, 9, 9]);
    }
  }

  class Connection {
    constructor(_url: string, _commitment: string) {}
    getLatestBlockhash = connectionCalls.getLatestBlockhash;
    sendRawTransaction = connectionCalls.sendRawTransaction;
    confirmTransaction = connectionCalls.confirmTransaction;
  }

  return { PublicKey, TransactionInstruction, Transaction, Connection };
});

describe("five-program-client boundary", () => {
  beforeEach(() => {
    mockFromABI.mockClear();
    connectionCalls.getLatestBlockhash.mockClear();
    connectionCalls.sendRawTransaction.mockClear();
    connectionCalls.confirmTransaction.mockClear();
  });

  it("builds instruction through FiveProgram fluent API", async () => {
    const result = await buildExecuteInstruction({
      network: "localnet",
      scriptAccount: "Script1111111111111111111111111111111111",
      abi: {
        name: "test",
        functions: [{ name: "run", index: 0, parameters: [] }],
      },
      functionName: "run",
      accounts: {},
      args: {},
    });

    expect(mockFromABI).toHaveBeenCalledTimes(1);
    expect(result.instruction.data).toEqual(Buffer.from([1, 2, 3]));
    expect(result.accounts).toHaveLength(1);
  });

  it("executes via wallet signTransaction + RPC submit path", async () => {
    const wallet = {
      publicKey: { toString: () => "Wallet1111111111111111111111111111111111" } as any,
      signTransaction: jest.fn(async (tx: any) => tx),
    };

    const result = await executeFunction({
      network: "localnet",
      scriptAccount: "Script1111111111111111111111111111111111",
      abi: {
        name: "test",
        functions: [{ name: "run", index: 0, parameters: [] }],
      },
      functionName: "run",
      accounts: {},
      args: {},
      wallet,
    });

    expect(result.success).toBe(true);
    expect(result.signature).toBe("sig-123");
    expect(wallet.signTransaction).toHaveBeenCalledTimes(1);
    expect(connectionCalls.getLatestBlockhash).toHaveBeenCalledTimes(1);
    expect(connectionCalls.sendRawTransaction).toHaveBeenCalledTimes(1);
    expect(connectionCalls.sendRawTransaction.mock.calls[0][1]).toEqual(
      expect.objectContaining({ skipPreflight: true }),
    );
    expect(connectionCalls.confirmTransaction).toHaveBeenCalledWith("sig-123", "confirmed");
  });

  it("creates FiveProgram with network-specific program id", () => {
    const abi = { name: "test", functions: [] };

    createFiveProgram(
      "Script1111111111111111111111111111111111",
      abi as any,
      "devnet",
    );

    expect(mockFromABI).toHaveBeenCalledWith(
      "Script1111111111111111111111111111111111",
      abi,
      expect.objectContaining({
        fiveVMProgramId: "2DXiYbzfSMwkDSxc9aWEaW7XgJjkNzGdADfRN4FbxMNN",
      }),
    );
  });
});
