# FiveProgram Usage Guide

## Overview

FiveProgram provides a simple, type-safe interface for interacting with Five VM scripts on Solana. It handles all the complexity of parameter encoding, account configuration, and instruction building.

## Basic Setup

### 1. Load Your Script's ABI

```typescript
import { FiveProgram } from '@five-vm/sdk';
import fs from 'fs';

// Load the compiled ABI
const abi = JSON.parse(fs.readFileSync('counter.abi.json', 'utf-8'));
```

### 2. Create a FiveProgram Instance

```typescript
// Minimal configuration
const program = FiveProgram.fromABI(scriptAccountAddress, abi);

// Full configuration (recommended)
const program = FiveProgram.fromABI(scriptAccountAddress, abi, {
  // Five VM Program ID (required for on-chain execution)
  fiveVMProgramId: 'HzC7dhS3gbcTPoLmwSGFcTSnAqdDpdtERP5n5r9wyY4k',

  // VM State account (optional - will be derived if not provided)
  vmStateAccount: '1p5JMJ475unWyiCJuR96V4LewG1qbsS4J1Gzw6u6zGt',

  // Fee receiver/admin account (receives transaction fees)
  feeReceiverAccount: payerAddress.toBase58(),

  // Enable debug logging (optional)
  debug: false
});
```

## Account Configuration

### Three Required Accounts

1. **Five VM Program ID** - The deployed Five VM Solana program
   - Mainnet: `7wVkyXsUiRcZtAHGZcXbTPCXnE7DBP6juN35H6FEUUZo`
   - Devnet: Varies by deployment
   - Can be overridden via options

2. **VM State Account** - The Five VM state PDA
   - Stores global VM state and configuration
   - Can be derived automatically from program ID
   - Can be overridden if using custom derivation

3. **Fee Receiver Account** - Admin account for transaction fees
   - Receives fees from executed transactions
   - Usually the payer address
   - Required for proper fee collection

### Setting Accounts

#### At Initialization

```typescript
const program = FiveProgram.fromABI(scriptAccount, abi, {
  fiveVMProgramId: 'HzC7dhS3gbcTPoLmwSGFcTSnAqdDpdtERP5n5r9wyY4k',
  vmStateAccount: 'VMStatePDAAddress...',
  feeReceiverAccount: 'PayerAddress...'
});
```

#### Dynamically After Creation

```typescript
// Update VM State
program.setVMStateAccount('NewVMStatePDA...');

// Update Fee Receiver
program.setFeeReceiverAccount('NewPayerAddress...');

// Query current values
const vmState = program.getVMStateAccount();
const feeReceiver = program.getFeeReceiverAccount();
const programId = program.getFiveVMProgramId();
```

## Building Instructions

### Basic Function Call

```typescript
const instructionData = await program
  .function('increment')
  .accounts({
    counter: counterAccountAddress,
    owner: ownerAddress
  })
  .instruction();
```

### Function with Data Parameters

```typescript
const instructionData = await program
  .function('add_amount')
  .accounts({
    counter: counterAccountAddress,
    owner: ownerAddress
  })
  .args({
    amount: 100  // Data parameter
  })
  .instruction();
```

### Convert to TransactionInstruction

```typescript
import { TransactionInstruction, PublicKey } from '@solana/web3.js';

const ix = new TransactionInstruction({
  programId: new PublicKey(instructionData.programId),
  keys: instructionData.keys.map((key) => ({
    pubkey: new PublicKey(key.pubkey),
    isSigner: key.isSigner,
    isWritable: key.isWritable
  })),
  data: Buffer.from(instructionData.data, 'base64')
});
```

### Send Transaction

```typescript
import { Connection, Transaction } from '@solana/web3.js';

const connection = new Connection('http://localhost:8899');
const tx = new Transaction().add(ix);

const signature = await connection.sendTransaction(tx, [signer], {
  skipPreflight: true,
  maxRetries: 3
});

await connection.confirmTransaction(signature, 'confirmed');
```

## Complete Example

```typescript
import { FiveProgram } from '@five-vm/sdk';
import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction
} from '@solana/web3.js';
import fs from 'fs';

async function executeCounterIncrement() {
  // 1. Load ABI
  const abi = JSON.parse(
    fs.readFileSync('counter.abi.json', 'utf-8')
  );

  // 2. Setup connection and keypairs
  const connection = new Connection('http://localhost:8899');
  const payer = Keypair.fromSecretKey(/* secret key bytes */);

  // 3. Create FiveProgram with all accounts configured
  const program = FiveProgram.fromABI(
    'GozdrELSNrs2emihAKxVQtcHzvAjz6CZNeDF4vTxfWFm', // Script account
    abi,
    {
      fiveVMProgramId: 'HzC7dhS3gbcTPoLmwSGFcTSnAqdDpdtERP5n5r9wyY4k',
      vmStateAccount: '1p5JMJ475unWyiCJuR96V4LewG1qbsS4J1Gzw6u6zGt',
      feeReceiverAccount: payer.publicKey.toBase58()
    }
  );

  // 4. Build instruction
  const instructionData = await program
    .function('increment')
    .accounts({
      counter: 'CounterAccountAddress...',
      owner: payer.publicKey.toBase58()
    })
    .instruction();

  // 5. Convert to Solana instruction
  const ix = new TransactionInstruction({
    programId: new PublicKey(instructionData.programId),
    keys: instructionData.keys.map((key) => ({
      pubkey: new PublicKey(key.pubkey),
      isSigner: key.isSigner,
      isWritable: key.isWritable
    })),
    data: Buffer.from(instructionData.data, 'base64')
  });

  // 6. Send transaction
  const tx = new Transaction().add(ix);
  const signature = await connection.sendTransaction(tx, [payer], {
    skipPreflight: true
  });

  console.log('Transaction signature:', signature);
  await connection.confirmTransaction(signature, 'confirmed');
}

executeCounterIncrement().catch(console.error);
```

## Best Practices

### 1. Always Configure Fee Receiver
```typescript
// ✅ Good - Fee receiver configured
const program = FiveProgram.fromABI(scriptAccount, abi, {
  feeReceiverAccount: payer.publicKey.toBase58()
});

// ❌ Bad - No fee receiver
const program = FiveProgram.fromABI(scriptAccount, abi);
```

### 2. Use Configured Accounts at Initialization
```typescript
// ✅ Good - All accounts set at init
const program = FiveProgram.fromABI(scriptAccount, abi, {
  fiveVMProgramId: programId,
  vmStateAccount: vmState,
  feeReceiverAccount: payer.publicKey.toBase58()
});

// ⚠️ Less ideal - Dynamic updates
const program = FiveProgram.fromABI(scriptAccount, abi);
program.setVMStateAccount(vmState);
program.setFeeReceiverAccount(payer.publicKey.toBase58());
```

### 3. Handle Account Metadata
```typescript
// ✅ Good - Let FiveProgram infer from ABI
const ix = await program
  .function('initialize')
  .accounts({
    counter: counterAccount,
    owner: ownerAddress
    // SystemProgram auto-injected for @init
  })
  .instruction();

// ❌ Avoid - Manual account metadata
// FiveProgram handles this automatically
```

### 4. Error Handling
```typescript
try {
  const ix = await program
    .function('increment')
    .accounts({ counter, owner })
    .instruction();
} catch (error) {
  if (error instanceof Error) {
    console.error('Failed to build instruction:', error.message);
  }
}
```

## Troubleshooting

### "Function not found in ABI"
```typescript
// Check available functions
console.log(program.getFunctions());

// Verify function name matches exactly
const ix = await program
  .function('increment') // Must match ABI exactly
  .accounts({ counter, owner })
  .instruction();
```

### "Missing required account"
```typescript
// Ensure all function parameters are provided
const ix = await program
  .function('transfer')
  .accounts({
    from: fromAccount,      // Required
    to: toAccount,          // Required
    // Don't forget any accounts!
  })
  .instruction();
```

### "Missing required argument"
```typescript
// Provide all data parameters
const ix = await program
  .function('add_amount')
  .accounts({ counter, owner })
  .args({
    amount: 100  // Required data parameter
  })
  .instruction();
```

## API Reference

### FiveProgram

```typescript
class FiveProgram {
  // Factory methods
  static fromABI(scriptAccount: string, abi: ScriptABI, options?: FiveProgramOptions): FiveProgram
  static load(fetcher: AccountFetcher, scriptAccount: string, options?: FiveProgramOptions): Promise<FiveProgram>

  // Function access
  function(functionName: string): FunctionBuilder
  getFunctions(): string[]
  getFunction(name: string): FunctionDefinition | undefined
  getAllFunctions(): FunctionDefinition[]

  // Type generation
  generateTypes(): string

  // Account accessors
  getScriptAccount(): string
  getFiveVMProgramId(): string
  getVMStateAccount(): string | undefined
  getFeeReceiverAccount(): string | undefined

  // Account setters (fluent)
  setVMStateAccount(account: string): this
  setFeeReceiverAccount(account: string): this

  // Accessors
  getABI(): ScriptABI
  getOptions(): FiveProgramOptions
}
```

### FunctionBuilder

```typescript
class FunctionBuilder {
  // Fluent API
  accounts(accounts: Record<string, string | PublicKey>): this
  args(args: Record<string, any>): this

  // Generate instruction
  instruction(): Promise<SerializedInstruction>

  // Accessors
  getFunctionDef(): FunctionDefinition
  getAccounts(): Record<string, string>
  getArgs(): Record<string, any>
}
```

### SerializedInstruction

```typescript
interface SerializedInstruction {
  programId: string
  keys: Array<{ pubkey: string; isSigner: boolean; isWritable: boolean }>
  data: string // base64-encoded
}
```

## Integration with Your Solana Client

FiveProgram returns serialized instruction data that works with **any** Solana client library:

```typescript
// With @solana/web3.js
import { TransactionInstruction, PublicKey } from '@solana/web3.js';

const ix = new TransactionInstruction({
  programId: new PublicKey(instructionData.programId),
  keys: instructionData.keys.map(k => ({
    pubkey: new PublicKey(k.pubkey),
    isSigner: k.isSigner,
    isWritable: k.isWritable
  })),
  data: Buffer.from(instructionData.data, 'base64')
});

// With @project-serum/anchor
const ixFromData = (ixData) => {
  return new web3.TransactionInstruction({
    programId: new web3.PublicKey(ixData.programId),
    keys: ixData.keys.map(k => ({
      pubkey: new web3.PublicKey(k.pubkey),
      isSigner: k.isSigner,
      isWritable: k.isWritable
    })),
    data: Buffer.from(ixData.data, 'base64')
  });
};

// With custom RPC client
const customSend = async (programId, keys, data) => {
  // Your custom implementation
};
```

This zero-dependency design means FiveProgram works everywhere, and you choose your own Solana client library!
