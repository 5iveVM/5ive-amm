import { Connection, PublicKey } from '@solana/web3.js';
import { CommandContext, CommandDefinition } from '../types.js';
import { ConfigManager } from '../config/ConfigManager.js';
import { VmClusterConfigResolver } from '../config/VmClusterConfigResolver.js';
import { keyValue, section } from '../utils/cli-ui.js';

const VM_SERVICE_REGISTRY_OFFSET = 56;
const SERVICE_REGISTRY_ENTRY_LEN = 80;

type ServiceEntry = {
  scriptAccount: string;
  codeHash: string;
  status: number;
  version: number;
  epoch: bigint;
};

function readServiceEntry(data: Buffer, slot: number): ServiceEntry {
  const offset = VM_SERVICE_REGISTRY_OFFSET + slot * SERVICE_REGISTRY_ENTRY_LEN;
  if (data.length < offset + SERVICE_REGISTRY_ENTRY_LEN) {
    return {
      scriptAccount: '11111111111111111111111111111111',
      codeHash: '11111111111111111111111111111111',
      status: 0,
      version: 0,
      epoch: 0n,
    };
  }

  const scriptAccount = new PublicKey(data.subarray(offset, offset + 32)).toBase58();
  const codeHash = new PublicKey(data.subarray(offset + 32, offset + 64)).toBase58();
  const status = data[offset + 64];
  const version = data[offset + 65];
  const epoch = data.readBigUInt64LE(offset + 72);
  return { scriptAccount, codeHash, status, version, epoch };
}

function statusLabel(status: number): string {
  return status === 1 ? 'active' : 'disabled';
}

export const serviceCommand: CommandDefinition = {
  name: 'service',
  description: 'Inspect canonical 5IVE service registry',
  options: [
    {
      flags: '-t, --target <target>',
      description: 'Target network (local, devnet, mainnet)',
      required: false,
    },
    {
      flags: '-n, --network <url>',
      description: 'Override RPC URL',
      required: false,
    },
    {
      flags: '--program-id <pubkey>',
      description: 'Override 5IVE VM program ID',
      required: false,
    },
    {
      flags: '--vm-state <pubkey>',
      description: 'Override VM state account',
      required: false,
    },
  ],
  arguments: [
    { name: 'action', description: 'show', required: true },
    { name: 'service', description: 'session_v1', required: true },
  ],
  examples: [
    {
      command: '5ive service show session_v1 --target mainnet',
      description: 'Show canonical session service metadata from VM state',
    },
  ],
  handler: async (args: string[], options: any, _context: CommandContext): Promise<void> => {
    const action = (args[0] || '').toLowerCase();
    const serviceName = (args[1] || '').toLowerCase();
    if (action !== 'show') {
      throw new Error('service command supports only: show');
    }
    if (serviceName !== 'session_v1') {
      throw new Error('supported service names: session_v1');
    }

    const configManager = ConfigManager.getInstance();
    const cfg = await configManager.applyOverrides({
      target: options.target,
      network: options.network,
    });

    const cluster = VmClusterConfigResolver.fromCliTarget(cfg.target as any);
    const profile = VmClusterConfigResolver.loadClusterConfig({ cluster });
    const derived = VmClusterConfigResolver.deriveVmAddresses(profile);
    const programId = new PublicKey(options.programId || profile.programId);
    const vmState = new PublicKey(options.vmState || derived.vmStatePda);
    const rpcUrl = cfg.networks[cfg.target].rpcUrl;
    const connection = new Connection(rpcUrl, 'confirmed');

    const accountInfo = await connection.getAccountInfo(vmState);
    if (!accountInfo) {
      throw new Error(`VM state account not found: ${vmState.toBase58()}`);
    }
    if (!accountInfo.owner.equals(programId)) {
      throw new Error('VM state owner does not match resolved 5IVE VM program ID');
    }

    const data = Buffer.from(accountInfo.data);
    const active = readServiceEntry(data, 0);
    const previous = readServiceEntry(data, 1);

    console.log(section('Service Registry'));
    console.log(keyValue('Cluster', cluster));
    console.log(keyValue('RPC', rpcUrl));
    console.log(keyValue('Program', programId.toBase58()));
    console.log(keyValue('VM state', vmState.toBase58()));
    console.log('');
    console.log(section('session_v1 (active)'));
    console.log(keyValue('Status', statusLabel(active.status)));
    console.log(keyValue('Script account', active.scriptAccount));
    console.log(keyValue('Code hash', active.codeHash));
    console.log(keyValue('Manager version', String(active.version)));
    console.log(keyValue('Epoch', active.epoch.toString()));
    console.log('');
    console.log(section('session_v1 (previous)'));
    console.log(keyValue('Status', statusLabel(previous.status)));
    console.log(keyValue('Script account', previous.scriptAccount));
    console.log(keyValue('Code hash', previous.codeHash));
    console.log(keyValue('Manager version', String(previous.version)));
    console.log(keyValue('Epoch', previous.epoch.toString()));
  },
};
