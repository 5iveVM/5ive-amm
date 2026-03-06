import fs from 'fs';
import path from 'path';

const STALE_PROGRAM_IDS = new Set([
  '9MHGM73eszNUtmJS6ypDCESguxWhCBnkUPpTMyLGqURH',
]);

function fail(msg) {
  throw new Error(`[sdk-validator-config] ${msg}`);
}

export function resolveNetwork(networkRaw = process.env.FIVE_NETWORK || 'localnet') {
  const network = String(networkRaw).trim();
  if (!['localnet', 'devnet', 'mainnet'].includes(network)) {
    fail(`invalid network "${network}" (expected localnet|devnet|mainnet)`);
  }
  return network;
}

export function defaultRpcUrlForNetwork(network) {
  if (network === 'localnet') return 'http://127.0.0.1:8899';
  if (network === 'devnet') return 'https://api.devnet.solana.com';
  if (network === 'mainnet') return 'https://api.mainnet-beta.solana.com';
  fail(`no RPC default for network "${network}"`);
}

export function loadSdkValidatorConfig(opts = {}) {
  const network = resolveNetwork(opts.network);
  const rpcUrl = opts.rpcUrl || process.env.FIVE_RPC_URL || defaultRpcUrlForNetwork(network);
  const keypairPath = opts.keypairPath || process.env.FIVE_KEYPAIR_PATH || path.join(process.env.HOME || '', '.config/solana/id.json');
  const programId = opts.programId || process.env.FIVE_PROGRAM_ID || '';
  const vmStatePda = opts.vmState || process.env.VM_STATE_PDA || '';
  const resultsFile = opts.resultsFile || process.env.FIVE_RESULTS_FILE || '';
  const scenariosRaw = opts.scenarios || process.env.FIVE_SCENARIOS || '';
  const scenarios = scenariosRaw
    ? scenariosRaw.split(',').map((s) => s.trim()).filter(Boolean)
    : [];

  if (!rpcUrl) fail('missing FIVE_RPC_URL');
  if (!keypairPath) fail('missing FIVE_KEYPAIR_PATH');
  if (!programId) fail('missing FIVE_PROGRAM_ID');
  if (!fs.existsSync(keypairPath)) fail(`keypair path not found: ${keypairPath}`);

  const allowStale = opts.allowStaleProgramId === true || process.env.FIVE_ALLOW_STALE_PROGRAM_ID === '1';
  if (!allowStale && STALE_PROGRAM_IDS.has(programId)) {
    fail(
      `program id ${programId} is blocked as stale. Set FIVE_PROGRAM_ID explicitly to an active deployment (or set FIVE_ALLOW_STALE_PROGRAM_ID=1).`
    );
  }

  return {
    network,
    rpcUrl,
    keypairPath,
    programId,
    vmStatePda,
    resultsFile,
    scenarios,
  };
}
