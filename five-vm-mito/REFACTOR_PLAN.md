# Refactoring Plan: Modular VM Architecture

## 1. Overview

The goal of this refactoring is to transition `five-vm-mito` from a monolithic `ExecutionContext` to a modular, "Game Engine" style architecture. This improves separation of concerns, maintainability, and allows for better resource tracking (stack depth, memory usage) without sacrificing performance.

The core principle is to split the `ExecutionContext` (the "God Object") into specialized **Systems** (Managers) that handle specific domains of the VM's state.

## 2. Architecture

### Current State
Currently, `ExecutionContext` holds direct references to `StackStorage` fields and implements all logic:
- Stack operations (push/pop)
- Memory management (alloc_temp)
- Account handling (get_account, lazy_validation)
- Instruction fetching
- CPI logic

### Proposed State
The `ExecutionContext` becomes a container/coordinator for initialized Systems. It delegates operations to these systems.

```rust
pub struct ExecutionContext<'a> {
    // --- Systems ---
    pub stack: StackManager<'a>,
    pub memory: MemoryManager<'a>,
    pub accounts: AccountManager<'a>,
    pub frame: FrameManager<'a>, // Optional: could be part of stack

    // --- Core State ---
    pub pc: u16,
    pub bytecode: &'a [u8],
    pub halted: bool,
    // ... other core flags
}
```

## 3. Subsystems

### 3.1 StackManager (`systems/stack.rs`)
**Responsibility:** Manages the operand stack.
**State:**
- `stack: &'a mut [ValueRef]` (Slice from `StackStorage`)
- `sp: u8` (Stack Pointer)
- `max_depth: usize` (derived from slice length)

**API:**
- `push(value) -> Result<()>`
- `pop() -> Result<ValueRef>`
- `peek()`, `dup()`, `swap()`, `pick(depth)`
- `len()`, `is_empty()`

**BPF Safety:**
- Enforces hard limits on stack depth.
- Can track "logical stack bytes" if needed.

### 3.2 MemoryManager (`systems/memory.rs`)
**Responsibility:** Manages temporary memory buffers and heap abstractions.
**State:**
- `buffer: &'a mut [u8]` (Slice from `StackStorage.temp_buffer`)
- `pos: usize` (Current allocation pointer)

**API:**
- `alloc(size) -> Result<u8>` (Returns offset)
- `get(offset, size) -> Result<&[u8]>`
- `write_value(value) -> Result<offset>`
- `reset()`

### 3.3 AccountManager (`systems/accounts.rs`)
**Responsibility:** Manages Solana accounts, lazy validation, and lookups.
**State:**
- `accounts: &'a [AccountInfo]`
- `validator: LazyAccountValidator`
- `program_id: Pubkey`

**API:**
- `get(index) -> Result<&AccountInfo>`
- `create_account(...)`
- `create_pda(...)`
- `check_authorization(...)`

### 3.4 FrameManager (`systems/frame.rs`)
**Responsibility:** Manages call frames and local variables.
**State:**
- `call_stack: &'a mut [CallFrame]`
- `locals: &'a mut [ValueRef]`
- `csp: u8` (Call Stack Pointer)

## 4. Data Layout & Performance

### Zero-Copy "Views"
We will **not** change `StackStorage`. It remains the owner of the data arrays on the Rust stack.
When `ExecutionContext` is initialized, it splits `StackStorage` into mutable slices and passes them to the Systems.

```rust
// In ExecutionContext::new
let stack_sys = StackManager::new(&mut storage.stack);
let mem_sys = MemoryManager::new(&mut storage.temp_buffer);
// ...
```

### Inlining
All System methods must be marked `#[inline(always)]`. This ensures that the abstraction layer dissolves during compilation, resulting in identical machine code to the monolithic version.

## 5. Migration Strategy

1.  **Create Systems Module:** Initialize `src/systems/`.
2.  **Move Logic incrementally:**
    - Move Stack logic -> `StackManager`.
    - Update `ExecutionContext` to hold `StackManager`.
    - Wrapper methods in `ExecutionContext` (e.g., `ctx.push`) will delegate to `ctx.stack.push`.
3.  **Repeat for Memory and Accounts.**
4.  **Final Cleanup:** Remove legacy fields from `ExecutionContext` once all logic is moved.

## 6. Benefits
- **Testability:** Systems can be unit-tested in isolation with mock data buffers.
- **Safety:** Bounds checks are encapsulated within the managers.
- **Clarity:** `execution.rs` becomes a high-level orchestration of opcode logic, not low-level byte shuffling.
