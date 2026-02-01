#!/usr/bin/env node
import { execSync } from 'child_process';

// Set environment variables from deployment config
process.env.FIVE_PROGRAM_ID = '6ndNfSrrGoFfTbS1sdJFybuJJyA6YQHtNgRdoXFREi8k';
process.env.RPC_URL = 'http://127.0.0.1:8899';

// Run the e2e test
console.log('\n╔════════════════════════════════════════════════════════════╗');
console.log('║     Running E2E Token Tests with Existing Deployment      ║');
console.log('╚════════════════════════════════════════════════════════════╝\n');

console.log('Configuration:');
console.log('  Token Script: CwV1etYkM7MvPoZTKhZZeho9pFmNp39PZFHeQxLsvQDo');
console.log('  Program ID:   6ndNfSrrGoFfTbS1sdJFybuJJyA6YQHtNgRdoXFREi8k');
console.log('  VM State:     G1m84QHbcUvJb5JDM7RYCPLxaSAwAVDayj7tkNPYKLRr');
console.log('  RPC:          http://127.0.0.1:8899\n');

try {
  execSync('node e2e-token-test.mjs', {
    cwd: process.cwd(),
    env: { ...process.env },
    stdio: 'inherit',
    timeout: 180000
  });
  console.log('\n✓ E2E tests completed');
} catch (error) {
  console.error('\n✗ E2E tests failed:', error.message);
  process.exit(1);
}
