# Five CLI - Five DSL Compilation and Deployment

This document covers the complete Five CLI workflow, including single-file and **multi-file compilation** with module support.

## Quickstart

### Single-File Compilation
```bash
five compile script.v                    # Compile to bytecode
five compile script.v -o script.five     # Output as .five format (includes ABI)
```

### Project-Based Workflow
```bash
five init my-project                     # Scaffold a project with five.toml
cd my-project
five compile                             # Auto-discovers modules and compiles
five deploy --project .                  # Deploy to Solana
five execute --project .                 # Execute on-chain
five test --project .                    # Run test suite
```

## Native SOL Fees (Deploy + Execute)

The on-chain Five VM program can charge native SOL fees for deploy and execute.

- **Deploy fee**: basis points of the rent-exempt minimum for the script account size
- **Execute fee**: basis points of the standard Solana transaction fee (5000 lamports)
- Fees are transferred to the VM authority/admin account if configured on-chain

If fees are enabled, make sure the payer signer has enough lamports and the admin
account is included in the transaction so the program can credit the fee.

## Program ID Management

The Five VM requires a program ID for on-chain deployments and execution. Five CLI provides multiple ways to specify and store program IDs, with flexible override options.

### Quick Setup

```bash
# Store program ID in config for current target
five config set --program-id HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg

# View all stored program IDs
five config get programIds

# Deploy using stored program ID
five deploy script.bin
```

### Resolution Order

Program IDs are resolved in the following priority order:

1. **CLI flag** (highest priority): `--program-id <pubkey>`
2. **Project config**: `five.toml` [deploy] section
3. **CLI config**: Stored via `five config set --program-id`
4. **Environment variable**: `FIVE_PROGRAM_ID`
5. **SDK default**: Set programmatically or via npm package
6. **Error**: Clear guidance with setup instructions (lowest priority)

### Configuration Methods

#### 1. Store Globally in CLI Config

Store program IDs per-target network:

```bash
# Set for current target (devnet by default)
five config set --program-id HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg

# Set for specific target
five config set --program-id HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg --target devnet
five config set --program-id <testnet-id> --target testnet
five config set --program-id <mainnet-id> --target mainnet

# View all stored IDs
five config get programIds

# View specific target
five config get programIds.devnet

# Clear program ID for target
five config clear --program-id --target devnet
```

Stored at: `~/.config/five/config.json`

#### 2. Per-Project Configuration

Add program ID to your `five.toml`:

```toml
[deploy]
program_id = "HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg"
cluster = "devnet"
```

#### 3. Command-Line Override

Pass program ID directly to any on-chain command:

```bash
five deploy script.bin --program-id <program-id>
five execute <script-account> -f 0 --program-id <program-id>
five namespace bind <name> --program-id <program-id>
```

#### 4. Environment Variable

Set globally for your session:

```bash
export FIVE_PROGRAM_ID=HJ5RXmE94poUCBoUSViKe1bmvs9pH7WBA9rRpCz3pKXg
five deploy script.bin              # Uses env var
five execute <script-account> -f 0  # Uses env var
```

### Complete Workflow Example

```bash
# 1. Initialize CLI (if first time)
five config init

# 2. Set program IDs for each network
five config set --program-id ABC... --target devnet
five config set --program-id DEF... --target testnet
five config set --program-id GHI... --target mainnet

# 3. View current config
five config get

# 4. Compile your script
five compile script.v

# 5. Deploy to devnet (uses stored devnet program ID)
five deploy script.bin --target devnet

# 6. Switch target and deploy to testnet
five deploy script.bin --target testnet
# (uses stored testnet program ID)

# 7. Execute on devnet (uses program ID from step 5)
five execute <script-account> -f 0
```

### Troubleshooting

#### Error: "Program ID required for deployment"

This means no program ID was found in the resolution chain. Fix with:

```bash
# Option 1: Set in config (recommended)
five config set --program-id <PROGRAM_ID>

# Option 2: Pass directly
five deploy script.bin --program-id <PROGRAM_ID>

# Option 3: Use environment variable
export FIVE_PROGRAM_ID=<PROGRAM_ID>

# Option 4: Add to five.toml
[deploy]
program_id = "<PROGRAM_ID>"
```

#### How to Find Your Program ID

If you've already deployed Five VM to Solana:

```bash
# Check program account address from deployment
solana address -k five-keypair.json

# Or view from your transaction
solana confirm <transaction-signature> -v
```

#### Multi-Network Workflows

Different networks require different program IDs:

```bash
# Devnet
five config set --program-id <devnet-id> --target devnet
five deploy script.bin --target devnet

# Testnet
five config set --program-id <testnet-id> --target testnet
five deploy script.bin --target testnet

# Mainnet (use with caution)
five config set --program-id <mainnet-id> --target mainnet
five deploy script.bin --target mainnet
```

#### View Current Configuration

```bash
# View all config
five config get

# View only program IDs
five config get programIds

# View specific target
five config get programIds.devnet
```

## Multi-File Compilation (Module System)

### Automatic Module Discovery
When a source file uses `use` or `import` statements, Five CLI automatically discovers and compiles all dependencies:

```bash
# Automatic discovery mode (recommended)
five compile src/main.v --auto-discover
five build                               # Uses auto mode from five.toml
```

Example Five DSL with imports:
```v
// src/main.v
use lib;                                 // Import module 'lib'
use utils::helpers;                      // Import nested module
use "ContractAddress"::{transfer};       // Import from external contract

pub execute() -> u64 {
    return lib::multiply(10, 20);
}
```

```v
// src/lib.v
pub multiply(a: u64, b: u64) -> u64 {
    return a * b;
}
```

### Explicit Module List
For advanced use cases, specify modules directly:

```bash
five compile src/main.v src/lib.v src/utils/helpers.v  # Explicit list
```

### Use Statement Syntax

Three forms of `use` statements are supported:

1. **Local imports**: `use lib;`
   - Looks for `lib.v` in the same directory or source_dir

2. **Nested imports**: `use utils::helpers;`
   - Looks for `utils/helpers.v` relative to source_dir
   - Supports multiple segments: `use a::b::c;`

3. **External contracts**: `use "ContractAddress"::{func1, func2};`
   - Imports functions from deployed contract bytecode
   - Optional specifier list (omit for wildcard): `use "ContractAddress";`

## five.toml Configuration

### Minimal Example
```toml
[project]
name = "my-app"
version = "0.1.0"

[build]
entry_point = "src/main.v"
source_dir = "src"
build_dir = "build"
multi_file_mode = "auto"  # Options: "auto", "explicit", "disabled"
target = "vm"
```

### Multi-File Configuration
```toml
[build]
# Auto-discover modules from use statements
multi_file_mode = "auto"          # (default) Automatically discover all imports
# OR
multi_file_mode = "explicit"      # Require explicit file list
# OR
multi_file_mode = "disabled"      # No multi-file support
```

### Complete Configuration Reference
- `project`: `name`, `version`, `source_dir`, `build_dir`, `target`, `entry_point`, `output_artifact_name`
- `build`: `multi_file_mode`, `output_artifact_name`
- `optimizations`: `enable_compression`, `enable_constraint_optimization`, `optimization_level`
- `deploy`: `cluster`, `rpc_url`, `commitment`, `program_id`, `keypair_path`

## Manifest
- Emitted at `.five/build.json` by `five compile` when a project is loaded.
- Fields: `artifact_path`, `abi_path`, `compiler_version`, `source_files`, `target`, `timestamp`, `hash`, `format` (`five` preferred), `entry_point`, `source_dir`.

## Discovery Order
- `--project <dir|file>` (explicit) > nearest `five.toml` upward from `cwd`.
- Artifact preference: `.five` (ABI + bytecode) preferred over `.bin`; manifest records format.

## Error Handling

### Common Module System Errors

**Circular Dependency**
```
Error: circular dependency detected in module graph: lib → utils → lib
```
Solution: Restructure modules to remove cycles. Consider extracting shared code to a separate module.

**Missing Module**
```
Error: Module 'lib' not found
Searched paths:
  - ./lib.v
  - src/lib.v
  - lib.v
```
Solution: Ensure the module file exists in one of the search paths, or adjust the import path.

**Invalid External Address**
```
Error: Invalid contract address '0xINVALID' - not a valid Solana public key
```
Solution: Use a valid Solana public key address (e.g., `AjJVHdYu7ASTWCDoNiZtNrEY2wnELYsZNf5s2pHJQPdt`).

**Duplicate Symbols**
```
Error: Symbol 'multiply' defined in multiple modules: lib.v:5, utils.v:12
```
Solution: Rename one of the functions or use namespace access (e.g., `lib::multiply` vs `utils::multiply`).

## Overrides
- CLI flags still override config (target/network/keypair/output paths).
- Clear errors when artifacts/config are missing; use `--project` to disambiguate roots.

## Command Reference

### Compile Command
```bash
five compile <files...>                  # Compile one or more files
five compile src/main.v --auto-discover  # Auto-discover all modules
five compile -o output.five              # Specify output format and name
five compile --target vm                 # Override target from config
```

### Build Command
```bash
five build                               # Build project with five.toml
five build --project path/to/proj        # Explicit project path
five build --target solana               # Override target
```

### Testing Module System

Create a test project:
```bash
mkdir my-app && cd my-app
five init .

# Create test modules
echo 'pub add(a: u64, b: u64) -> u64 { return a + b; }' > src/lib.v
echo 'use lib; pub test() { lib::add(1, 2); }' > src/main.v

# Compile with auto-discovery
five compile src/main.v --auto-discover

# Or with build command
five build
```
