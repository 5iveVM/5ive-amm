import { describe, it, expect } from "@jest/globals";
import { Keypair } from "@solana/web3.js";
import { deployLargeProgramToSolana } from "../../modules/deploy.js";

describe("deploy commitment resilience", () => {
  it("uses finalized confirmations and finalized account reads in chunked deploy flow", async () => {
    const bytecode = new Uint8Array(900).fill(1); // Force chunked path (> 800 bytes)
    const deployer = Keypair.generate();

    let sigCounter = 0;
    let accountInfoCall = 0;
    const getAccountInfoCalls: Array<{ pubkey: any; commitment?: string }> = [];
    const getSignatureStatusesCalls: Array<{ signatures: string[] }> = [];

    const vmStateData = Buffer.alloc(56);
    vmStateData[50] = 1; // shardCount = 1

    const connection = {
      async getMinimumBalanceForRentExemption(size: number) {
        return 1_000_000 + size;
      },
      async getLatestBlockhash(_commitment: string) {
        return { blockhash: `bh-${sigCounter}`, lastValidBlockHeight: 99999 };
      },
      async sendRawTransaction(_payload: Buffer, _opts: any) {
        sigCounter += 1;
        return `sig-${sigCounter}`;
      },
      async confirmTransaction(signature: string, commitment: string) {
        return { value: { err: null } };
      },
      async getSignatureStatus(_signature: string) {
        return { value: { confirmationStatus: "confirmed", confirmations: 1, err: null } };
      },
      async getSignatureStatuses(_signatures: string[]) {
        getSignatureStatusesCalls.push({ signatures: _signatures });
        return { value: [{ confirmationStatus: "finalized", confirmations: null, err: null }] };
      },
      async getAccountInfo(pubkey: any, commitment?: string) {
        getAccountInfoCalls.push({ pubkey, commitment });
        accountInfoCall += 1;

        // ensureCanonicalVmStateAccount: first call should report missing account.
        if (accountInfoCall === 1) return null;
        // readVMStateFeeConfig: next call should provide fee config data.
        if (accountInfoCall === 2) return { data: vmStateData, lamports: 1_000_000 };
        // Per-chunk rent sizing read.
        if (accountInfoCall === 3) return { data: Buffer.alloc(64), lamports: 1_000_000 };
        // Final verification read.
        return { data: Buffer.alloc(64 + bytecode.length), lamports: 1_000_000 };
      },
    };

    const result = await deployLargeProgramToSolana(bytecode, connection, deployer, {
      chunkSize: 500,
      debug: false,
      forceChunkedSmallProgram: true,
    });

    expect(result.success).toBe(true);
    expect(getSignatureStatusesCalls.length).toBeGreaterThan(0);

    const commitmentReads = getAccountInfoCalls
      .map((c) => c.commitment)
      .filter((c): c is string => typeof c === "string");
    expect(commitmentReads.length).toBeGreaterThan(0);
    for (const commitment of commitmentReads) {
      expect(commitment).toBe("finalized");
    }
  });
});
