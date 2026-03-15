import { createHash } from "crypto";
import bs58 from "bs58";

export const __calls = {
  getLatestBlockhash: [] as any[],
  sendRawTransaction: [] as any[],
  confirmTransaction: [] as any[],
};

export class PublicKey {
  private readonly value: string;

  constructor(value: string | Uint8Array | number[]) {
    if (typeof value === "string") {
      this.value = value;
      return;
    }
    const bytes = value instanceof Uint8Array ? value : Uint8Array.from(value);
    this.value = bs58.encode(bytes);
  }

  toString() {
    return this.value;
  }

  toBase58() {
    return this.value;
  }

  static findProgramAddressSync(
    seeds: Array<Buffer | Uint8Array>,
    programId: PublicKey,
  ): [PublicKey, number] {
    const marker = Buffer.from("ProgramDerivedAddress");
    const programIdBytes = bs58.decode(programId.toBase58());

    for (let bump = 255; bump >= 1; bump--) {
      const parts = [
        ...seeds.map((seed) => Buffer.from(seed)),
        Buffer.from([bump]),
        Buffer.from(programIdBytes),
        marker,
      ];
      const hash = createHash("sha256").update(Buffer.concat(parts)).digest();
      return [new PublicKey(hash), bump];
    }

    throw new Error("Unable to find valid program address");
  }

  static isOnCurve(_pubkeyData: Uint8Array): boolean {
    // Deterministic mock behavior: treat derived addresses as off-curve.
    return false;
  }

  static async createWithSeed(
    fromPublicKey: PublicKey,
    seed: string,
    programId: PublicKey,
  ): Promise<PublicKey> {
    const hash = createHash("sha256")
      .update(Buffer.concat([
        bs58.decode(fromPublicKey.toBase58()),
        Buffer.from(seed, "utf8"),
        bs58.decode(programId.toBase58()),
      ]))
      .digest();
    return new PublicKey(hash);
  }
}

export class TransactionInstruction {
  keys: any[];
  programId: PublicKey;
  data: Buffer;
  constructor(config: any) {
    this.keys = config.keys;
    this.programId = config.programId;
    this.data = config.data;
  }
}

export class Transaction {
  instructions: any[] = [];
  feePayer: PublicKey | undefined;
  recentBlockhash: string | undefined;
  signatures: Array<{ signature: Buffer | null }> = [{ signature: Buffer.from([1]) }];
  add(ix: any) {
    this.instructions.push(ix);
    return this;
  }
  partialSign(..._signers: any[]) {
    return this;
  }
  serialize() {
    return Buffer.from([9, 9, 9]);
  }
}

export class Keypair {
  publicKey: PublicKey;

  private constructor(publicKey?: PublicKey) {
    this.publicKey = publicKey || new PublicKey(Buffer.alloc(32, 7));
  }

  static generate() {
    return new Keypair(new PublicKey(Buffer.alloc(32, 8)));
  }
}

export class Connection {
  constructor(_url: string, _commitment: string) {}

  async getLatestBlockhash() {
    __calls.getLatestBlockhash.push([]);
    return { blockhash: "latest-bh" };
  }

  async sendRawTransaction(payload: Buffer, options: any) {
    __calls.sendRawTransaction.push([payload, options]);
    return "sig-123";
  }

  async confirmTransaction(signature: string, commitment: string) {
    __calls.confirmTransaction.push([signature, commitment]);
    return { value: { err: null } };
  }
}

export const SystemProgram = {
  programId: new PublicKey("11111111111111111111111111111111"),
  createAccount: (params: any) =>
    new TransactionInstruction({
      keys: [
        { pubkey: params.fromPubkey, isSigner: true, isWritable: true },
        { pubkey: params.newAccountPubkey, isSigner: true, isWritable: true },
      ],
      programId: new PublicKey("11111111111111111111111111111111"),
      data: Buffer.from([0]),
    }),
  transfer: (params: any) =>
    new TransactionInstruction({
      keys: [
        { pubkey: params.fromPubkey, isSigner: true, isWritable: true },
        { pubkey: params.toPubkey, isSigner: false, isWritable: true },
      ],
      programId: new PublicKey("11111111111111111111111111111111"),
      data: Buffer.from([2]),
    }),
  createAccountWithSeed: (params: any) =>
    new TransactionInstruction({
      keys: [
        { pubkey: params.fromPubkey, isSigner: true, isWritable: true },
        { pubkey: params.newAccountPubkey, isSigner: false, isWritable: true },
        { pubkey: params.basePubkey, isSigner: true, isWritable: false },
      ],
      programId: new PublicKey("11111111111111111111111111111111"),
      data: Buffer.concat([
        Buffer.from([3]),
        Buffer.from(params.seed, "utf8"),
        Buffer.from("|"),
        Buffer.from(String(params.lamports), "utf8"),
        Buffer.from("|"),
        Buffer.from(String(params.space), "utf8"),
        Buffer.from("|"),
        Buffer.from(params.programId.toBase58(), "utf8"),
      ]),
    }),
};

export const ComputeBudgetProgram = {
  setComputeUnitLimit: (_params: any) =>
    new TransactionInstruction({
      keys: [],
      programId: new PublicKey("ComputeBudget111111111111111111111111111111"),
      data: Buffer.from([3]),
    }),
};
