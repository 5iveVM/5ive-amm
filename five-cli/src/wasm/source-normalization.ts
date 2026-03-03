const SPL_TOKEN_PROGRAM_ID = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';

function normalizeAccountContextAccess(source: string): string {
  const placeholders = new Map<string, string>();
  let nextId = 0;
  const protect = (match: string) => {
    const token = `__FIVE_CTX_${nextId++}__`;
    placeholders.set(token, match);
    return token;
  };

  let normalized = source.replace(
    /(\b[A-Za-z_][A-Za-z0-9_]*)\.ctx\.(key|lamports|owner|data|bump)\b/g,
    protect,
  );

  normalized = normalized.replace(
    /(\b[A-Za-z_][A-Za-z0-9_]*)\.(key|lamports|owner|data|bump)\b/g,
    '$1.ctx.$2',
  );

  for (const [token, original] of placeholders.entries()) {
    normalized = normalized.split(token).join(original);
  }

  return normalized;
}

function normalizeSplTokenModule(source: string): string {
  const importPattern = /^\s*use\s+std::interfaces::spl_token;\s*$/m;
  if (!importPattern.test(source) && !/\bspl_token::[A-Za-z_][A-Za-z0-9_]*\s*\(/.test(source)) {
    return source;
  }

  let normalized = source.replace(importPattern, '');

  const methods = new Set<string>();
  for (const match of normalized.matchAll(/\bspl_token::([A-Za-z_][A-Za-z0-9_]*)\s*\(/g)) {
    methods.add(match[1]);
  }

  if (methods.size === 0) {
    return normalized;
  }

  const signatures = Array.from(methods)
    .sort()
    .map((method) => {
      switch (method) {
        case 'transfer':
          return '  transfer(source: account @mut, destination: account @mut, authority: account @signer, amount: u64);';
        case 'mint_to':
          return '  mint_to(mint: account @mut, destination: account @mut, authority: account @signer, amount: u64);';
        case 'burn':
          return '  burn(source: account @mut, mint: account @mut, authority: account @signer, amount: u64);';
        case 'approve':
          return '  approve(source: account @mut, delegate: account, authority: account @signer, amount: u64);';
        case 'revoke':
          return '  revoke(source: account @mut, authority: account @signer);';
        case 'freeze_account':
          return '  freeze_account(account_to_freeze: account @mut, mint: account @mut, freeze_authority: account @signer);';
        case 'thaw_account':
          return '  thaw_account(account_to_thaw: account @mut, mint: account @mut, freeze_authority: account @signer);';
        case 'transfer_checked':
          return '  transfer_checked(source: account @mut, mint: account @mut, destination: account @mut, authority: account @signer, amount: u64, decimals: u8);';
        default:
          return `  ${method}(source: account @mut, destination: account @mut, authority: account @signer, amount: u64);`;
      }
    })
    .join('\n');

  normalized = normalized.replace(/\bspl_token::([A-Za-z_][A-Za-z0-9_]*)\s*\(/g, 'SPLToken.$1(');

  if (/^\s*interface\s+SPLToken\b/m.test(normalized)) {
    return normalized;
  }

  const interfaceDecl =
    `interface SPLToken @program("${SPL_TOKEN_PROGRAM_ID}") {\n${signatures}\n}\n\n`;

  return `${interfaceDecl}${normalized.trimStart()}`;
}

export function normalizeWasmCompilerSource(source: string): string {
  let normalized = source;
  normalized = normalizeAccountContextAccess(normalized);
  normalized = normalizeSplTokenModule(normalized);
  return normalized;
}
