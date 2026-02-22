# 5IVE CLI

CLI for building, testing, deploying, and executing 5ive DSL programs.

## Install

```bash
npm install -g @5ive-tech/cli
5ive --version
```

Or run without global install:

```bash
npx @5ive-tech/cli --help
```

## Quick Start

### 1) Initialize a project

```bash
5ive init my-program
cd my-program
```

`5ive init` generates an `AGENTS.md` playbook in every new project so agents can immediately compile, test, deploy, and execute 5ive DSL programs with the same workflow as developers.

### 2) Compile to a `.five` artifact (recommended)

```bash
5ive build
```

The `.five` artifact contains bytecode and ABI and is the best default for deployment and SDK integration.

### 3) Run locally

```bash
5ive execute build/main.five --local -f 0
```

### 4) Configure on-chain target

```bash
5ive config init
5ive config set --target devnet
5ive config set --keypair ~/.config/solana/id.json
5ive config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
```

### 5) Deploy and execute on-chain

```bash
5ive deploy build/main.five --target devnet
5ive execute build/main.five --target devnet -f 0
```

If you already have a deployed script account:

```bash
5ive execute --script-account <SCRIPT_ACCOUNT_PUBKEY> --target devnet -f 0
```

## Program ID Resolution

For on-chain commands (`deploy`, `execute`, `namespace`), program ID precedence is:

1. `--program-id` flag
2. `five.toml` `[deploy].program_id`
3. `5ive config` stored value for current target
4. `FIVE_PROGRAM_ID` environment variable

If none are set, on-chain commands fail fast with a program ID setup error.

## Standard Workflows

### Project build flow

```bash
5ive build
5ive deploy build/main.five --project .
5ive execute build/main.five --project . -f 0
```

`5ive build` / `--project` use `five.toml` and the generated manifest to resolve artifacts.

### Run tests

```bash
5ive test --sdk-runner
5ive test --filter "test_*" --verbose
5ive test --on-chain --target local
5ive test tests/ --on-chain --target devnet
5ive test tests/ --on-chain --target mainnet --allow-mainnet-tests --max-cost-sol 0.5
```

### Namespace operations

```bash
5ive namespace register @your-domain
5ive namespace bind @your-domain/program --script <SCRIPT_ACCOUNT_PUBKEY>
5ive namespace resolve @your-domain/program
```

## Advanced Workflows (Optional)

### Compile diagnostics and machine-readable metrics

```bash
5ive build \
  --analyze \
  --metrics-output build/compile-metrics.json \
  --metrics-format json \
  --error-format json
```

### Project-aware execution from `five.toml` context

```bash
5ive execute --project . -f 0
5ive execute --project . -f 0 --params params.json --target devnet
```

### Deploy large artifacts with chunk/optimization controls

```bash
5ive deploy build/main.five --target devnet --optimized --progress
5ive deploy build/main.five --target devnet --force-chunked --chunk-size 900
5ive deploy build/main.five --target devnet --dry-run --format json
```

### Advanced test modes

```bash
5ive test --sdk-runner --format json
5ive test test-scripts/ --on-chain --target devnet --batch --analyze-costs
5ive test tests/ --on-chain --target mainnet --allow-mainnet-tests --max-cost-sol 0.5
5ive test --watch --parallel 4
```

### Namespace manager and lockfile modes

```bash
# On-chain manager flow
5ive namespace register @acme --manager <MANAGER_SCRIPT_ACCOUNT>
5ive namespace bind @acme/payments --script <SCRIPT_ACCOUNT_PUBKEY> --manager <MANAGER_SCRIPT_ACCOUNT>

# Local lockfile-only flow (no manager RPC)
5ive namespace resolve @acme/payments --local
```

### Config layering and explicit RPC overrides

```bash
5ive config set --rpc-url https://api.devnet.solana.com --target devnet
5ive config set --show-config true
5ive deploy build/main.five --target devnet --network https://your-rpc.example.com
```

## Artifact and SDK Interop

`@5ive-tech/cli` and `@5ive-tech/sdk` work best with `.five` artifacts.

## Common Commands

```bash
5ive help <command>
5ive help compile
5ive help deploy
5ive help execute
5ive help config
```

## Troubleshooting

### Global `5ive` is stale vs monorepo source

```bash
# Run local CLI dist directly from this repository
node ./dist/index.js --version
node ./dist/index.js init my-program
```

### `Program ID required` or `owner/program mismatch`

```bash
5ive config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
# or
5ive deploy build/main.five --program-id <FIVE_VM_PROGRAM_ID> --target devnet
```

### `Keypair file not found`

```bash
5ive config set --keypair ~/.config/solana/id.json
```

### Command-specific help

```bash
5ive build --help
5ive execute --help
5ive config --help
```
