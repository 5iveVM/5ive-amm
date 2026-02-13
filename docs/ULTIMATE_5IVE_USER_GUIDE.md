# Ultimate 5ive User Guide (Code-Truth Canonical)

This guide is the canonical implementation guide for building 5IVE DSL programs, testing them, deploying to devnet/mainnet, and integrating via CLI, SDK, and frontend surfaces.

## 0) Canonical Rules

### 0.1 Source-of-truth precedence
When documentation conflicts, resolve in this order:
1. CLI/SDK/compiler source code
2. Package manifests and command definitions
3. README/examples/docs

### 0.2 Command naming
Use `5ive` as canonical command in this guide.
Compatibility note: `five` is an alias (`five-cli/package.json` maps both `5ive` and `five` to the same binary).

### 0.3 Artifact standard
Use `.five` as the default artifact for deployment and SDK interop.
It carries both bytecode and ABI.

## 1) Operating Model

### 1.1 What 5ive compiles to
- Input: `.v` DSL source (single or multi-file)
- Output: `.five` artifact (bytecode + ABI)
- Optional outputs: `.bin`, ABI JSON, diagnostics/metrics

### 1.2 Lifecycle
1. Author DSL source
2. Compile (`5ive compile` or `5ive build`)
3. Test locally/runtime harness
4. Deploy on-chain (`5ive deploy`)
5. Execute (`5ive execute`)
6. Integrate via SDK/frontend

### 1.3 CLI vs SDK responsibilities
- CLI: project scaffolding, compile/build/test/deploy/execute ops, config layering
- SDK: programmatic compile/load, typed instruction construction, transaction assembly helpers, metadata helpers, testing utilities

### 1.4 Testing boundaries
- Runtime harness/local-first: fastest feedback, preferred preflight
- On-chain tests: integration correctness against real accounts/programs/network conditions

## 2) Fastest Path (Copy-Paste Runbook)

### 2.1 Initialize
```bash
5ive init my-program
cd my-program
```

### 2.2 Compile to canonical artifact
```bash
5ive compile src/main.v -o build/main.five
```

### 2.3 Local execute
```bash
5ive execute build/main.five --local -f 0
```

### 2.4 Configure devnet
```bash
5ive config init
5ive config set --target devnet
5ive config set --keypair ~/.config/solana/id.json
5ive config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
```

### 2.5 Deploy + execute devnet
```bash
5ive deploy build/main.five --target devnet
5ive execute build/main.five --target devnet -f 0
```

### 2.6 SDK load + invoke
```ts
import fs from "fs";
import {
  Connection,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { FiveSDK, FiveProgram } from "@5ive-tech/sdk";

const connection = new Connection("https://api.devnet.solana.com", "confirmed");
const fiveFileText = fs.readFileSync("build/main.five", "utf8");
const { abi } = await FiveSDK.loadFiveFile(fiveFileText);

const program = FiveProgram.fromABI("<SCRIPT_ACCOUNT>", abi, {
  fiveVMProgramId: "<FIVE_VM_PROGRAM_ID>",
});

const serialized = await program
  .function("<FUNCTION_NAME>")
  .accounts({ /* required accounts */ })
  .args({ /* required args */ })
  .instruction();

const ix = new TransactionInstruction({
  programId: new PublicKey(serialized.programId),
  keys: serialized.keys.map((k) => ({
    pubkey: new PublicKey(k.pubkey),
    isSigner: k.isSigner,
    isWritable: k.isWritable,
  })),
  data: Buffer.from(serialized.data, "base64"),
});

const sig = await sendAndConfirmTransaction(connection, new Transaction().add(ix), [payer], {
  skipPreflight: false,
  commitment: "confirmed",
});

const tx = await connection.getTransaction(sig, {
  maxSupportedTransactionVersion: 0,
  commitment: "confirmed",
});

if (tx?.meta?.err) throw new Error(JSON.stringify(tx.meta.err));
console.log("signature", sig, "computeUnits", tx?.meta?.computeUnitsConsumed);
```

### 2.7 Frontend minimum integration
1. Load `.five` ABI through SDK helper.
2. Use `FiveProgram` for instruction generation.
3. Submit transaction via wallet adapter/web3.js.
4. Display signature + compute units + error surface.

## 3) DSL Mastery (Agent-Focused)

### 3.1 Core language areas
1. Accounts and account constraints (`@mut`, `@signer`, `@init`)
2. Control flow (`if`, `while`, branching)
3. Data types (`u*`, `i*`, `bool`, `pubkey`, arrays, optional fields)
4. Multi-file/module layouts (entrypoint + module files)

### 3.2 External calls and interfaces
- Use `interface ... @program("...")` for CPI boundaries
- Keep discriminators explicit
- Keep serializer explicit (`@serializer(...)`) for deterministic behavior
- Prefer explicit account ordering and required account comments in code

### 3.3 CPI vs external bytecode calls
- CPI: invokes external Solana programs with serialized instruction payloads
- External bytecode call path: calls deployed 5IVE bytecode accounts through import metadata
- Rule: choose the minimal boundary that preserves safety and composability

### 3.4 Common pitfalls
1. Implicit program ID assumptions on deploy/execute
2. ABI function name mismatch in multi-file programs (`module::func`)
3. Missing required accounts/args in SDK invocation
4. Relying on send success without confirmed transaction error check
5. Serializer default assumptions across conflicting docs

### 3.5 Bytecode efficiency checklist
1. Keep account set minimal per instruction
2. Minimize repeated state loads and writes
3. Keep instruction payloads compact and deterministic
4. Prefer simple, auditable branching and authority checks
5. Measure CU in runtime/on-chain tests and keep regression budgets

## 4) Design Patterns for Complex Contracts

Pattern references should be implemented with explicit invariants and account-role definitions.

### 4.1 Vault
- Required accounts: vault state, authority signer, destination accounts
- State model: total assets + authority + optional policy fields
- Authority model: strict signer check + optional delegated controls
- Invariants: non-negative balances, authority-only withdrawal
- Testing: deposit/withdraw with negative-path auth tests

### 4.2 Escrow
- Required accounts: escrow state, maker/taker accounts, asset accounts
- State model: lifecycle status (`init`, `funded`, `settled`, `cancelled`)
- Authority model: maker/taker signatures per transition
- Invariants: one-way state transitions, no double-settlement
- Testing: full lifecycle + timeout/cancel branches

### 4.3 Token / mint authority
- Required accounts: mint, token accounts, authorities
- State model: supply/metadata/flags/delegation state
- Authority model: mint/freeze/delegate authority separation
- Invariants: supply math safety, authority gating, freeze behavior
- Testing: mint/transfer/burn/freeze/revoke/authority rotation

### 4.4 AMM / orderbook
- Required accounts: pool/orderbook state, user positions, fee accounts
- State model: reserves/book levels, fee accumulation, position snapshots
- Authority model: protocol admin + user signer paths
- Invariants: conservation checks, deterministic matching/settlement
- Testing: swap/match edge cases, slippage and partial fills

### 4.5 Lending / perps / stablecoin risk systems
- Required accounts: collateral positions, debt state, oracle/state feeds
- State model: collateral ratio, debt index, liquidation metadata
- Authority model: borrower actions + liquidator/admin controls
- Invariants: solvency thresholds and bounded liquidation behavior
- Testing: stress paths (volatility, liquidation, edge collateral values)

## 5) Template Strategy and Maturity Matrix

Use templates as either:
- **Verified runbook**: concrete compile/test/deploy flow is documented and exercised
- **Reference pattern**: architecture/layout guidance, adapt and validate in your own environment

| Template Area | Classification | Notes |
|---|---|---|
| Token (`five-templates/token`) | Verified runbook | Rich E2E/runtime fixture docs and scripts |
| CPI examples (`five-templates/cpi-examples`) | Verified runbook | Concrete interface/CPI patterns |
| CPI integration tests (`five-templates/cpi-integration-tests`) | Verified runbook | Localnet/devnet integration paths |
| Orderbook/Perps/Lending/Stablecoin/Vault/etc | Reference pattern | Strong architecture summaries and module layouts |

### 5.1 Entrypoint/module mapping rule
Always verify:
1. `five.toml` entrypoint
2. `modules` ordering (if multi-file)
3. artifact output names and deploy defaults

## 6) Devnet Runbook (Production-Like)

### 6.1 Program ID resolution (on-chain commands)
Canonical precedence:
1. `--program-id`
2. `five.toml [deploy].program_id`
3. `5ive config` program ID for target
4. `FIVE_PROGRAM_ID`

If unresolved, on-chain commands should fail fast.

### 6.2 Deployment modes
```bash
5ive deploy build/main.five --target devnet
5ive deploy build/main.five --target devnet --optimized --progress
5ive deploy build/main.five --target devnet --force-chunked --chunk-size 900
5ive deploy build/main.five --target devnet --dry-run --format json
```

### 6.3 Execute + verify
```bash
5ive execute build/main.five --target devnet -f 0
# or
5ive execute --script-account <SCRIPT_ACCOUNT_PUBKEY> --target devnet -f 0
```
Verification pattern:
1. send with preflight enabled
2. fetch confirmed transaction
3. assert `meta.err == null`
4. record `computeUnitsConsumed`

### 6.4 Troubleshooting
1. Program ID setup error:
   - set `--program-id` or configure target program ID
2. Keypair not found:
   - `5ive config set --keypair ~/.config/solana/id.json`
3. owner/program mismatch:
   - ensure deployment owner program ID is the expected Five VM ID

## 7) Mainnet Production Runbook

### 7.1 Preflight gates (must-pass)
1. Bytecode freeze:
   - immutable build artifact hash recorded
2. Config lock:
   - target, RPC URL, keypair scope, program ID pinned
3. Key custody:
   - signer environment and approval policy confirmed
4. RPC plan:
   - primary + fallback endpoints validated
5. Dry-run/simulation path executed where supported
6. Monitoring and rollback plan documented before deployment

### 7.2 Staged rollout
1. Final devnet rehearsal with release artifact
2. Mainnet deploy during controlled window
3. Post-deploy smoke execute
4. Confirm tx metadata and CU envelope

### 7.3 Post-deploy observability
1. Store signatures for deployment + first executes
2. Record execution CU baselines
3. Monitor error rates and authority-sensitive operations

### 7.4 Rollback/containment policy
1. Stop new writes/executes if critical invariant breaks
2. Rotate/disable authorities where contract design allows
3. Publish incident summary with signature timeline

## 8) Testing Stack

### 8.1 Preferred order
1. Runtime harness (validator-free) where available
2. CLI local/sdk test runner
3. On-chain integration tests

### 8.2 CLI testing modes
```bash
5ive test --sdk-runner
5ive test --sdk-runner --format json
5ive test tests/ --on-chain --target devnet
5ive test test-scripts/ --on-chain --target devnet --batch --analyze-costs
```

### 8.3 CPI testing patterns
- SPL Token CPI paths
- INVOKE_SIGNED/PDA authority paths
- import/program-ID verification paths

### 8.4 CU/cost regression workflow
1. Persist baseline CU per critical instruction
2. Compare CU deltas in CI or release checks
3. Fail/flag when thresholds are exceeded

## 9) SDK Deep Integration

### 9.1 Canonical client construction
1. Load ABI from `.five`
2. Build `FiveProgram.fromABI(...)`
3. Generate instruction via `.function().accounts().args().instruction()`
4. Convert serialized data to `TransactionInstruction`
5. Submit + verify confirmed metadata

### 9.2 Function naming in multi-file ABI
Use exact ABI names, including namespaced forms like `module::function`.

### 9.3 Account wiring behavior
`FunctionBuilder` can auto-handle system account injection, signer/writable derivation, and some PDA resolution from ABI metadata, but required business accounts still must be provided.

### 9.4 Useful SDK helper areas
- program ID defaulting via `FiveSDK.setDefaultProgramId(...)`
- deploy/execute convenience methods
- metadata fetch/decode helpers
- namespace helper methods
- testing utilities (`FiveTestRunner`, `TestDiscovery`, account fixtures)

## 10) Frontend Integration

### 10.1 Architecture checkpoints
Use frontend as orchestration layer around SDK/web3, not custom serialization logic.

### 10.2 Network strategy
Support explicit `localnet` and `devnet` first; mainnet only after runbook gates.

### 10.3 Wallet/deploy/execute flow
1. Connect wallet
2. Compile/load artifact
3. Build instruction via SDK
4. Send transaction and confirm
5. Surface signature/errors/CU in UI

### 10.4 LSP-backed authoring
Use LSP-backed editing features for diagnostics, completion, hover, references, and rename to reduce DSL authoring errors.

### 10.5 Safe production boundaries
1. Never sign mainnet txs from ambiguous UI state
2. Show target/program ID/key account context before submit
3. Log tx outcomes and preserve audit trail

## 11) Contradictions and Canonical Resolutions

### 11.1 `5ive` vs `five`
- Conflict: docs use both
- Canonical: use `5ive` in guides and scripts
- Compatibility: `five` remains a supported alias

### 11.2 CPI serializer default conflict
- Conflict: some docs claim Borsh default, others claim Bincode default
- Canonical resolution: do not rely on implicit default in production docs
- Required practice: always set `@serializer(...)` explicitly in interfaces

### 11.3 Legacy command examples
- Some historical docs/scripts use legacy or inconsistent command forms
- Canonical resolution: validate against current command implementations and README baseline before reuse

### 11.4 Program ID setup ambiguity
- Many failures trace back to implicit program ID assumptions
- Canonical resolution: make program ID source explicit in every deployment runbook

## 12) Appendices

### 12.1 Devnet preflight checklist
1. `five.toml` entrypoint/modules validated
2. Build artifact exists (`build/*.five`)
3. Keypair and balance validated
4. Program ID resolution path identified
5. Dry-run or rehearsal execution completed

### 12.2 Mainnet go-live checklist
1. Artifact hash frozen and reviewed
2. Program ID/target/RPC pinned
3. Key custody and signer policy verified
4. Rollback/containment plan written
5. Post-deploy smoke test plan ready

### 12.3 Command cheat sheet
```bash
# Init
5ive init my-program

# Compile/build
5ive compile src/main.v -o build/main.five
5ive build

# Test
5ive test --sdk-runner
5ive test tests/ --on-chain --target devnet

# Deploy/execute
5ive deploy build/main.five --target devnet
5ive execute build/main.five --target devnet -f 0

# Config
5ive config init
5ive config set --target devnet
5ive config set --keypair ~/.config/solana/id.json
5ive config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
```

### 12.4 Failure signatures and likely fixes
1. `No program ID resolved`:
   - set `--program-id`, config target value, or `FIVE_PROGRAM_ID`
2. `Function '<name>' not found in ABI`:
   - use exact ABI function name (including namespace)
3. `Missing required account/argument`:
   - satisfy `.accounts(...)` and `.args(...)` contract
4. `owner/program mismatch`:
   - validate target program ID and deployed account owner alignment

---

This document is intentionally implementation-first and can be used directly by both human developers and coding agents.
