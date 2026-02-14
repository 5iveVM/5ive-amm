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
  add(ix: any) {
    this.instructions.push(ix);
    return this;
  }
  serialize() {
    return Buffer.from([9, 9, 9]);
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
