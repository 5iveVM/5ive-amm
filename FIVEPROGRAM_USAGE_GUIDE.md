# FiveProgram Usage Guide

Detailed guide for developers building 5ive DSL contracts and interacting with deployed script accounts via `five-sdk`.

## Preconditions

1. You have a compiled `.five` artifact from `five-cli`.
2. You have a deployed script account address.
3. You have a resolvable Five VM program ID.

## Program ID Requirement

On-chain operations require a resolved Five VM program ID.
Resolution precedence:
1. Explicit `fiveVMProgramId`
2. `FiveSDK.setDefaultProgramId(...)`
3. `FIVE_PROGRAM_ID` env var
4. Release-baked default

```ts
import { FiveSDK } from 'five-sdk';

FiveSDK.setDefaultProgramId('YourFiveVMProgramIdBase58');
```

## ABI Sources

### Preferred: `.five` artifact

```ts
import fs from 'fs';
import { FiveSDK } from 'five-sdk';

const fiveFileText = fs.readFileSync('build/my-program.five', 'utf-8');
const { abi } = await FiveSDK.loadFiveFile(fiveFileText);
```

### Compile directly with SDK (optional)

```ts
import { FiveSDK } from 'five-sdk';
import fs from 'fs';

const source = `
pub increment(x: u64) -> u64 {
  return x + 1;
}
`;

const compiled = await FiveSDK.compile(source, { optimize: true });

if (!compiled.success || !compiled.fiveFile) {
  throw new Error(`Compilation failed: ${JSON.stringify(compiled.errors ?? [])}`);
}

fs.writeFileSync('build/my-program.five', JSON.stringify(compiled.fiveFile, null, 2));
```

Compile from file path:

```ts
const fromFile = await FiveSDK.compileFile('src/main.v', { optimize: true });
```

Compile multi-file/module source:

```ts
const multi = await FiveSDK.compileModules(
  { filename: 'main.v', content: mainSource },
  [{ name: 'math', source: mathModuleSource }],
  { optimize: true }
);
```

### Optional: ABI JSON

```ts
import fs from 'fs';
const abi = JSON.parse(fs.readFileSync('my-program.abi.json', 'utf-8'));
```

## Create Program Client

```ts
import { FiveProgram } from 'five-sdk';

const program = FiveProgram.fromABI('ScriptAccountBase58', abi, {
  fiveVMProgramId: 'YourFiveVMProgramIdBase58',
  vmStateAccount: 'VmStatePdaBase58',
  feeReceiverAccount: 'FeeReceiverBase58',
  debug: false,
});
```

## Core Interaction Flow

```ts
const ixData = await program
  .function('add_amount')
  .accounts({
    counter: counterPubkey,
    owner: ownerPubkey,
  })
  .args({
    amount: 100,
  })
  .instruction();
```

Then convert and send with your Solana client.

```ts
import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from '@solana/web3.js';

const connection = new Connection('http://127.0.0.1:8899', 'confirmed');

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
  throw new Error(`Execution failed: ${JSON.stringify(txDetails.meta.err)}`);
}

console.log('computeUnits', txDetails?.meta?.computeUnitsConsumed);
```

## Multi-file DSL Function Names

Function names must match ABI exactly.
In multi-file/module projects, names may be qualified (for example `module::function`).

```ts
console.log(program.getFunctions());
```

If ABI contains `math::add`, calling `.function('add')` will fail.

## Account Wiring Behavior

`FunctionBuilder` handles:
- system account auto-injection when required by ABI constraints
- PDA account resolution from ABI seed metadata
- signer/writable flags from ABI account attributes

You must still provide all required business accounts and args.

## Advanced APIs (Optional)

`FiveProgram` is the primary interface for most applications. The APIs below are useful for tooling, testing, and custom infrastructure.

### Local-only execution (no validator/RPC)

```ts
const local = await FiveSDK.compileAndExecuteLocally(
  sourceCode,
  'increment',
  [1],
  { optimize: true, trace: true, computeUnitLimit: 1_000_000 }
);
```

### Serialized instruction generation

```ts
const deployIx = await FiveSDK.generateDeployInstruction(bytecode, deployerPubkeyBase58);

const execIx = await FiveSDK.generateExecuteInstruction(
  scriptAccountBase58,
  'increment',
  [1],
  [ownerPubkeyBase58],
  connection,
  { computeUnitLimit: 1_000_000 }
);
```

### On-chain convenience methods

```ts
const deployed = await FiveSDK.deployToSolana(bytecode, connection, payerKeypair, {
  fiveVMProgramId: 'YourFiveVMProgramIdBase58',
});

const executed = await FiveSDK.executeScriptAccount(
  deployed.scriptAccount,
  0,
  [],
  connection,
  payerKeypair,
  { fiveVMProgramId: 'YourFiveVMProgramIdBase58' }
);
```

### Metadata and decoding helpers

```ts
const metadata = await FiveSDK.getScriptMetadataWithConnection(scriptAccountBase58, connection);
const functionNames = await FiveSDK.getFunctionNamesFromScriptAccount(scriptAccountBase58, connection);
const decoded = await FiveSDK.fetchAccountAndDeserialize(scriptAccountBase58, connection, {
  parseMetadata: true,
  validateEncoding: true,
});
```

### Namespace helpers (5NS)

```ts
const parsed = FiveSDK.canonicalizeNamespace('@acme/payments');
const lockfileMatch = FiveSDK.resolveNamespaceFromLockfile(parsed.canonical, lockfileJson);
```

For on-chain namespace manager flows:
- `FiveSDK.registerNamespaceTldOnChain(...)`
- `FiveSDK.bindNamespaceOnChain(...)`
- `FiveSDK.resolveNamespaceOnChain(...)`

## Troubleshooting

### `No program ID resolved for Five VM`
Provide explicit `fiveVMProgramId`, set SDK default, or set `FIVE_PROGRAM_ID`.

### `Function '<name>' not found in ABI`
Use exact ABI function name (including `module::` prefix when present).

### `Missing required account '<name>'`
Add the missing account to `.accounts(...)`.

### `Missing required argument '<name>'`
Add the missing argument to `.args(...)`.

## API Reference (Current Signatures)

### FiveProgram

```ts
class FiveProgram {
  static fromABI(
    scriptAccount: string,
    abi: ScriptABI,
    options?: FiveProgramOptions
  ): FiveProgram;

  static load(
    scriptAddress: string,
    connection: any,
    options?: FiveProgramOptions
  ): Promise<FiveProgram>;

  function(functionName: string): FunctionBuilder;
  account(structName: string): ProgramAccount;

  getFunctions(): string[];
  getFunction(name: string): FunctionDefinition | undefined;
  getAllFunctions(): FunctionDefinition[];

  generateTypes(): string;

  getScriptAccount(): string;
  getFiveVMProgramId(): string;
  getVMStateAccount(): string | undefined;
  getFeeReceiverAccount(): string | undefined;

  setVMStateAccount(account: string): this;
  setFeeReceiverAccount(account: string): this;

  findAddress(
    seeds: (string | Uint8Array | Buffer)[],
    programId?: string
  ): Promise<[string, number]>;

  getABI(): ScriptABI;
  getOptions(): FiveProgramOptions;
}
```

### FunctionBuilder

```ts
class FunctionBuilder {
  accounts(accounts: Record<string, string | { toBase58(): string }>): this;
  args(args: Record<string, any>): this;

  instruction(): Promise<SerializedInstruction>;
  transaction(options?: { computeUnits?: number }): Promise<any>;
  rpc(options?: {
    signers?: any[];
    skipPreflight?: boolean;
    computeUnits?: number;
  }): Promise<string>;

  getFunctionDef(): FunctionDefinition;
  getAccounts(): Record<string, string>;
  getArgs(): Record<string, any>;
}
```

### SerializedInstruction

```ts
interface SerializedInstruction {
  programId: string;
  keys: Array<{ pubkey: string; isSigner: boolean; isWritable: boolean }>;
  data: string; // base64
}
```

## Ecosystem

- CLI/tooling: `five-cli`
- SDK/client interactions: `five-sdk`
- Frontend/UI workflows: [5ive.tech](https://5ive.tech)
