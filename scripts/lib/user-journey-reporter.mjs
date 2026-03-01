import fs from 'fs';
import path from 'path';

export const USER_JOURNEY_STEP_PREFIX = 'USER_JOURNEY_STEP_JSON:';

function nowIso() {
  return new Date().toISOString();
}

export function emitUserJourneyStep(step) {
  const payload = {
    timestamp: nowIso(),
    scenario: step.scenario || process.env.FIVE_SCENARIO || 'unknown',
    step: step.step || 'unnamed',
    status: step.status || 'UNKNOWN',
    signature: step.signature || null,
    computeUnits: Number.isFinite(step.computeUnits) ? Number(step.computeUnits) : null,
    missingCuReason: step.missingCuReason || null,
    error: step.error || null,
    failureClass: step.failureClass || null,
  };
  if (payload.computeUnits === null && !payload.missingCuReason) {
    payload.missingCuReason = 'compute units unavailable';
  }
  console.log(`${USER_JOURNEY_STEP_PREFIX}${JSON.stringify(payload)}`);
}

export class UserJourneyReporter {
  constructor(meta = {}) {
    this.meta = {
      startedAt: nowIso(),
      network: meta.network || process.env.FIVE_NETWORK || 'unknown',
      rpcUrl: meta.rpcUrl || process.env.FIVE_RPC_URL || '',
      programId: meta.programId || process.env.FIVE_PROGRAM_ID || '',
      vmStatePda: meta.vmStatePda || process.env.VM_STATE_PDA || '',
      keypairPath: meta.keypairPath || process.env.FIVE_KEYPAIR_PATH || '',
    };
    this.scenarios = [];
    this.current = null;
  }

  startScenario(name, details = {}, blocking = true, family = 'token') {
    this.current = {
      name,
      family,
      blocking,
      details,
      startedAt: nowIso(),
      finishedAt: null,
      status: 'RUNNING',
      exitCode: null,
      steps: [],
      errors: [],
    };
    this.scenarios.push(this.current);
  }

  recordStep(step) {
    if (!this.current) throw new Error('recordStep called before startScenario');
    const row = {
      timestamp: nowIso(),
      scenario: this.current.name,
      step: step.step || 'unnamed',
      status: step.status || 'UNKNOWN',
      signature: step.signature || null,
      computeUnits: Number.isFinite(step.computeUnits) ? Number(step.computeUnits) : null,
      missingCuReason: step.missingCuReason || null,
      error: step.error || null,
      failureClass: step.failureClass || null,
    };
    if (row.computeUnits === null && !row.missingCuReason) {
      row.missingCuReason = 'compute units unavailable';
    }
    this.current.steps.push(row);
  }

  finalizeScenario(data = {}) {
    if (!this.current) throw new Error('finalizeScenario called before startScenario');
    this.current.finishedAt = nowIso();
    this.current.exitCode = Number.isInteger(data.exitCode) ? data.exitCode : 1;
    this.current.status = data.status || (this.current.exitCode === 0 ? 'PASS' : 'FAIL');
    if (data.error) this.current.errors.push(String(data.error));
    this.current = null;
  }

  toJSON(logDir = '') {
    const totals = { PASS: 0, FAIL: 0, PARTIAL: 0, SKIPPED: 0 };
    for (const scenario of this.scenarios) {
      totals[scenario.status] = (totals[scenario.status] || 0) + 1;
    }
    const allGreen = this.scenarios
      .filter((scenario) => scenario.blocking)
      .every((scenario) => scenario.status === 'PASS');
    return {
      ...this.meta,
      finishedAt: nowIso(),
      totals,
      allGreen,
      artifacts: {
        logDir,
      },
      scenarios: this.scenarios,
    };
  }

  writeReport(basePath, logDir = '') {
    const jsonPath = basePath.endsWith('.json') ? basePath : `${basePath}.json`;
    const mdPath = jsonPath.replace(/\.json$/, '.md');
    fs.mkdirSync(path.dirname(jsonPath), { recursive: true });
    const payload = this.toJSON(logDir);
    fs.writeFileSync(jsonPath, `${JSON.stringify(payload, null, 2)}\n`);
    fs.writeFileSync(mdPath, `${this.toMarkdown(payload)}\n`);
    return { jsonPath, mdPath };
  }

  toMarkdown(payload) {
    const lines = [];
    lines.push('# User Journey Report');
    lines.push('');
    lines.push('## Environment');
    lines.push(`- Network: ${payload.network}`);
    lines.push(`- RPC URL: ${payload.rpcUrl || 'n/a'}`);
    lines.push(`- Program ID: ${payload.programId || 'n/a'}`);
    lines.push(`- VM State PDA: ${payload.vmStatePda || 'n/a'}`);
    lines.push('');
    lines.push('## Summary');
    lines.push(`- PASS: ${payload.totals.PASS || 0}`);
    lines.push(`- FAIL: ${payload.totals.FAIL || 0}`);
    lines.push(`- PARTIAL: ${payload.totals.PARTIAL || 0}`);
    lines.push(`- SKIPPED: ${payload.totals.SKIPPED || 0}`);
    lines.push(`- Blocking all green: ${payload.allGreen ? 'yes' : 'no'}`);
    lines.push('');
    lines.push('## Scenarios');
    lines.push('| Scenario | Family | Blocking | Status | Exit | Steps |');
    lines.push('| --- | --- | --- | --- | --- | --- |');
    for (const scenario of payload.scenarios) {
      lines.push(`| ${scenario.name} | ${scenario.family || 'token'} | ${scenario.blocking ? 'yes' : 'no'} | ${scenario.status} | ${scenario.exitCode ?? ''} | ${scenario.steps.length} |`);
    }
    lines.push('');
    lines.push('## Steps');
    lines.push('| Scenario | Step | Status | Failure Class | Signature | CU |');
    lines.push('| --- | --- | --- | --- | --- | --- |');
    for (const scenario of payload.scenarios) {
      for (const step of scenario.steps) {
        lines.push(
          `| ${scenario.name} | ${step.step} | ${step.status} | ${step.failureClass || ''} | ${step.signature || ''} | ${step.computeUnits ?? ''} |`
        );
      }
    }
    return lines.join('\n');
  }
}
