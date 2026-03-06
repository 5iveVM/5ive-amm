export interface ScopedNamespace {
  symbol: "!" | "@" | "#" | "$" | "%";
  domain: string;
  subprogram?: string;
  canonical: string;
}

const SYMBOLS = new Set(["!", "@", "#", "$", "%"]);
const NAMESPACE_CONFIG_SEED = "5ns_config";
const NAMESPACE_TLD_SEED = "5ns_tld";
const NAMESPACE_BINDING_SEED = "5ns_binding";
const LAMPORTS_PER_SOL = 1_000_000_000;

export function canonicalizeScopedNamespace(input: string): ScopedNamespace {
  const value = input.trim();
  if (value.length < 2) {
    throw new Error("namespace is too short");
  }
  const symbol = value[0];
  if (!SYMBOLS.has(symbol)) {
    throw new Error("namespace symbol must be one of ! @ # $ %");
  }
  const parts = value.slice(1).split("/");
  if (parts.length === 0 || parts.length > 2) {
    throw new Error("namespace must be !domain or !domain/subprogram");
  }
  const normalize = (seg: string) => seg.trim().toLowerCase();
  const domain = normalize(parts[0]);
  const subprogram = parts[1] ? normalize(parts[1]) : undefined;
  const validSegment = (seg: string) =>
    seg.length > 0 && /^[a-z0-9-]+$/.test(seg);
  if (!validSegment(domain)) {
    throw new Error("domain must be lowercase alphanumeric + hyphen");
  }
  if (subprogram && !validSegment(subprogram)) {
    throw new Error("subprogram must be lowercase alphanumeric + hyphen");
  }
  const canonical = subprogram ? `${symbol}${domain}/${subprogram}` : `${symbol}${domain}`;
  return {
    symbol: symbol as ScopedNamespace["symbol"],
    domain,
    subprogram,
    canonical,
  };
}

export function namespaceSeedBytes(namespaceValue: string): Uint8Array {
  const parsed = canonicalizeScopedNamespace(namespaceValue);
  if (!parsed.subprogram) {
    throw new Error("namespace seed requires !domain/subprogram");
  }
  const seed = `5NS/${parsed.canonical}`;
  return new TextEncoder().encode(seed);
}

export function resolveNamespaceFromLockfile(
  namespaceValue: string,
  lockfile: any,
): string | undefined {
  const parsed = canonicalizeScopedNamespace(namespaceValue);
  if (!parsed.subprogram) return undefined;
  const namespaces = Array.isArray(lockfile?.namespaces) ? lockfile.namespaces : [];
  const match = namespaces.find((entry: any) => entry?.namespace === parsed.canonical);
  return match?.address;
}

function asBuffer(value: string): Buffer {
  return Buffer.from(value, "utf8");
}

function validateSymbol(symbol: string): ScopedNamespace["symbol"] {
  if (!SYMBOLS.has(symbol)) {
    throw new Error("namespace symbol must be one of ! @ # $ %");
  }
  return symbol as ScopedNamespace["symbol"];
}

async function deriveNamespaceConfigAccount(fiveVMProgramId: string): Promise<string> {
  const { PDAUtils } = await import("../crypto/index.js");
  const cfg = await PDAUtils.findProgramAddress(
    [asBuffer(NAMESPACE_CONFIG_SEED)],
    fiveVMProgramId,
  );
  return cfg.address;
}

export interface NamespaceDerivedAccounts {
  config: string;
  tld: string;
  binding?: string;
}

export async function deriveNamespaceAccounts(
  namespaceValue: string,
  fiveVMProgramId: string,
): Promise<NamespaceDerivedAccounts> {
  const { PDAUtils } = await import("../crypto/index.js");
  const parsed = canonicalizeScopedNamespace(namespaceValue);

  const cfg = await PDAUtils.findProgramAddress(
    [asBuffer(NAMESPACE_CONFIG_SEED)],
    fiveVMProgramId,
  );
  const tld = await PDAUtils.findProgramAddress(
    [asBuffer(NAMESPACE_TLD_SEED), asBuffer(parsed.symbol), asBuffer(parsed.domain)],
    fiveVMProgramId,
  );

  if (!parsed.subprogram) {
    return {
      config: cfg.address,
      tld: tld.address,
    };
  }

  const binding = await PDAUtils.findProgramAddress(
    [
      asBuffer(NAMESPACE_BINDING_SEED),
      asBuffer(parsed.symbol),
      asBuffer(parsed.domain),
      asBuffer(parsed.subprogram),
    ],
    fiveVMProgramId,
  );

  return {
    config: cfg.address,
    tld: tld.address,
    binding: binding.address,
  };
}

interface NamespaceOnChainOptions {
  managerScriptAccount: string;
  connection: any;
  signerKeypair: any;
  fiveVMProgramId?: string;
  debug?: boolean;
}

function nowUnix(): number {
  return Math.floor(Date.now() / 1000);
}

export async function registerNamespaceTldOnChain(
  namespaceValue: string,
  options: NamespaceOnChainOptions,
): Promise<{ transactionId?: string; tldAddress: string; owner: string; treasuryAccount: string }> {
  const { ProgramIdResolver } = await import("../config/ProgramIdResolver.js");
  const { executeOnSolana } = await import("./execute.js");
  const { getVMState } = await import("./vm-state.js");

  const parsed = canonicalizeScopedNamespace(namespaceValue);
  if (parsed.subprogram) {
    throw new Error("register expects top-level namespace like @domain");
  }

  const vmProgramId = ProgramIdResolver.resolve(options.fiveVMProgramId);
  const addresses = await deriveNamespaceAccounts(parsed.canonical, vmProgramId);
  const owner = options.signerKeypair.publicKey.toBase58();
  const now = nowUnix();
  const vmState = await getVMState(options.connection, vmProgramId);
  const treasuryAccount = vmState.authority;
  if (!treasuryAccount) {
    throw new Error("VM state authority (treasury) is unavailable");
  }

  const result = await executeOnSolana(
    options.managerScriptAccount,
    options.connection,
    options.signerKeypair,
    "register_tld",
    [
      addresses.config,
      addresses.tld,
      owner,
      treasuryAccount,
      parsed.symbol,
      parsed.domain,
      now,
    ],
    [addresses.config, addresses.tld, owner, treasuryAccount],
    {
      debug: options.debug,
      fiveVMProgramId: vmProgramId,
      computeUnitLimit: 500000,
    },
  );

  if (!result.success) {
    throw new Error(result.error || "register_tld failed");
  }

  return {
    transactionId: result.transactionId,
    tldAddress: addresses.tld,
    owner,
    treasuryAccount,
  };
}

export async function bindNamespaceOnChain(
  namespaceValue: string,
  scriptAccount: string,
  options: NamespaceOnChainOptions,
): Promise<{ transactionId?: string; bindingAddress: string; owner: string }> {
  const { ProgramIdResolver } = await import("../config/ProgramIdResolver.js");
  const { executeOnSolana } = await import("./execute.js");

  const parsed = canonicalizeScopedNamespace(namespaceValue);
  if (!parsed.subprogram) {
    throw new Error("bind expects namespace with subprogram like @domain/subprogram");
  }

  const vmProgramId = ProgramIdResolver.resolve(options.fiveVMProgramId);
  const addresses = await deriveNamespaceAccounts(parsed.canonical, vmProgramId);
  const owner = options.signerKeypair.publicKey.toBase58();
  const now = nowUnix();

  const result = await executeOnSolana(
    options.managerScriptAccount,
    options.connection,
    options.signerKeypair,
    "bind_subprogram",
    [
      addresses.tld,
      addresses.binding,
      owner,
      parsed.symbol,
      parsed.domain,
      parsed.subprogram,
      scriptAccount,
      now,
    ],
    [addresses.tld, addresses.binding!, owner],
    {
      debug: options.debug,
      fiveVMProgramId: vmProgramId,
      computeUnitLimit: 650000,
    },
  );

  if (!result.success) {
    throw new Error(result.error || "bind_subprogram failed");
  }

  return {
    transactionId: result.transactionId,
    bindingAddress: addresses.binding!,
    owner,
  };
}

export async function resolveNamespaceOnChain(
  namespaceValue: string,
  options: NamespaceOnChainOptions,
): Promise<{ transactionId?: string; resolvedScript?: string; bindingAddress: string }> {
  const { ProgramIdResolver } = await import("../config/ProgramIdResolver.js");
  const { executeOnSolana } = await import("./execute.js");

  const parsed = canonicalizeScopedNamespace(namespaceValue);
  if (!parsed.subprogram) {
    throw new Error("resolve expects namespace with subprogram like @domain/subprogram");
  }

  const vmProgramId = ProgramIdResolver.resolve(options.fiveVMProgramId);
  const addresses = await deriveNamespaceAccounts(parsed.canonical, vmProgramId);

  const result = await executeOnSolana(
    options.managerScriptAccount,
    options.connection,
    options.signerKeypair,
    "resolve",
    [addresses.binding],
    [addresses.binding!],
    {
      debug: options.debug,
      fiveVMProgramId: vmProgramId,
      computeUnitLimit: 300000,
    },
  );

  if (!result.success) {
    throw new Error(result.error || "resolve failed");
  }

  const raw = result.result;
  let resolvedScript: string | undefined;
  if (typeof raw === "string" && raw.length > 0) {
    resolvedScript = raw;
  } else if (raw && typeof raw === "object") {
    if (typeof raw.script_account === "string") resolvedScript = raw.script_account;
    if (typeof raw.scriptAccount === "string") resolvedScript = raw.scriptAccount;
    if (typeof raw.value === "string") resolvedScript = raw.value;
  }

  return {
    transactionId: result.transactionId,
    resolvedScript,
    bindingAddress: addresses.binding!,
  };
}

export async function setNamespaceSymbolPriceOnChain(
  symbol: string,
  priceLamports: number,
  options: NamespaceOnChainOptions,
): Promise<{ transactionId?: string; symbol: ScopedNamespace["symbol"]; priceLamports: number }> {
  const { ProgramIdResolver } = await import("../config/ProgramIdResolver.js");
  const { executeOnSolana } = await import("./execute.js");

  const validatedSymbol = validateSymbol(symbol);
  if (!Number.isFinite(priceLamports) || priceLamports <= 0 || !Number.isInteger(priceLamports)) {
    throw new Error("priceLamports must be a positive integer");
  }

  const vmProgramId = ProgramIdResolver.resolve(options.fiveVMProgramId);
  const configAddress = await deriveNamespaceConfigAccount(vmProgramId);
  const admin = options.signerKeypair.publicKey.toBase58();

  const result = await executeOnSolana(
    options.managerScriptAccount,
    options.connection,
    options.signerKeypair,
    "set_symbol_price",
    [configAddress, admin, validatedSymbol, priceLamports],
    [configAddress, admin],
    {
      debug: options.debug,
      fiveVMProgramId: vmProgramId,
      computeUnitLimit: 300000,
    },
  );

  if (!result.success) {
    throw new Error(result.error || "set_symbol_price failed");
  }

  return {
    transactionId: result.transactionId,
    symbol: validatedSymbol,
    priceLamports,
  };
}

export async function getNamespaceSymbolPriceOnChain(
  symbol: string,
  options: NamespaceOnChainOptions,
): Promise<{ transactionId?: string; symbol: ScopedNamespace["symbol"]; priceLamports: number; priceSol: number }> {
  const { ProgramIdResolver } = await import("../config/ProgramIdResolver.js");
  const { executeOnSolana } = await import("./execute.js");

  const validatedSymbol = validateSymbol(symbol);
  const vmProgramId = ProgramIdResolver.resolve(options.fiveVMProgramId);
  const configAddress = await deriveNamespaceConfigAccount(vmProgramId);

  const result = await executeOnSolana(
    options.managerScriptAccount,
    options.connection,
    options.signerKeypair,
    "get_symbol_price",
    [configAddress, validatedSymbol],
    [configAddress],
    {
      debug: options.debug,
      fiveVMProgramId: vmProgramId,
      computeUnitLimit: 300000,
    },
  );

  if (!result.success) {
    throw new Error(result.error || "get_symbol_price failed");
  }

  const priceLamportsRaw = typeof result.result === "number"
    ? result.result
    : Number(result.result);
  if (!Number.isFinite(priceLamportsRaw) || priceLamportsRaw < 0) {
    throw new Error(`invalid get_symbol_price result: ${String(result.result)}`);
  }
  const priceLamports = Math.trunc(priceLamportsRaw);

  return {
    transactionId: result.transactionId,
    symbol: validatedSymbol,
    priceLamports,
    priceSol: priceLamports / LAMPORTS_PER_SOL,
  };
}
