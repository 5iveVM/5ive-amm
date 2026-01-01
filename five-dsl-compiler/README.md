# Five DSL Compiler

A Rust-based compiler for the Five Domain Specific Language (Five DSL), which compiles to WebAssembly bytecode for execution on the Five VM (Mito) on Solana.

## Overview

The Five DSL Compiler provides:
- **Multi-file compilation** with automatic module discovery
- **Module namespaces** using qualified function names (`module::function`)
- **Type checking** with cross-module symbol resolution
- **Bytecode generation** optimized for Solana execution
- **VLE encoding** for compact instruction representation

## Quick Start

### Single-File Compilation

```bash
# Compile a single Five DSL file
five compile src/main.v --output build/main.five
```

### Multi-File Compilation

```bash
# Compile all modules in a project
five compile-multi src/main.v --output build/project.five
```

## Project Structure

For a multi-module project, organize your files by module:

```
my_project/
├── src/
│   ├── main.v              # Entry point
│   ├── types/
│   │   ├── token.v         # Module: types
│   │   └── pool.v          # Module: types
│   └── logic/
│       ├── swap.v          # Module: logic
│       └── quote.v         # Module: logic
└── build/
    └── my_project.five     # Compiled output
```

## Module Namespaces

Functions are automatically namespaced by their module path. This prevents naming collisions and makes dependencies explicit.

### How Namespaces Work

**File Structure:**
```
src/
  math/calculator.v      # module: math
  types/token.v          # module: types
  main.v                 # entry point (no module prefix)
```

**Compiled Function Names:**
- `src/math/calculator.v::add` → `math::add`
- `src/types/token.v::transfer` → `types::transfer`
- `src/main.v::initialize` → `initialize` (entry point, no prefix)

### ABI Output

The compiled bytecode includes an ABI with qualified function names:

```json
{
  "functions": [
    {
      "name": "initialize",
      "index": 0,
      "parameters": [],
      "returnType": "void"
    },
    {
      "name": "math::add",
      "index": 1,
      "parameters": [
        { "name": "a", "type": "u32" },
        { "name": "b", "type": "u32" }
      ],
      "returnType": "u32"
    },
    {
      "name": "types::transfer",
      "index": 2,
      "parameters": [
        { "name": "from", "type": "u64" },
        { "name": "to", "type": "u64" },
        { "name": "amount", "type": "u64" }
      ],
      "returnType": "bool"
    }
  ]
}
```

## Calling Functions

### From TypeScript SDK

```typescript
import { FiveSDK } from '@five-vm/sdk';

// Execute a namespaced function
const result = await FiveSDK.executeLocally(
  bytecode,
  'math::add',        // Qualified function name
  [5, 3],             // Parameters
  { debug: true }
);

// Execute entry point function (no namespace)
const result = await FiveSDK.executeLocally(
  bytecode,
  'initialize',       // Entry point
  [],
  { debug: true }
);
```

### From CLI

```bash
# Execute a namespaced function
five execute build/project.five --function "math::add" --args "[5, 3]"

# Execute entry point
five execute build/project.five --function "initialize"
```

## Visibility Rules

Functions can be marked with visibility modifiers:

- **`pub fn`**: Public - callable on-chain and importable by other modules
- **`fn`**: Internal - importable by other modules but NOT on-chain callable
- **`private fn`**: Private - not importable, only available within the module

```five
pub fn public_function() {
  // Can be called on-chain
  // Can be imported by other modules
}

fn internal_function() {
  // Cannot be called on-chain
  // Can be imported by other modules
}

private fn private_function() {
  // Cannot be called on-chain
  // Cannot be imported by other modules
}
```

## Backward Compatibility

For legacy projects using flat namespace (no module prefixes), use the `--flat-namespace` flag:

```bash
# Compile with flat namespace (legacy mode)
five compile-multi src/main.v --output build/project.five --flat-namespace
```

This generates function names without module prefixes:
- `src/math/calculator.v::add` → `add` (no prefix)
- `src/types/token.v::transfer` → `transfer` (no prefix)

**In code:**

```rust
let config = CompilationConfig::new(CompilationMode::Deployment)
    .with_module_namespaces(false);  // Disable namespace qualification

let bytecode = DslCompiler::compile_modules(
    vec!["src/main.v"],
    "src/main.v",
    &config
)?;
```

## Examples

### Simple DEX Example

**src/types/token.v**
```five
pub fn get_balance(token: u64) -> u64 {
  // Token balance logic
  0
}

pub fn transfer(from: u64, to: u64, amount: u64) -> bool {
  // Transfer logic
  true
}
```

**src/logic/swap.v**
```five
pub fn swap_exact_in(amount_in: u64, min_out: u64) -> u64 {
  // Swap logic using types::get_balance and types::transfer
  amount_in * 2  // Simplified
}

pub fn quote(amount_in: u64) -> u64 {
  // Price oracle logic
  amount_in
}
```

**src/main.v**
```five
pub fn initialize(admin: u64) {
  // Initialization logic
}

pub fn execute_swap(amount_in: u64, min_out: u64) -> u64 {
  // Main entry point that calls logic::swap_exact_in
  0
}
```

**Compiled Functions:**
```
initialize                # Entry point
types::get_balance        # Token module
types::transfer           # Token module
logic::swap_exact_in      # Logic module
logic::quote              # Logic module
execute_swap              # Main module
```

## Development Workflow

### 1. Create Multi-Module Project

```bash
mkdir my_project && cd my_project
mkdir -p src/types src/logic
touch src/main.v src/types/token.v src/logic/swap.v
```

### 2. Implement Modules

Add functions to each module file.

### 3. Compile

```bash
five compile-multi src/main.v --output build/project.five
```

### 4. Test Locally

```typescript
import { FiveSDK } from '@five-vm/sdk';

const bytecode = await FiveSDK.compile(sourceCode);
const result = await FiveSDK.executeLocally(
  bytecode,
  'logic::swap_exact_in',
  [1000, 900],
  { debug: true }
);
```

### 5. Deploy

```bash
five deploy build/project.five --network devnet
```

## Namespace Collision Prevention

The namespace system prevents accidental function name collisions:

```
src/
  auth/permissions.v     -> auth::can_approve
  governance/permissions.v -> governance::can_approve
```

These are **distinct** functions and won't conflict.

Without namespaces (legacy mode), these would collide:
```bash
# Would fail - duplicate 'can_approve' function
five compile-multi src/main.v --flat-namespace
```

## Commands

### Compilation

```bash
# Compile single file
five compile <INPUT> [--output <OUTPUT>] [--debug]

# Compile multi-file project
five compile-multi <ENTRY_POINT> [--output <OUTPUT>] [--flat-namespace] [--debug]
```

### Execution

```bash
# Execute function locally
five execute <BYTECODE> --function <NAME> [--args <JSON>]

# With namespace
five execute build/project.five --function "math::add" --args "[5, 3]"
```

### Inspection

```bash
# Show compiled functions and metadata
five inspect <BYTECODE>
```

## Configuration

### Environment Variables

- `FIVE_COMPILER_DEBUG` - Enable debug output during compilation
- `FIVE_COMPILER_OPTIMIZE` - Optimization level (0-3)

### CompilationConfig

```rust
use five_dsl_compiler::CompilationConfig;

let config = CompilationConfig::new(CompilationMode::Deployment)
    .with_module_namespaces(true)           // Enable namespaces
    .with_optimization_level(OptimizationLevel::V2);

let bytecode = DslCompiler::compile_modules(
    vec!["src/main.v"],
    "src/main.v",
    &config
)?;
```

## Error Handling

The compiler provides detailed error messages with line numbers and suggestions:

```
Error [UNDEFINED_FUNCTION]: Function 'swap' not found in scope
  at src/main.v:12:3
  help: Did you mean 'logic::swap_exact_in'?
```

For multi-module projects, errors include module context:

```
Error [VISIBILITY_ERROR]: Function 'types::private_helper' is private
  at src/logic/swap.v:5:10
  module: logic
  help: Private functions cannot be imported across modules
```

## Performance

The compiler is optimized for fast multi-file projects:

- **Parallel compilation** of independent modules
- **Incremental type checking** with cached symbol tables
- **VLE encoding** reduces bytecode size by 30-50%
- **Symbol caching** prevents redundant lookups

## Testing

Run the compiler test suite:

```bash
cargo test                          # All tests
cargo test test_multi_module        # Multi-module tests
cargo test test_namespace           # Namespace tests
```

## Contributing

See [CLAUDE.md](./CLAUDE.md) for detailed development guidance and architecture information.

## Architecture

The compiler consists of these phases:

1. **Tokenization** - Break source into tokens
2. **Parsing** - Build AST from tokens
3. **Module Discovery** - Find all modules in project
4. **Module Merging** - Combine modules with namespace qualification
5. **Type Checking** - Validate types across modules using ModuleScope
6. **Bytecode Generation** - Emit optimized bytecode

For architecture details, see [CLAUDE.md](./CLAUDE.md).
