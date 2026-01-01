# Five Frontend 2 - Project Context

## Project Overview

**Five Frontend 2** is a Next.js 16 web application that serves as a comprehensive **Integrated Development Environment (IDE)** for the **Five DSL** (Domain Specific Language). It allows developers to write, compile, debug, and deploy smart contracts for the Five VM directly from the browser.

The application leverages **WebAssembly (WASM)** to run the Five Compiler and Virtual Machine client-side, enabling near-instant feedback without backend dependencies. It also integrates with the **Solana** blockchain for on-chain deployment and execution.

### Key Features
*   **Browser-Based IDE:** Monaco Editor with Five DSL syntax highlighting.
*   **Virtual File System:** In-memory file management (create, open, delete, rename) persisted to LocalStorage.
*   **Local Execution:** WASM-powered compilation and VM execution with stack/memory visualization.
*   **On-Chain Integration:** Deploy scripts to Solana (Devnet/Localnet) and execute transactions via Wallet Adapter.
*   **Theme System:** "Rose Pine" aesthetic with dark/light mode support using Tailwind CSS v4.

## Tech Stack & Architecture

*   **Framework:** Next.js 16 (App Router), React 19.
*   **Language:** TypeScript.
*   **State Management:** `zustand` (with persistence middleware).
*   **Styling:** Tailwind CSS v4.
*   **Core Dependencies (Local):**
    *   `five-vm-wasm`: WASM bindings for the Five VM and Compiler.
    *   `five-sdk`: TypeScript SDK for interacting with the VM.
*   **Blockchain:** `@solana/web3.js`, `@solana/wallet-adapter`.
*   **Deployment:** Cloudflare Pages (Static Export).

## Building and Running

### Prerequisites
*   Node.js (v20+ recommended).
*   The `five-vm-wasm` and `five-sdk` packages must be built and available in the parent directory (as they are referenced via `file:../` in `package.json`).

### Commands

*   **Install Dependencies:**
    ```bash
    npm install
    ```

*   **Development Server:**
    ```bash
    npm run dev
    # Runs on http://localhost:3000
    ```

*   **Production Build:**
    ```bash
    npm run build
    # Outputs static files to the `out/` directory
    ```

*   **Linting:**
    ```bash
    npm run lint
    ```

*   **Deploy (Cloudflare Pages):**
    ```bash
    npm run deploy
    # Builds and deploys to Cloudflare Pages using Wrangler
    ```

## Key Directory Structure

*   **`src/app`**: Next.js App Router pages.
    *   `ide/page.tsx`: The main IDE interface.
    *   `docs/page.tsx`: Documentation viewer.
    *   `point-break/page.tsx`: A CTF-style game mode.
*   **`src/components`**: React components.
    *   `editor/`: Monaco editor wrappers and file tree.
    *   `vm/`: VM state visualization (stack, memory).
    *   `deploy/`: Solana deployment managers.
*   **`src/lib`**: Core utilities.
    *   `five-wasm-loader.ts`: Handles dynamic loading of the WASM module.
    *   `onchain-client.ts`: Solana interaction logic (deploy/execute).
*   **`src/stores`**: Global state.
    *   `ide-store.ts`: The central store managing code, filesystem, VM state, and compilation results.
*   **`five-wasm/` & `five-sdk/`**: (External) Sibling directories containing the core logic referenced by this frontend.

## Development Notes

*   **WASM Loading:** The app uses dynamic imports (`await import('five-vm-wasm')`) to load the WASM module client-side. Ensure the WASM package is correctly built in the sibling directory before starting dev.
*   **State Persistence:** The `ide-store` persists the virtual file system and user preferences to `localStorage` key `five-ide-storage`.
*   **Virtual File System:** Files are stored as a flat map (`path -> content`) in the store. Folders are implicit based on path strings (e.g., `src/main.five`).
