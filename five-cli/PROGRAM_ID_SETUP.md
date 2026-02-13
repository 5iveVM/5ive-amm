# Program ID Setup Guide

This guide explains how to configure and manage program IDs for Five CLI deployments and execution on Solana.

## What is a Program ID?

A **program ID** is the on-chain address of the Five VM program. It's required for:
- Deploying Five scripts to Solana
- Executing deployed scripts on-chain
- Managing namespaces on-chain

Your program ID is a Solana public key (base58 encoded address, 32-44 characters).

## Quick Start (30 seconds)

```bash
# 1. Set your program ID once
five config set --program-id HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg

# 2. Deploy - no need to specify program ID again
five deploy script.bin

# 3. Execute - uses the same stored program ID
five execute <script-account> -f 0
```

Done! Your program ID is now configured.

## Finding Your Program ID

### If You Deployed Five VM Yourself

```bash
# Get the address from your keypair
solana address -k ~/my-five-program-keypair.json

# Or check deployment output
# Look for the "Program ID:" line in your deployment logs
```

### If Using a Public Network

Ask your infrastructure provider for the program ID, or check their documentation:

- **Devnet**: `HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg` (example)
- **Testnet**: Contact network operator
- **Mainnet**: Contact Solana Foundation or use official releases

### Verify Your Program ID

Check that a program exists at your ID:

```bash
solana account <program-id> --url https://api.devnet.solana.com
```

You should see:
```
Public Key: HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg
Balance: X SOL
Owner: BPFLoaderUpgradeab1e11111111111111111111111
...
```

## Configuration Methods

### Method 1: Global CLI Config (Recommended)

Store program ID in your local CLI config for future use:

```bash
five config set --program-id HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg
```

**Stored at:** `~/.config/five/config.json`

**Pros:**
- Works for all future commands
- Per-target support (devnet, testnet, mainnet)
- Survives shell restarts

**View/manage:**
```bash
five config get programIds                          # See all stored IDs
five config get programIds.devnet                   # See specific target
five config clear --program-id --target devnet     # Remove for target
```

### Method 2: Per-Project Configuration

Store in your `five.toml` for team consistency:

```toml
[project]
name = "my-dapp"

[deploy]
program_id = "HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg"
cluster = "devnet"
```

**Pros:**
- Shared with your team via git
- Different ID per environment (dev, staging, prod)
- Visible in code review

### Method 3: Command-Line Flag

Override on each command:

```bash
five deploy script.bin --program-id HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg
five execute <account> -f 0 --program-id HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg
```

**Pros:**
- One-off overrides
- No permanent storage

### Method 4: Environment Variable

Set for your shell session:

```bash
export FIVE_PROGRAM_ID=HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg
five deploy script.bin     # Uses env var
five execute <account> -f 0  # Uses env var
```

**Pros:**
- Session-wide
- Good for CI/CD

## Multi-Network Setup

Different networks (devnet, testnet, mainnet) use different program IDs.

### Setup Per-Target

```bash
# Devnet
five config set --program-id HJ5RXmE... --target devnet

# Testnet
five config set --program-id 5ive1XYZ... --target testnet

# Mainnet
five config set --program-id 5ive1ABC... --target mainnet
```

### Deploy to Different Networks

```bash
# Deploy to devnet (uses devnet program ID automatically)
five deploy script.bin --target devnet

# Deploy to testnet (uses testnet program ID)
five deploy script.bin --target testnet

# Deploy to mainnet
five deploy script.bin --target mainnet
```

### View All Configured IDs

```bash
$ five config get programIds

Program IDs:
  ● devnet:  HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg
  ○ testnet: 5ive1XYZ...
  ○ mainnet: (not configured)
```

The `●` shows your current target, `○` shows others.

## Resolution Order (Priority)

When Five CLI runs, it looks for your program ID in this order:

```
1. CLI flag (--program-id)         ← Highest priority
   ↓
2. five.toml [deploy].program_id
   ↓
3. Config file (five config set)
   ↓
4. FIVE_PROGRAM_ID environment var
   ↓
5. SDK default (rarely set)
   ↓
Error: No program ID found         ← Lowest priority, shows setup guide
```

**This means:**
- `--program-id` flag always wins
- If not set, checks five.toml
- If not set, checks CLI config
- And so on...

### Example: Precedence in Action

```bash
# Scenario: Config has devnet ID, but we override

# Option 1: Config + CLI flag (flag wins)
five config set --program-id CONFIG_ID --target devnet
five deploy script.bin --program-id CLI_FLAG_ID
# → Uses CLI_FLAG_ID

# Option 2: CLI flag + environment (flag wins)
export FIVE_PROGRAM_ID=ENV_ID
five deploy script.bin --program-id CLI_FLAG_ID
# → Uses CLI_FLAG_ID

# Option 3: Config + environment (config wins - checked first)
five config set --program-id CONFIG_ID
export FIVE_PROGRAM_ID=ENV_ID
five deploy script.bin
# → Uses CONFIG_ID

# Option 4: Only environment set
export FIVE_PROGRAM_ID=ENV_ID
five deploy script.bin
# → Uses ENV_ID
```

## Common Workflows

### Personal Development on Devnet

```bash
# Setup once
five config set --program-id <your-devnet-id>

# Use everywhere
five deploy script.bin
five execute <account> -f 0
five namespace bind myname
```

### Team Development (five.toml)

```toml
# five.toml (in git)
[deploy]
program_id = "HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg"
cluster = "devnet"
```

```bash
# Team member clones and just deploys
five deploy script.bin  # Uses shared config
```

### CI/CD Deployment

```bash
# .github/workflows/deploy.yml
- name: Deploy Five script
  env:
    FIVE_PROGRAM_ID: ${{ secrets.FIVE_PROGRAM_ID }}
  run: |
    five deploy script.bin
```

### Testing Multiple Environments

```bash
# Shell script for multi-environment testing
for target in devnet testnet mainnet; do
  echo "Deploying to $target..."
  five deploy script.bin --target $target
done
```

## Troubleshooting

### Error: "Program ID required for deployment"

**Cause:** No program ID found in the resolution chain.

**Fix:**
```bash
# Fastest fix: Use environment variable
export FIVE_PROGRAM_ID=<your-id>
five deploy script.bin

# Or store in config
five config set --program-id <your-id>
five deploy script.bin
```

### Error: "Invalid Solana public key"

**Cause:** Program ID is malformed (wrong format, wrong length, invalid characters).

**Fix:**
1. Verify the program ID is base58 encoded (no 0, O, I, l)
2. Check length (should be 32-44 characters)
3. Copy-paste from `solana address` output

```bash
# Verify format
echo "HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg" | wc -c  # Should be 44+1 (newline)

# Test with solana CLI
solana account <program-id> --url https://api.devnet.solana.com
```

### Program ID Works Locally but Not in CI/CD

**Cause:** Environment variable not set or wrong value.

**Fix:**
1. Verify secret is set in CI/CD platform
2. Check secret name matches your script
3. Log the first few chars to debug:
```bash
echo "Program ID (first 8 chars): ${FIVE_PROGRAM_ID:0:8}"
```

### Multiple Program IDs for Different Users

**Scenario:** Team members have different program IDs.

**Solution:** Use environment variables per user
```bash
# ~/.bashrc or ~/.zshrc
export FIVE_PROGRAM_ID=$(cat ~/.five/program-id.txt)

# Or use a config file per user
five config set --program-id <user-specific-id>
```

### Switching Program IDs Frequently

**Scenario:** Testing with different programs.

**Solution:** Create shell aliases
```bash
# ~/.bashrc or ~/.zshrc
alias five-devnet='five --target devnet'
alias five-testnet='five --target testnet'

# Then use
five-devnet deploy script.bin
five-testnet deploy script.bin
```

Or use a shell function:
```bash
five-with-id() {
  local id=$1
  shift
  FIVE_PROGRAM_ID=$id five "$@"
}

five-with-id HJ5RXmE... deploy script.bin
```

## Advanced Topics

### Setting Program ID for Published npm Package

If you're publishing Five SDK, inject a program ID at build time:

```bash
./scripts/set-default-program-id.sh HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg
npm run build
npm publish
```

### Programmatic Usage (SDK)

If using Five SDK in code:

```typescript
import { FiveSDK } from 'five-sdk';

// Option 1: Pass per-call
const result = await FiveSDK.deployToSolana(
  bytecode,
  connection,
  keypair,
  { fiveVMProgramId: 'HJ5RXmE...' }
);

// Option 2: Set SDK-wide default
FiveSDK.setDefaultProgramId('HJ5RXmE...');
const result = await FiveSDK.deployToSolana(
  bytecode,
  connection,
  keypair
);

// Option 3: Environment variable + SDK auto-resolution
// (SDK checks FIVE_PROGRAM_ID env var automatically)
```

### Viewing Your Configuration

```bash
# See all stored program IDs
five config get programIds

# See full config (including program IDs)
five config show

# Check what program ID will be used
five config get programIds.<target>

# For debugging, set verbose mode
five deploy script.bin -v 2>&1 | grep "program"
```

## Best Practices

1. **Use CLI config for personal development**
   ```bash
   five config set --program-id <id>
   ```

2. **Use five.toml for team projects**
   ```toml
   [deploy]
   program_id = "..."
   ```

3. **Use environment variables in CI/CD**
   ```bash
   export FIVE_PROGRAM_ID=${{ secrets.PROGRAM_ID }}
   ```

4. **Use CLI flags for one-off overrides**
   ```bash
   five deploy script.bin --program-id <different-id>
   ```

5. **Store secrets securely**
   - Never commit program IDs to git (unless safe/public)
   - Use `.gitignore` for local config: `~/.config/five/config.json`
   - Use CI/CD secrets for production IDs

6. **Document your setup**
   - Add a comment in five.toml explaining the program ID
   - Document in README for new team members
   - Include setup instructions in CONTRIBUTING.md

## See Also

- [Five CLI README](./README.md) - Complete CLI reference
- [Configuration Guide](./CONFIG.md) - All config options
- [Solana CLI Docs](https://docs.solana.com/cli) - For `solana address`, `solana account` commands
