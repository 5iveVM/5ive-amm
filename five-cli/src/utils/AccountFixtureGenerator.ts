// Account fixture generator for test runner scripts.

import * as fs from 'fs';
import * as path from 'path';
import { AccountTestFixture, FixtureTemplates } from '@5ive-tech/sdk';

function detectAccountPattern(scriptContent: string): string | null {
  // Look for account definitions in the script
  const hasSignerConstraint = /@signer/g.test(scriptContent);
  const hasMutConstraint = /@mut/g.test(scriptContent);
  const hasInitConstraint = /@init/g.test(scriptContent);

  // Count signers and mutable accounts
  const signerMatches = scriptContent.match(/account\s+@signer/g) || [];
  const mutMatches = scriptContent.match(/account\s+@mut/g) || [];

  // Detect pattern
  if (hasInitConstraint && hasSignerConstraint) {
    return 'account-creation';
  }

  if (signerMatches.length >= 2) {
    return 'multi-sig';
  }

  if (hasSignerConstraint && hasMutConstraint) {
    return 'authorization';
  }

  if (hasMutConstraint && signerMatches.length === 0) {
    return 'state-mutation';
  }

  if (hasSignerConstraint) {
    return 'authorization';
  }

  // Default: try to identify from filename
  const fileName = path.basename(scriptContent);
  if (fileName.includes('signer')) return 'authorization';
  if (fileName.includes('mut')) return 'state-mutation';
  if (fileName.includes('init')) return 'account-creation';

  return null;
}

function parseAccountDefinitions(scriptContent: string): Map<string, any> {
  const accounts = new Map<string, any>();

  // Regex to find account definitions
  const accountRegex = /account\s+(\w+)\s*\{([^}]*)\}/g;
  let match;

  while ((match = accountRegex.exec(scriptContent)) !== null) {
    const accountName = match[1];
    const accountFields = match[2];

    // Parse fields
    const fields: any = {};
    const fieldRegex = /(\w+)\s*:\s*(\w+)/g;
    let fieldMatch;

    while ((fieldMatch = fieldRegex.exec(accountFields)) !== null) {
      const fieldName = fieldMatch[1];
      const fieldType = fieldMatch[2];

      // Set default values based on type
      if (fieldType === 'u64') {
        fields[fieldName] = 0;
      } else if (fieldType === 'pubkey') {
        fields[fieldName] = '11111111111111111111111111111111';
      } else if (fieldType === 'bool') {
        fields[fieldName] = false;
      } else if (fieldType.startsWith('Option')) {
        fields[fieldName] = null;
      } else {
        fields[fieldName] = null;
      }
    }

    accounts.set(accountName, fields);
  }

  return accounts;
}

function parseFunctionSignatures(scriptContent: string): Map<string, string[]> {
  const functions = new Map<string, string[]>();

  // Regex to find function definitions with account parameters
  const funcRegex = /pub\s+(\w+)\s*\(([^)]*)\)/g;
  let match;

  while ((match = funcRegex.exec(scriptContent)) !== null) {
    const funcName = match[1];
    const paramsStr = match[2];

    // Extract account parameter names and constraints
    const accountParams: string[] = [];
    const paramRegex = /(\w+)\s*:\s*account\s*(@\w+)?/g;
    let paramMatch;

    while ((paramMatch = paramRegex.exec(paramsStr)) !== null) {
      const paramName = paramMatch[1];
      const constraint = paramMatch[2] || '';
      accountParams.push(`${paramName}${constraint}`);
    }

    functions.set(funcName, accountParams);
  }

  return functions;
}

export async function generateFixtureForScript(
  scriptPath: string,
  options: {
    debug?: boolean;
    mode?: 'local' | 'onchain';
    connection?: any;
    payer?: any;
  } = {}
): Promise<{
  fixture: any;
  accountAddresses: string[];
  accountNames: string[];
  pattern: string | null;
}> {
  // Read script content
  if (!fs.existsSync(scriptPath)) {
    throw new Error(`Script not found: ${scriptPath}`);
  }

  const scriptContent = fs.readFileSync(scriptPath, 'utf-8');

  // Detect pattern
  const pattern = detectAccountPattern(scriptContent);
  if (options.debug) {
    console.log(`[AccountFixtureGenerator] Detected pattern: ${pattern}, mode: ${options.mode || 'local'}`);
  }

  // Use template if detected
  let fixture;
  const buildOptions = {
    debug: options.debug,
    mode: options.mode || 'local',
    connection: options.connection,
    payer: options.payer,
    cleanup: true  // Auto-cleanup after test
  };

  if (pattern) {
    switch (pattern) {
      case 'state-mutation':
        fixture = await FixtureTemplates.stateCounter().build(buildOptions);
        break;
      case 'authorization':
        fixture = await FixtureTemplates.authorization().build(buildOptions);
        break;
      case 'account-creation':
        fixture = await FixtureTemplates.accountCreation().build(buildOptions);
        break;
      case 'multi-sig':
        fixture = await FixtureTemplates.multiSigPattern().build(buildOptions);
        break;
      default:
        fixture = await FixtureTemplates.stateCounter().build(buildOptions);
    }
  } else {
    // Fallback: create minimal fixture
    fixture = await new AccountTestFixture()
      .addStateAccount('state')
      .build(buildOptions);
  }

  // Extract account addresses and names
  const accountAddresses = fixture.accounts.map((a: any) => a.pubkey);
  const accountNames = fixture.accounts.map((a: any) => a.name || 'unknown');

  if (options.debug) {
    const modeStr = options.mode === 'onchain' ? '[ON-CHAIN]' : '[LOCAL]';
    console.log(`[AccountFixtureGenerator] ${modeStr} Generated ${accountAddresses.length} accounts`);
    accountNames.forEach((name, i) => {
      console.log(`  ${i}: ${name} -> ${accountAddresses[i].substring(0, 8)}...`);
    });
  }

  return {
    fixture,
    accountAddresses,
    accountNames,
    pattern
  };
}

/**
 * Generate fixture and return as CLI-compatible format
 */
export async function generateFixtureForCLI(
  scriptPath: string,
  options: {
    debug?: boolean;
    mode?: 'local' | 'onchain';
    connection?: any;
    payer?: any;
  } = {}
): Promise<{
  accounts: string;  // Comma-separated pubkeys for --accounts flag
  keypairs: Array<{ name: string; secret: string }>;  // For signing
  summary: string;   // Human-readable summary
  fixture: any;      // Raw fixture for cleanup
  cleanup?: () => Promise<void>;  // Optional cleanup function
}> {
  const { fixture, accountAddresses, accountNames, pattern } = await generateFixtureForScript(
    scriptPath,
    {
      debug: options.debug,
      mode: options.mode || 'local',
      connection: options.connection,
      payer: options.payer
    }
  );

  // Format accounts as comma-separated string
  const accounts = accountAddresses.join(',');

  // Extract keypairs for signers
  const keypairs: Array<{ name: string; secret: string }> = [];
  fixture.accounts.forEach((acc: any, i: number) => {
    if (acc.keypair) {
      const secretArray = Array.from(acc.keypair.secretKey);
      keypairs.push({
        name: accountNames[i],
        secret: JSON.stringify(secretArray)
      });
    }
  });

  // Create summary
  const modeStr = options.mode === 'onchain' ? '[ON-CHAIN]' : '[LOCAL]';
  const summary = [
    `${modeStr} Fixture for: ${path.basename(scriptPath)}`,
    `Pattern: ${pattern || 'custom'}`,
    `Accounts: ${accountAddresses.length}`,
    accountNames.map((name, i) => `  ${i}: ${name} [${accountAddresses[i].substring(0, 8)}...]`).join('\n')
  ].join('\n');

  return {
    accounts,
    keypairs,
    summary,
    fixture,
    cleanup: fixture.cleanup
  };
}

/**
 * Check if a script is an account-system test
 */
export function isAccountSystemScript(scriptPath: string): boolean {
  if (!fs.existsSync(scriptPath)) return false;

  const scriptContent = fs.readFileSync(scriptPath, 'utf-8');

  // Check for account definitions or constraints
  return (
    /account\s+\w+\s*\{/g.test(scriptContent) ||
    /@signer/g.test(scriptContent) ||
    /@mut/g.test(scriptContent) ||
    /@init/g.test(scriptContent)
  );
}

/**
 * Generate fixtures for all scripts in a directory
 */
export async function generateFixturesForDirectory(
  directory: string,
  options: { debug?: boolean; filter?: (name: string) => boolean } = {}
): Promise<
  Map<
    string,
    {
      accounts: string;
      keypairs: Array<{ name: string; secret: string }>;
      summary: string;
    }
  >
> {
  const results = new Map();

  // Find all .v files
  const files = fs.readdirSync(directory).filter(f => f.endsWith('.v'));

  for (const file of files) {
    if (options.filter && !options.filter(file)) continue;

    const scriptPath = path.join(directory, file);

    try {
      if (isAccountSystemScript(scriptPath)) {
        const result = await generateFixtureForCLI(scriptPath, options);
        results.set(file, result);

        if (options.debug) {
          console.log(`\n${result.summary}\n`);
        }
      }
    } catch (error) {
      console.error(`Error generating fixture for ${file}: ${error}`);
    }
  }

  return results;
}

/**
 * Main entry point when run as CLI tool
 */
async function main() {
  const args = process.argv.slice(2);
  const scriptPath = args[0];

  if (!scriptPath) {
    console.error('Usage: node AccountFixtureGenerator.ts <script-path>');
    process.exit(1);
  }

  try {
    const result = await generateFixtureForCLI(scriptPath, { debug: true });

    console.log('\n' + result.summary);
    console.log(`\nAccounts: ${result.accounts}`);

    if (result.keypairs.length > 0) {
      console.log('\nSigner Keypairs:');
      result.keypairs.forEach(kp => {
        console.log(`  ${kp.name}: ${kp.secret.substring(0, 50)}...`);
      });
    }
  } catch (error) {
    console.error(`Error: ${error}`);
    process.exit(1);
  }
}

// Run if invoked directly
if (import.meta.url === `file://${process.argv[1]}`) {
  main();
}
