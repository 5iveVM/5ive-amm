#!/usr/bin/env node

/**
 * Deploy token template to localnet using the robust chunked deploy flow.
 * This avoids transaction-size limits from single-transaction deploys.
 */

import path from 'path';
import { fileURLToPath } from 'url';
import { spawnSync } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const tokenTemplateDir = path.join(__dirname, '..', 'five-templates', 'token');
const deployScript = path.join(tokenTemplateDir, 'deploy-to-five-vm.mjs');

const env = {
  ...process.env,
  RPC_URL: process.env.RPC_URL || 'http://127.0.0.1:8899',
  FIVE_PROGRAM_ID: process.env.FIVE_PROGRAM_ID || '3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1',
  VM_STATE_PDA: process.env.VM_STATE_PDA || 'AJm3tpMgv9mXCWK2Sj9dZ2DxtUWXuQBXiK5HcYtHmKit',
};

const result = spawnSync('node', [deployScript, 'Token'], {
  cwd: tokenTemplateDir,
  env,
  stdio: 'inherit',
});

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}
