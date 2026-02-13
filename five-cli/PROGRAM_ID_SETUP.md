# Program ID Setup (Five CLI)

This guide covers how to configure the Five VM program ID for on-chain use with `five-cli`.

## What is the Program ID?

The program ID is the on-chain address of the Five VM program (a Solana public key).

You need it for:
- `five deploy`
- `five execute` (on-chain)
- `five deploy-and-execute`
- `five namespace` flows that derive/check on-chain state

## Fast Setup

```bash
five config init
five config set --target devnet
five config set --keypair ~/.config/solana/id.json
five config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
```

Verify:

```bash
five config get programIds
```

## Resolution Order (Highest to Lowest)

1. `--program-id <PUBKEY>`
2. `five.toml` `[deploy].program_id`
3. Stored CLI config (`five config set --program-id ...` per target)
4. `FIVE_PROGRAM_ID` environment variable

If none resolve, on-chain commands fail with a setup error.

## Configuration Methods

### 1) CLI config (recommended)

```bash
five config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
five config set --program-id <FIVE_VM_PROGRAM_ID_TESTNET> --target testnet
five config set --program-id <FIVE_VM_PROGRAM_ID_MAINNET> --target mainnet
```

Read values:

```bash
five config get programIds
five config get programIds.devnet
```

### 2) Project-level config (`five.toml`)

```toml
[deploy]
program_id = "<FIVE_VM_PROGRAM_ID>"
cluster = "devnet"
```

Use this when you want project defaults checked into version control.

### 3) Per-command override

```bash
five deploy build/main.five --target devnet --program-id <FIVE_VM_PROGRAM_ID>
five execute --script-account <SCRIPT_ACCOUNT> --target devnet --program-id <FIVE_VM_PROGRAM_ID>
```

### 4) Environment variable

```bash
export FIVE_PROGRAM_ID=<FIVE_VM_PROGRAM_ID>
five deploy build/main.five --target devnet
```

Useful for CI.

## Practical Workflows

### One-time personal setup

```bash
five config init
five config set --target devnet
five config set --keypair ~/.config/solana/id.json
five config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
```

### CI/CD setup

```bash
export FIVE_PROGRAM_ID=<FIVE_VM_PROGRAM_ID>
five deploy build/main.five --target devnet
```

### Switch networks

```bash
five config set --target devnet
five deploy build/main.five --target devnet

five config set --target testnet
five deploy build/main.five --target testnet
```

## Advanced Program ID Usage

### One-off override for smoke tests

```bash
five deploy-and-execute build/main.five --target devnet --program-id <FIVE_VM_PROGRAM_ID>
```

### Namespace PDA derivation against a custom VM program

```bash
five namespace resolve @acme/payments --program-id <FIVE_VM_PROGRAM_ID>
```

### Combine custom RPC + explicit program ID

```bash
five deploy build/main.five \
  --target devnet \
  --network https://your-rpc.example.com \
  --program-id <FIVE_VM_PROGRAM_ID>
```

## Troubleshooting

### `Program ID required for deployment` / `Program ID missing`

Set one of:

```bash
five config set --program-id <FIVE_VM_PROGRAM_ID> --target devnet
# or
export FIVE_PROGRAM_ID=<FIVE_VM_PROGRAM_ID>
# or
five deploy build/main.five --program-id <FIVE_VM_PROGRAM_ID> --target devnet
```

### `Invalid Solana public key`

Your value is not valid base58 or not a valid public key string. Re-copy from a trusted source and retry.

### `Works locally but fails in CI`

Check the CI secret value and confirm the deploy step actually exports `FIVE_PROGRAM_ID` before running `five deploy`.

## Related

- CLI overview: [README.md](./README.md)
- Frontend integration path: [5ive.tech](https://5ive.tech)
