#!/usr/bin/env node
import fs from 'fs';
import path from 'path';
import { UserJourneyReporter, USER_JOURNEY_STEP_PREFIX } from './lib/user-journey-reporter.mjs';

function parseArgs(argv) {
  const args = {};
  for (let i = 2; i < argv.length; i += 1) {
    const key = argv[i];
    const value = argv[i + 1];
    if (key.startsWith('--')) {
      args[key.slice(2)] = value;
      i += 1;
    }
  }
  return args;
}

function parseStepEvents(logText) {
  const out = [];
  for (const line of logText.split('\n')) {
    if (!line.startsWith(USER_JOURNEY_STEP_PREFIX)) continue;
    const raw = line.slice(USER_JOURNEY_STEP_PREFIX.length);
    try {
      out.push(JSON.parse(raw));
    } catch {
      out.push({
        step: 'unparseable_step_event',
        status: 'FAIL',
        computeUnits: null,
        missingCuReason: 'invalid step event payload',
        failureClass: 'unknown',
      });
    }
  }
  return out;
}

function classifyError(logText, exitCode) {
  if (exitCode === 0) return null;
  if (/insufficient funds|insufficient lamports|debit an account/i.test(logText)) return 'funding';
  if (/signature verification failed|must sign|not signer|missing signature/i.test(logText)) return 'authority';
  if (/already initialized|already in use/i.test(logText)) return 'already_initialized';
  if (/script account not found|vm state not found|account not found on-chain/i.test(logText)) return 'account_fixture';
  if (/missing required account|account .* not provided/i.test(logText)) return 'missing_account';
  if (/duplicate/i.test(logText)) return 'duplicate_submit';
  if (/fetch failed|429|timeout|timed out|econnreset/i.test(logText)) return 'rpc';
  if (/program not found|invalid program argument/i.test(logText)) return 'program_id';
  return 'unknown';
}

function main() {
  const args = parseArgs(process.argv);
  const statusFile = args['status-file'];
  const resultsJson = args['results-json'];
  if (!statusFile || !resultsJson) {
    throw new Error('missing required --status-file and --results-json');
  }

  const reporter = new UserJourneyReporter({
    network: args.network,
    rpcUrl: args['rpc-url'],
    programId: args['program-id'],
    vmStatePda: args['vm-state'],
    keypairPath: args.keypair,
  });

  const lines = fs.readFileSync(statusFile, 'utf8').split('\n').map((line) => line.trim()).filter(Boolean);
  for (const line of lines) {
    const row = JSON.parse(line);
    reporter.startScenario(row.scenario, { command: row.command, log: row.log }, row.blocking !== false, row.family || 'token');
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
        failureClass: classifyError(logText, row.exit_code),
      });
    } else {
      for (const step of steps) reporter.recordStep(step);
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
  const logDir = path.join(path.dirname(resultsJson), 'logs');
  reporter.writeReport(base, logDir);
  const md = `${base}.md`;
  if (!fs.existsSync(path.dirname(resultsJson))) fs.mkdirSync(path.dirname(resultsJson), { recursive: true });
  console.log(`wrote ${resultsJson}`);
  console.log(`wrote ${md}`);
}

main();
