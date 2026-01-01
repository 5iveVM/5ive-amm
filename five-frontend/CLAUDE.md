# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Quick Start Commands

**Development**
- `npm run dev` - Start dev server with Webpack (http://localhost:3000)
- `npm run lint` - Run ESLint across the codebase
- `npm run build` - Build for production (outputs to `out/` directory)
- `npm start` - Start production server
- `npm run deploy` - Build and deploy to Cloudflare Pages using Wrangler

## Project Overview

**5ive Frontend** is a Next.js 16 application providing an IDE and ecosystem tools for the Five DSL (a Solana/blockchain-focused smart contract language). The app features:
- A full-featured IDE for writing and compiling Five DSL code
- Integration with Five WASM VM for local execution and bytecode generation
- On-chain deployment and execution via Solana integration
- Theme support (light/dark mode) with Tailwind CSS
- Project file system management with virtual file explorer

### Tech Stack
- **Frontend Framework**: Next.js 16 (App Router), React 19
- **Styling**: Tailwind CSS v4, Tailwind Merge
- **State Management**: Zustand with persistence middleware
- **Editor**: Monaco Editor via `@monaco-editor/react`
- **Blockchain**: Solana Web3.js, Solana Wallet Adapter
- **Build Bundler**: Webpack (custom config in next.config.ts for WASM support)
- **VM/Compilation**: `five-vm-wasm` (local dependency) and `five-sdk` (local dependency)
- **UI Libraries**: Lucide React, Framer Motion, html2canvas
- **Deployment**: Cloudflare Pages via Wrangler

## Architecture

### State Management (Zustand Stores)

**`stores/ide-store.ts`** - Main application state for IDE, project, compilation, and execution
- **Files/Project**: Virtual file system with `files` (Map<path, content>), `activeFile`, `openFiles`, `expandedFolders`
- **Code/Compilation**: `code`, `bytecode`, `abi`, `isCompiling`, `logs`
- **Execution**: `vmState` (stack, instructionPointer, computeUnits, memory), `isExecuting`, `selectedFunctionIndex`, `executionParams`
- **On-Chain**: `deployments`, `isDeploying`, `isOnChainExecuting`, `onChainLogs`, `contractAddress`, `rpcEndpoint`
- **Compiler Options**: `v2Preview`, `enhancedErrors`, `analysisVisible`, `enableConstraintCache`, `includeDebugInfo`, `includeMetrics`, `optimizationLevel`
- **Cost Estimation**: `estimatedCost`, `solPrice`
- **Persistence**: Stores to localStorage under key `five-ide-storage`; persisted fields are explicitly listed via `partialize`

**`stores/theme-store.ts`** - Theme management
- Persists to localStorage under key `five-theme-storage`
- Supports 'dark' and 'light' themes (defaults to 'dark')

### Page Routes

- **`app/page.tsx`** - Landing page with Hero component
- **`app/ide/page.tsx`** - Main IDE interface (largest component, orchestrates editor, file explorer, VM visualizer, deployment manager)
- **`app/docs/page.tsx`** - Documentation page with DocsEditor component
- **`app/point-break/page.tsx`** - Point Break/CTF game interface

### Key Components

**Editor Components** (`components/editor/`)
- **GlassEditor.tsx** - Monaco Editor wrapper with syntax highlighting for Five DSL
- **ProjectExplorer.tsx** - Virtual file system tree view with create/delete/rename/open operations
- **EditorTabs.tsx** - Tab bar showing open files with close buttons
- **ProjectConfigModal.tsx** - Modal for managing five.toml project configuration
- **ScriptBrowserModal.tsx** - Modal for browsing example scripts
- **DocsEditor.tsx** - Editor for documentation content

**IDE Components** (`components/ide/`)
- **ExecutionControls.tsx** - Controls for running functions, parameter input, execution results

**Deployment** (`components/deploy/`)
- **DeployManager.tsx** - Handles on-chain deployment via Solana, cost estimation, transaction signing

**VM Visualization** (`components/vm/`)
- **VMVisualizer.tsx** - Displays VM state (stack, instruction pointer, compute units, memory)

**Providers** (`components/providers/`)
- **ThemeProvider.tsx** - Client-side theme application by adding class to document root and setting `color-scheme`
- **WalletProvider.tsx** - Solana wallet adapter setup

**UI Components** (`components/ui/`)
- **glass-card.tsx** - Reusable glass-morphism card component
- **ThemeToggle.tsx** - Dark/light mode toggle button

### Blockchain Integration

**`lib/onchain-client.ts`** - Solana interaction layer
- Deployment via SPL programs using `PublicKey`, `Transaction`, `TransactionInstruction`
- PDA derivation for script accounts
- Execution result handling with compute unit tracking
- Default program ID: `J99pDwVh1PqcxyBGKRvPKk8MUvW8V8KF6TmVEavKnzaF`

**Five SDK/WASM Integration**
- `five-sdk` (local dependency) for Five DSL compilation
- `five-vm-wasm` (local dependency) for WASM-based bytecode execution
- `lib/five-wasm-loader.ts` - Dynamic WASM module loader with caching
- `hooks/useFiveWasm.ts` - React hook wrapping WASM loader with loading/error states
- Webpack configured to support asyncWebAssembly (see next.config.ts)

### Build & Deployment

**`next.config.ts`** - Next.js configuration
- Webpack customization: WASM support, fallbacks for fs/path/os modules (browser environment)
- Static export output (`output: 'export'`)
- Unoptimized images for static builds

**`wrangler.toml`** - Cloudflare Pages config
- Project name: "5ive"
- Output directory: "out/" (must match Next.js build output)
- Compatibility date: 2024-09-23

**`eslint.config.mjs`** - ESLint configuration
- Extends `eslint-config-next/core-web-vitals` and `eslint-config-next/typescript`
- Ignores `.next/**`, `out/**`, `build/**`, `next-env.d.ts`

### Styling

**`app/globals.css`** - Global styles using Tailwind CSS v4
- Custom theme colors via CSS variables (Rose Pine palette: base, surface, overlay, muted, subtle, text, love, gold, rose, pine, foam, iris, highlight levels)
- Custom animations: `float`, `pulse-glow`, `glow`
- Tailwind 4 feature: @theme block for custom colors/animations

## Common Workflows

### Adding a Feature to the IDE
1. Update `ide-store.ts` with new state fields and actions if persistent state is needed
2. Create/update component in `components/editor/` or `components/ide/`
3. Use store actions in component via `useIdeStore()` hook
4. Import and integrate component into `app/ide/page.tsx`
5. Style using Tailwind CSS classes (reference Rose Pine theme colors)

### Modifying Compilation/Execution
- Five DSL compilation is handled via `five-sdk` (external dependency)
- VM execution uses `five-vm-wasm` (external dependency)
- Execution logic orchestrated in IDE page component; state managed via `useIdeStore`
- Logs are appended to `logs` array; VM state updates via `updateVmState` action

### Working with Files/Project Explorer
- Virtual file system is stored as Map in `files` state
- All file operations (create, delete, rename, update) have corresponding store actions
- File paths use forward slashes (e.g., "src/main.five", "five.toml")
- `activeFile` tracks currently edited file; `openFiles` tracks tabs; `expandedFolders` tracks tree state

### On-Chain Integration
- Wallet connection via Solana Wallet Adapter (setup in WalletProvider)
- Use `OnChainClient` from `lib/onchain-client.ts` for deployment and execution
- Default RPC: Devnet (`https://api.devnet.solana.com`)
- Cost estimation based on account size + rent + transaction fees

## Key Dependencies and Configuration

- **five-sdk** (local): Five DSL compiler and utilities
- **five-vm-wasm** (local): WASM-compiled Five VM for bytecode execution
- **Next.js 16**: App Router for pages, static export support
- **Zustand**: Global state with localStorage persistence
- **@solana/wallet-adapter-***: Wallet integration and UI
- **Tailwind CSS 4**: Utility-first styling with @theme and custom animations

## Notes for Future Development

- **WASM Loading**: The WASM module is cached after first load (see `five-wasm-loader.ts`); consider invalidation strategy if updates occur
- **State Persistence**: IDE store persists most editor state; be mindful of localStorage limits and what data should be ephemeral vs. persistent
- **Theme Flashing**: ThemeProvider applies theme class on hydration; for faster theme application in browser, consider a script tag in HTML head
- **Solana Network**: Default is Devnet; production deployments should adjust RPC and program ID
- **Static Export**: App is built as static site for Cloudflare Pages; no server-side functions or dynamic routes
- **Error Handling**: VM execution and on-chain operations log errors to `logs` state; UI displays these in execution panels
- **Bytecode Format**: Bytecode is stored as `Uint8Array`; ensure serialization/deserialization compatibility when persisting or sending to chain
