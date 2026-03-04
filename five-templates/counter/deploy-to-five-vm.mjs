import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import {
  deployFiveVmScript,
  loadExplicitDeployEnv,
} from '../../scripts/lib/five-vm-deploy.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const env = loadExplicitDeployEnv(path.join(__dirname, 'build', 'five-counter-template.five'));

deployFiveVmScript({ ...env, label: 'counter template' })
  .then((result) => {
    const config = {
      counterScriptAccount: result.scriptAccount,
      fiveProgramId: result.fiveProgramId,
      vmStatePda: result.vmStatePda,
      rpcUrl: result.rpcUrl,
      timestamp: new Date().toISOString(),
    };
    fs.writeFileSync(path.join(__dirname, 'deployment-config.json'), JSON.stringify(config, null, 2));

    console.log(`counterScriptAccount=${result.scriptAccount}`);
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
