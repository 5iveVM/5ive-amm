#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

const ARTIFACT_PATH = path.join(__dirname, 'five-templates/token/build/five-token-registers.five');
const DEPLOY_SCRIPT = path.join(__dirname, 'five-templates/token/deploy-to-five-vm.mjs');

// Check if artifact exists
if (!fs.existsSync(ARTIFACT_PATH)) {
    console.error(`❌ Register-optimized artifact not found: ${ARTIFACT_PATH}`);
    console.error('Run: cd five-templates/token && node create-artifact-registers.js');
    process.exit(1);
}

console.log('🚀 Deploying register-optimized token bytecode...\n');

// Temporarily backup and replace the bytecode file path in deploy script
const deployContent = fs.readFileSync(DEPLOY_SCRIPT, 'utf-8');
const modifiedContent = deployContent.replace(
    "path.join(__dirname, 'build/five-token-template.five')",
    "path.join(__dirname, 'build/five-token-registers.five')"
);

// Create temporary deploy script
const tempDeployScript = path.join(__dirname, 'five-templates/token/deploy-registers-temp.mjs');
fs.writeFileSync(tempDeployScript, modifiedContent);

try {
    // Run the modified deploy script
    execSync(`node ${tempDeployScript}`, { cwd: path.join(__dirname, 'five-templates/token'), stdio: 'inherit' });
    console.log('\n✅ Deployment complete!');
} catch (error) {
    console.error('\n❌ Deployment failed!');
    process.exit(1);
} finally {
    // Clean up temporary script
    if (fs.existsSync(tempDeployScript)) {
        fs.unlinkSync(tempDeployScript);
    }
}
