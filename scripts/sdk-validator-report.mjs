#!/usr/bin/env node
import fs from 'fs';
import path from 'path';
import { SdkValidatorReporter, STEP_EVENT_PREFIX } from './lib/sdk-validator-reporter.mjs';

function parseArgs(argv) {
  const args = {};
  for (let i = 2; i < argv.length; i += 1) {
    const k = argv[i];
    const v = argv[i + 1];
    if (k.startsWith('--')) {
      args[k.slice(2)] = v;
      i += 1;
    }
  }
  return args;
}

function parseStepEvents(logText) {
  const out = [];
  for (const line of logText.split('\n')) {
    if (!line.startsWith(STEP_EVENT_PREFIX)) continue;
    const raw = line.slice(STEP_EVENT_PREFIX.length);
    try {
      out.push(JSON.parse(raw));
    } catch {
      out.push({
        step: 'unparseable_step_event',
        status: 'FAIL',
        computeUnits: null,
        missingCuReason: 'invalid step event payload',
      });
    }
  }
  return out;
}

function classifyError(logText, exitCode) {
  if (exitCode === 0) return null;
  if (/program not found/i.test(logText)) return 'Program/account ownership mismatch';
  if (/InvalidInstructionData/i.test(logText)) return 'InvalidInstructionData/encoding mismatch';
  if (/insufficient funds|airdrop|lamports/i.test(logText)) return 'Insufficient funds';
  if (/timeout|timed out|429|ECONNRESET|fetch failed/i.test(logText)) return 'RPC/rate-limit/intermittent network';
  if (/error TS\d+|TypeScript/i.test(logText)) return 'Build/type-check failure';
  if (/testCases is not iterable|Failed to parse test file/i.test(logText)) return 'Missing fixtures/accounts';
  return 'Unknown';
}

function main() {
  const args = parseArgs(process.argv);
  const statusFile = args['status-file'];
  const resultsJson = args['results-json'];
  if (!statusFile || !resultsJson) {
    throw new Error('missing required --status-file and --results-json');
  }

  const reporter = new SdkValidatorReporter({
    network: args.network,
    rpcUrl: args['rpc-url'],
    programId: args['program-id'],
    vmStatePda: args['vm-state'],
    keypairPath: args.keypair,
  });

  const lines = fs.readFileSync(statusFile, 'utf8').split('\n').map((s) => s.trim()).filter(Boolean);
  for (const line of lines) {
    const row = JSON.parse(line);
    reporter.startScenario(row.scenario, { command: row.command, log: row.log });
    let logText = '';
    if (row.log && fs.existsSync(row.log)) logText = fs.readFileSync(row.log, 'utf8');
    const steps = parseStepEvents(logText);
    if (steps.length === 0) {
      reporter.recordStep({
        step: 'scenario_command',
        status: row.status,
        computeUnits: null,
        missingCuReason: row.status === 'PASS' ? 'no step events emitted' : 'scenario failed before step emission',
        error: classifyError(logText, row.exit_code),
      });
    } else {
      for (const s of steps) reporter.recordStep(s);
    }
    const finalStatus = row.status === 'PASS'
      ? 'PASS'
      : (steps.length > 0 ? 'PARTIAL' : 'FAIL');
    reporter.finalizeScenario({
      status: finalStatus,
      exitCode: row.exit_code,
      error: classifyError(logText, row.exit_code),
    });
  }

  const base = resultsJson.replace(/\.json$/, '');
  reporter.writeReport(base);
  const md = `${base}.md`;
  if (!fs.existsSync(path.dirname(resultsJson))) fs.mkdirSync(path.dirname(resultsJson), { recursive: true });
  console.log(`wrote ${resultsJson}`);
  console.log(`wrote ${md}`);
}

main();

