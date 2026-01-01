#!/usr/bin/env node

/**
 * Account Generation Helper for Test Runner
 * 
 * Generates AccountMeta JSON files for Five VM scripts that require accounts.
 * Used by test-runner.sh to provide proper account structures.
 */

import fs from 'fs/promises';
import path from 'path';
import crypto from 'crypto';

/**
 * Generate accounts file for a Five VM script
 * @param {string} scriptPath - Path to .v script file
 * @param {string} outputPath - Path to write accounts JSON file
 */
async function generateAccountsForScript(scriptPath, outputPath) {
  try {
    // Determine corresponding .five file path
    const baseDir = path.dirname(scriptPath);
    const baseName = path.basename(scriptPath, '.v');
    const fiveFilePath = path.join(baseDir, `${baseName}.five`);
    
    // Check if .five file exists
    try {
      await fs.access(fiveFilePath);
    } catch {
      console.error(`❌ .five file not found: ${fiveFilePath}`);
      console.error(`   Compile the script first: ./dist/index.js compile ${scriptPath} --output ${fiveFilePath}`);
      process.exit(1);
    }
    
    // Read and parse .five file
    const fiveContent = await fs.readFile(fiveFilePath, 'utf8');
    const fiveData = JSON.parse(fiveContent);
    
    if (!fiveData.abi || !fiveData.abi.functions) {
      console.error(`❌ Invalid .five file: missing ABI or functions`);
      process.exit(1);
    }
    
    // Find function with index 0 (test function)
    const functions = fiveData.abi.functions;
    const testFunction = Object.values(functions).find(f => f.index === 0);
    
    if (!testFunction) {
      console.error(`❌ No function with index 0 found`);
      process.exit(1);
    }
    
    const accounts = testFunction.accounts || [];
    
    if (accounts.length === 0) {
      console.log(`✅ No accounts required for ${baseName}`);
      // Create empty accounts file
      await fs.writeFile(outputPath, JSON.stringify({ accounts: [] }, null, 2));
      return;
    }
    
    console.log(`🔧 Generating accounts for ${baseName} (${accounts.length} required)`);
    
    // Generate AccountMeta for each required account
    const accountMetas = [];
    
    for (const [index, accountSpec] of accounts.entries()) {
      console.log(`   📄 Account ${index}: ${accountSpec.name} (signer: ${accountSpec.signer}, writable: ${accountSpec.writable})`);
      
      let accountMeta;
      
      if (accountSpec.signer) {
        // Generate keypair for signer accounts
        const keypair = await generateKeypair();
        accountMeta = {
          pubkey: keypair.publicKey,
          isSigner: true,
          isWritable: accountSpec.writable,
          // Include private key for signing (Five CLI format)
          privateKey: keypair.privateKey
        };
      } else {
        // Generate random address for non-signer accounts
        accountMeta = {
          pubkey: await generateRandomAddress(),
          isSigner: false,
          isWritable: accountSpec.writable
        };
      }
      
      accountMetas.push(accountMeta);
    }
    
    // Write accounts file in Five CLI format
    const accountsData = {
      accounts: accountMetas,
      metadata: {
        scriptPath: scriptPath,
        functionIndex: 0,
        generatedAt: new Date().toISOString()
      }
    };
    
    await fs.writeFile(outputPath, JSON.stringify(accountsData, null, 2));
    console.log(`✅ Generated accounts file: ${outputPath}`);
    
  } catch (error) {
    console.error(`❌ Error generating accounts: ${error.message}`);
    process.exit(1);
  }
}

/**
 * Generate Ed25519 keypair for signer accounts
 */
async function generateKeypair() {
  // Generate 32 random bytes for private key
  const privateKey = crypto.randomBytes(32);
  
  // For testing, we'll generate a random public key
  // In production, this would derive from the private key using Ed25519
  const publicKey = crypto.randomBytes(32);
  
  // Encode as base58
  const { default: bs58 } = await import('bs58');
  
  return {
    publicKey: bs58.encode(publicKey),
    privateKey: bs58.encode(privateKey)
  };
}

/**
 * Generate random Solana address
 */
async function generateRandomAddress() {
  const randomBytes = crypto.randomBytes(32);
  const { default: bs58 } = await import('bs58');
  return bs58.encode(randomBytes);
}

// Check if bs58 is available
async function ensureDependencies() {
  try {
    await import('bs58');
  } catch {
    console.error('❌ bs58 package not found. Please install it with: npm install bs58');
    process.exit(1);
  }
}

// Main execution
async function main() {
  const args = process.argv.slice(2);
  
  if (args.length < 2) {
    console.log('Usage: node generate-accounts-for-script.mjs <script.v> <accounts.json>');
    console.log('');
    console.log('Examples:');
    console.log('  node generate-accounts-for-script.mjs test-scripts/04-account-system/state-access.v accounts.json');
    process.exit(1);
  }
  
  const [scriptPath, outputPath] = args;
  
  await ensureDependencies();
  await generateAccountsForScript(scriptPath, outputPath);
}

if (import.meta.url === `file://${process.argv[1]}`) {
  await main();
}