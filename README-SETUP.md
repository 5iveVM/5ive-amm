# Five Frontend Setup Guide

## Quick Start

After cloning the repository, set up the frontend LSP integration:

```bash
# From repository root
./scripts/setup-lsp-wasm.sh
cd five-frontend
npm install
npm run dev
```

## What the Setup Script Does

The `setup-lsp-wasm.sh` script handles:

1. **Dependency Check**: Verifies `wasm-pack` and `cargo` are installed
2. **WASM Build**: Compiles Five LSP to WebAssembly using `wasm-pack`
3. **File Copy**: Copies WASM binaries to `five-frontend/public/wasm/`
4. **Asset Generation**: Creates missing `noise.png` texture if needed
5. **Verification**: Confirms all required files are present

## Prerequisites

- **Rust toolchain**: Install from https://rustup.rs/
- **wasm-pack**: Install with `cargo install wasm-pack`
- **Node.js**: For frontend development
- **Python 3**: For generating assets (optional, can be skipped)

## Troubleshooting

### wasm-pack not found

```bash
cargo install wasm-pack
```

### WASM build fails with bulk memory error

This is expected and handled automatically. The build succeeds, but `wasm-opt` post-processing fails. The binary is still usable.

### Need to rebuild WASM after compiler changes

```bash
./scripts/setup-lsp-wasm.sh clean
./scripts/setup-lsp-wasm.sh setup
```

## Development Workflow

### Make changes to Five LSP

```bash
cd five-lsp
# Edit source code in src/

# Rebuild and copy to frontend
./build-wasm.sh

# Or rebuild and reload frontend:
cd ../five-frontend
npm run dev
# Browser will hot-reload with new WASM
```

### Frontend-only development (no LSP changes)

```bash
cd five-frontend
npm run dev
# No need to rebuild WASM - it's already in public/
```

## File Organization

```
five-mono/
├── five-lsp/
│   ├── src/
│   │   ├── lib.rs         # WASM entry point
│   │   ├── wasm.rs        # WASM bindings
│   │   ├── bridge.rs      # Compiler bridge
│   │   ├── features/      # LSP features (hover, completion, etc.)
│   │   └── ...
│   ├── Cargo.toml
│   ├── build-wasm.sh      # Build script (builds and copies to frontend)
│   └── pkg/               # Generated WASM files (build output)
│
├── five-frontend/
│   ├── public/
│   │   ├── wasm/          # WASM binaries (served statically)
│   │   │   ├── five_lsp.js
│   │   │   ├── five_lsp_bg.wasm
│   │   │   ├── five_lsp.d.ts
│   │   │   ├── five_lsp_bg.wasm.d.ts
│   │   │   └── package.json
│   │   ├── noise.png      # Generated texture (if missing)
│   │   └── ...
│   ├── src/
│   │   ├── lib/
│   │   │   ├── lsp-client.ts          # WASM client wrapper
│   │   │   ├── monaco-lsp.ts          # Monaco editor integration
│   │   │   └── monaco-*.ts            # Feature providers
│   │   └── ...
│   ├── next.config.ts     # Webpack config for WASM
│   ├── package.json
│   └── LSP_SETUP.md       # Detailed LSP documentation
│
├── scripts/
│   ├── setup-lsp-wasm.sh  # Main setup script
│   └── ...
└── README-SETUP.md        # This file
```

## Common Tasks

### Check if WASM is properly installed

```bash
./scripts/setup-lsp-wasm.sh check
```

Output should show all files found:
```
✓ Found: five_lsp.js
✓ Found: five_lsp_bg.wasm
✓ Found: five_lsp.d.ts
✓ Found: five_lsp_bg.wasm.d.ts
```

### Clean up and rebuild everything

```bash
./scripts/setup-lsp-wasm.sh clean
./scripts/setup-lsp-wasm.sh setup
cd five-frontend
npm install
npm run dev
```

### Just rebuild WASM (after making LSP changes)

```bash
cd five-lsp
./build-wasm.sh
# Frontend automatically sees the new files
```

## Architecture

### How LSP Works in the Browser

1. **WASM Module** (`five_lsp_bg.wasm`): Contains compiled Five LSP
2. **Glue Code** (`five_lsp.js`): Bindings between JavaScript and WASM
3. **Client** (`lsp-client.ts`): TypeScript wrapper for LSP operations
4. **Monaco Integration** (`monaco-lsp.ts`): Connects LSP to Monaco editor

### Compilation Flow

```
Five DSL Source Code
    ↓
Five Compiler (Rust)
    ↓
Diagnostics, Hover, Completions, etc.
    ↓
JSON Response
    ↓
Monaco Editor (Real-time error highlighting, suggestions, etc.)
```

## Performance Notes

- **Initial Load**: WASM compilation takes 1-2 seconds in browser
- **After Load**: LSP analyses typically complete in <100ms
- **Memory**: WASM instance uses ~50-100 MB depending on code complexity
- **Binary Size**: WASM binary is 897 KB (already optimized)

## Next Steps

1. Run the setup script: `./scripts/setup-lsp-wasm.sh`
2. Install frontend dependencies: `cd five-frontend && npm install`
3. Start development: `npm run dev`
4. Open http://localhost:3000 in your browser
5. Navigate to `/ide` for the full IDE experience

## Support & Documentation

- **LSP Setup Details**: See `five-frontend/LSP_SETUP.md`
- **Five DSL Docs**: See `docs/` directory
- **CLAUDE.md**: Project-specific development guide
- **GitHub Issues**: Report bugs or request features
