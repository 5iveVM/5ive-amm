# Five LSP Integration Guide

## Overview

The Five frontend automatically includes the latest LSP WASM build for real-time language features including:
- ✅ **Constraint annotation autocomplete** - `@signer`, `@mut`, `@init`, `@writable`
- ✅ **Real-time diagnostics** - Syntax and type errors as you type
- ✅ **Hover tooltips** - Type information and documentation
- ✅ **Go-to-definition** - Jump to symbol definitions
- ✅ **Find references** - See all usages of a symbol
- ✅ **Rename refactoring** - Safe cross-file renames
- ✅ **Code completion** - Keywords, symbols, and types
- ✅ **Document symbols** - Outline view navigation

## Quick Start

### Development Workflow

**1. Standard development (LSP already built):**
```bash
cd five-frontend
npm run dev
```
The dev server starts at http://localhost:3000 with LSP features active.

**2. After modifying LSP source code:**
```bash
# From five-frontend directory:
npm run rebuild:lsp      # Rebuild LSP and copy to public/wasm/
npm run dev              # Start dev server with updated LSP
```

**3. Fast iteration during LSP development:**
```bash
# Terminal 1: Watch-rebuild LSP (dev mode, faster builds)
cd five-lsp
./build-wasm.sh --dev && cp pkg/* ../five-frontend/public/wasm/

# Terminal 2: Run frontend dev server
cd five-frontend
npm run dev

# After each LSP change, rebuild in Terminal 1, then refresh browser
```

### Production Build

```bash
cd five-frontend
npm run build    # Automatically rebuilds LSP first (via prebuild hook)
npm start        # Serve production build
```

## File Locations

### LSP WASM Files
The LSP is compiled to WebAssembly and served from:
```
five-frontend/public/wasm/
├── five_lsp.js           # JavaScript bindings (28KB)
├── five_lsp.d.ts         # TypeScript definitions
├── five_lsp_bg.wasm      # Compiled WASM binary (928KB)
└── five_lsp_bg.wasm.d.ts # WASM type definitions
```

### Integration Files
```
five-frontend/src/lib/
├── lsp-client.ts              # LSP client wrapper
├── monaco-lsp.ts              # Provider registration
├── monaco-completion.ts       # Completion provider (constraint annotations)
├── monaco-hover.ts            # Hover provider
├── monaco-goto-definition.ts  # Definition provider
├── monaco-find-references.ts  # References provider
├── monaco-rename.ts           # Rename provider
├── monaco-code-actions.ts     # Code actions provider
├── monaco-document-symbols.ts # Document symbols provider
└── types/lsp.ts               # TypeScript LSP types
```

## How It Works

### 1. WASM Loading (Runtime)

The LSP is loaded dynamically at runtime:

```typescript
// In lsp-client.ts
async initialize() {
  // Load WASM from /wasm/five_lsp.js (public directory)
  const wasmUrl = new URL('/wasm/five_lsp.js', window.location.href).href;
  this.wasmModule = await import(/* webpackIgnore: true */ wasmUrl);

  // Initialize WASM
  await this.wasmModule.default();

  // Create LSP instance
  this.lsp = new this.wasmModule.FiveLspWasm();
}
```

**Why dynamic loading?**
- Avoids bundling 928KB WASM in main bundle
- Loads on-demand when Monaco editor is initialized
- Works around webpack WASM import limitations

### 2. Provider Registration (Setup)

Providers are registered when Monaco loads:

```typescript
// In monaco-lsp.ts
export async function setupFiveLsp(monacoInstance: typeof monaco) {
  const lspClient = new FiveLspClient();
  await lspClient.initialize();

  // Register all providers
  registerCompletionProvider(monacoInstance, lspClient);
  registerHoverProvider(monacoInstance, lspClient);
  registerDefinitionProvider(monacoInstance, lspClient);
  // ... etc
}
```

### 3. Feature Activation (User Interaction)

When the user types, Monaco triggers the appropriate provider:

```typescript
// Example: Constraint annotation completion
monacoInstance.languages.registerCompletionItemProvider('five', {
  triggerCharacters: ['@'],  // Auto-trigger when typing '@'

  async provideCompletionItems(model, position) {
    // Call LSP via WASM
    const completions = await lspClient.getCompletions(
      model.uri.toString(),
      model.getValue(),
      position.lineNumber - 1,
      position.column - 1
    );

    // Convert to Monaco format
    return { suggestions: completions.items };
  }
});
```

## LSP Build Process

### Build Scripts

**five-lsp/build-wasm.sh:**
```bash
# Build WASM and copy to frontend
./build-wasm.sh              # Release build
./build-wasm.sh --dev        # Dev build (faster, larger)
./build-wasm.sh --no-copy    # Build without copying
```

**five-frontend/scripts/rebuild-lsp.sh:**
```bash
# Wrapper script for convenience
npm run rebuild:lsp          # Calls ../five-lsp/build-wasm.sh
npm run rebuild:lsp:dev      # Dev mode
```

### Build Pipeline

```
┌─────────────────┐
│ Rust LSP Source │
│ (five-lsp/src)  │
└────────┬────────┘
         │
         ▼
    wasm-pack build
         │
         ▼
┌─────────────────┐
│  WASM Bindings  │
│ (five-lsp/pkg)  │
└────────┬────────┘
         │
         │ copy
         ▼
┌──────────────────────┐
│ Frontend Public Dir  │
│ (public/wasm/)       │
└───────────┬──────────┘
            │
            │ serve
            ▼
       ┌─────────┐
       │ Browser │
       └─────────┘
```

### Automatic Rebuild

The `prebuild` npm script ensures LSP is always fresh:

```json
{
  "scripts": {
    "prebuild": "bash scripts/rebuild-lsp.sh",
    "build": "next build --webpack"
  }
}
```

When you run `npm run build`, it:
1. Runs `prebuild` → rebuilds LSP WASM
2. Copies WASM to `public/wasm/`
3. Builds Next.js app with latest LSP

## Testing LSP Changes

### 1. Unit Tests (Rust)

Test LSP features in isolation:

```bash
cd five-lsp

# Test constraint completion
cargo test -p five-lsp --lib completion::tests -- --nocapture

# Test all features
cargo test -p five-lsp --lib -- --nocapture
```

### 2. Integration Test (Browser)

Test in the actual Monaco editor:

```bash
# Rebuild LSP
cd five-lsp
./build-wasm.sh --dev

# Start dev server
cd ../five-frontend
npm run dev

# Open browser to http://localhost:3000
# Open IDE, create a .v file, and test:
```

**Test constraint completion:**
```v
pub transfer(from: account @
                           ^ Type '@' here - should see 4 suggestions
```

**Test hover:**
```v
let x: u64 = 10;
    ^ Hover over 'x' - should see type info
```

**Test diagnostics:**
```v
let x = "unterminated string
                              ^ Should see syntax error
```

### 3. Performance Testing

Monitor LSP performance in browser DevTools:

```javascript
// Console commands to check LSP metrics
performance.getEntriesByName('lsp:completion')
performance.getEntriesByName('lsp:diagnostics')
```

Expected performance:
- **Completion:** < 50ms
- **Diagnostics:** < 120ms (medium files)
- **Hover:** < 60ms
- **Definition:** < 80ms

## Troubleshooting

### LSP Not Loading

**Symptom:** No autocomplete, no diagnostics, console error: `Failed to load WASM module`

**Fix:**
```bash
# Rebuild and verify files exist
cd five-lsp
./build-wasm.sh
ls -lh ../five-frontend/public/wasm/

# Should show:
# five_lsp.js           (28KB)
# five_lsp_bg.wasm      (928KB)
```

### Stale LSP Features

**Symptom:** Changes to LSP code don't appear in browser

**Fix:**
```bash
# Hard refresh browser cache
# Chrome/Firefox: Cmd+Shift+R (Mac) or Ctrl+Shift+R (Windows)

# Or rebuild and restart dev server
cd five-frontend
npm run rebuild:lsp
npm run dev
```

### WASM Init Failed

**Symptom:** Console error: `Failed to initialize Five LSP`

**Check:**
1. WASM files are in `public/wasm/`
2. Dev server is serving `/wasm/five_lsp.js` correctly
3. Browser supports WASM (all modern browsers do)

**Debug:**
```javascript
// In browser console
fetch('/wasm/five_lsp.js')
  .then(r => r.text())
  .then(t => console.log(t.substring(0, 100)))
  // Should show JavaScript module code
```

### Constraint Completion Not Working

**Symptom:** Typing `@` doesn't show constraint suggestions

**Check:**
1. LSP is initialized: Check console for `[FiveLspClient] Initialized successfully`
2. Provider is registered: Look for `[Monaco Completion] Completion provider registered`
3. Trigger character is set: Verify `triggerCharacters: ['@']` in monaco-completion.ts

**Debug:**
```javascript
// In browser console, check completion provider
monaco.languages.getLanguages()  // Should include 'five'
```

## Advanced: Custom Build Configuration

### Build for Specific Target

```bash
# Optimize for size (slower build, smaller WASM)
cd five-lsp
wasm-pack build --release --target web --out-dir pkg -- --features size-opt

# Optimize for speed (faster build, larger WASM)
wasm-pack build --dev --target web --out-dir pkg
```

### Skip Optimization

For rapid development, skip wasm-opt:

```bash
# Set WASM_OPT=0 to skip optimization
WASM_OPT=0 wasm-pack build --release --target web --out-dir pkg
```

## NPM Scripts Reference

```json
{
  "dev": "next dev --webpack",
  "build": "next build --webpack",
  "prebuild": "bash scripts/rebuild-lsp.sh",
  "rebuild:lsp": "bash scripts/rebuild-lsp.sh",
  "rebuild:lsp:dev": "bash scripts/rebuild-lsp.sh --dev"
}
```

**Usage:**
- `npm run dev` - Start dev server (uses existing LSP build)
- `npm run build` - Production build (auto-rebuilds LSP)
- `npm run rebuild:lsp` - Manually rebuild LSP (release mode)
- `npm run rebuild:lsp:dev` - Manually rebuild LSP (dev mode, faster)

## Related Documentation

- **LSP Contract:** `five-lsp/docs/LSP_CONTRACT.md` - API specification
- **Constraint Completion:** `five-lsp/docs/CONSTRAINT_COMPLETION.md` - Feature documentation
- **WASM Bindings:** `five-lsp/src/wasm.rs` - Rust WASM exports
- **Monaco Providers:** `five-frontend/src/lib/monaco-*.ts` - Provider implementations

## Next Steps

After understanding this integration:
1. Read `CONSTRAINT_COMPLETION.md` to learn about the constraint annotation feature
2. Check `LSP_CONTRACT.md` for full API reference
3. Explore `five-lsp/src/features/` for LSP feature implementations
4. Review `five-frontend/src/lib/monaco-*.ts` for Monaco integration patterns
