#!/usr/bin/env node

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  deployFiveVmScript,
  loadExplicitDeployEnv,
} from '../../scripts/lib/five-vm-deploy.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const artifactName = process.env.FIVE_ARTIFACT || 'five-amm-baseline.five';
const env = loadExplicitDeployEnv(path.join(__dirname, 'build', artifactName));

deployFiveVmScript({ ...env, label: 'amm template' })
  .then((result) => {
    console.log(`ammScriptAccount=${result.scriptAccount}`);
    console.log(`fiveProgramId=${result.fiveProgramId}`);
    console.log(`vmStatePda=${result.vmStatePda}`);
    console.log(`rpcUrl=${result.rpcUrl}`);
    console.log(`bytecodeLength=${result.bytecodeLength}`);
    console.log(`metadataLength=${result.metadataLength}`);
  })
  .catch((error) => {
    console.error(error.message || error);
    process.exit(1);
  });
