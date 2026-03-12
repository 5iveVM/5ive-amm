#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';

const required = [
  'dist/extension.js',
  'language-configuration.json',
  'syntaxes/five.tmLanguage.json',
  'package.json',
];

for (const entry of required) {
  const target = path.join(process.cwd(), entry);
  if (!fs.existsSync(target)) {
    console.error(`Missing required file: ${entry}`);
    process.exit(1);
  }
}

console.log('five-vscode smoke check passed');
