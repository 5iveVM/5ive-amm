# Five SDK

Client-agnostic TypeScript SDK for interacting with 5ive DSL programs on Solana.

This README is for external developers using:
- `five-cli`
- `five-sdk`
- [5ive.tech](https://5ive.tech)

## Install

```bash
npm install five-sdk @solana/web3.js
```

Note: Some older examples may use `@five-vm/sdk`. Use the package name your registry currently publishes for your release channel.

## Quick Start

### 1. Compile with `five-cli` to `.five` artifact

```bash
five compile src/main.v -o build/my-program.five
```

### 1b. Compile directly with `five-sdk` (optional)

```ts
import { FiveSDK } from 'five-sdk';
import fs from 'fs';

const source = `
pub add(a: u64, b: u64) -> u64 {
  return a + b;
}
`;

const result = await FiveSDK.compile(source, {
  optimize: true,
  debug: false,
});

if (!result.success || !result.fiveFile) {
  throw new Error(`Compilation failed: ${JSON.stringify(result.errors ?? [])}`);
}

fs.writeFileSync('build/my-program.five', JSON.stringify(result.fiveFile, null, 2));
```

Compile from file path:

```ts
const fileResult = await FiveSDK.compileFile('src/main.v', { optimize: true });
```

Compile multi-file/module source:

```ts
const multi = await FiveSDK.compileModules(
  { filename: 'main.v', content: mainSource },
  [{ name: 'math', source: mathModuleSource }],
  { optimize: true }
);
```

### 2. Load ABI from `.five`

```ts
import fs from 'fs';
import { FiveSDK } from 'five-sdk';

const fiveFileText = fs.readFileSync('build/my-program.five', 'utf-8');
const { abi } = await FiveSDK.loadFiveFile(fiveFileText);
```

### 3. Configure program ID resolution

On-chain instruction generation requires a resolvable Five VM program ID.
Resolution precedence:
1. Explicit `fiveVMProgramId`
2. `FiveSDK.setDefaultProgramId(...)`
3. `FIVE_PROGRAM_ID` environment variable
4. Release-baked default

```ts
import { FiveSDK } from 'five-sdk';

FiveSDK.setDefaultProgramId('YourFiveVMProgramIdBase58');
```

### 4. Create `FiveProgram`

```ts
import { FiveProgram } from 'five-sdk';

const program = FiveProgram.fromABI('ScriptAccountBase58', abi, {
  fiveVMProgramId: 'YourFiveVMProgramIdBase58',
  vmStateAccount: 'VmStatePdaBase58',
  feeReceiverAccount: 'FeeReceiverBase58',
  debug: false,
});
```

## Build and Send Instructions

```ts
import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';

const connection = new Connection('http://127.0.0.1:8899', 'confirmed');

const ixData = await program
  .function('transfer')
  .accounts({
    source_account: sourcePubkey,
    destination_account: destinationPubkey,
    owner: ownerPubkey,
  })
  .args({ amount: 100 })
  .instruction();

const ix = new TransactionInstruction({
  programId: new PublicKey(ixData.programId),
  keys: ixData.keys.map((k) => ({
    pubkey: new PublicKey(k.pubkey),
    isSigner: k.isSigner,
    isWritable: k.isWritable,
  })),
  data: Buffer.from(ixData.data, 'base64'),
});

const tx = new Transaction().add(ix);
const signature = await sendAndConfirmTransaction(connection, tx, [payer], {
  skipPreflight: false,
  commitment: 'confirmed',
});

const txDetails = await connection.getTransaction(signature, {
  maxSupportedTransactionVersion: 0,
  commitment: 'confirmed',
});

if (txDetails?.meta?.err) {
  throw new Error(`On-chain error: ${JSON.stringify(txDetails.meta.err)}`);
}

console.log('signature', signature);
console.log('computeUnits', txDetails?.meta?.computeUnitsConsumed);
```

## Operational Patterns

### Function naming in multi-file programs

Function names must match ABI names exactly, including namespaced forms such as `math::add`.

```ts
console.log(program.getFunctions());
```

### Account wiring behavior

`FunctionBuilder` automatically handles:
- system-account injection when ABI constraints require it
- PDA account resolution from ABI metadata
- signer/writable metadata derivation from ABI attributes

You still must pass all required business accounts and arguments.

### Deterministic error handling

Recommended send pattern:
1. send with `skipPreflight: false`
2. fetch confirmed transaction
3. assert `meta.err` is null
4. record CU from `meta.computeUnitsConsumed`

## Advanced SDK APIs (Optional)

Most builders should use `FiveProgram` first. Use these lower-level APIs when you need finer control.

### 1) Local VM testing without RPC

```ts
const run = await FiveSDK.compileAndExecuteLocally(
  sourceCode,
  'transfer',
  [100],
  { optimize: true, trace: true, computeUnitLimit: 1_000_000 }
);
```

### 2) Instruction-only generation (bring your own sender)

```ts
const deploy = await FiveSDK.generateDeployInstruction(bytecode, deployerPubkeyBase58, {
  debug: false,
});

const exec = await FiveSDK.generateExecuteInstruction(
  scriptAccountBase58,
  'transfer',
  [100],
  [ownerPubkeyBase58],
  connection,
  { computeUnitLimit: 1_000_000 }
);
```

### 3) On-chain convenience helpers

```ts
const deployResult = await FiveSDK.deployToSolana(bytecode, connection, payerKeypair, {
  fiveVMProgramId: 'YourFiveVMProgramIdBase58',
});

const execResult = await FiveSDK.executeOnSolana(
  deployResult.scriptAccount,
  connection,
  payerKeypair,
  'transfer',
  [100],
  [ownerPubkeyBase58],
  { fiveVMProgramId: 'YourFiveVMProgramIdBase58' }
);
```

### 4) Metadata, decoding, and diagnostics

```ts
const meta = await FiveSDK.getScriptMetadataWithConnection(scriptAccountBase58, connection);
const names = await FiveSDK.getFunctionNamesFromScriptAccount(scriptAccountBase58, connection);
const account = await FiveSDK.fetchAccountAndDeserialize(scriptAccountBase58, connection, {
  parseMetadata: true,
});
```

### 5) Namespace helpers (5NS)

```ts
const ns = FiveSDK.canonicalizeNamespace('@acme/payments');
const derived = await FiveSDK.deriveNamespaceAccounts(ns.canonical, 'YourFiveVMProgramIdBase58');
```

### 6) Test utilities

```ts
import { FiveTestRunner } from 'five-sdk';

const runner = new FiveTestRunner({ verbose: true, maxComputeUnits: 1_000_000 });
```

## Frontend Usage (`5ive.tech`)

For frontend integration patterns and UI workflows, use [5ive.tech](https://5ive.tech).
Typical flow:
1. compile/deploy with `five-cli`
2. use `five-sdk` in your app/backend to generate instructions
3. submit signed transactions from wallet-enabled frontend

## Troubleshooting

### `No program ID resolved for Five VM`
Set one via explicit `fiveVMProgramId`, `FiveSDK.setDefaultProgramId`, or `FIVE_PROGRAM_ID`.

### `Function '<name>' not found in ABI`
Use exact ABI function names (including namespace prefixes).

### `Missing required account` / `Missing required argument`
Provide all required fields in `.accounts(...)` and `.args(...)`.

## Next Doc

For API-level usage details, see `FIVEPROGRAM_USAGE_GUIDE.md`.
