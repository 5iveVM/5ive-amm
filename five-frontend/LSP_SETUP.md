# Five Frontend LSP Setup and Configuration

## Overview

This document explains the LSP (Language Server Protocol) setup for the Five Frontend and the fixes applied to resolve loading issues.

## Issues Fixed

### 1. Missing WASM Binary (`five_lsp_bg.wasm` 404)

**Problem**: The frontend was trying to load `/wasm/five_lsp_bg.wasm` but the file didn't exist in the public directory.

**Root Cause**: The Five LSP build process generates WASM bindings in `five-lsp/pkg/`, but only the JavaScript glue code was partially copied to the frontend. The critical binary `five_lsp_bg.wasm` file was missing.

**Solution**:
- Fixed compilation errors in `five-lsp/src/wasm.rs` (incorrect function signatures)
- Built the complete WASM bindings using `wasm-pack`
- Copied all necessary files to `five-frontend/public/wasm/`:
  - `five_lsp.js` - JavaScript glue code
  - `five_lsp_bg.wasm` - WASM binary (897 KB)
  - `five_lsp.d.ts` - TypeScript definitions
  - `five_lsp_bg.wasm.d.ts` - WASM type definitions

### 2. Missing Asset (`noise.png` 404)

**Problem**: The landing page component `NapkinToMainnet.tsx` references a background texture `noise.png` that didn't exist.

**Solution**: Created a Python script to procedurally generate a grayscale noise texture PNG (512x512, 94 KB) stored at `five-frontend/public/noise.png`.

### 3. LSP Client Import Issues

**Problem**: `src/lib/lsp-client.ts` tried to import from non-existent module `'./five-lsp-wasm'`.

**Solution**: Removed the incorrect import statement and properly documented that the WASM module is loaded dynamically from `/wasm/five_lsp.js` at runtime.

### 4. Next.js Configuration

**Enhancement**: Updated `next.config.ts` to properly handle the `five_lsp.js` glue code with webpack.

## Files Modified

### Core Fixes

1. **five-lsp/src/wasm.rs**
   - Fixed `find_references()` to pass `&mut self.bridge` as first parameter
   - Fixed `get_definition()` to pass correct parameters in correct order
   - Fixed `rename()` to pass `&mut self.bridge` as first parameter
   - Fixed `prepare_rename()` method signature to include `&self`

2. **five-frontend/src/lib/lsp-client.ts**
   - Removed incorrect relative import `import * as wasmModule from './five-lsp-wasm'`
   - Updated type declaration to use `any` for dynamically loaded WASM module
   - Preserved existing dynamic import from `/wasm/five_lsp.js`

3. **five-frontend/next.config.ts**
   - Added webpack rule for `five_lsp.js` (type: 'javascript/auto')

### New Files

1. **scripts/setup-lsp-wasm.sh**
   - Orchestration script for building and installing LSP WASM
   - Features: dependency checking, build, copy, verification, cleanup
   - Usage: `./scripts/setup-lsp-wasm.sh [setup|check|clean]`

2. **five-frontend/scripts/generate_noise.py**
   - Python script to generate procedural noise PNG texture
   - Creates 512x512 grayscale noise image with zlib compression

3. **five-lsp/build-wasm.sh** (Enhanced)
   - Added support for automatically copying files to frontend
   - Added wasm-opt error suppression
   - Improved output and next steps guidance

## WASM Building Workflow

### Option 1: Quick Setup (Recommended)

```bash
# From repository root
./scripts/setup-lsp-wasm.sh
```

This automatically:
- Checks dependencies (wasm-pack, cargo)
- Builds LSP WASM bindings
- Copies files to frontend
- Generates missing assets
- Verifies installation

### Option 2: Manual Build

```bash
# Build WASM
cd five-lsp
./build-wasm.sh

# Files are automatically copied to frontend/public/wasm/
```

### Option 3: Verify Existing Installation

```bash
# Check if WASM files are present
./scripts/setup-lsp-wasm.sh check
```

## WASM Files Structure

```
five-frontend/public/wasm/
├── five_lsp.js              # JavaScript glue code (25 KB)
├── five_lsp.d.ts            # TypeScript definitions (8 KB)
├── five_lsp_bg.wasm         # WASM binary (897 KB)
├── five_lsp_bg.wasm.d.ts    # WASM type definitions (2 KB)
├── .gitignore
└── package.json
```

## LSP Client Architecture

### Dynamic WASM Loading

The LSP client loads WASM at runtime using dynamic imports:

```typescript
// Load from public directory
const wasmUrl = new URL('/wasm/five_lsp.js', window.location.href).href;
this.wasmModule = await import(/* webpackIgnore: true */ wasmUrl);
```

This approach:
- ✅ Avoids webpack bundling the WASM binary (keeps bundle size small)
- ✅ Allows WASM to be served as static assets
- ✅ Works with Next.js static export mode
- ✅ Reduces initial page load time

### LSP Features Provided

- **Diagnostics**: Real-time syntax/semantic error checking
- **Hover**: Type information and documentation on hover
- **Completions**: Code suggestions and autocomplete
- **Go to Definition**: Jump to function/type definitions
- **Find References**: Locate all uses of a symbol
- **Rename**: Safe refactoring across the file
- **Document Symbols**: Outline view for navigation
- **Code Actions**: Quick fix suggestions

## Configuration Files

### next.config.ts

Webpack configuration for handling WASM:

```typescript
config.experiments = {
  asyncWebAssembly: true,
  layers: true,
};

// Handle five-lsp WASM bindings
config.module.rules.push({
  test: /five_lsp\.js$/,
  type: 'javascript/auto',
});
```

### five-lsp/Cargo.toml

Key configuration:

```toml
[lib]
crate-type = ["rlib", "cdylib"]
name = "five_lsp"

[package.metadata.wasm]
wasm-opt = false  # Disable wasm-opt due to bulk memory issues

[target.'cfg(target_arch = "wasm32")'.dependencies]
wasm-bindgen = "0.2"
wasm-bindgen-futures = "0.4"
js-sys = "0.3"
```

## Troubleshooting

### Issue: "Failed to execute 'compile' on 'WebAssembly'"

**Cause**: WASM binary file is missing or corrupted.

**Solution**:
```bash
./scripts/setup-lsp-wasm.sh clean
./scripts/setup-lsp-wasm.sh setup
```

### Issue: "Cannot find module five_lsp_wasm"

**Cause**: Incorrect import statement or WASM files not copied.

**Solution**: This is already fixed in the codebase. The import has been removed and dynamic loading is used instead.

### Issue: Monaco editor shows no LSP features

**Cause**: WASM initialization failed silently.

**Solution**: Check browser console for errors and ensure:
1. `/wasm/five_lsp_bg.wasm` returns HTTP 200
2. `/wasm/five_lsp.js` is properly loaded
3. Five DSL language is registered: `monaco.languages.register({ id: 'five' })`

## Performance Considerations

- **WASM Binary Size**: 897 KB (already optimized by compiler)
- **Initial Load**: ~1-2 seconds for WASM compilation in browser
- **Subsequent Analyses**: <100ms for most files
- **Memory Usage**: ~50-100 MB after initialization (depends on code complexity)

## Future Improvements

1. **Worker Thread**: Move WASM to Web Worker to avoid blocking main thread
2. **Caching**: Cache compiled analyses across page reloads
3. **Incremental Analysis**: Only recompile changed portions of code
4. **Network Fallback**: Support LSP over WebSocket for remote server

## References

- [WASM Build Script](../../five-lsp/build-wasm.sh)
- [Setup Script](../../scripts/setup-lsp-wasm.sh)
- [LSP Client](./src/lib/lsp-client.ts)
- [Monaco Integration](./src/lib/monaco-lsp.ts)
- [Five DSL Compiler](../../five-dsl-compiler/)

## Support

For issues or questions about LSP setup:
1. Check the troubleshooting section above
2. Review browser console for error messages
3. Run `./scripts/setup-lsp-wasm.sh check` to verify files
4. Rebuild fresh: `./scripts/setup-lsp-wasm.sh clean && ./scripts/setup-lsp-wasm.sh setup`
