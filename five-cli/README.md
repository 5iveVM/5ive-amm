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
