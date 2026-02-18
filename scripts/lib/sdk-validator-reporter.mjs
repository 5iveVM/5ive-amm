import fs from 'fs';
import path from 'path';

export const STEP_EVENT_PREFIX = 'SDK_VALIDATOR_STEP_JSON:';

function nowIso() {
  return new Date().toISOString();
}

export function emitStepEvent(step) {
  const payload = {
    timestamp: nowIso(),
    scenario: step.scenario || process.env.FIVE_SCENARIO || 'unknown',
    step: step.step || 'unnamed',
    status: step.status || 'UNKNOWN',
    signature: step.signature || null,
    computeUnits: Number.isFinite(step.computeUnits) ? Number(step.computeUnits) : null,
    missingCuReason: step.missingCuReason || null,
    error: step.error || null,
  };
  if (payload.computeUnits === null && !payload.missingCuReason) {
    payload.missingCuReason = 'compute units unavailable';
  }
  console.log(`${STEP_EVENT_PREFIX}${JSON.stringify(payload)}`);
}

export class SdkValidatorReporter {
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

  startScenario(name, details = {}) {
    this.current = {
      name,
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
    if (!this.current) {
      throw new Error('recordStep called before startScenario');
    }
    const row = {
      timestamp: nowIso(),
      scenario: this.current.name,
      step: step.step || 'unnamed',
      status: step.status || 'UNKNOWN',
      signature: step.signature || null,
      computeUnits: Number.isFinite(step.computeUnits) ? Number(step.computeUnits) : null,
      missingCuReason: step.missingCuReason || null,
      error: step.error || null,
    };
    if (row.computeUnits === null && !row.missingCuReason) {
      row.missingCuReason = 'compute units unavailable';
    }
    this.current.steps.push(row);
  }

  finalizeScenario(data = {}) {
    if (!this.current) {
      throw new Error('finalizeScenario called before startScenario');
    }
    this.current.finishedAt = nowIso();
    this.current.exitCode = Number.isInteger(data.exitCode) ? data.exitCode : 1;
    this.current.status = data.status || (this.current.exitCode === 0 ? 'PASS' : 'FAIL');
    if (data.error) this.current.errors.push(String(data.error));
    this.current = null;
  }

  toJSON() {
    const totals = { PASS: 0, FAIL: 0, PARTIAL: 0, SKIPPED: 0 };
    for (const s of this.scenarios) {
      totals[s.status] = (totals[s.status] || 0) + 1;
    }
    const allSteps = this.scenarios.flatMap((s) => s.steps);
    const cuValues = allSteps.map((s) => s.computeUnits).filter((n) => Number.isFinite(n));
    return {
      ...this.meta,
      finishedAt: nowIso(),
      totals,
      cu: {
        max: cuValues.length ? Math.max(...cuValues) : null,
        min: cuValues.length ? Math.min(...cuValues) : null,
      },
      scenarios: this.scenarios,
    };
  }

  writeReport(basePath) {
    const jsonPath = basePath.endsWith('.json') ? basePath : `${basePath}.json`;
    const mdPath = jsonPath.replace(/\.json$/, '.md');
    fs.mkdirSync(path.dirname(jsonPath), { recursive: true });
    const payload = this.toJSON();
    fs.writeFileSync(jsonPath, `${JSON.stringify(payload, null, 2)}\n`);
    fs.writeFileSync(mdPath, `${this.toMarkdown(payload)}\n`);
    return { jsonPath, mdPath };
  }

  toMarkdown(payload) {
    const lines = [];
    lines.push('# SDK Validator Suite Report');
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
    lines.push(`- CU max: ${payload.cu.max ?? 'n/a'}`);
    lines.push(`- CU min: ${payload.cu.min ?? 'n/a'}`);
    lines.push('');
    lines.push('## Scenarios');
    lines.push('| Scenario | Status | Exit | Steps |');
    lines.push('| --- | --- | --- | --- |');
    for (const s of payload.scenarios) {
      lines.push(`| ${s.name} | ${s.status} | ${s.exitCode ?? ''} | ${s.steps.length} |`);
    }
    lines.push('');
    lines.push('## Steps');
    lines.push('| Scenario | Step | Status | Signature | CU | Missing CU Reason |');
    lines.push('| --- | --- | --- | --- | --- | --- |');
    for (const s of payload.scenarios) {
      for (const st of s.steps) {
        lines.push(
          `| ${s.name} | ${st.step} | ${st.status} | ${st.signature || ''} | ${st.computeUnits ?? ''} | ${st.missingCuReason || ''} |`
        );
      }
    }
    return lines.join('\n');
  }
}

