import { afterEach, beforeEach, describe, expect, it } from "@jest/globals";
import bs58 from "bs58";
import { ProgramIdResolver } from "../../config/ProgramIdResolver.js";
import * as Admin from "../../modules/admin.js";

describe("admin instruction builders", () => {
  const programId = "TokenkegQfeZyiNwAJsyFbPVwwQQnmjV7B8B65C7TnP";
  const authority = "11111111111111111111111111111112";
  const payer = "11111111111111111111111111111113";
  const recipient = "11111111111111111111111111111114";
  const script = "11111111111111111111111111111111";

  beforeEach(() => {
    ProgramIdResolver.setDefault(programId);
  });

  afterEach(() => {
    ProgramIdResolver.clearDefault();
  });

  it("builds initialize vm_state instruction with canonical PDA + bump", async () => {
    const built = await Admin.generateInitializeVmStateInstruction(authority, { payer });
    const raw = Buffer.from(built.instruction.data, "base64");

    expect(raw.length).toBe(2);
    expect(raw[0]).toBe(0);
    expect(raw[1]).toBe(built.bump);
    expect(built.instruction.accounts).toHaveLength(4);
    expect(built.instruction.accounts[1].pubkey).toBe(authority);
    expect(built.instruction.accounts[2].pubkey).toBe(payer);
    expect(built.requiredSigners.sort()).toEqual([authority, payer].sort());
  });

  it("builds set_fees instruction data", async () => {
    const built = await Admin.generateSetFeesInstruction(authority, 1234, 5678);
    const raw = Buffer.from(built.instruction.data, "base64");

    expect(raw.length).toBe(9);
    expect(raw[0]).toBe(6);
    expect(raw.readUInt32LE(1)).toBe(1234);
    expect(raw.readUInt32LE(5)).toBe(5678);
    expect(built.instruction.accounts).toHaveLength(2);
    expect(built.instruction.accounts[1]).toMatchObject({
      pubkey: authority,
      isSigner: true,
      isWritable: false,
    });
  });

  it("builds init_fee_vault instruction with canonical vault bump", async () => {
    const built = await Admin.generateInitFeeVaultInstruction(payer, 3);
    const raw = Buffer.from(built.instruction.data, "base64");

    expect(raw.length).toBe(3);
    expect(raw[0]).toBe(11);
    expect(raw[1]).toBe(3);
    expect(raw[2]).toBe(built.bump);
    expect(built.instruction.accounts).toHaveLength(4);
    expect(built.instruction.accounts[2].pubkey).toBe(built.feeVaultAccount);
  });

  it("builds withdraw_script_fees instruction payload", async () => {
    const built = await Admin.generateWithdrawScriptFeesInstruction(
      authority,
      recipient,
      script,
      0,
      42,
    );
    const raw = Buffer.from(built.instruction.data, "base64");

    expect(raw.length).toBe(42);
    expect(raw[0]).toBe(12);
    expect(Buffer.compare(raw.subarray(1, 33), Buffer.from(bs58.decode(script)))).toBe(0);
    expect(raw[33]).toBe(0);
    expect(Number(raw.readBigUInt64LE(34))).toBe(42);
    expect(built.instruction.accounts).toHaveLength(4);
  });

  it("rejects non-canonical vm_state override", async () => {
    await expect(
      Admin.generateSetFeesInstruction(authority, 1, 1, {
        vmStateAccount: recipient,
      }),
    ).rejects.toThrow(/vmStateAccount must be canonical PDA/);
  });
});
