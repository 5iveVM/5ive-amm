5ive Testing Guide
This guide provides step-by-step instructions for testing the Five VM ecosystem, from validator setup to running end-to-end (E2E) verification tests for templates.

0. Validator-Free Runtime Harness (Preferred for CI)
Run the in-process onchain harness tests in `five-solana` (no `solana-test-validator` required):

```bash
./five-solana/scripts/run-runtime-tests.sh
```

If you need to run a single suite:

```bash
cargo test -p five --test runtime_template_fixture_tests -- --nocapture
```

Run only one template's runtime fixtures from that template folder:

```bash
cd five-templates/token
./run-runtime-fixtures.sh
```

Use localnet tests as parity checks after the runtime harness passes.

Add new script coverage by dropping JSON fixtures under:

```bash
five-templates/<template>/runtime-fixtures/*.json
```

The generic runtime test auto-discovers those fixtures and executes them through `five-solana` deploy/execute entrypoints.

0.1 Validator-Backed CU Harness (Localnet + Devnet, Manual)
Use this when you want CU measurements from a real validator while keeping the fast in-process harness unchanged.

Entry point:

```bash
cargo test -p five --test runtime_validator_cu_tests -- --ignored --nocapture
```

Required environment:

```bash
export FIVE_CU_NETWORK=localnet   # localnet | devnet
export FIVE_CU_PAYER_KEYPAIR="$HOME/.config/solana/id.json"
export FIVE_CU_PROGRAM_ID="<predeployed-five-program-id>"
```

Optional environment:

```bash
export FIVE_CU_RPC_URL="http://127.0.0.1:8899"   # localnet default
export FIVE_CU_SCENARIOS="token_full_e2e,external_non_cpi,external_interface_mapping_non_cpi,external_burst_non_cpi,memory_string_heavy,arithmetic_intensive"
export FIVE_CU_RESULTS_FILE="/tmp/localnet-cu.json"
```

Devnet safety gate (manual opt-in required):

```bash
export FIVE_CU_NETWORK=devnet
export FIVE_CU_DEVNET_OPT_IN=1
```

Notes:
- Devnet mode uses a pre-deployed program ID and does not auto-deploy.
- Every deploy/execute transaction is validated with confirmed `meta.err == null`.
- CU results + signatures are written to `five-solana/tests/benchmarks/validator-runs/` unless `FIVE_CU_RESULTS_FILE` is set.

1. Local Validator Setup
Before running any tests, ensure you have a clean local Solana validator running.

Reset & Start
To start a fresh validator (clearing all previous state):

# Stop any running validator
pkill -f solana-test-validator
# Remove old ledger data
rm -rf test-ledger
# Start new validator (run in a separate terminal)
solana-test-validator -r
Configure CLI
Ensure your Solana CLI is targeting localhost:

solana config set --url localhost
2. Building the Five VM
You can build the VM in Debug mode (with extensive logs) or Production mode (minimal/no logs).

Option A: Production Build (Recommended for Performance)
Use this for benchmarking CU usage or when "clean" execution is required.

Prerequisite: Ensure debug-logs is NOT in the default features in 
five-solana/Cargo.toml
.

cd five-solana
cargo build-sbf
Option B: Debug Build
Use this when you need msg! logs and debug_log! output to diagnose issues.

cd five-solana
cargo build-sbf --features debug-logs
3. Deploying the VM
Deploy the compiled program to your local validator.

# From five-mono root
solana program deploy \
    target/deploy/five.so \
    --program-id G7NFhT9ZBbrM1oqtNnWgd8mbB7A5FbbNt4XChvaPhA3A
Note: The Program ID G7NFhT9ZBbrM1oqtNnWgd8mbB7A5FbbNt4XChvaPhA3A is the default for local development.

4. Running Template E2E Tests
These scripts handle:

Compiling the Five DSL script (e.g., 
token.v
).
Deploying the bytecode to the Five VM.
Initializing the script functionality.
Executing verifications (init_mint, transfer, burn, etc.).
Token Template
cd five-templates/token
./e2e-token-test.sh
Counter Template (if applicable)
cd five-templates/counter
./e2e-counter-test.sh
5. Troubleshooting common issues
"Account not found"
Did you reset the validator?
Did you deploy the VM program?
"Logs truncated"
If running a Debug build, Solana logs may be truncated.
Use solana logs in a separate terminal to stream logs.
"Instruction Error: Custom(x)"
Check five-protocol/src/error.rs or VMErrorCode enum to map the error code.
