# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Five** is a blockchain virtual machine ecosystem for Solana, consisting of:
- **Five DSL** - A domain-specific language for writing smart contracts
- **Five VM (Mito)** - A stack-based virtual machine optimized for Solana execution
- **Five Protocol** - Shared protocol definitions, opcodes, and types
- **Five SDK** - TypeScript SDK for client-side interaction
- **Five CLI** - Command-line tools for compilation, deployment, and execution
- **Five Frontend** - Web-based IDE for writing and testing Five DSL contracts
- **Five LSP** - Language Server Protocol for editor integration and code completion

The system compiles Five DSL source code (`.v` files) to compact bytecode that executes on-chain via the Five Solana program. Developers can use the CLI, web IDE, or integrate the SDK directly.

## Repository Structure

```
five-mono/
├── five-protocol/          # Shared protocol: opcodes, types, varint encoding, headers
├── five-dsl-compiler/      # Rust compiler: DSL → bytecode
├── five-vm-mito/           # Core VM: zero-allocation Solana execution engine
├── five-solana/            # Solana program wrapper for on-chain execution
├── five-wasm/              # WASM bindings for browser/Node.js execution
├── five-sdk/               # TypeScript SDK (client-agnostic)
├── five-cli/               # CLI tools and test infrastructure
├── five-templates/         # Example contracts (counter, token, AMM, bridge, etc.)
├── five-frontend/          # Web IDE and UI for Five development
├── five-dex-frontend/      # DEX-specific frontend components
├── five-lsp/               # Language Server Protocol implementation for Five DSL
├── five-scripts/           # Utility scripts and automation
├── docs/                   # Project documentation
├── scripts/                # Build and deployment scripts
├── release-audit/          # Release management and audit tools
└── third_party/            # Vendored dependencies (pinocchio fork)
```

## Essential Commands

### Building

```bash
# Build all Rust crates
cargo build --release

# Build specific crate
cargo build -p five-dsl-compiler
cargo build -p five-vm-mito
cargo build -p five --release  # Solana program (five-solana)

# Build Solana program
cd five-solana
cargo-build-sbf --no-default-features --features production --sbf-out-dir target/deploy

# Build WASM bindings
cd five-wasm && ./build.sh

# Build TypeScript SDK
cd five-sdk && npm run build

# Build CLI
cd five-cli && npm run build
```

### Testing

```bash
# Run all Rust tests
cargo test

# Test specific crate
cargo test -p five-protocol
cargo test -p five-dsl-compiler
cargo test -p five-vm-mito

# Run compiler tests with output
cargo test -p five-dsl-compiler -- --nocapture

# Focused regressions for external import/use call resolution
cargo test -p five-dsl-compiler --lib bytecode_generator::ast_generator::functions::tests -- --nocapture
cargo test -p five-dsl-compiler --test lib test_external_imported_items_allow_unqualified_call -- --nocapture
cargo test -p five-dsl-compiler --test lib test_external_imported_items_unqualified_ambiguous_call_fails -- --nocapture

# Run E2E template tests (requires localnet)
cd five-templates/counter && node e2e-counter-test.mjs
cd five-templates/token && node e2e-token-test.mjs

# Run CLI test suite
cd five-cli && npm run test:scripts
```

### Benchmarking (BPF CU, no localnet)

Use this workflow when measuring on-chain compute for VM + bytecode execution in-process (via `solana-program-test`) without `solana-test-validator`.

```bash
# 1) Compile benchmark/template bytecode
CARGO_TARGET_DIR=/tmp/five-target cargo run -q -p five-dsl-compiler --bin five -- compile \
  five-templates/defi-bench/src/defi_bench.v \
  -o five-templates/defi-bench/src/defi_bench.bin \
  --v2-preview

# Optional: compile token template too
CARGO_TARGET_DIR=/tmp/five-target cargo run -q -p five-dsl-compiler --bin five -- compile \
  five-templates/token/src/token.v \
  -o five-templates/token/src/token.bin \
  --v2-preview

# 2) Build SBF program used by runtime CU tests
cargo build-sbf --manifest-path five-solana/Cargo.toml

# 3) Run CU harness with a specific fixture
# DeFi benchmark fixture
CARGO_TARGET_DIR=/tmp/five-target \
FIVE_BPF_FIXTURE=five-templates/defi-bench/runtime-fixtures/defi_bench.json \
cargo test -p five --test runtime_bpf_cu_tests -- --nocapture

# Token fixture (default if FIVE_BPF_FIXTURE is unset)
CARGO_TARGET_DIR=/tmp/five-target \
FIVE_BPF_FIXTURE=five-templates/token/runtime-fixtures/init_mint.json \
cargo test -p five --test runtime_bpf_cu_tests -- --nocapture

# Direct external-call CU tests (non-CPI) for token.v
# Requires token bytecode present at five-templates/token/src/token.bin
cargo-build-sbf --manifest-path five-solana/Cargo.toml
cargo test -p five --test runtime_bpf_cu_tests external_token_transfer_non_cpi_bpf_compute_units -- --nocapture
cargo test -p five --test runtime_bpf_cu_tests external_token_transfer_burst_non_cpi_bpf_compute_units -- --nocapture
cargo test -p five --test runtime_bpf_cu_tests external_token_transfer_mass_non_cpi_bpf_compute_units -- --nocapture
```

What the harness prints:
- `BPF_CU minimal_execute_floor=...` = runtime baseline overhead floor.
- `BPF_CU step=<name> ... units=<n>` = per public function call CU.
- `BPF_CU deploy=...` = script deployment CU in the harness.
- `BPF_CU fixture=<name> total_units=...` = aggregate run cost.
- `BPF_CU external_*` lines = non-CPI external call runs (transfer, burst, mass).

Fixture location and format:
- Primary fixture file for DeFi math benchmarking:
  - `five-templates/defi-bench/runtime-fixtures/defi_bench.json`
- Test implementation:
  - `five-solana/tests/runtime_bpf_cu_tests.rs`
- Fixtures specify:
  - `bytecode_path`, `permissions`, `steps[]` with `function_index`, params, and expected outcome.

Important notes for stable CU measurements:
- Use a fixed target dir (`CARGO_TARGET_DIR=/tmp/five-target`) to reduce rebuild noise.
- Always recompile `.v -> .bin` before measuring after compiler/opcode changes.
- The test function name is `token_e2e_bpf_compute_units`, but it runs whichever fixture is selected by `FIVE_BPF_FIXTURE`.
- Keep fixture inputs valid for expected-success steps (for example, avoid values that violate `require(...)` constraints), or mark those steps with expected error.
- The mass external transfer test currently uses 10 transfer pairs to stay within `MAX_FUNCTION_PARAMS=24`.

Common failures:
- `failed reading fixture ... No such file or directory`
  - Fix `FIVE_BPF_FIXTURE` path.
- Step fails with custom program error (for example `0x232b`)
  - Fixture parameters violated DSL `require(...)` checks.
- CU unexpectedly regresses
  - Rebuild SBF and recompile fixture bytecode, then rerun with same fixture and target dir.
- `invalid instruction data` in external-call tests
  - Ensure `token.bin` is built and the SBF program is rebuilt.
  - Note: `external_token_all_public_non_cpi_bpf_compute_units` is `#[ignore]` pending support for non-`transfer` external calls.

### Benchmarking (Validator-backed CU: localnet/devnet)

Use this when you want CU numbers from a real validator RPC (not `ProgramTest`), while keeping the fast in-process harness unchanged.

Files:
- Orchestrator: `five-solana/tests/runtime_validator_cu_tests.rs`
- Backend: `five-solana/tests/harness/validator.rs`
- Output reports: `five-solana/tests/benchmarks/validator-runs/*.json`

Default validator scenarios (V1):
- `token_full_e2e`
- `external_non_cpi`
- `external_interface_mapping_non_cpi`
- `external_burst_non_cpi`
- `memory_string_heavy`
- `arithmetic_intensive`

Required env:
```bash
export FIVE_CU_NETWORK=localnet              # localnet | devnet
export FIVE_CU_PAYER_KEYPAIR="$HOME/.config/solana/id.json"
export FIVE_CU_PROGRAM_ID="<predeployed-five-program-id>"
```

Optional env:
```bash
export FIVE_CU_RPC_URL="http://127.0.0.1:8899"   # defaults by network
export FIVE_CU_SCENARIOS="token_full_e2e,external_non_cpi,external_interface_mapping_non_cpi,external_burst_non_cpi,memory_string_heavy,arithmetic_intensive"
export FIVE_CU_RESULTS_FILE="/tmp/localnet-cu.json"
```

Run command:
```bash
cargo test -p five --test runtime_validator_cu_tests validator_cu_orchestrator -- --ignored --nocapture
```

#### Localnet quick start (copy/paste)
```bash
# 1) Start local validator
solana-test-validator -r

# 2) Build and deploy Five program to local validator
cargo build-sbf --manifest-path five-solana/Cargo.toml
solana program deploy target/deploy/five.so \
  --program-id target/deploy/five-keypair.json \
  --url http://127.0.0.1:8899 \
  --keypair "$HOME/.config/solana/id.json"

# 3) Run validator CU harness
FIVE_CU_NETWORK=localnet \
FIVE_CU_RPC_URL=http://127.0.0.1:8899 \
FIVE_CU_PAYER_KEYPAIR="$HOME/.config/solana/id.json" \
FIVE_CU_PROGRAM_ID="$(solana-keygen pubkey target/deploy/five-keypair.json)" \
cargo test -p five --test runtime_validator_cu_tests validator_cu_orchestrator -- --ignored --nocapture
```

#### Devnet run (manual opt-in only)
Devnet is intentionally gated and does not auto-airdrop.

```bash
FIVE_CU_NETWORK=devnet \
FIVE_CU_DEVNET_OPT_IN=1 \
FIVE_CU_PAYER_KEYPAIR="$HOME/.config/solana/id.json" \
FIVE_CU_PROGRAM_ID="<devnet-five-program-id>" \
cargo test -p five --test runtime_validator_cu_tests validator_cu_orchestrator -- --ignored --nocapture
```

#### What success looks like
- Per-step lines:
  - `BPF_CU validator step=<name> signature=<sig> units=<n> success=true`
- Per-scenario lines:
  - `BPF_CU validator scenario=<name> deploy=<n> execute=<n> total=<n> steps=<k>`
- Report line:
  - `BPF_CU validator report=<path>`
- Test status:
  - `test result: ok. 1 passed; 0 failed`

#### Troubleshooting
- `RPC response error ... transaction too large`
  - The validator harness now auto-falls back to chunked upload (`init_large_program` + append/finalize). If this still appears, verify you are on latest harness code.
- `custom program error: 0x453` during execute
  - This is typically missing execute-fee recipient when fees are non-zero. The harness now zeroes VM fees on setup (or applies fixture fee override).
- `invalid instruction data` in external scenarios
  - Ensure external imported bytecode accounts are passed as `*_script` extras (read-only imported script behavior).
- `not enough signers`
  - Ensure fixture extra signer accounts are created as signers and forwarded in execute extras; latest harness includes signer resolution for step extras.
- Devnet run exits early for balance
  - Expected. Devnet mode does not auto-airdrop; pre-fund the payer wallet.

### Unified BPF-CU Benchmark Suite (micro + scenario + regression gates)

Use this suite as the default performance workflow for VM hotpath work.

Files:
- Harness utilities: `five-solana/tests/harness/perf.rs`
- Micro opcode suite: `five-solana/tests/runtime_bpf_opcode_micro_cu_tests.rs`
- Scenario suite: `five-solana/tests/runtime_bpf_cu_tests.rs`
- Baseline snapshots: `five-solana/tests/benchmarks/baseline/<commit>.json`
- Regression allowlist: `five-solana/tests/benchmarks/allowlist/<commit>.json`
- Runner script: `scripts/ci-bpf-bench.sh`

Standard output lines:
- `BENCH family=<...> opcode=<...> variant=<...> deploy=<...> execute=<...> total=<...>`
- `SCENARIO name=<...> execute=<...> total=<...>`

Run workflow:

```bash
# Build SBF + run micro + scenario suites (default baseline key: local)
./scripts/ci-bpf-bench.sh

# Use a named baseline snapshot key
FIVE_BENCH_BASELINE_COMMIT=pre-opt-2026-02-12 ./scripts/ci-bpf-bench.sh

# Run suites individually
cargo test -p five --test runtime_bpf_opcode_micro_cu_tests -- --nocapture
cargo test -p five --test runtime_bpf_cu_tests -- --nocapture
```

Regression policy implemented in harness:
- Strict no-regression check compares current CU vs baseline for `deploy`, `execute`, and `total`.
- Missing baseline file or test entry is non-fatal and prints:
  - `BENCH baseline_missing ...`
  - `BENCH baseline_entry_missing ...`
- Allowlist entries can exempt specific fields (`deploy`/`execute`/`total`) per test.

Baseline management:
1. Create/update baseline file at `five-solana/tests/benchmarks/baseline/<commit>.json`.
2. Add per-test metrics in `tests` map (key = test name used in `assert_no_regression`).
3. If a regression is intentional, add allowlist entry at:
   `five-solana/tests/benchmarks/allowlist/<commit>.json`
   with owner/rationale/expiry commit and optional field list.

Current scenario notes:
- `scenario_high_cpi_density_bpf_compute_units` and `scenario_memory_string_heavy_bpf_compute_units` run full fixture flows and emit `SCENARIO` lines.
- `scenario_high_external_call_fanout_bpf_compute_units` currently acts as a regression hook line; high-fanout external execution is still measured by:
  - `external_token_transfer_burst_non_cpi_bpf_compute_units`
  - `external_token_transfer_mass_non_cpi_bpf_compute_units`

### Hotpath Optimization Playbook (BPF-first, zero-copy safe)

When optimizing VM execution:

1. Measure first
- Add/adjust micro benchmark in `runtime_bpf_opcode_micro_cu_tests.rs`.
- Confirm scenario impact in `runtime_bpf_cu_tests.rs`.
- Capture BENCH/SCENARIO before changing code.

2. Prioritize safe wins
- Remove intermediate copies.
- Convert byte-by-byte decoding to bounded slice decode.
- Collapse repeated bounds checks into one check when sound.
- Keep account/signer/writable/owner checks unchanged.
- Prefer immutable slice reads and stack-local temporaries over heap allocation.

3. Re-run both suites every change
- Micro suite catches opcode-level regressions.
- Scenario suite catches system-level regressions and interaction effects.

4. Gate and document
- If CU worsens, either fix or explicitly allowlist with rationale and expiry.
- Keep emitted BENCH/SCENARIO format stable for tooling and diffability.

Recommended hotspots to inspect first:
- Dispatch + locals/stack handlers (`five-vm-mito/src/execution.rs`, `five-vm-mito/src/handlers/locals.rs`, `five-vm-mito/src/handlers/stack_ops.rs`)
- Input/memory decode paths (`five-vm-mito/src/context.rs`, `five-vm-mito/src/handlers/memory.rs`)
- External/system call paths (`five-vm-mito/src/handlers/functions.rs`, `five-vm-mito/src/handlers/system/invoke.rs`)

### Interface CPI CU tests (SPL + Anchor, no validator)

Use this when validating interface-based CPI CU usage for SPL Token and Anchor program calls.

```bash
# 1) Recompile CPI example bytecode (always do this after compiler changes)
CARGO_TARGET_DIR=/tmp/five-target cargo run -p five-dsl-compiler --bin five -- compile \
  five-templates/cpi-examples/spl-token-mint-e2e.v \
  -o five-templates/cpi-examples/spl-token-mint-e2e.bin

CARGO_TARGET_DIR=/tmp/five-target cargo run -p five-dsl-compiler --bin five -- compile \
  five-templates/cpi-examples/anchor-program-call-e2e.v \
  -o five-templates/cpi-examples/anchor-program-call-e2e.bin

# 2) Build the Five SBF program used by runtime_bpf_cu_tests
cargo-build-sbf --manifest-path five-solana/Cargo.toml --sbf-out-dir target/deploy

# 3) Build external Anchor comparison program .so used by fixture
cargo-build-sbf \
  --manifest-path five-templates/anchor-token-comparison/programs/anchor-token-comparison/Cargo.toml \
  --sbf-out-dir target/deploy

# 4) Run both interface CU tests
CARGO_TARGET_DIR=/tmp/five-target \
cargo test -p five --test runtime_bpf_cu_tests interface_cpi_bpf_compute_units -- --nocapture
```

Notes:
- Use `-p five` (crate name from `five-solana/Cargo.toml`), not `-p five-solana`.
- Keep `CARGO_TARGET_DIR=/tmp/five-target` to avoid Cargo lock contention and reduce rebuild noise.
- The interface fixtures are:
  - `five-templates/cpi-examples/runtime-fixtures/spl-token-mint-e2e.json`
  - `five-templates/cpi-examples/runtime-fixtures/anchor-program-call-e2e.json`
- Current passing Anchor fixture covers mint/transfer/burn/freeze/thaw CPI flow against `anchor_token_comparison.so`.

### Five CLI Usage

```bash
# Compile Five DSL to bytecode
five compile script.v
five compile script.v -o script.five

# Project-based workflow
five init my-project
five build
five deploy --project .
five execute --project .

# Local WASM execution
five local execute script.v 0

# On-chain execution
five deploy script.five
five execute <SCRIPT_ACCOUNT> -f 0 --params "[10, 20]"
```

## Architecture

### Compilation Pipeline

```
Five DSL (.v) → Tokenizer → Parser → Type Checker → Bytecode Generator → .fbin/.five
```

1. **Tokenization** - Lexical analysis into tokens
2. **Parsing** - Build Abstract Syntax Tree (AST)
3. **Type Checking** - Semantic analysis with cross-module symbol resolution
4. **Bytecode Generation** - Emit optimized bytecode with varint encoding

### VM Execution Model

The Five VM is a **stack-based virtual machine** with:
- **64-byte temp buffer** for intermediate values
- **Zero-allocation design** for Solana compute efficiency
- **Lazy-loading** for account data (AccountRef pattern)
- **varint encoding** reduces bytecode size by 30-50%

Key opcode categories (see `five-protocol/OPCODE_SPEC.md`):
- Control flow: `HALT`, `JUMP`, `JUMP_IF`, `REQUIRE`, `RETURN`
- Stack ops: `PUSH_U8/U16/U32/U64`, `POP`, `DUP`, `SWAP`
- Arithmetic: `ADD`, `SUB`, `MUL`, `DIV`, checked variants
- Memory: `LOAD_FIELD`, `STORE_FIELD`, `LOAD_FIELD_PUBKEY`
- Accounts: `GET_KEY`, `CHECK_SIGNER`, `CHECK_WRITABLE`, `TRANSFER`
- System: `INVOKE`, `DERIVE_PDA`, `INIT_ACCOUNT`

### On-Chain Execution Flow

```
Client → five-solana program → Five VM (Mito) → Account state changes
```

The `five-solana` crate wraps the VM and handles:
- Instruction parsing and dispatch
- Account constraint validation
- System program CPI for account creation

## Key Files by Component

### five-protocol
- `src/opcodes.rs` - All VM opcode definitions
- `src/types.rs` - Type constants and `ImportableAccountHeader`
- `src/encoding.rs` - varint encoding/decoding
- `OPCODE_SPEC.md` - RFC-1 opcode specification

### five-dsl-compiler
- `src/compiler/pipeline.rs` - Unified compilation pipeline
- `src/parser/` - DSL parser (expressions, statements, blocks)
- `src/type_checker/` - Type validation and inference
- `src/bytecode_generator/` - Bytecode emission (modular by AST node type)
- `src/error/` - Structured error system with templates

### five-vm-mito
- `src/lib.rs` - VM entry point and execution loop
- `src/context.rs` - ExecutionContext and state management
- `src/handlers/` - Opcode handlers (memory, arithmetic, accounts, system)
- `src/utils.rs` - Stack operations and utilities

### five-solana
- `src/lib.rs` - Solana program entry point
- `src/instructions.rs` - Instruction parsing and dispatch

### five-sdk
- `src/FiveSDK.ts` - Main SDK class (compilation, execution, instruction generation)
- `src/encoding/ParameterEncoder.ts` - varint parameter encoding
- `src/lib/varint-encoder.js` - varint utility implementation

### five-frontend
- Web-based IDE for Five DSL development
- Includes code editor, compiler integration, and testing interface
- LSP-enabled for real-time language features

### five-lsp
- Language Server Protocol implementation for Five DSL
- Provides editor integration (VS Code, etc.) with code completion, diagnostics, and navigation

### five-cli
- `src/commands/` - CLI command implementations
- TypeScript-based command-line interface for Five development workflow

## Template Examples

Five includes a comprehensive template library in `five-templates/`:

**Core Templates (Actively Maintained):**
- `counter/` - Simple counter with account initialization
- `token/` - Full SPL Token-compatible implementation
- `defi-bench/` - DeFi math benchmarking suite
- `cpi-examples/` - Cross-program invocation examples (SPL Token, Anchor)
- `anchor-token-comparison/` - Comparative performance testing

**DeFi Applications:**
- `amm/` - Automated Market Maker
- `lending/` - Lending protocol
- `staking/` - Token staking
- `oracle/` - Price oracle
- `vault/` - Vault contract

**Other Applications:**
- `nft/` - NFT minting and trading
- `governance/` - DAO governance
- `bridge/` - Cross-chain bridge
- `escrow/` - Payment escrow

Each template includes source code (`.v`), fixtures for testing, and integration tests.

## Five DSL Language

### Basic Syntax

```v
// Global state
mut counter: u64;

// Initialization block
init {
    counter = 0;
}

// Public function (callable on-chain)
pub increment() -> u64 {
    counter = counter + 1;
    return counter;
}

// Internal function
fn helper(x: u64) -> u64 {
    return x * 2;
}
```

### Account Constraints

```v
pub transfer(
    from: account @mut @signer,
    to: account @mut,
    amount: u64
) {
    // @mut = writable, @signer = must sign transaction
    // @init(payer=X, space=N) for account creation
}
```

### Module System

```v
use lib;                    // Import local module
use utils::helpers;         // Nested import
use "PubkeyAddress"::{fn};  // External contract import

pub main() {
    lib::calculate(10);     // Qualified function call
}
```

## Development Guidelines

### Adding New Opcodes

1. Define opcode constant in `five-protocol/src/opcodes.rs`
2. Add handler in `five-vm-mito/src/handlers/`
3. Update compiler emission in `five-dsl-compiler/src/bytecode_generator/`
4. Add tests in both crates
5. Update `OPCODE_SPEC.md`

### Modifying Bytecode Generation

- AST generation is modular: `bytecode_generator/ast_generator/*.rs`
- Each AST node type has its own module (expressions, statements, functions)
- Test with `cargo test --test golden_bytecode` for regressions

### Error Handling

- Compiler errors go through `five-dsl-compiler/src/error/`
- Use error codes from `error/registry.rs`
- VM errors use `five_protocol::VMError`

### Testing Workflow

1. Write Five DSL test script in `five-templates/` or `five-cli/test-scripts/`
2. Add `// @test-params X Y` comments for parameterized tests
3. Run locally with WASM: `five local execute script.v 0`
4. Test on-chain with localnet after `solana-test-validator`

### SDK Usage with Parameter Encoding

When using `FiveSDK.generateExecuteInstruction()` with functions that have mixed account/data parameters:

```javascript
import { FiveSDK } from 'five-sdk';

// Load the ABI from compiled .five file
const fiveFile = JSON.parse(fs.readFileSync('build/contract.five', 'utf-8'));
const abi = fiveFile.abi;

// Get function definition to determine parameter order
const functionDef = abi.functions.find(f => f.name === 'myFunction');

// Build merged parameters array in correct order (accounts and data mixed per ABI)
const mergedParams = [];
functionDef.parameters.forEach(param => {
  if (param.is_account || param.isAccount) {
    mergedParams.push(accountPublicKey);  // Account parameter
  } else {
    mergedParams.push(dataValue);          // Data parameter (u64, pubkey, string, etc.)
  }
});

// Generate instruction with ABI metadata
const instruction = await FiveSDK.generateExecuteInstruction(
  scriptAccountPubkey,
  functionIndex,
  mergedParams,         // All parameters in correct order
  accountPubkeys,       // Also pass account list
  connection,
  {
    scriptMetadata: abi,  // IMPORTANT: Pass ABI for proper parameter mapping
    vmStateAccount: vmStatePda,
    fiveVMProgramId: programId,
    adminAccount: payerPubkey
  }
);
```

**Key Points:**
- Always pass `scriptMetadata: abi` in options
- Merge account and data parameters in correct order from function definition
- The SDK will identify accounts via ABI and map them to indices
- All parameters are encoded via WASM encoder for reliability

## Current Status

### Fully Functional
- ✅ Full compilation pipeline (Five DSL → Bytecode)
- ✅ Local WASM execution via CLI and SDK
- ✅ On-chain deployment and execution on Solana
- ✅ SDK parameter encoding for mixed account/data parameters
- ✅ Account constraint validation (@mut, @signer, @init)
- ✅ CPI interface integration (SPL Token, Anchor)
- ✅ Web IDE and LSP for development workflow

### Benchmarking & Metrics
- ✅ BPF compute unit (CU) measurement harness (`runtime_bpf_cu_tests`)
- ✅ Token template E2E test suite (352,972 CU total)
- ✅ Counter template with account initialization
- ✅ CPI performance testing against SPL and Anchor programs

### Recent Improvements
- Removed VLE (Variable Length Encoding) terminology; normalized to varint encoding
- Consolidated parameter encoding through unified WASM encoder
- Validated protocol/VM/compiler alignment across all three components
- Expanded template library (token, counter, AMM, bridge, DAO, etc.)

### Known Issues

Previously resolved:
- ✅ **SDK parameter encoding** - Fixed by passing `scriptMetadata` ABI to SDK
- ✅ **@init constraint** - Works correctly (counter template demonstrates this)
- ✅ **Token template** - Was blocked by string parameter handling in DSL, not @init

**Current Focus Areas:**
- Protocol/VM alignment validation
- CPI interface integration and optimization
- Template expansion and benchmark suite
- LSP and IDE feature completeness

## Deployment

### Local Development

```bash
# Start Solana localnet
solana-test-validator

# Deploy Five VM program
cd five-solana
cargo build --release
solana program deploy target/deploy/five.so --url http://127.0.0.1:8899
```

### Program IDs
- Five VM Program: `HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg` (localnet)

## Mandatory Fee Enforcement (Updated)

Fee payment is now strict for non-zero deploy/execute fees:
- Fee payer **must not** equal fee recipient.
- Fee recipient account must be present and writable on fee-bearing txs.
- System Program account (`11111111111111111111111111111111`) must be present for fee CPI transfers.

VM state now stores:
- `authority` (admin authority)
- `fee_recipient` (treasury recipient)
- fee lamports + initialization/version fields

### Devnet/localnet fee recipient management

```bash
# Set deploy + execute fees
node scripts/vm-state-set-fees.mjs \
  --rpc-url https://api.devnet.solana.com \
  --program-id 4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d \
  --vm-state 8ip3qGGETf8774jo6kXbsTTrMm5V9bLuGC4znmyZjT3z \
  --keypair ~/.config/solana/id.json \
  --deploy-fee 10000 \
  --execute-fee 85734

# Set explicit fee recipient treasury
node scripts/vm-state-set-fee-recipient.mjs \
  --rpc-url https://api.devnet.solana.com \
  --program-id 4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d \
  --vm-state 8ip3qGGETf8774jo6kXbsTTrMm5V9bLuGC4znmyZjT3z \
  --keypair ~/.config/solana/id.json \
  --fee-recipient <TREASURY_PUBKEY>

# Verify canonical vm_state fields (authority + fee recipient + fees)
node scripts/vm-state-parity-check.mjs \
  --rpc-url https://api.devnet.solana.com \
  --program-id 4Qxf3pbCse2veUgZVMiAm3nWqJrYo2pT4suxHKMJdK1d \
  --vm-state 8ip3qGGETf8774jo6kXbsTTrMm5V9bLuGC4znmyZjT3z \
  --expected-authority <AUTHORITY_PUBKEY> \
  --expected-fee-recipient <TREASURY_PUBKEY> \
  --expected-deploy-fee 10000 \
  --expected-execute-fee 85734
```
