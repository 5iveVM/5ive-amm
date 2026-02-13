# AGENTS.md - Complete 5IVE DSL Agent Bootstrap

This file is designed for agents with zero prior 5IVE knowledge.
It is intentionally self-contained and should be treated as the minimum operating spec for building and deploying 5IVE bytecode safely.

## 1. What 5IVE Is

5IVE is a DSL and toolchain for building compact Solana-executed program logic compiled into 5IVE bytecode.

Core outputs and surfaces:
1. Source files: `.v`
2. Canonical artifact: `.five` (bytecode + ABI)
3. CLI: `@5ive-tech/cli` (`5ive` canonical command, `five` alias also works)
4. SDK: `@5ive-tech/sdk` (`FiveSDK`, `FiveProgram`)

## 2. Source of Truth Policy

When references conflict, resolve in this order:
1. CLI/SDK/compiler source code
2. Package manifests and command definitions
3. README/docs/examples

Do not assume older docs are correct without verifying against active command implementations.

## 3. Non-Negotiable Workflow

1. Inspect `five.toml` first.
2. Compile to `.five` artifact.
3. Run local/runtime tests.
4. Deploy with explicit target and program ID resolution path.
5. Execute and verify confirmed transaction metadata (`meta.err == null`).
6. Record signatures + compute units.

## 4. 5IVE DSL Syntax and Semantics (Cold-Start Primer)

### 4.1 Top-level structure
Current examples and grammar support top-level declarations directly.

```five
account Counter {
    value: u64;
    authority: pubkey;
}

pub increment(counter: Counter @mut, authority: account @signer) {
    require(counter.authority == authority.key);
    counter.value = counter.value + 1;
}
```

Notes:
1. Legacy wrapper forms may exist in old examples; prefer direct top-level declarations.
2. Keep one clear entrypoint file for project builds (`five.toml` `entry_point`).

### 4.2 Core declarations
1. Accounts:
```five
account Position {
    owner: pubkey;
    amount: u64;
}
```
2. Functions:
```five
pub add(a: u64, b: u64) -> u64 {
    return a + b;
}
```
3. Init block (used in many examples):
```five
init {
    // initial setup
}
```

### 4.3 Types
Commonly used types from docs/examples:
1. Unsigned ints: `u8..u128`
2. Signed ints: `i8..i64`
3. `bool`
4. `pubkey`
5. Strings with sizing in account fields: `string<N>`
6. Fixed arrays: `[T; N]`
7. Optional account fields: `field?: type`
8. Option/Result in signatures in advanced examples: `Option<T>`, `Result<T,E>`

Use conservative, template-proven type patterns for production paths.

### 4.4 Expressions and control flow
Supported in examples:
1. Arithmetic and comparisons
2. Boolean logic
3. `if` and nested conditionals
4. `while`
5. Function calls and account field access

Example:
```five
pub accumulate(limit: u64) -> u64 {
    let mut i: u64 = 0;
    let mut total: u64 = 0;
    while (i < limit) {
        total = total + i;
        i = i + 1;
    }
    return total;
}
```

### 4.5 Guards and validation
Use `require(...)` aggressively for invariant protection.

```five
require(amount > 0);
require(vault.authority == authority.key);
```

### 4.6 Account parameters and constraints
Canonical constraint patterns:
1. `@mut` mutable account
2. `@signer` required signer
3. `@init` initialize account
4. Extended patterns in templates:
   - `@init(payer=..., space=..., seeds=[...])`
   - `@has(field)` ownership/authority relation checks

Example:
```five
pub init_counter(
    counter: Counter @mut @init(payer=owner, space=56, seeds=["counter", owner.key]),
    owner: account @signer
) {
    counter.value = 0;
    counter.authority = owner.key;
}
```

### 4.7 External bytecode imports
5IVE supports import-style external calls to deployed bytecode accounts.

```five
use "11111111111111111111111111111111"::{transfer};

pub settle(from: account @mut, to: account @mut, owner: account @signer) {
    transfer(from, to, owner, 50);
}
```

### 4.8 CPI interfaces
Define external program interfaces explicitly:

```five
interface SPLToken @program("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA") @serializer(bincode) {
    transfer @discriminator(3) (
        from: account,
        to: account,
        authority: account,
        amount: u64
    );
}

pub cpi_transfer(from: account @mut, to: account @mut, authority: account @signer) {
    SPLToken.transfer(from, to, authority, 50);
}
```

Critical CPI rules:
1. Always set `@program(...)`.
2. Always set `@serializer(...)` explicitly.
3. Always set `@discriminator(...)` explicitly.
4. Keep account ordering deterministic.

### 4.9 Serializer conflict handling (important)
Docs in this repo historically conflict on default serializer (`borsh` vs `bincode`).
Canonical rule for agents:
1. Never rely on default serializer.
2. Always specify serializer explicitly in each interface.
3. Match serializer/discriminator to target program spec.

### 4.10 DSL safety baseline
For each state-mutating function, include:
1. authority check
2. value/range check
3. state-transition check
4. arithmetic safety check (overflow/underflow-aware patterns)

## 5. Project Structure and Build Model

Typical project layout:
1. `src/` DSL source
2. `tests/` test scripts
3. `build/` compiled artifacts
4. `five.toml` project config

Multi-file model:
1. Set `entry_point` in `five.toml`.
2. If using module list, keep stable ordering.
3. Ensure all imported modules are part of build context.

## 6. CLI Canonical Usage

### 6.1 Install and identity
```bash
npm install -g @5ive-tech/cli
5ive --version
```

### 6.2 Initialize
```bash
5ive init my-program
cd my-program
```

### 6.3 Compile
```bash
5ive compile src/main.v -o build/main.five
# or project-aware
5ive build
```

### 6.4 Local execute
```bash
5ive execute build/main.five --local -f 0
```

### 6.5 Configure devnet
```bash
5ive config init
5ive config set --target devnet
5ive config set --keypair ~/.config/solana/id.json
5ive config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
```

### 6.6 Deploy and execute on-chain
```bash
5ive deploy build/main.five --target devnet
5ive execute build/main.five --target devnet -f 0
```

### 6.7 Advanced deploy modes
```bash
5ive deploy build/main.five --target devnet --optimized --progress
5ive deploy build/main.five --target devnet --force-chunked --chunk-size 900
5ive deploy build/main.five --target devnet --dry-run --format json
```

### 6.8 Tests
```bash
5ive test --sdk-runner
5ive test tests/ --on-chain --target devnet
5ive test --sdk-runner --format json
```

## 7. Program ID and Target Resolution

For on-chain commands (`deploy`, `execute`, `namespace`) resolve program ID in this order:
1. `--program-id`
2. `five.toml [deploy].program_id`
3. `5ive config` stored value for current target
4. `FIVE_PROGRAM_ID`

If unresolved, fail fast and do not continue.

## 8. SDK Canonical Usage

### 8.1 Load artifact
```ts
import fs from "fs";
import { FiveSDK } from "@5ive-tech/sdk";

const fiveFileText = fs.readFileSync("build/main.five", "utf8");
const { abi } = await FiveSDK.loadFiveFile(fiveFileText);
```

### 8.2 Program client
```ts
import { FiveProgram } from "@5ive-tech/sdk";

const program = FiveProgram.fromABI("<SCRIPT_ACCOUNT>", abi, {
  fiveVMProgramId: "<FIVE_VM_PROGRAM_ID>",
  vmStateAccount: "<VM_STATE_ACCOUNT>",
  feeReceiverAccount: "<FEE_RECEIVER_ACCOUNT>",
});
```

### 8.3 Instruction build + send
1. Build with `.function().accounts().args().instruction()`.
2. Convert to `TransactionInstruction`.
3. Send with preflight.
4. Fetch confirmed transaction.
5. Assert `meta.err == null`.
6. Record `meta.computeUnitsConsumed`.

### 8.4 SDK program ID defaults
Precedence in SDK paths:
1. explicit `fiveVMProgramId`
2. `FiveSDK.setDefaultProgramId(...)`
3. `FIVE_PROGRAM_ID`
4. released package baked default (if set)

## 9. Frontend Integration Baseline

1. Build instructions through SDK (`FiveProgram`) instead of custom serializers.
2. Keep network selection explicit (`localnet`, `devnet`, `mainnet`).
3. Surface signatures, errors, and CU metrics in UI.
4. For editor workflows, use LSP-backed diagnostics/completion features where available.

## 10. Design Pattern Mapping (for Complex Programs)

### 10.1 Vault
- Accounts: vault state, authority signer, source/destination token accounts
- Invariants: authority-only withdrawals, no negative balances

### 10.2 Escrow
- Accounts: escrow state, counterparties, settlement accounts
- Invariants: valid lifecycle transitions, no double settlement

### 10.3 Token/mint authority
- Accounts: mint, token accounts, authorities/delegates
- Invariants: supply accounting, authority checks, freeze/delegate behavior

### 10.4 AMM/orderbook
- Accounts: pool/book state, user positions, fee accounts
- Invariants: conservation, deterministic settlement, fee accounting

### 10.5 Lending/perps/stablecoin
- Accounts: collateral/debt/position state, oracle/risk accounts
- Invariants: collateral thresholds, liquidation boundaries

## 11. Testing Strategy

Execution order:
1. Runtime harness (validator-free) where available
2. Local CLI/SDK tests
3. On-chain integration tests

Always include:
1. happy path
2. auth failure path
3. value-range failure path
4. state transition failure path
5. CU regression capture for critical instructions

## 12. Mainnet Safety Policy

Never deploy mainnet blindly.

Required preflight gates:
1. Artifact hash freeze (`.five` file chosen and immutable)
2. Config lock (target, RPC, program ID, keypair source)
3. Key custody validation
4. Dry-run/simulate path complete
5. Rollback/containment plan defined

Post-deploy requirements:
1. smoke execute
2. confirmed transaction validation
3. CU baseline capture
4. incident path if unexpected errors appear

## 13. Common Failures and Fixes

1. `No program ID resolved for Five VM`:
   - Set one via `--program-id`, config, or `FIVE_PROGRAM_ID`.
2. `Function '<name>' not found in ABI`:
   - Use exact ABI name (including namespace prefixes).
3. `Missing required account` / `Missing required argument`:
   - satisfy all `.accounts(...)` and `.args(...)` fields.
4. owner/program mismatch:
   - check target program ID and deployed account ownership.
5. CPI serialization mismatch:
   - ensure explicit `@serializer(...)` and correct discriminator format.

## 14. Definition of Done

A task is complete only when:
1. Program compiles to `.five`.
2. Tests pass with evidence.
3. Deployment is confirmed (if requested).
4. Execution is confirmed and `meta.err == null` (if requested).
5. Signatures + CU data are recorded.
6. SDK/frontend integration snippet is provided if integration is in scope.

## 15. Agent Behavior Rules

1. Prefer minimal, reproducible command paths.
2. Do not skip verification after sending transactions.
3. Do not assume defaults for critical deploy/CPI parameters.
4. Keep all outputs deterministic and auditable.
5. If uncertain, inspect compiler/CLI source before making assumptions.
