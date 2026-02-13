# AGENTS.md - Complete 5IVE DSL Agent Bootstrap

This file is written for agents with zero prior 5IVE knowledge.
It is intentionally self-contained and should be treated as the baseline technical specification for authoring, testing, deploying, and integrating 5IVE programs.

## 1) What 5IVE Is

5IVE is a DSL and toolchain for compiling compact bytecode programs for Solana execution paths.

Core surfaces:
1. Source language: `.v`
2. Canonical artifact: `.five` (bytecode + ABI)
3. CLI: `@5ive-tech/cli` (`5ive` canonical command; `five` alias supported)
4. SDK: `@5ive-tech/sdk` (`FiveSDK`, `FiveProgram`)

## 2) Source of Truth Policy

When docs conflict, resolve in this order:
1. Compiler/CLI/SDK source code
2. Package manifests + command definitions
3. READMEs/examples/docs

Never rely on stale docs when behavior is high-stakes (deploy/execute/CPI encoding).

## 3) Non-Negotiable Workflow

1. Inspect `five.toml` before code changes.
2. Compile to `.five`.
3. Run local/runtime tests.
4. Deploy with explicit target + program ID resolution path.
5. Execute and verify confirmed tx metadata (`meta.err == null`).
6. Record signatures + compute units.

## 4) DSL Feature Inventory (Deep)

This section enumerates language features discovered from parser/compiler code and repo examples.

### 4.1 Top-level declarations
Observed and/or parsed:
1. `account Name { ... }`
2. Global fields/variables (including `mut`)
3. `init { ... }` block
4. `constraints { ... }` block
5. Function/instruction definitions (`pub`, `fn`, optional `instruction` keyword)
6. `event Name { ... }` definitions
7. `interface Name ... { ... }` definitions
8. `use` / `import` statements
9. Legacy `script Name { ... }` wrapper (parser-supported)

### 4.2 Function definition forms
Parser accepts flexible forms:
1. `pub add(...) -> ... { ... }`
2. `fn add(...) { ... }`
3. `instruction add(...) { ... }`
4. `pub fn add(...) { ... }`

### 4.3 Parameter system
Each parameter supports:
1. Name + type: `x: u64`
2. Optional marker: `x?: u64`
3. Default value: `x: u64 = 10`
4. Attributes before or after type

Common attributes:
1. `@signer`
2. `@mut`
3. `@init`
4. Generic form: `@attribute(args...)`
5. Template-observed relation constraints: `@has(field)`

### 4.4 `@init` config support
`@init` can include config arguments:
1. `payer=...`
2. `space=...`
3. `seeds=[...]`
4. `bump=...`

Examples also show legacy bracket seed forms after `@init`.

### 4.5 Types
Supported/parsed type families:
1. Primitive numeric/bool/pubkey/string types (`u8..u128`, `i8..i64`, `bool`, `pubkey`, `string`)
2. `Account` type and account-typed params
3. Sized strings: `string<32>`
4. Arrays:
   - Rust style: `[T; N]`
   - TypeScript-style sized: `T[N]`
   - TypeScript-style dynamic: `T[]`
5. Tuples: `(T1, T2, ...)`
6. Inline struct types: `{ field: Type, ... }`
7. Generic types:
   - `Option<T>`
   - `Result<T, E>`
   - nested generics (`Option<Option<u64>>` etc.)
8. Namespaced/custom types: `module::Type`
9. Optional account fields in structs/accounts: `field?: Type`

### 4.6 Statements
Observed and parser-supported:
1. `let` declarations (with `mut` and optional type annotation)
2. Assignment:
   - direct: `x = y`
   - compound: `+=`, `-=`, `*=`, `/=`, `<<=`, `>>=`, `&=`, `|=`, `^=`
3. Field assignment: `obj.field = value`
4. Return statements (`return`, `return value`)
5. Guard/assertion: `require(condition)`
6. Conditionals:
   - `if (...) {}`
   - `else if (...) {}`
   - `else {}`
7. Pattern matching: `match expr { ... }`, with optional arm guards (`if ...`)
8. Loops:
   - `while (...) { ... }`
   - `for (init; cond; update) { ... }`
   - `for (item in iterable) { ... }`
   - `do { ... } while (...);`
9. Tuple destructuring:
   - declaration style: `let (a, b) = expr`
   - assignment style for tuple targets
10. Event emission: `emit EventName { field: value, ... }`
11. Expression statements (function/method calls, constructors like `Some(...)`)

### 4.7 Expressions and operators
Parser handles:
1. Arithmetic: `+`, `-`, `*`, `/`, `%`
2. Checked-arithmetic tokens in grammar: `+?`, `-?`, `*?`
   - Some repo tests indicate these were replaced/legacy in current examples.
3. Comparison: `==`, `!=`, `<`, `<=`, `>`, `>=`
4. Boolean: `&&`, `||`, `!`
5. Bitwise: `&`, `|`, `^`, `~`
6. Shifts/bit ops: `<<`, `>>`, `>>>`, `<<<`
7. Range operator: `..`
8. Unary `+`/`-`
9. Cast syntax: `expr as Type`
10. Error propagation postfix: `expr?`
11. Field access: `obj.field`
12. Tuple access: `obj.0`
13. Array indexing: `arr[idx]`
14. Function calls
15. Method calls: `obj.method(args...)`
16. Namespaced calls: `module::function(...)`
17. Struct literals: `{ field: expr, ... }`
18. Array literals: `[a, b, c]`
19. Tuple literals: `(a, b)`
20. Option/Result constructors and patterns:
   - `Some(...)`, `None`
   - `Ok(...)`, `Err(...)`

### 4.8 Imports and modules
`use`/`import` system supports:
1. External module specifier via quoted literal
2. Local module specifier via identifier path
3. Nested local module paths using `::`
4. Scoped namespace forms with symbols: `!`, `@`, `#`, `$`, `%`
5. Member imports:
   - single: `::name`
   - list: `::{a, b}`
   - typed list entries: `method foo`, `interface Bar`

### 4.9 Interfaces and CPI features
Interface parser supports:
1. `interface Name ... { methods... }`
2. Program binding:
   - `program("...")`
   - `@program("...")`
3. Serializer hints:
   - `serializer(...)`
   - `@serializer(...)`
4. Anchor marker:
   - `@anchor interface ...`
5. Method discriminators:
   - `@discriminator(u8)`
   - `@discriminator([byte,...])`
   - `discriminator_bytes(...)` forms in parser/compiler AST
6. Optional interface method return types

CPI hard rule for agents:
1. Always set `@program(...)`
2. Always set `@serializer(...)` explicitly
3. Always set discriminator explicitly

### 4.10 Events and error/enums
Parser/AST include:
1. Event definitions + `emit` statements
2. Enum/error-style definitions (`enum` path in parser)

Production note:
Treat event/error enum features as available in syntax, but verify runtime/compiler behavior in your exact toolchain version before relying on them for critical flows.

### 4.11 Testing-oriented language features
From tokenizer/parser support:
1. `#[...]` test attributes
2. `test` function parse path
3. Test attribute names/tokens include:
   - `ignore`
   - `should_fail`
   - `timeout`
4. Assertion tokens:
   - `assert_eq`
   - `assert_true`
   - `assert_false`
   - `assert_fails`
   - `assert_approx_eq`

Repository tests also use comment-based param conventions (`// @test-params ...`) in many scripts.

### 4.12 Blockchain-oriented built-ins seen in examples
Observed in scripts/templates:
1. `derive_pda(...)` (including bump-return and bump-specified variants)
2. `get_clock()`
3. `get_key(...)`
4. account key access: `authority.key`

Treat built-ins as compiler/runtime coupled features; verify signatures in current examples before use.

## 5) Feature Maturity Matrix (Agent Safety)

### 5.1 Generally production-oriented (widely used in templates)
1. Accounts, `@mut`, `@signer`, `@init`
2. `require`
3. Basic control flow (`if`, `while`)
4. Arithmetic/comparison/boolean expressions
5. `.five` compile/deploy/execute path
6. `interface` + explicit discriminator + explicit serializer CPI patterns

### 5.2 Available but validate per-version before critical use
1. Match expressions with `Option`/`Result`
2. Tuple destructuring and tuple returns
3. Advanced loop forms (`for`, `do-while`)
4. Event definition/emit workflows
5. Namespaced imports and scoped namespace symbols
6. Bitwise/shift operator-heavy code

### 5.3 Parser tokens exist; treat as reserved/experimental unless proven in your path
1. `query`, `when`, `realloc`, `pda` keywords
2. Some assertion/test keyword paths in non-test production code
3. Legacy checked-arithmetic operators (`+?`, `-?`, `*?`) where examples indicate migration

## 6) CLI Canonical Usage

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

### 6.6 Deploy + execute on-chain
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

### 6.8 Test modes
```bash
5ive test --sdk-runner
5ive test tests/ --on-chain --target devnet
5ive test --sdk-runner --format json
```

## 7) Program ID and Target Resolution

On-chain command precedence (`deploy`, `execute`, `namespace`):
1. `--program-id`
2. `five.toml [deploy].program_id`
3. `5ive config` value for current target
4. `FIVE_PROGRAM_ID`

Never run on-chain commands with ambiguous target/program-id context.

## 8) SDK Canonical Usage

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

### 8.3 Execution verification pattern
1. Build instruction via `program.function(...).accounts(...).args(...).instruction()`
2. Submit with preflight enabled
3. Fetch confirmed tx
4. Assert `meta.err == null`
5. Record `meta.computeUnitsConsumed`

### 8.4 SDK program ID resolution precedence
1. Explicit `fiveVMProgramId`
2. `FiveSDK.setDefaultProgramId(...)`
3. `FIVE_PROGRAM_ID`
4. release-baked default (if present)

## 9) Frontend Integration Baseline

1. Build execute instructions via SDK (`FiveProgram`), not custom serializers.
2. Keep network selection explicit (`localnet`, `devnet`, `mainnet`).
3. Surface signatures, CU metrics, and rich error states.
4. Use LSP-backed editing where available to reduce DSL mistakes.

## 10) Pattern Mapping for Complex Contracts

1. Vault:
   - authority-gated custody, withdraw invariants
2. Escrow:
   - lifecycle transitions, dual-party authorization
3. Token/mint authority:
   - supply accounting, freeze/delegate controls
4. AMM/orderbook:
   - conservation math, deterministic settlement
5. Lending/perps/stablecoin:
   - collateral/liquidation thresholds, risk checks

## 11) Mainnet Safety Policy

Required preflight gates:
1. Freeze artifact hash
2. Lock target/program-id/RPC/key source
3. Validate key custody
4. Run simulation/dry-run path
5. Predefine rollback/containment actions

Post-deploy:
1. smoke execute
2. confirmed tx validation
3. CU baseline capture
4. incident process if anomalies appear

## 12) Common Failure Signatures

1. `No program ID resolved for Five VM`:
   - set explicit program-id source
2. `Function '<name>' not found in ABI`:
   - use exact ABI name (including namespace)
3. `Missing required account/argument`:
   - satisfy `.accounts(...)` and `.args(...)`
4. owner/program mismatch:
   - verify target program ownership assumptions
5. CPI mismatch:
   - verify explicit serializer/discriminator/account order

## 13) Definition of Done

Complete means:
1. `.five` artifact produced
2. tests passed with evidence
3. deployment confirmed (if in scope)
4. execution confirmed with `meta.err == null` (if in scope)
5. signatures + CU metrics recorded
6. integration snippet delivered (SDK/frontend when requested)

## 14) Agent Behavior Rules

1. Prefer deterministic, minimal command paths.
2. Verify tx outcomes; do not assume send success == execution success.
3. Avoid hidden defaults for deploy/CPI critical parameters.
4. Keep changes auditable and reproducible.
5. If uncertain, inspect compiler/CLI source directly.
