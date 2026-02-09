import { beforeAll, beforeEach, describe, expect, it, jest } from "@jest/globals";
import { resolve } from "path";
import { pathToFileURL } from "url";
import { __setFromABIImpl } from "five-sdk";
import { __calls as solanaCalls } from "@solana/web3.js";

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

let FrontendBoundary: any;

describe("frontend boundary via five-program-client", () => {
  beforeAll(async () => {
    __setFromABIImpl((...args: any[]) => mockFromABI(...args));
    const frontendClientPath = resolve(
      process.cwd(),
      "../five-frontend/src/lib/five-program-client.ts",
    );
    FrontendBoundary = await import(pathToFileURL(frontendClientPath).href);
  });

  beforeEach(() => {
    mockFromABI.mockClear();
    solanaCalls.getLatestBlockhash.length = 0;
    solanaCalls.sendRawTransaction.length = 0;
    solanaCalls.confirmTransaction.length = 0;
  });

  it("builds execution instruction through FiveProgram fluent API", async () => {
    const result = await FrontendBoundary.buildExecuteInstruction({
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

  it("executes with wallet signTransaction + RPC submit path", async () => {
    const wallet = {
      publicKey: { toString: () => "Wallet1111111111111111111111111111111111" } as any,
      signTransaction: jest.fn(async (tx: any) => tx),
    };

    const result = await FrontendBoundary.executeFunction({
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
    expect(solanaCalls.getLatestBlockhash).toHaveLength(1);
    expect(solanaCalls.sendRawTransaction).toHaveLength(1);
    expect(solanaCalls.sendRawTransaction[0][1]).toEqual(
      expect.objectContaining({ skipPreflight: true }),
    );
    expect(solanaCalls.confirmTransaction).toEqual([["sig-123", "confirmed"]]);
  });
});
