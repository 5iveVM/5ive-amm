import { PDAUtils, Base58Utils } from "../crypto/index.js";
import { ProgramIdResolver } from "../config/ProgramIdResolver.js";

export async function getVMState(connection: any, fiveVMProgramId?: string): Promise<{
  authority: string;
  feeRecipient: string;
  scriptCount: number;
  deployFeeBps: number;
  executeFeeBps: number;
  isInitialized: boolean;
}> {
  const programId = ProgramIdResolver.resolve(fiveVMProgramId);
  const vmStatePDA = await PDAUtils.deriveVMStatePDA(programId);

  let accountData: Uint8Array;
  try {
    if (typeof connection.getAccountInfo === 'function') {
      let pubkey: any = vmStatePDA.address;
      try {
        const { PublicKey } = await import("@solana/web3.js");
        pubkey = new PublicKey(vmStatePDA.address);
      } catch { }

      const info = await connection.getAccountInfo(pubkey);
      if (!info) throw new Error("VM State account not found");
      accountData = new Uint8Array(info.data);
    } else if (typeof connection.getAccountData === 'function') {
      const info = await connection.getAccountData(vmStatePDA.address);
      if (!info) throw new Error("VM State account not found");
      accountData = new Uint8Array(info.data);
    } else {
      throw new Error("Invalid connection object: must support getAccountInfo or getAccountData");
    }

    if (accountData.length < 88) throw new Error(`VM State account data too small: expected 88, got ${accountData.length}`);

    const authority = Base58Utils.encode(accountData.slice(0, 32));
    const feeRecipient = Base58Utils.encode(accountData.slice(32, 64));
    const view = new DataView(accountData.buffer, accountData.byteOffset, accountData.byteLength);

    return {
      authority,
      feeRecipient,
      scriptCount: Number(view.getBigUint64(64, true)),
      deployFeeBps: view.getUint32(72, true),
      executeFeeBps: view.getUint32(76, true),
      isInitialized: accountData[80] === 1
    };
  } catch (error) {
    throw new Error(`Failed to fetch VM state: ${error instanceof Error ? error.message : "Unknown error"}`);
  }
}
