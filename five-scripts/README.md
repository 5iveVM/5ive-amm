# Five Ecosystem Build & Deployment Scripts

Comprehensive scripts for building, testing, and deploying the Five DSL ecosystem components.

## Scripts Overview

### 🔨 Build Scripts

#### `build-workspace.sh`
Builds all components in the Five Protocol ecosystem in a single command.

**Usage:**
```bash
./five-scripts/build-workspace.sh
```

**What it builds:**
- five-protocol (Rust)
- five-vm-mito (Rust VM)
- five-dsl-compiler (Rust DSL Compiler)
- five-solana (On-chain Solana Program)
- five-cli (TypeScript CLI with WASM)
- five-wasm (WebAssembly bindings)
- five-mcp (MCP Server)

**Features:**
- Parallel compilation with cargo workspace
- Automatic dependency resolution
- Color-coded output for easy debugging
- Summary of all built components

---

#### `build-five-solana.sh`
Builds the Solana program implementation of the Five VM.

**Usage:**
```bash
# Production build (optimized, default)
./five-scripts/build-five-solana.sh

# Debug build (with debug logs)
./five-scripts/build-five-solana.sh debug
```

**What it builds:**
- five-solana Rust program (Solana Program Framework)
- Generates `five-solana/target/deploy/five.so` executable

**Build modes:**
- **prod** (default): Production-optimized binary with minimal overhead
- **debug**: Includes debug logging for troubleshooting

**Output:**
- Program binary: `five-solana/target/deploy/five.so`
- Program keypair: `five-solana/target/deploy/five-keypair.json`
- Program size display (typically 100-200 KB)

---

#### `build-production-vm.sh`
Builds production-optimized binaries with all debug features removed.

**Usage:**
```bash
./five-scripts/build-production-vm.sh
```

**What it does:**
1. Cleans previous production builds
2. Builds five-vm-mito without debug features
3. Builds five-dsl-compiler with minimal features
4. Builds on-chain Solana program for production
5. Verifies build artifacts
6. Displays performance analysis
7. Optionally validates production build
8. Optionally deploys and initializes VM state

**Build optimizations applied:**
- ✗ Disabled: debug-logs (~286 debug statements removed)
- ✗ Disabled: type-checking (runtime validation removed)
- ✗ Disabled: execution-tracing (instruction history removed)
- ✗ Disabled: benchmark-mode (test state modification removed)
- ✓ Enabled: Release mode (-O3 optimization)
- ✓ Enabled: Dead code elimination
- ✓ Enabled: Link-time optimization (LTO)
- ✓ Enabled: Stateless VM design

**Expected improvements:**
- ~43% memory reduction per VM instance
- 150-800+ compute unit savings per transaction
- Elimination of debug/tracing overhead
- Faster program loading due to smaller binary size

**Output directories:**
- VM library: `target/production/release/libfive_vm_mito.rlib`
- Program binary: `target/production/deploy/five.so`

---

### 🚀 Deployment Scripts

#### `deploy-and-init.sh`
Complete deployment pipeline for the Five Solana Program with VM state initialization.

**Usage:**
```bash
# Deploy to localnet (default)
./five-scripts/deploy-and-init.sh

# Deploy to devnet with custom payer
./five-scripts/deploy-and-init.sh devnet ~/.config/solana/devnet-keypair.json

# Deploy with production build
./five-scripts/deploy-and-init.sh localnet ~/.config/solana/id.json prod

# Deploy with debug build
./five-scripts/deploy-and-init.sh localnet ~/.config/solana/id.json debug
```

**Parameters:**
1. **Network** (default: `localnet`)
   - `localnet` - Local validator (surfpool or solana-test-validator)
   - `devnet` - Solana devnet
   - `testnet` - Solana testnet

2. **Payer keypair** (default: `~/.config/solana/id.json`)
   - Path to Solana keypair file for paying deployment fees
   - Supports tilde expansion

3. **Build mode** (optional)
   - `prod` - Production build
   - `debug` - Debug build with logging
   - Omit to skip building

**Deployment steps:**
1. Set Solana cluster (localnet/devnet/testnet)
2. Ensure local validator is running (localnet only)
3. Fund payer wallet (localnet only, 50 SOL airdrop)
4. Build program (optional, if build mode specified)
5. Deploy Solana program
6. Verify deployment
7. Initialize VM state account
8. Display summary with Program ID and VM State PDA

**Supported validators:**
- **Localnet:** Surfpool (preferred) or solana-test-validator
- **Devnet:** Solana devnet API
- **Testnet:** Solana testnet API

**Environment variables:**
- `FIVE_LOCAL_RPC_URL` - Override localnet RPC URL
- `SOLANA_URL` - Fallback RPC URL
- `SKIP_SOLANA_CONFIG_SET` - Set to 1 to skip solana config set
- `SKIP_BUILD` - Set to 1 to skip building
- `MAX_SIGN_ATTEMPTS` - Maximum signature attempts (default: 50)
- `VM_STATE_KEYPAIR` - VM state keypair path

**Output example:**
```
✨ FIVE Program Deployment Complete!
=========================================
Program ID: J99pDwVh1PqcxyBGKRvPKk8MUvW8V8KF6TmVEavKnzaF
Network: localnet
Binary: five-solana/target/deploy/five.so
VM State PDA: 3dTiHw3aHYUmqFmLsvvswHa789HQDmeJG8ZCS5frFtkX
```

**Next steps after deployment:**
```bash
# Check validator status
FIVE_VALIDATOR=surfpool ./five-surfpool/surfpool instance status

# Deploy and execute a Five script
five deploy-and-execute examples/add.v --target localnet

# Execute on-chain
five execute <script_account> -f 0 --target localnet
```

---

## Quick Start

### 1. Build everything
```bash
./five-scripts/build-workspace.sh
```

### 2. Build and deploy to localnet
```bash
./five-scripts/deploy-and-init.sh localnet ~/.config/solana/id.json prod
```

### 3. Start using Five
```bash
# Compile a Five DSL script
five compile examples/add.v

# Deploy to the VM
five deploy-and-execute examples/add.v --target localnet

# Execute a function
five execute <script_account> -f 0 --target localnet
```

---

## Prerequisites

### Required tools
- Rust 1.79.0+ (or use `rustup override set 1.79.0`)
- Cargo with SBF support (`cargo install cargo-build-sbf`)
- Node.js 16+
- Solana CLI tools
  ```bash
  sh -c "$(curl -sSfL https://release.solana.com/v1.18.26/install)"
  ```

### Optional tools
- Surfpool (for managed localnet)
  ```bash
  cargo install --git https://github.com/5iveVM/surfpool surfpool
  ```
- jq (for JSON processing in production builds)

### Development setup
```bash
# Clone the monorepo
git clone https://github.com/5iveVM/five-mono.git
cd five-mono

# Install Rust dependencies
rustup override set 1.79.0

# Make scripts executable
chmod +x five-scripts/*.sh

# Run build
./five-scripts/build-workspace.sh

# Deploy
./five-scripts/deploy-and-init.sh
```

---

## Common Workflows

### Local development with localnet
```bash
# 1. Start surfpool validator (in another terminal)
FIVE_VALIDATOR=local ./five-surfpool/surfpool instance start localnet

# 2. Deploy and initialize
./five-scripts/deploy-and-init.sh localnet ~/.config/solana/id.json prod

# 3. Compile and deploy script
five compile examples/add.v
five deploy examples/add.v --target localnet

# 4. Execute
five execute <script_account> -f 0 --target localnet
```

### Production deployment to devnet
```bash
# Build production binary
./five-scripts/build-production-vm.sh

# Deploy to devnet
./five-scripts/deploy-and-init.sh devnet ~/devnet-keypair.json prod

# Verify deployment
solana program show <PROGRAM_ID> --url https://api.devnet.solana.com
```

### Testing the VM
```bash
# Build with debug features
./five-scripts/build-five-solana.sh debug

# Deploy with debug
./five-scripts/deploy-and-init.sh localnet ~/.config/solana/id.json debug

# Check program logs
solana logs <PROGRAM_ID> --url http://localhost:8899
```

---

## Troubleshooting

### Build fails with "stack overflow"
Reduce stack usage in ExecutionContext. Edit `five-solana/src/execution.rs` and optimize memory allocations.

### "Program not deployed" error
Ensure the Solana validator is running:
```bash
# Check validator status
solana cluster-version

# Start localnet if not running
solana-test-validator
```

### Deployment timeout
Increase max sign attempts:
```bash
./five-scripts/deploy-and-init.sh localnet ~/.config/solana/id.json prod
# or
MAX_SIGN_ATTEMPTS=100 ./five-scripts/deploy-and-init.sh localnet ~/.config/solana/id.json prod
```

### VM state initialization fails
Ensure Node.js is installed and @solana/web3.js is available:
```bash
node --version
npm list @solana/web3.js
```

---

## Environment Configuration

### Localnet setup
```bash
# Configure for local validator
export FIVE_LOCAL_RPC_URL="http://127.0.0.1:8899"
export FIVE_VALIDATOR=local

# Or use surfpool
export FIVE_VALIDATOR=surfpool
./five-surfpool/surfpool instance start localnet
```

### Network configuration
```bash
# Devnet
solana config set --url devnet

# Testnet
solana config set --url testnet

# Custom RPC
solana config set --url https://your-rpc-url.com
```

---

## Performance Metrics

### Build times (approximate)
- Full workspace build: 3-5 minutes
- Production VM build: 2-3 minutes
- Solana program build: 1-2 minutes
- WASM build: 1 minute

### Binary sizes
- Production Solana program: 100-150 KB
- Debug Solana program: 200-250 KB
- VM Mito library: 2-3 MB (unlinked)

### Deployment
- Initial deployment: 10-30 seconds (includes signature gathering)
- VM state initialization: 3-5 seconds
- Subsequent deployments: 5-10 seconds

---

## Support

For issues with these scripts:
1. Check the script output carefully
2. Verify all prerequisites are installed
3. Try running with `set -x` for debugging: `bash -x five-scripts/deploy-and-init.sh`
4. Check Solana program logs: `solana logs <PROGRAM_ID>`
5. Report issues at https://github.com/5iveVM/five-mono/issues

---

## License

These scripts are part of the Five ecosystem and are licensed under the MIT License.
