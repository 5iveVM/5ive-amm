#!/usr/bin/env node

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);

// Read the raw bytecode
const bytecodeFile = path.join(__dirname, 'src', 'token.bin');
const bytecodeRaw = fs.readFileSync(bytecodeFile);

// Convert to base64
const bytecodeBase64 = bytecodeRaw.toString('base64');

// Create the ABI with correct attributes for init_token_account
const abi = {
  "program_name": "Module",
  "functions": [
    {
      "name": "init_mint",
      "index": 0,
      "parameters": [
        {
          "name": "mint_account",
          "param_type": "Mint",
          "is_account": true,
          "attributes": ["mut", "init", "signer"]
        },
        {
          "name": "authority",
          "param_type": "Account",
          "is_account": true,
          "attributes": ["mut", "signer"]
        },
        {
          "name": "freeze_authority",
          "param_type": "pubkey",
          "is_account": false,
          "attributes": []
        },
        {
          "name": "decimals",
          "param_type": "u8",
          "is_account": false,
          "attributes": []
        },
        {
          "name": "name",
          "param_type": "string",
          "is_account": false,
          "attributes": []
        },
        {
          "name": "symbol",
          "param_type": "string",
          "is_account": false,
          "attributes": []
        },
        {
          "name": "uri",
          "param_type": "string",
          "is_account": false,
          "attributes": []
        }
      ],
      "return_type": "pubkey",
      "is_public": true,
      "bytecode_offset": 0
    },
    {
      "name": "init_token_account",
      "index": 1,
      "parameters": [
        {
          "name": "token_account",
          "param_type": "TokenAccount",
          "is_account": true,
          "attributes": ["mut", "init", "signer"]
        },
        {
          "name": "owner",
          "param_type": "Account",
          "is_account": true,
          "attributes": ["mut", "signer"]
        },
        {
          "name": "mint",
          "param_type": "pubkey",
          "is_account": false,
          "attributes": []
        }
      ],
      "return_type": "pubkey",
      "is_public": true,
      "bytecode_offset": 0
    },
    {
      "name": "mint_to",
      "index": 2,
      "parameters": [
        {
          "name": "mint_state",
          "param_type": "Mint",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "destination_account",
          "param_type": "TokenAccount",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "mint_authority",
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
      "name": "transfer",
      "index": 3,
      "parameters": [
        {
          "name": "source_account",
          "param_type": "TokenAccount",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "destination_account",
          "param_type": "TokenAccount",
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
      "name": "transfer_from",
      "index": 4,
      "parameters": [
        {
          "name": "source_account",
          "param_type": "TokenAccount",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "destination_account",
          "param_type": "TokenAccount",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "authority",
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
      "name": "approve",
      "index": 5,
      "parameters": [
        {
          "name": "source_account",
          "param_type": "TokenAccount",
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
          "name": "delegate",
          "param_type": "pubkey",
          "is_account": false,
          "attributes": []
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
      "name": "revoke",
      "index": 6,
      "parameters": [
        {
          "name": "source_account",
          "param_type": "TokenAccount",
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
      "name": "burn",
      "index": 7,
      "parameters": [
        {
          "name": "mint_state",
          "param_type": "Mint",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "source_account",
          "param_type": "TokenAccount",
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
      "name": "freeze_account",
      "index": 8,
      "parameters": [
        {
          "name": "mint_state",
          "param_type": "Mint",
          "is_account": true,
          "attributes": []
        },
        {
          "name": "account_to_freeze",
          "param_type": "TokenAccount",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "freeze_authority",
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
      "name": "thaw_account",
      "index": 9,
      "parameters": [
        {
          "name": "mint_state",
          "param_type": "Mint",
          "is_account": true,
          "attributes": []
        },
        {
          "name": "account_to_thaw",
          "param_type": "TokenAccount",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "freeze_authority",
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
      "name": "set_mint_authority",
      "index": 10,
      "parameters": [
        {
          "name": "mint_state",
          "param_type": "Mint",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "current_authority",
          "param_type": "Account",
          "is_account": true,
          "attributes": ["signer"]
        },
        {
          "name": "new_authority",
          "param_type": "pubkey",
          "is_account": false,
          "attributes": []
        }
      ],
      "return_type": null,
      "is_public": true,
      "bytecode_offset": 0
    },
    {
      "name": "set_freeze_authority",
      "index": 11,
      "parameters": [
        {
          "name": "mint_state",
          "param_type": "Mint",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "current_freeze_authority",
          "param_type": "Account",
          "is_account": true,
          "attributes": ["signer"]
        },
        {
          "name": "new_freeze_authority",
          "param_type": "pubkey",
          "is_account": false,
          "attributes": []
        }
      ],
      "return_type": null,
      "is_public": true,
      "bytecode_offset": 0
    },
    {
      "name": "disable_mint",
      "index": 12,
      "parameters": [
        {
          "name": "mint_state",
          "param_type": "Mint",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "current_authority",
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
      "name": "disable_freeze",
      "index": 13,
      "parameters": [
        {
          "name": "mint_state",
          "param_type": "Mint",
          "is_account": true,
          "attributes": ["mut"]
        },
        {
          "name": "current_freeze_authority",
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

// Write the JSON file to build directory
const outputFile = path.join(__dirname, 'build', 'five-token-template.five');
fs.writeFileSync(outputFile, JSON.stringify(fiveFile, null, 2));
console.log(`✅ Created five-token-template.five with ${bytecodeBase64.length} base64 bytes`);
console.log(`   ABI includes ${abi.functions.length} functions with updated attributes`);
