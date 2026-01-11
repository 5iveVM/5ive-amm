#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { TypeGenerator } from '../program/TypeGenerator.js';

async function main() {
    const args = process.argv.slice(2);

    if (args.length < 1) {
        console.error('Usage: five-gen-types <abi-file.json> [output-file.ts]');
        process.exit(1);
    }

    const abiPath = args[0];
    const outputPath = args[1] || abiPath.replace('.json', '.d.ts');

    // Read ABI
    const abiContent = fs.readFileSync(abiPath, 'utf8');
    let abi;
    try {
        abi = JSON.parse(abiContent);
    } catch (e) {
        console.error(`Error parsing ABI file ${abiPath}:`, e);
        process.exit(1);
    }

    // Generate Types
    const generator = new TypeGenerator(abi, {
        scriptName: abi.name,
        includeJSDoc: true
    });

    const code = generator.generate();

    // Write output
    fs.writeFileSync(outputPath, code);
    console.log(`Generated types to: ${outputPath}`);
}

main().catch(err => {
    console.error("Error:", err);
    process.exit(1);
});
