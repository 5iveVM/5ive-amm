# Five SDK Roadmap

This roadmap outlines the strategic direction for `five-sdk`. Our mission is to build the **Universal Toolchain** for the Five VM—bridging the gap between local development, browser-based environments, and on-chain execution.

We aim to provide a developer experience on par with top-tier frameworks like Anchor, but with the unique advantage of running the full Compiler and VM entirely in the browser via WASM.

## Pillar 1: Universal Compilation & Local Execution (Foundation)

The core differentiator of Five SDK is that it doesn't just "talk" to a blockchain; it *contains* the blockchain's execution environment.

### 1. Robust WASM Integration
**Goal:** Ensure the SDK can compile and execute scripts instantly in any JS environment (Node.js, Browser, Edge).
**Implementation:**
- [x] Integrate `five-dsl-compiler` and `five-vm-mito` via WASM.
- [x] Implement "Client-Agnostic" design (zero dependencies on `web3.js`).
- [ ] **Lazy Loading Optimization**: Ensure WASM binaries are only loaded when strictly necessary to improve startup time.

### 2. Hybrid Execution Model ("Forking")
**Goal:** Run tests locally with state fetched on-demand from a live network.
**Implementation:**
- [x] `executeLocally()` for pure execution.
- [ ] **Forking Provider**: Middleware that intercepts account reads in the WASM VM and fetches missing account data from RPC.
- [ ] **Trace Visualization**: Return detailed execution traces (stack, memory) from the WASM VM to debug failures visually.

---

## Pillar 2: Developer Experience (Anchor Parity)

We want to move from "assembling transactions" to "calling functions."

### 3. The `Program` Abstraction
**Goal:** High-level, typed interaction with deployed scripts.
**Implementation:**
- Introduce a `Program` class that wraps the static `FiveSDK` methods.
- Allow injection of a `Provider` (Connection + Wallet) to handle signing.
- **Outcome**: `program.methods.myFunc(args).rpc()` syntax.

### 4. Automatic Transaction Management
**Goal:** Eliminate boilerplate around specific Solana mechanics.
**Implementation:**
- **CU Budgeting**: Automatically simulate transactions and set optimal specific Compute Unit types.
- **Pre-flight Checks**: Verify account ownership/discriminators before sending.
- **Instruction Composition**: allow chaining `.instruction()` calls to build complex atomic transactions easily.

---

## Pillar 3: Ecosystem & Tooling (Future)

### 5. Type-Safe Client Generation (`five-gen-ts`)
**Goal:** Compile-time safety for your specific Five DSL script.
**Implementation:**
- CLI tool that reads a `.v` file or `.abi.json`.
- Generates a bespoke TypeScript client ensuring arguments match the schema.
- ```typescript
  // No more any[]
  await program.methods.transfer(recipient, amount).rpc(); 
  ```

### 6. State Fetching & Decoding
**Goal:** Read Five VM account state as native JS objects.
**Implementation:**
- `program.account.MyStruct.fetch(pubkey)`
- Auto-decode the raw byte array using the ABI's field layout.
- Support for complex nested types and enums.

---

## Appendix: Versioning Strategy

- **v1.x**: Focus on stability of the underlying WASM compilation and execution (Current).
- **v2.x**: Introduction of the high-level `Program` API and `five-gen-ts` (Planned).
