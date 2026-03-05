// Generate starter Five DSL (.v) templates for common patterns.

import { mkdir, writeFile, access, readFile } from 'fs/promises';
import { join, resolve, dirname } from 'path';
import { fileURLToPath } from 'url';
import ora from 'ora';
import { success as uiSuccess, warning as uiWarning, uiColors } from '../utils/cli-ui.js';

import {
  CommandDefinition,
  CommandContext,
} from '../types.js';

type TemplateName = 'vault' | 'escrow' | 'amm' | 'token' | 'nft' | 'nft-globals' | 'multisig' | 'vesting' | 'auction-english' | 'staking' | 'airdrop-merkle' | 'system-lamports' | 'interface' | 'spl-token';

export const templateCommand: CommandDefinition = {
  name: 'template',
  description: 'Generate Five DSL templates for common 5IVE and CPI patterns, including stdlib-based and interface-based examples.',
  aliases: ['tmpl', 'scaffold'],

  options: [
    {
      flags: '-t, --type <name>',
      description: 'Template type',
      choices: ['vault', 'escrow', 'amm', 'token', 'nft', 'nft-globals', 'multisig', 'vesting', 'auction-english', 'staking', 'airdrop-merkle', 'system-lamports', 'interface', 'spl-token'],
      required: false,
    },
    {
      flags: '--all',
      description: 'Generate all templates',
      defaultValue: false,
    },
    {
      flags: '-o, --out-dir <dir>',
      description: 'Output directory',
      defaultValue: '.',
    },
    {
      flags: '-f, --force',
      description: 'Overwrite existing files',
      defaultValue: false,
    },
  ],

  arguments: [
    {
      name: 'name',
      description: 'Optional base filename (without extension) when generating single template',
      required: false,
    },
  ],

  examples: [
    {
      command: 'five template --type vault',
      description: 'Generate a vault.v template in current directory',
    },
    {
      command: 'five template --type escrow -o examples',
      description: 'Generate escrow.v under examples/',
    },
    {
      command: 'five template --all -o templates',
      description: 'Generate all templates into templates/',
    },
    {
      command: 'five template --type token my_token',
      description: 'Generate my_token.v for a single template',
    },
    {
      command: 'five template --type nft-globals',
      description: 'Generate nft-globals.v (transfer-only; metadata assumed pre-set)',
    },
    // Quickstart flows (Token)
    {
      command: '# Token: generate → compile → run locally',
      description: '—',
    },
    {
      command: 'five template --type token -o templates',
      description: 'Create token.v in ./templates',
    },
    {
      command: 'five compile templates/token.v -o build/token.five',
      description: 'Compile to bytecode (.five)',
    },
    {
      command: 'five execute build/token.five --local',
      description: 'Local execution (use -f to pick a function index)',
    },
    // Quickstart flows (AMM)
    {
      command: '# AMM: generate → compile → run locally',
      description: '—',
    },
    {
      command: 'five template --type amm -o templates',
      description: 'Create amm.v in ./templates',
    },
    {
      command: 'five compile templates/amm.v -o build/amm.five',
      description: 'Compile AMM template',
    },
    {
      command: 'five execute build/amm.five --local',
      description: 'Local execution for AMM',
    },
    // Quickstart flows (NFT)
    {
      command: '# NFT: generate → compile → run locally',
      description: '—',
    },
    {
      command: 'five template --type nft -o templates',
      description: 'Create nft.v in ./templates',
    },
    {
      command: 'five compile templates/nft.v -o build/nft.five',
      description: 'Compile NFT template',
    },
    {
      command: 'five execute build/nft.five --local',
      description: 'Local execution for NFT',
    },
    // Deploy + on-chain execution (mainnet)
    {
      command: '# Deploy + execute on-chain (generic)',
      description: '—',
    },
    {
      command: 'five deploy build/token.five --target mainnet',
      description: 'Deploy compiled bytecode to mainnet',
    },
    {
      command: 'five execute --script-account <ACCOUNT_ID> -f 0 --target mainnet',
      description: 'Execute function 0 of deployed script (replace <ACCOUNT_ID>)',
    },
  ],

  handler: async (args: string[], options: any, context: CommandContext): Promise<void> => {
    const { logger } = context;

    const outDir = resolve(options.outDir || options['out-dir'] || '.');
    const baseNameArg = args[0];
    const force = !!options.force;
    const all = !!options.all;
    const type = options.type as TemplateName | undefined;

    if (!all && !type) {
      logger.error('Please specify --type <vault|escrow|amm|token|nft|nft-globals> or use --all');
      throw new Error('Template type not specified');
    }

    // Determine which templates to generate
    const templates: TemplateName[] = all
      ? ['vault', 'escrow', 'amm', 'token', 'nft', 'nft-globals', 'multisig', 'vesting', 'auction-english', 'staking', 'airdrop-merkle', 'system-lamports', 'interface', 'spl-token']
      : [type as TemplateName];

    const spinner = ora(`Generating ${all ? 'all templates' : `${type} template`}...`).start();
    try {
      await mkdir(outDir, { recursive: true });

      const results: { file: string; created: boolean }[] = [];
      for (const t of templates) {
        const filename = buildFilename(t, baseNameArg);
        const filepath = join(outDir, filename);
        const content = await getTemplateContent(t);
        const created = await writeFileSafe(filepath, content, force);
        results.push({ file: filepath, created });
      }

      spinner.succeed('Template generation complete');
      for (const r of results) {
        if (r.created) {
          console.log(`${uiSuccess('Created')} ${uiColors.info(r.file)}`);
        } else {
          console.log(uiWarning(`Skipped (exists): ${r.file}`));
        }
      }

      console.log('\nNext steps:');
      console.log(`- Edit generated .v files to fit your use case`);
      console.log(`- Compile: ${uiColors.info('five compile <file>.v -o build/<file>.five')}`);
      console.log(`- Execute locally: ${uiColors.info('five execute <file>.five --local')}`);

    } catch (err) {
      spinner.fail('Failed to generate templates');
      logger.error((err as Error).message);
      throw err;
    }
  },
};

function buildFilename(type: TemplateName, base?: string): string {
  if (base && base.trim().length > 0) {
    return `${base.trim()}.v`;
  }
  return `${type}.v`;
}

async function writeFileSafe(path: string, content: string, force: boolean): Promise<boolean> {
  try {
    if (!force) {
      await access(path);
      // Exists and not forcing
      return false;
    }
  } catch {
    // Does not exist, proceed
  }
  await writeFile(path, content);
  return true;
}

async function getTemplateContent(name: TemplateName): Promise<string> {
  // Prefer external template files for easier debugging and iteration
  try {
    const __filename = fileURLToPath(import.meta.url);
    const __dirname = dirname(__filename); // .../dist/commands at runtime
    const candidatePaths = [
      // When running from dist
      resolve(__dirname, '../templates', `${name}.v`),
      // When running from src via ts-node or tests
      resolve(__dirname, '../../src/templates', `${name}.v`),
      // When executed from repository root
      resolve(process.cwd(), 'templates', `${name}.v`),
    ];

    for (const p of candidatePaths) {
      try {
        const data = await readFile(p, 'utf8');
        if (data && data.trim().length > 0) return data;
      } catch {
        // try next
      }
    }
  } catch {
    // ignore and fall back to inline
  }

  // Fallback to inline templates if files not found
  switch (name) {
    case 'vault':
      return TEMPLATE_VAULT;
    case 'escrow':
      return TEMPLATE_ESCROW;
    case 'amm':
      return TEMPLATE_AMM;
    case 'token':
      return TEMPLATE_TOKEN;
    case 'nft':
      return TEMPLATE_NFT;
    case 'nft-globals':
      // Minimal inline fallback; prefer file template
      return `// NFT (globals) inline fallback\nmut collection_symbol: string;\nmut base_uri: string;\naccount NFT { token_id: pubkey; owner_key: pubkey; uri: string; }\nconfigure(symbol: string, base: string) { collection_symbol = symbol; base_uri = base; }\nmint_from_globals(state: NFT @mut, owner: pubkey) { state.token_id = owner; state.owner_key = owner; state.uri = base_uri; }\n`;
    case 'multisig':
      return `account MultisigState { threshold: u8; approvals: u64; last_proposal_id: u64; proposal_hash: u64; executed: bool; }\ninit_multisig(state: MultisigState @mut, t: u8) { state.threshold = t; state.approvals = 0; state.last_proposal_id = 0; state.proposal_hash = 0; state.executed = false; }\nopen_proposal(state: MultisigState @mut, h: u64) { state.last_proposal_id = state.last_proposal_id + 1; state.proposal_hash = h; state.approvals = 0; state.executed = false; }\napprove(state: MultisigState @mut) { state.approvals = state.approvals + 1; }\nexecute(state: MultisigState @mut) { require(!state.executed); require(state.approvals >= state.threshold); state.executed = true; }\n`;
    case 'vesting':
      return `account VestingState { beneficiary: pubkey; start_time: u64; cliff_seconds: u64; duration_seconds: u64; total_amount: u64; released_amount: u64; }\ninit_vesting(state: VestingState @mut, b: pubkey, s: u64, c: u64, d: u64, t: u64) { state.beneficiary = b; state.start_time = s; state.cliff_seconds = c; state.duration_seconds = d; state.total_amount = t; state.released_amount = 0; }\nrelease(state: VestingState @mut, amount: u64) -> u64 { require(amount > 0); state.released_amount = state.released_amount + amount; return amount; }\n`;
    case 'auction-english':
      return `account AuctionState { seller: pubkey; end_time: u64; min_increment: u64; highest_bid: u64; highest_bidder: pubkey; settled: bool; }\ninit_auction(state: AuctionState @mut, s: pubkey, e: u64, m: u64, r: u64) { state.seller = s; state.end_time = e; state.min_increment = m; state.highest_bid = r; state.highest_bidder = s; state.settled = false; }\nbid(state: AuctionState @mut, b: pubkey, a: u64) { let now = get_clock().slot; require(now < state.end_time); require(a >= state.highest_bid + state.min_increment); state.highest_bid = a; state.highest_bidder = b; }\nsettle(state: AuctionState @mut) { let now = get_clock().slot; require(now >= state.end_time); require(!state.settled); state.settled = true; }\n`;
    case 'staking':
      return `account Pool { reward_rate_per_slot: u64; last_update_slot: u64; acc_reward_per_share: u64; scale: u64; }\naccount StakeAccount { owner_key: pubkey; amount: u64; reward_debt: u64; }\ninit_pool(state: Pool @mut, r: u64, sc: u64) { state.reward_rate_per_slot = r; state.last_update_slot = get_clock().slot; state.acc_reward_per_share = 0; state.scale = sc; }\naccrue(state: Pool @mut, slots: u64) { state.acc_reward_per_share = state.acc_reward_per_share + (state.reward_rate_per_slot * slots); state.last_update_slot = state.last_update_slot + slots; }\ninit_staker(state: StakeAccount @mut, o: pubkey) { state.owner_key = o; state.amount = 0; state.reward_debt = 0; }\nstake(state: StakeAccount @mut, o: pubkey, a: u64, acc: u64) { require(state.owner_key == o); state.reward_debt = state.reward_debt + (a * acc); state.amount = state.amount + a; }\nunstake(state: StakeAccount @mut, o: pubkey, a: u64, acc: u64) { require(state.owner_key == o); require(state.amount >= a); state.amount = state.amount - a; state.reward_debt = state.reward_debt - (a * acc); }\nclaimable(state: StakeAccount, acc: u64) -> u64 { let accrued = state.amount * acc; if (accrued <= state.reward_debt) { return 0; } return accrued - state.reward_debt; }\nrecord_claim(state: StakeAccount @mut, c: u64) { state.reward_debt = state.reward_debt + c; }\n`;
    case 'airdrop-merkle':
      return `account AirdropConfig { merkle_root: u64; total_claimed: u64; }\naccount ClaimRecord { claimer: pubkey; amount: u64; claimed: bool; }\ninit_airdrop(state: AirdropConfig @mut, r: u64) { state.merkle_root = r; state.total_claimed = 0; }\nclaim(state: ClaimRecord @mut, c: pubkey, a: u64, expected: u64, cfg_root: u64) { require(expected == cfg_root); require(!state.claimed); state.claimer = c; state.amount = a; state.claimed = true; }\n`;
    case 'system-lamports':
      return `quote_transfer(from: account, to: account, amount: u64) -> (u64, u64) { require(amount > 0); require(from.ctx.lamports >= amount); let nf = from.ctx.lamports - amount; let nt = to.ctx.lamports + amount; return (nf, nt); }\ncheck_min_balance(acc: account, min: u64) -> bool { return acc.ctx.lamports >= min; }\ntopup_needed(acc: account, min: u64) -> u64 { if (acc.ctx.lamports >= min) { return 0; } return min - acc.ctx.lamports; }\n`;
    case 'interface':
      return `interface ExampleProgram @program("11111111111111111111111111111111") @serializer(raw) {\n    do_thing @discriminator_bytes([1]) (authority: account, value: u64);\n}\n\npub call_example(authority: account @signer, value: u64) {\n    ExampleProgram::do_thing(authority, value);\n}\n`;
    case 'spl-token':
      return `use std::interfaces::spl_token;\n\npub mint_tokens(mint: account @mut, destination: account @mut, authority: account @signer, amount: u64) {\n    require(amount > 0);\n    spl_token::SPLToken::mint_to(mint, destination, authority, amount);\n}\n\npub transfer_tokens(source: account @mut, destination: account @mut, authority: account @signer, amount: u64) {\n    require(amount > 0);\n    spl_token::SPLToken::transfer(source, destination, authority, amount);\n}\n`;
    
  }
}

// --- Templates ---

const COMMON_HEADER = `// Generated by five template
// Starter template in Five DSL. Adjust constraints and accounts
// to match your application and follow state layout best practices.
`;

const TEMPLATE_VAULT = `${COMMON_HEADER}
// Vault template: lamport custody via System Program CPI

use std::interfaces::system_program;

account VaultState {
    balance: u64;
    authorized_user: pubkey;
}

// Initialize vault state (sets authority; vault_account provided during deposit/withdraw)
init_vault(state: VaultState @mut, authority: account @signer) {
    state.balance = 0;
    state.authorized_user = authority.ctx.key;
}

// Deposit lamports into the vault: transfer from payer to vault_account
// - payer: signer funding the deposit
// - vault_account: the on-chain account holding lamports for the vault
// Updates internal balance for accounting
deposit(state: VaultState @mut, payer: account @signer @mut, vault_account: account @mut, amount: u64) {
    require(amount > 0);
    system_program::SystemProgram::transfer(payer, vault_account, amount);
    state.balance = state.balance + amount;
}

// Withdraw lamports from the vault to a recipient (requires authority)
// - authority: must match configured authorized_user
// - vault_account: source of lamports (vault's account)
// - recipient: destination account to receive lamports
withdraw(state: VaultState @mut, authority: account @signer, vault_account: account @mut, recipient: account @mut, amount: u64) {
    require(state.authorized_user == authority.ctx.key);
    require(amount > 0);
    require(state.balance >= amount);
    system_program::SystemProgram::transfer(vault_account, recipient, amount);
    state.balance = state.balance - amount;
}
`;

const TEMPLATE_ESCROW = `${COMMON_HEADER}
// Escrow template: maker locks funds for a designated taker

account EscrowState {
    maker: pubkey;
    taker: pubkey;
    amount: u64;
    is_funded: bool;
    is_settled: bool;
}

init_escrow(state: EscrowState @mut, maker: account @signer, taker: pubkey, amount: u64) {
    state.maker = maker.ctx.key;
    state.taker = taker;
    state.amount = amount;
    state.is_funded = false;
    state.is_settled = false;
}

fund_escrow(state: EscrowState @mut, maker: account @signer, amount: u64) {
    require(state.maker == maker.ctx.key);
    require(amount == state.amount);
    state.is_funded = true;
}

claim_escrow(state: EscrowState @mut, taker: account @signer) {
    require(state.is_funded);
    require(!state.is_settled);
    require(state.taker == taker.ctx.key);
    state.is_settled = true;
}

cancel_escrow(state: EscrowState @mut, maker: account @signer) {
    require(!state.is_settled);
    require(state.maker == maker.ctx.key);
    state.is_funded = false;
}
`;

const TEMPLATE_AMM = `${COMMON_HEADER}
// Constant-product AMM template (x*y=k) with simple fee

account Pool {
    token_a: u64;
    token_b: u64;
    total_shares: u64;
    fee_bps: u64;
}

init_pool(state: Pool @mut, fee_bps: u64) {
    state.token_a = 0;
    state.token_b = 0;
    state.total_shares = 0;
    state.fee_bps = fee_bps;
}

add_liquidity(state: Pool @mut, amount_a: u64, amount_b: u64) -> u64 {
    // Share calc for template
    let shares = amount_a;
    state.token_a = state.token_a + amount_a;
    state.token_b = state.token_b + amount_b;
    state.total_shares = state.total_shares + shares;
    return shares;
}

swap(state: Pool @mut, amount_in: u64, a_for_b: bool) -> u64 {
    // Skeleton implementation for validator compatibility
    let fee = (amount_in * state.fee_bps) / 10000;
    let net_in = amount_in - fee;
    // No state changes to avoid multi-account/multi-branch rules in validator
    return net_in;
}

// Quote liquidity shares without mutating state
quote_add_liquidity(state: Pool, amount_a: u64, amount_b: u64) -> u64 {
    if (amount_b < amount_a) {
        return amount_b;
    }
    return amount_a;
}

// Remove liquidity (simplified)
remove_liquidity(state: Pool @mut, share: u64) -> u64 {
    require(state.total_shares >= share);
    state.total_shares = state.total_shares - share;
    return share;
}
`;

const TEMPLATE_TOKEN = `${COMMON_HEADER}
// Fungible token template with simple mint and transfer

account Mint {
    authority: pubkey;
    supply: u64;
    decimals: u8;
}

account TokenAccount {
    owner_key: pubkey;
    bal: u64;
}

// Initialize mint state
init_mint(state: Mint @mut, authority: account @signer, decimals: u8) {
    state.authority = authority.ctx.key;
    state.supply = 0;
    state.decimals = decimals;
}

// Initialize a token account
init_account(state: TokenAccount @mut, owner: account @signer) {
    state.owner_key = owner.ctx.key;
    state.bal = 0;
}

// Split flows to satisfy current validator constraints
mint_increase_supply(state: Mint @mut, authority: account @signer, amount: u64) {
    require(state.authority == authority.ctx.key);
    require(amount > 0);
    state.supply = state.supply + amount;
}

credit_account(state: TokenAccount @mut, amount: u64) {
    state.bal = state.bal + amount;
}

debit_account(state: TokenAccount @mut, signer: account @signer, amount: u64) {
    require(state.owner_key == signer.ctx.key);
    require(state.bal >= amount);
    require(amount > 0);
    state.bal = state.bal - amount;
}

credit_after_debit(state: TokenAccount @mut, amount: u64) {
    state.bal = state.bal + amount;
}

// Burn reduces supply
burn_supply(state: Mint @mut, authority: account @signer, amount: u64) {
    require(state.authority == authority.ctx.key);
    require(state.supply >= amount);
    require(amount > 0);
    state.supply = state.supply - amount;
}

// Change mint authority
set_mint_authority(state: Mint @mut, current: account @signer, new_auth: pubkey) {
    require(state.authority == current.ctx.key);
    state.authority = new_auth;
}

// Read-only helpers
get_supply(state: Mint) -> u64 { return state.supply; }
get_balance(state: TokenAccount) -> u64 { return state.bal; }
`;

const TEMPLATE_NFT = `${COMMON_HEADER}
// NFT template with simple mint and transfer

account NFT {
    token_id: pubkey;
    owner_key: pubkey;
    uri: string;
}

// Initialize NFT fields
mint_nft(state: NFT @mut, owner: pubkey, uri: string) {
    // For template simplicity, assign token_id deterministically
    state.token_id = owner;
    state.owner_key = owner;
    state.uri = uri;
}

// Transfer ownership
transfer_nft(state: NFT @mut, from: pubkey, to: pubkey) {
    require(state.owner_key == from);
    state.owner_key = to;
}

// Update token metadata URI
set_uri(state: NFT @mut, owner: pubkey, new_uri: string) {
    require(state.owner_key == owner);
    state.uri = new_uri;
}

// Read-only helpers
get_uri(state: NFT) -> string { return state.uri; }
get_owner(state: NFT) -> pubkey { return state.owner_key; }
`;

export default templateCommand;
