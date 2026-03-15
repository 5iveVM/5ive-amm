import { FiveProgram } from './FiveProgram.js';
import { PublicKey } from '@solana/web3.js';

export interface SessionScope {
  functions: string[];
  ttlSlots?: number;
  bindAccount?: string;
  nonce?: string;
}

export interface SessionCreateParams {
  authority: string;
  delegate: string;
  targetProgram: string;
  expiresAtSlot: number;
  scopeHash: string;
  bindAccount?: string;
  nonce?: string;
  payer?: string;
  managerScriptAccount?: string;
  managerCodeHash?: string;
  managerVersion?: number;
}

export interface SessionRecord {
  sessionAddress: string;
  authority: string;
  delegate: string;
  targetProgram: string;
  expiresAtSlot: number;
}

export interface CanonicalSessionService {
  cluster: 'localnet' | 'devnet' | 'mainnet';
  scriptAccount: string;
  codeHash: string;
  version: number;
  status: 'active' | 'disabled';
}

export interface SessionManagerOptions {
  identity?: CanonicalSessionService;
  enforceCanonical?: boolean;
  allowUnsafeOverride?: boolean;
}

/**
 * Lightweight helper around a deployed session-manager script.
 * Uses normal Five execute flow; no VM opcode/runtime changes required.
 */
export class SessionManager {
  readonly identity: CanonicalSessionService;
  private readonly enforceCanonical: boolean;
  private readonly allowUnsafeOverride: boolean;

  constructor(
    readonly managerProgram: FiveProgram,
    readonly defaultTtlSlots: number = 3000, // ~20m on Solana-like slot timings
    options: SessionManagerOptions = {},
  ) {
    this.identity =
      options.identity ||
      SessionManager.resolveCanonicalIdentity({
        vmProgramId: this.managerProgram.getFiveVMProgramId(),
      });
    this.allowUnsafeOverride = options.allowUnsafeOverride ?? false;
    const strictByDefault = this.identity.cluster === 'mainnet';
    this.enforceCanonical = options.enforceCanonical ?? strictByDefault;
    if (
      this.identity.cluster === 'mainnet' &&
      options.enforceCanonical === false &&
      !this.allowUnsafeOverride
    ) {
      throw new Error('Disabling canonical session manager on mainnet requires allowUnsafeOverride');
    }
    if (this.enforceCanonical && this.identity.status !== 'active') {
      throw new Error('Canonical session service is disabled for current cluster');
    }
    const hasPinnedScript =
      this.identity.scriptAccount !== '11111111111111111111111111111111';
    if (
      this.enforceCanonical &&
      hasPinnedScript &&
      this.managerProgram.getScriptAccount() !== this.identity.scriptAccount
    ) {
      throw new Error('SessionManager program does not match canonical session_v1 service');
    }
  }

  static resolveCanonicalIdentity(input: {
    cluster?: 'localnet' | 'devnet' | 'mainnet';
    vmProgramId?: string;
    scriptAccount?: string;
    codeHash?: string;
    status?: 'active' | 'disabled';
    version?: number;
  } = {}): CanonicalSessionService {
    const vmProgramId = input.vmProgramId || process.env.FIVE_PROGRAM_ID;
    if (!vmProgramId) {
      throw new Error('SessionManager canonical identity requires vmProgramId or FIVE_PROGRAM_ID');
    }
    const vmProgram = new PublicKey(vmProgramId);
    const [scriptPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('session_v1', 'utf-8')],
      vmProgram,
    );
    const cluster = input.cluster || ((process.env.FIVE_VM_CLUSTER as any) || 'localnet');

    return {
      cluster,
      scriptAccount: input.scriptAccount || scriptPda.toBase58(),
      codeHash: input.codeHash || '11111111111111111111111111111111',
      version: input.version ?? 1,
      status: input.status || 'active',
    };
  }

  static scopeHashForFunctions(functions: string[]): string {
    const sorted = [...functions].sort();
    // Stable v1 hash seed; caller may replace with stronger domain-specific hashing if desired.
    let acc = 0n;
    for (const ch of sorted.join('|')) {
      acc = (acc * 131n + BigInt(ch.charCodeAt(0))) & ((1n << 256n) - 1n);
    }
    return '0x' + acc.toString(16).padStart(64, '0');
  }

  async deriveSessionAddress(
    authority: string,
    delegate: string,
    targetProgram: string,
  ): Promise<string> {
    const [pda] = await this.managerProgram.findAddress(
      ['session', authority, delegate, targetProgram],
      this.managerProgram.getFiveVMProgramId(),
    );
    return pda;
  }

  async buildCreateSessionInstruction(params: SessionCreateParams) {
    const sessionAddress = await this.deriveSessionAddress(
      params.authority,
      params.delegate,
      params.targetProgram,
    );
    const builder = this.managerProgram
      .function('create_session')
      .accounts({
        session: sessionAddress,
        authority: params.authority,
        delegate: params.delegate,
      })
      .args({
        target_program: params.targetProgram,
        expires_at_slot: params.expiresAtSlot,
        scope_hash: params.scopeHash,
        bind_account: params.bindAccount || '11111111111111111111111111111111',
        nonce: params.nonce || '0x00',
        manager_script_account: params.managerScriptAccount || this.identity.scriptAccount,
        manager_code_hash: params.managerCodeHash || this.identity.codeHash,
        manager_version: params.managerVersion ?? this.identity.version,
      });
    if (params.payer) {
      builder.payer(params.payer);
    }
    return builder.instruction();
  }

  async buildRevokeSessionInstruction(
    authority: string,
    delegate: string,
    targetProgram: string,
    payer?: string,
  ) {
    const sessionAddress = await this.deriveSessionAddress(authority, delegate, targetProgram);
    const builder = this.managerProgram
      .function('revoke_session')
      .accounts({
        session: sessionAddress,
        authority,
      });
    if (payer) {
      builder.payer(payer);
    }
    return builder.instruction();
  }
}
