/**
 * Five CLI Donate Command
 *
 * Sends SOL to the Five VM donation address.
 */

import ora from 'ora';
import { readFile } from 'fs/promises';
import { homedir } from 'os';
import { join } from 'path';
import {
  Connection,
  LAMPORTS_PER_SOL,
  PublicKey,
  SystemProgram,
  Keypair,
  sendAndConfirmTransaction,
  Transaction,
} from '@solana/web3.js';

import { CommandDefinition, CommandContext } from '../types.js';
import { section, success as uiSuccess, error as uiError, uiColors } from '../utils/cli-ui.js';

const DONATION_ADDRESS = '2A3mhuakqQMCaY3ZeeqyLZP2jmxcS478Jnfd6ZerBaFm';

function getDefaultKeypairPath(): string {
  return join(homedir(), '.config', 'solana', 'id.json');
}

function getRpcUrl(network?: string, custom?: string): string {
  if (custom) return custom;
  switch ((network || '').toLowerCase()) {
    case 'mainnet':
    case 'mainnet-beta':
      return 'https://api.mainnet-beta.solana.com';
    case 'testnet':
      return 'https://api.testnet.solana.com';
    case 'devnet':
    default:
      return 'https://api.devnet.solana.com';
  }
}

async function loadKeypair(path: string): Promise<Keypair> {
  const raw = await readFile(path, 'utf8');
  // Standard solana-keygen JSON array format
  const secret = Uint8Array.from(JSON.parse(raw));
  return Keypair.fromSecretKey(secret);
}

export const donateCommand: CommandDefinition = {
  name: 'donate',
  description: 'Donate SOL to support Five VM development',
  aliases: [],

  options: [
    {
      flags: '--network <network>',
      description: 'Solana network (devnet|testnet|mainnet)',
      defaultValue: 'devnet',
    },
    {
      flags: '--rpc <url>',
      description: 'Custom RPC URL (overrides --network)',
      required: false,
    },
    {
      flags: '--keypair <path>',
      description: 'Path to keypair JSON (default: ~/.config/solana/id.json)',
      required: false,
    },
  ],

  arguments: [
    {
      name: 'amount',
      description: 'Amount in whole SOL (integer)',
      required: true,
    },
  ],

  examples: [
    { command: 'five donate 1', description: 'Donate 1 SOL on devnet' },
    { command: 'five donate 2 --network mainnet', description: 'Donate 2 SOL on mainnet' },
    { command: 'five donate 1 --rpc https://... --keypair ~/.config/solana/id.json', description: 'Use custom RPC and keypair' },
  ],

  handler: async (args: string[], options: any, _context: CommandContext): Promise<void> => {
    const spinner = ora('Preparing donation...').start();
    try {
      const amountStr = args[0];
      if (!amountStr) throw new Error('Amount (whole SOL) is required');

      const amountSol = parseInt(String(amountStr), 10);
      if (!Number.isFinite(amountSol) || amountSol <= 0) {
        throw new Error(`Invalid amount: ${amountStr}. Provide a positive integer number of SOL.`);
      }

      const lamports = BigInt(amountSol) * BigInt(LAMPORTS_PER_SOL);

      const keypairPath = options.keypair || process.env.SOLANA_KEYPAIR || getDefaultKeypairPath();
      const payer = await loadKeypair(keypairPath);

      const rpcUrl = getRpcUrl(options.network, options.rpc);
      const connection = new Connection(rpcUrl, 'confirmed');
      const toPubkey = new PublicKey(DONATION_ADDRESS);

      spinner.text = 'Building transaction...';
      const ix = SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey, lamports: Number(lamports) });
      const tx = new Transaction().add(ix);

      spinner.text = 'Sending donation transaction...';
      const signature = await sendAndConfirmTransaction(connection, tx, [payer], {
        commitment: 'confirmed',
        skipPreflight: false,
      });

      spinner.succeed('Donation sent');

      console.log('\n' + section('Thanks'));
      console.log(`  Amount: ${uiColors.info(amountSol.toString())} ${uiColors.info('SOL')}`);
      console.log(`  To: ${uiColors.info(DONATION_ADDRESS)}`);
      console.log(`  Tx: ${uiColors.info(signature)}`);
      console.log('\n' + uiColors.muted('Your support helps us ship faster. Thank you.') + '\n');

    } catch (err: any) {
      spinner.fail('Donation failed');
      console.error(uiError(err?.message || String(err)));
      throw err;
    }
  },
};

export default donateCommand;
