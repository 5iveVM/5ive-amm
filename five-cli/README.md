# Five CLI

CLI for building, testing, deploying, and executing 5ive DSL programs.

This README is for external users working with:
- `five-cli`
- `five-sdk`
- [5ive frontend](https://5ive.tech)

## Install

```bash
npm install -g @five-vm/cli
five --version
```

Or run without global install:

```bash
npx @five-vm/cli --help
```

## Quick Start

### 1) Initialize a project

```bash
five init my-program
cd my-program
```

### 2) Compile to a `.five` artifact (recommended)

```bash
five compile src/main.v -o build/main.five
```

The `.five` artifact contains bytecode and ABI and is the best default for deployment and SDK integration.

### 3) Run locally

```bash
five execute build/main.five --local -f 0
```

### 4) Configure on-chain target

```bash
five config init
five config set --target devnet
five config set --keypair ~/.config/solana/id.json
five config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
```

### 5) Deploy and execute on-chain

```bash
five deploy build/main.five --target devnet
five execute build/main.five --target devnet -f 0
```

If you already have a deployed script account:

```bash
five execute --script-account <SCRIPT_ACCOUNT_PUBKEY> --target devnet -f 0
```

## Program ID Resolution

For on-chain commands (`deploy`, `execute`, `deploy-and-execute`, `namespace`), program ID precedence is:

1. `--program-id` flag
2. `five.toml` `[deploy].program_id`
3. `five config` stored value for current target
4. `FIVE_PROGRAM_ID` environment variable

If none are set, on-chain commands fail fast with a program ID setup error.

See: [PROGRAM_ID_SETUP.md](./PROGRAM_ID_SETUP.md)

## Standard Workflows

### Project build flow

```bash
five build
five deploy build/main.five --project .
five execute build/main.five --project . -f 0
```

`five build` / `--project` use `five.toml` and the generated manifest to resolve artifacts.

### One-command deploy+execute (great for smoke tests)

```bash
five deploy-and-execute build/main.five --target devnet -f 0
```

### Run tests

```bash
five test --sdk-runner
five test tests/ --on-chain --target devnet
```

### Namespace operations

```bash
five namespace register @your-domain
five namespace bind @your-domain/program --script <SCRIPT_ACCOUNT_PUBKEY>
five namespace resolve @your-domain/program
```

## Advanced CLI Workflows (Optional)

Most teams can stay on the quick-start path. Use these features when you need deeper control.

### 1) Compile diagnostics and machine-readable metrics

```bash
five compile src/main.v \
  --analyze \
  --metrics-output build/compile-metrics.json \
  --metrics-format json \
  --error-format json
```

### 2) Project-aware execution from `five.toml` context

```bash
five execute --project . -f 0
five execute --project . -f 0 --params params.json --target devnet
```

### 3) Deploy large artifacts with chunk/optimization controls

```bash
five deploy build/main.five --target devnet --optimized --progress
five deploy build/main.five --target devnet --force-chunked --chunk-size 900
five deploy build/main.five --target devnet --dry-run --format json
```

### 4) Deploy-and-execute for fast integration checks

```bash
five deploy-and-execute build/main.five --target devnet -f 0 -p "[100]"
five deploy-and-execute src/main.v --target local --debug --cleanup
```

### 5) Advanced test modes

```bash
five test --sdk-runner --format json
five test test-scripts/ --on-chain --target devnet --batch --analyze-costs
five test --watch --parallel 4
```

### 6) Namespace manager and lockfile modes

```bash
# On-chain manager flow
five namespace register @acme --manager <MANAGER_SCRIPT_ACCOUNT>
five namespace bind @acme/payments --script <SCRIPT_ACCOUNT_PUBKEY> --manager <MANAGER_SCRIPT_ACCOUNT>

# Local lockfile-only flow (no manager RPC)
five namespace resolve @acme/payments --local
```

### 7) Config layering and explicit RPC overrides

```bash
five config set --rpc-url https://api.devnet.solana.com --target devnet
five config set --show-config true
five deploy build/main.five --target devnet --network https://your-rpc.example.com
```

## Artifact and SDK Interop

`five-cli` and `five-sdk` work best with `.five` artifacts:

- Compile with CLI to `.five`
- Load and interact in SDK via ABI-aware helpers
- Use the same artifact in frontend flows on [5ive.tech](https://5ive.tech)

You can still use `.bin` where needed, but `.five` is the preferred default.

## Common Commands

```bash
five help <command>
five help compile
five help deploy
five help execute
five help config
```

## Troubleshooting

### `Program ID required` or `owner/program mismatch`

Set or override the VM program ID:

```bash
five config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
# or
five deploy build/main.five --program-id <FIVE_VM_PROGRAM_ID> --target devnet
```

### `Keypair file not found`

```bash
five config set --keypair ~/.config/solana/id.json
```

### Command-specific help from global help flow

```bash
five compile --help
five execute --help
five config --help
```

## Links

- Program ID setup: [PROGRAM_ID_SETUP.md](./PROGRAM_ID_SETUP.md)
- 5ive frontend: [5ive.tech](https://5ive.tech)
