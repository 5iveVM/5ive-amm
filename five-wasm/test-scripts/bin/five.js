#!/usr/bin/env node

// Five CLI - Binary Entry Point
// This script forwards all calls to the compiled TypeScript entry point

import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

// Import and run the main CLI
const { main } = await import(join(__dirname, '..', '..', 'dist', 'index.js'));
await main();