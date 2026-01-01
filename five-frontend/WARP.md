# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Quick Start Commands

**Development**
- `npm run dev` - Start dev server with Webpack at http://localhost:3000
- `npm run build` - Build for production (outputs to `out/` directory)
- `npm start` - Start production server
- `npm run lint` - Run ESLint
- `npm run deploy` - Build and deploy to Cloudflare Pages using Wrangler

**Common Workflows**
- Build CLI JS-only for testing: `npm -C ../five-cli run build:js` (if working with CLI integration)
- WASM local execution test: `node ../five-cli/dist/index.js execute <file.v> --function 0 --params "[3,4]" --local --trace`

## Project Overview

**5ive Frontend** is a Next.js 16 application providing an IDE and ecosystem tools for Five DSL (a Solana blockchain smart contract language). Core features:
- Full-featured IDE with Monaco Editor for writing and compiling Five DSL code
- Integration with Five WASM VM for local bytecode execution
- On-chain deployment and execution via Solana
- Virtual file system with project explorer
- Theme support (Rose Pine dark/light mode)

### Tech Stack
- **Framework**: Next.js 16 (App Router), React 19
- **Styling**: Tailwind CSS v4 with custom Rose Pine theme
- **State**: Zustand with localStorage persistence
- **Editor**: Monaco Editor (`@monaco-editor/react`)
- **Blockchain**: Solana Web3.js, Wallet Adapter
- **Build**: Webpack with WASM support
- **VM**: `five-vm-wasm` (local dependency) and `five-sdk` (local dependency)
- **Deployment**: Cloudflare Pages via Wrangler

## Architecture

### State Management (Zustand)

**`stores/ide-store.ts`** - Main application state
- **Virtual File System**: `files` (Record<path, content>), `activeFile`, `openFiles`, `expandedFolders`
- **Code/Compilation**: `code`, `bytecode`, `abi`, `isCompiling`, `logs`, `compilerOptions`
- **VM Execution**: `vmState` (stack, instructionPointer, computeUnits, memory), `isExecuting`, `executionParams`
- **On-Chain**: `deployments`, `isDeploying`, `contractAddress`, `rpcEndpoint` (defaults to Solana devnet)
- **Persistence**: Stored in localStorage under `five-ide-storage`; only specified fields are persisted via `partialize`

**`stores/theme-store.ts`** - Theme management
- Persisted to localStorage under `five-theme-storage`
- Supports 'dark' (default) and 'light' themes

### Page Structure

- `app/page.tsx` - Landing page
- `app/ide/page.tsx` - Main IDE interface (primary entry point for development)
- `app/docs/page.tsx` - Documentation editor
- `app/point-break/page.tsx` - Point Break/CTF game interface

### Key Component Directories

**`components/editor/`** - IDE editor components
- **GlassEditor.tsx** - Monaco Editor wrapper with Five DSL syntax highlighting
- **ProjectExplorer.tsx** - Virtual file system tree view with CRUD operations
- **EditorTabs.tsx** - Tab bar for open files
- **ProjectConfigModal.tsx** - Manage five.toml project configuration
- **ScriptBrowserModal.tsx** - Browse example scripts
- **DocsEditor.tsx** - Documentation content editor

**`components/ide/`** - IDE feature components
- **ExecutionControls.tsx** - Function execution controls, parameter input, results display

**`components/deploy/`** - Blockchain deployment
- **DeployManager.tsx** - Handles on-chain deployment, cost estimation, transaction signing

**`components/vm/`** - VM visualization
- **VMVisualizer.tsx** - Displays VM state (stack, instruction pointer, compute units, memory)

**`components/providers/`** - Context providers
- **ThemeProvider.tsx** - Client-side theme application
- **WalletProvider.tsx** - Solana wallet adapter setup

### Blockchain Integration

**`lib/onchain-client.ts`** - Solana interaction layer
- Deployment via SPL programs using PublicKey, Transaction, TransactionInstruction
- PDA derivation for script accounts
- Default program ID: `J99pDwVh1PqcxyBGKRvPKk8MUvW8V8KF6TmVEavKnzaF` (localnet)
- VM state PDA derived using seed `["vm_state"]`
- Instruction discriminators: Deploy=8, Execute=9
- Account order for execute: `[script_account (RO), vm_state (RW), signer (RW)]`

**Five SDK/WASM Integration**
- `five-sdk` (local dependency at `../five-sdk`) - Five DSL compilation
- `five-vm-wasm` (local dependency at `../five-wasm/pkg`) - WASM-based bytecode execution
- `lib/five-wasm-loader.ts` - Dynamic WASM module loader with caching
- `hooks/useFiveWasm.ts` - React hook wrapping WASM loader with loading/error states
- Webpack configured for asyncWebAssembly support (see `next.config.ts`)

### Build Configuration

**`next.config.ts`**
- Webpack WASM support and fallbacks for fs/path/os (browser environment)
- Static export (`output: 'export'`)
- Unoptimized images for static builds

**`wrangler.toml`**
- Project: "5ive"
- Output: "out/" (matches Next.js build output)

**`eslint.config.mjs`**
- Extends `eslint-config-next/core-web-vitals` and `eslint-config-next/typescript`
- Ignores `.next/**`, `out/**`, `build/**`, `next-env.d.ts`

### Styling

**`app/globals.css`**
- Tailwind CSS v4 with `@theme` block
- Custom Rose Pine theme colors (base, surface, overlay, muted, subtle, text, love, gold, rose, pine, foam, iris, highlight levels)
- Custom animations: `float`, `pulse-glow`, `glow`
- Utility classes: `.glass-panel`, `.glass-panel-heavy`, `.glass-button`, `.glass-input`

## Development Workflows

### Adding IDE Features
1. Update `stores/ide-store.ts` with new state fields/actions if needed
2. Create/update component in appropriate `components/` subdirectory
3. Use store via `useIdeStore()` hook
4. Import and integrate into `app/ide/page.tsx`
5. Style using Tailwind with Rose Pine theme colors

### Working with Virtual File System
- File system is stored as `Record<string, string>` in `files` state
- All file operations have corresponding store actions (create, delete, rename, update)
- File paths use forward slashes (e.g., "src/main.five", "five.toml")
- `activeFile` = currently edited file; `openFiles` = tab state; `expandedFolders` = tree state

### Compilation and Execution
- Five DSL compilation handled via `five-sdk` (external dependency)
- VM execution uses `five-vm-wasm` (external dependency)
- Execution logic orchestrated in IDE page component
- Logs appended to `logs` array; VM state updated via `updateVmState` action
- VLE (Variable Length Encoding) used for parameter encoding in execute instructions

### On-Chain Integration
- Wallet connection via Solana Wallet Adapter (configured in WalletProvider)
- Use `OnChainClient` from `lib/onchain-client.ts` for deployment and execution
- Default RPC: Devnet (`https://api.devnet.solana.com`)
- Cost estimation based on account size + rent + transaction fees
- Deploy instruction format: `[8, bytecode_len(u32_le), permissions(u8), bytecode]`
- Execute instruction format: `[9, VLE(function_index), VLE(param_count), VLE(param1), ...]`

### Testing and Validation
- WASM module is cached after first load (see `five-wasm-loader.ts`)
- When making on-chain changes, verify with local WASM execution first
- For on-chain debugging, enable `--debug` flag in CLI commands to see accounts and serialized data
- Check transaction logs using `getLogsForSignature()` when on-chain operations fail

## Important Notes

### State Persistence
- IDE store persists most editor state to localStorage
- Be mindful of localStorage size limits
- Consider what data should be ephemeral vs. persistent when adding new state

### WASM Loading
- WASM module is cached after first load
- Consider cache invalidation strategy if WASM updates frequently

### Solana Integration
- Default network is Devnet
- Program ID is currently localnet-specific (`J99pDwVh1PqcxyBGKRvPKk8MUvW8V8KF6TmVEavKnzaF`)
- Production deployments need updated RPC and program ID

### Static Export
- App is built as static site for Cloudflare Pages
- No server-side functions or dynamic routes
- All runtime logic must be client-side

### Bytecode Format
- Bytecode stored as `Uint8Array`
- Ensure serialization/deserialization compatibility when persisting or sending to chain
- Execute payload uses VLE encoding for function index and parameters

### Theme Application
- ThemeProvider applies theme class on client-side hydration
- For faster theme application, consider script tag in HTML head to prevent flash

### Error Handling
- VM execution and on-chain operations log errors to `logs` state
- UI displays logs in execution panels
- On-chain failures should capture program logs for debugging
