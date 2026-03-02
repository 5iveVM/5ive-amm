#!/usr/bin/env node

import path from 'node:path';
import { fileURLToPath } from 'node:url';
import { loadFiveArtifact, splitDeployPayload } from './lib/five-vm-deploy.mjs';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

function inspect(label, artifactPath, expectImportMetadata) {
  const { bytecode } = loadFiveArtifact(artifactPath);
  const split = splitDeployPayload(bytecode);
  const ok = expectImportMetadata ? split.metadata.length > 0 : split.metadata.length === 0;

  console.log(
    `${label}: bytecode=${split.bytecode.length} metadata=${split.metadata.length} importMetadata=${split.hadImportMetadata}`,
  );

  if (!ok) {
    throw new Error(
      `${label} metadata expectation failed (expected import metadata ${expectImportMetadata ? 'present' : 'absent'})`,
    );
  }
}

try {
  inspect(
    'token',
    path.join(__dirname, '..', 'five-templates', 'token', 'build', 'five-token-template.five'),
    false,
  );
  inspect(
    '5ive-amm',
    path.join(__dirname, '..', '5ive-amm', 'build', '5ive-amm.five'),
    true,
  );
  inspect(
    '5ive-lending-2',
    path.join(__dirname, '..', '5ive-lending-2', 'build', '5ive-lending-2.five'),
    true,
  );
} catch (error) {
  console.error(error.message || error);
  process.exit(1);
}
