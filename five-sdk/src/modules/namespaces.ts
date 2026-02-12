export interface ScopedNamespace {
  symbol: "!" | "@" | "#" | "$" | "%";
  domain: string;
  subprogram?: string;
  canonical: string;
}

const SYMBOLS = new Set(["!", "@", "#", "$", "%"]);

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

