#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Read the raw bytecode
const bytecodeFile = path.join(__dirname, 'build', 'five-counter-template.five');
const bytecodeRaw = fs.readFileSync(bytecodeFile);

// Convert to base64
const bytecodeBase64 = bytecodeRaw.toString('base64');

// Create the ABI - this is the same as before
const abi = {
  "program_name": "Module",
  "functions": [
    {
      "name": "initialize",
      "index": 0,
      "parameters": [
        {
          "name": "counter",
          "param_type": "Counter",
          "is_account": true,
          "attributes": ["mut", "init", "signer"]
        },
        {
          "name": "owner",
          "param_type": "Account",
          "is_account": true,
          "attributes": ["signer"]
        }
      ],
      "return_type": "pubkey",
      "is_public": true,
      "bytecode_offset": 0
    },
    {
      "name": "increment",
      "index": 1,
      "parameters": [
        {
          "name": "counter",
          "param_type": "Counter",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "owner",
          "param_type": "Account",
          "is_account": true,
          "attributes": ["signer"]
        }
      ],
      "return_type": null,
      "is_public": true,
      "bytecode_offset": 0
    },
    {
      "name": "decrement",
      "index": 2,
      "parameters": [
        {
          "name": "counter",
          "param_type": "Counter",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "owner",
          "param_type": "Account",
          "is_account": true,
          "attributes": ["signer"]
        }
      ],
      "return_type": null,
      "is_public": true,
      "bytecode_offset": 0
    },
    {
      "name": "add_amount",
      "index": 3,
      "parameters": [
        {
          "name": "counter",
          "param_type": "Counter",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "owner",
          "param_type": "Account",
          "is_account": true,
          "attributes": ["signer"]
        },
        {
          "name": "amount",
          "param_type": "u64",
          "is_account": false,
          "attributes": []
        }
      ],
      "return_type": null,
      "is_public": true,
      "bytecode_offset": 0
    },
    {
      "name": "get_count",
      "index": 4,
      "parameters": [
        {
          "name": "counter",
          "param_type": "Counter",
          "is_account": true,
          "attributes": []
        }
      ],
      "return_type": "u64",
      "is_public": true,
      "bytecode_offset": 0
    },
    {
      "name": "reset",
      "index": 5,
      "parameters": [
        {
          "name": "counter",
          "param_type": "Counter",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "owner",
          "param_type": "Account",
          "is_account": true,
          "attributes": ["signer"]
        }
      ],
      "return_type": null,
      "is_public": true,
      "bytecode_offset": 0
    }
  ],
  "fields": [],
  "version": "1.0"
};

// Create the five file
const fiveFile = {
  bytecode: bytecodeBase64,
  abi: abi,
  version: "1.0",
  metadata: {}
};

// Write the JSON file
fs.writeFileSync(bytecodeFile, JSON.stringify(fiveFile, null, 2));
console.log(`✅ Created five-counter-template.five with ${bytecodeBase64.length} base64 bytes`);
