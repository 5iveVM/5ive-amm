# Five-VM-Mito CU Optimization Bible

## Goal
Beat SPL Token CU performance. Target: **<4500 CU for transfer, <4500 CU for mint_to**.

## Current Baseline (Five VM vs SPL Token vs p-token)

| Operation | Five VM | SPL Token | p-token | Gap vs SPL |
|-----------|---------|-----------|---------|------------|
| init_mint | 11,437 | 2,967 | 100 | +285% |
| init_token_account | 9,936 | 4,527 | 185 | +119% |
| mint_to | 6,338 | 4,538 | 155 | +40% |
| transfer | 7,341 | 4,645 | 155 | +58% |
| approve | 5,968 | 2,904 | 122 | +106% |
| revoke | 5,688 | 2,677 | 97 | +113% |
| burn | 8,635 | 4,753 | 168 | +82% |

**To beat SPL:** Need 1800-3000+ CU savings depending on operation.

---

## CRITICAL: Stateless Execution Model

Solana smart contracts are **stateless like serverless workers**:
- Each transaction starts fresh with zero state
- No caching between transactions
- No warm starts - every execution is cold
- All "optimizations" below are per-execution improvements, NOT cross-transaction caching
- Focus on: fewer instructions, less memory, faster code paths within ONE transaction

---

# PART 1: PINOCCHIO PATTERNS TO ADOPT

## 1.1 Zero-Copy Account Data Access (HIGH IMPACT)

**Pinocchio pattern** (`third_party/pinocchio/src/account_info.rs:684-686`):
```rust
pub fn data_ptr(&self) -> *mut u8 {
    unsafe { (self.raw as *mut u8).add(core::mem::size_of::<Account>()) }
}
```

**Apply to Five VM**: Replace all account data copies with direct pointer arithmetic.
- Current: Bounds check → borrow → copy
- Target: Single pointer dereference

**Files to modify:**
- `five-vm-mito/src/handlers/memory.rs:105-127` (LOAD_FIELD)
- `five-vm-mito/src/context.rs:722-732` (AccountRef resolution)

**Estimated savings:** 50-100 CU for token transfers (3-4 account reads)

---

## 1.2 Ultra-Efficient Pubkey Comparison (MEDIUM IMPACT)

**Pinocchio pattern** (`third_party/pinocchio/src/pubkey.rs:44-54`):
```rust
pub const fn pubkey_eq(p1: &Pubkey, p2: &Pubkey) -> bool {
    let p1_ptr = p1.as_ptr() as *const u64;
    let p2_ptr = p2.as_ptr() as *const u64;
    unsafe {
        read_unaligned(p1_ptr) == read_unaligned(p2_ptr)
            && read_unaligned(p1_ptr.add(1)) == read_unaligned(p2_ptr.add(1))
            && read_unaligned(p1_ptr.add(2)) == read_unaligned(p2_ptr.add(2))
            && read_unaligned(p1_ptr.add(3)) == read_unaligned(p2_ptr.add(3))
    }
}
```

**Why faster:** 4 × 64-bit comparisons vs 32 byte comparisons = 80% fewer instructions

**Files to modify:**
- `five-vm-mito/src/handlers/accounts.rs` (key comparisons)
- Any pubkey validation in handlers

**Estimated savings:** 20-50 CU per pubkey comparison

---

## 1.3 Lazy Account Parsing (MEDIUM IMPACT)

**Pinocchio pattern** (`third_party/pinocchio/src/entrypoint/lazy.rs:145-157`):
```rust
pub unsafe fn new_unchecked(input: *mut u8) -> Self {
    Self {
        buffer: (input as *mut u8).add(size_of::<u64>()),
        remaining: *(input as *const u64),
    }
}
```

**Concept:** Only parse account data when actually accessed, not upfront.

**Apply to Five VM:** Don't parse all accounts on initialization - defer until first use.

**Files to modify:**
- `five-vm-mito/src/execution.rs:74-131` (initialize_execution_context)
- `five-vm-mito/src/systems/accounts.rs`

**Estimated savings:** 100+ CU when not all accounts needed

---

## 1.4 Branch Hints for Hot Paths (LOW-MEDIUM IMPACT)

**Pinocchio pattern** (`third_party/pinocchio/src/lib.rs:268-299`):
```rust
pub mod hint {
    #[inline(always)]
    pub const fn unlikely(b: bool) -> bool {
        if b { cold_path(); true } else { false }
    }

    #[inline(always)]
    #[cold]
    fn cold_path() {}
}
```

**Apply to Five VM:** Mark error paths as cold, success paths as hot.

**Files to modify:**
- All handlers in `five-vm-mito/src/handlers/`
- Error checking in `five-vm-mito/src/context.rs`

**Implementation:**
```rust
if hint::unlikely(account_idx >= accounts.len()) {
    return Err(VMErrorCode::InvalidAccountIndex);
}
```

**Estimated savings:** 10-30 CU overall (better branch prediction)

---

## 1.5 Borrow State Bitfield Tracking (LOW IMPACT)

**Pinocchio pattern** (`third_party/pinocchio/src/account_info.rs:28-44`):
```rust
#[repr(u8)]
pub enum BorrowState {
    Borrowed = 0b_1111_1111,
    MutablyBorrowed = 0b_1000_1000,
}
// Single byte tracks: 1 mutable lamport + 3 immutable lamport + 1 mutable data + 3 immutable data
```

**Apply to Five VM:** Replace Cell<u64> in LazyAccountValidator with bitfield operations.

**Estimated savings:** 5-10 CU per validation

---

## 1.6 Unchecked Invoke for Pre-Validated CPIs (HIGH IMPACT)

**Pinocchio pattern** (`third_party/pinocchio/src/cpi.rs:357-435`):
```rust
pub unsafe fn invoke_signed_unchecked(
    instruction: &Instruction,
    accounts: &[Account],
    signers_seeds: &[Signer],
) {
    // Direct syscall - no validation
    unsafe { crate::syscalls::sol_invoke_signed_c(...) };
}
```

**Apply to Five VM:** When accounts are pre-validated, skip all CPI borrow checks.

**Estimated savings:** 50-200 CU per CPI call

---

# PART 2: MEMORY & DATA STRUCTURE OPTIMIZATIONS

## 2.1 ValueRef U128 Bloat Fix (HIGH IMPACT)

**Problem** (`five-protocol/src/value.rs:62`):
```rust
pub enum ValueRef {
    // ...
    U128(u128),  // This variant expands enum from 8 to 16 bytes!
}
```

U128 is rarely used but doubles the size of ALL ValueRef instances.

**Solution A - Tagged Pointer:**
```rust
pub enum ValueRef {
    // Most variants stay inline
    U128Ref(u8),  // Index into temp buffer where u128 is stored
}
```

**Solution B - Split Enum:**
```rust
pub enum ValueRef { /* 8-byte variants only */ }
pub enum LargeValue { U128(u128), ... }  // Separate path
```

**Savings:**
- Stack: 8 bytes × 32 slots = 256 bytes saved
- Locals: 8 bytes × 32 slots = 256 bytes saved
- **Total: 512 bytes per execution**

---

## 2.2 CallFrame Packing (MEDIUM IMPACT)

**Current** (`five-vm-mito/src/types.rs:14-33`):
```rust
pub struct CallFrame<'a> {
    pub return_address: usize,    // 8 bytes - wasteful!
    pub local_count: u8,          // 1 byte
    pub local_base: u8,           // 1 byte
    pub param_start: u8,          // 1 byte
    pub param_len: u8,            // 1 byte
    pub bytecode: &'a [u8],       // 16 bytes
}  // = 32 bytes with alignment
```

**Optimized:**
```rust
pub struct CallFrame<'a> {
    pub return_address: u16,      // 2 bytes (64KB max script)
    pub local_count: u8,
    pub local_base: u8,
    pub param_start: u8,
    pub param_len: u8,
    pub bytecode: &'a [u8],       // 16 bytes
}  // = 24 bytes → fits in cache line better
```

**Savings:** 8 bytes × 8 frames = 64 bytes

---

## 2.3 ResourceManager Vec Elimination (HIGH IMPACT)

**Current** (`five-vm-mito/src/systems/resource.rs:23,44`):
```rust
heap_chunks: Vec<(*mut u8, usize, usize)>,  // Heap allocation!
heap_chunks: Vec::with_capacity(4),          // ~400 CU
```

**Optimized:**
```rust
heap_chunks: [(*mut u8, usize, usize); 4],  // Fixed on stack
heap_chunk_count: u8,
```

**Savings:** 200-400 CU per execution (eliminates heap allocation)

---

## 2.4 Skip Heap Zeroing (MEDIUM IMPACT)

**Current** (`five-vm-mito/src/systems/resource.rs:225`):
```rust
unsafe { ptr::write_bytes(ptr, 0, new_chunk_size) };  // Zeros 2KB!
```

**Optimized:** Remove or make conditional:
```rust
#[cfg(debug_assertions)]
unsafe { ptr::write_bytes(ptr, 0, new_chunk_size) };
// In release: trust allocator or initialize on use
```

**Savings:** 100-200 CU per heap chunk allocation

---

## 2.5 Parameter Storage Consolidation (MEDIUM IMPACT)

**Problem** (`five-vm-mito/src/systems/frame.rs:19`):
Parameters stored in 3 places: stack, FrameManager, and CallFrame.

**Solution:** Single shared parameter array indexed by call depth.

**Savings:** 144 bytes per context

---

## 2.6 Account Data Pointer Reuse Within Execution (MEDIUM IMPACT)

**Problem** (`five-vm-mito/src/context.rs:722-732`):
Each AccountRef access re-borrows and bounds-checks, even when reading multiple fields from the same account in one execution.

**Solution - Local variable reuse (not cross-transaction caching):**
When loading multiple fields from same account in sequence:
```rust
// Instead of: 3 separate get_account + borrow_data calls
// Do: 1 get_account, store data ptr in local, read all fields

let data = unsafe { account.borrow_data_unchecked() };
let field1 = u64::from_le_bytes(data[0..8].try_into().unwrap());
let field2 = u64::from_le_bytes(data[8..16].try_into().unwrap());
```

This is purely within a single opcode handler or execution - NOT persisted between transactions.

**Savings:** 10-30% for handlers that access multiple fields (e.g., token transfer reads balance from 2 accounts)

---

# PART 3: DISPATCH & EXECUTION OPTIMIZATIONS

## 3.1 Hot Opcode Fast Path (HIGH IMPACT)

**Current** (`five-vm-mito/src/execution.rs:134-199`):
```rust
fn dispatch_opcode_range(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    match opcode & 0xF0 {  // 14-way match (branch misprediction)
        0x00 => handle_control_flow(opcode, ctx),
        // ... then another match inside each handler
    }
}
```

**Optimized - Fast path first:**
```rust
fn dispatch_opcode_range(opcode: u8, ctx: &mut ExecutionManager) -> CompactResult<()> {
    // Hot opcodes inline (no second dispatch)
    match opcode {
        PUSH_U64 => { /* inline */ }
        ADD | SUB | MUL => { /* inline */ }
        LOAD_FIELD | STORE_FIELD => { /* inline */ }
        EQ | NEQ | LT | GT => { /* inline */ }
        JUMP_IF => { /* inline */ }
        _ => dispatch_cold_path(opcode, ctx)
    }
}
```

**Estimated savings:** 500-1500 CU for typical operations (10-30 opcodes)

---

## 3.2 Remove Dead Stack Check (QUICK WIN)

**Current** (`five-vm-mito/src/handlers/functions.rs`):
```rust
ctx.check_stack_limit()?;  // Called every CALL
```

**But** (`five-vm-mito/src/systems/resource.rs:73-78`):
```rust
pub fn check_stack_limit(&self) -> CompactResult<()> {
    // DISABLED: ...
    Ok(())  // Does nothing!
}
```

**Solution:** Remove the call entirely.

**Savings:** 20-50 CU per function call

---

## 3.3 Inline Critical Handlers (MEDIUM IMPACT)

**Current:**
- `handle_memory` marked `#[inline(never)]`
- Large functions like `store_value_into_buffer` marked `#[inline(always)]`

**Optimized:**
- Change `handle_memory` to `#[inline]` (let compiler decide)
- Break up large `#[inline(always)]` into smaller specialized functions

**Savings:** 50-100 CU

---

# PART 4: DEPLOY-TIME VS EXECUTION-TIME

## 4.1 Pre-Compiled Function Table (HIGH IMPACT)

**Current** (`five-vm-mito/src/handlers/functions.rs:81-99`):
```rust
let param_count = ctx.fetch_byte()?;           // 1 cycle
let func_addr = ctx.fetch_u16()? as usize;     // 3 cycles
if func_addr >= ctx.script().len() { ... }     // 5 cycles (bounds check)
if param_count as usize > MAX_PARAMETERS { ... } // 2 cycles
// Total: ~11 cycles + validation
```

**Optimized - Table lookup:**
```rust
let entry = FUNCTION_TABLE[func_idx];  // Pre-validated at deploy
// Total: ~1 cycle
```

**Bytecode format addition:**
```
[function_table_marker: 1]   // 0xFE
[function_count: 1]
For each function:
  [address: 2 bytes]         // u16
  [param_count: 1]
  [local_count: 1]
```

**Savings:** ~10 cycles per function call × N calls

---

## 4.2 Parameter Schema Validation (HIGH IMPACT)

**Current** (`five-vm-mito/src/context.rs:976-1040`):
- 13 if-let chains for type checking per parameter
- Full loop on every call

**Optimized - Schema once:**
```
[param_schema: after bytecode]
  [param_count: 1]
  For each param: [type_id: 1]
```

At execution: Just decode values without type checking (schema validated at deploy).

**Savings:** 20-30 CU per parameter

---

## 4.3 Constraint Pre-Validation (HIGH IMPACT)

**Current** (`five-vm-mito/src/handlers/constraints.rs:21-31`):
Each CHECK_SIGNER, CHECK_WRITABLE does full account lookup + validation.

**Optimized - Bitmap in instruction data:**
```
Constraint bitmap (8 bytes):
  bits 0-7:   Required account count
  bits 8-23:  Signer bitmap
  bits 24-39: Writable bitmap
```

SDK validates bitmap BEFORE VM execution. VM just trusts.

**Savings:** 25-50 CU per constraint check

---

## 4.4 Import Hash Table (MEDIUM IMPACT)

**Current** (`five-vm-mito/src/metadata.rs:73-183`):
Linear search through imports O(n).

**Optimized:**
```
[imports_marker: 1]
[import_count: 1]
For each import:
  [hash: 8 bytes]  // First 8 bytes of pubkey for fast rejection
```

**Savings:** 30-100 CU per import verification

---

## 4.5 Header Offset Pre-Computation (LOW IMPACT)

**Current** (`five-vm-mito/src/execution.rs:532-555`):
VLE metadata offset computed every execution.

**Optimized:** Store pre-computed offset in extended header:
```
[instruction_start: 2 bytes]  // Pre-computed
```

**Savings:** ~20-50 CU per execution

---

# PART 5: CARGO & BUILD OPTIMIZATIONS

## 5.1 Release Profile (HIGH IMPACT)

**Add to workspace Cargo.toml:**
```toml
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true
```

**Savings:** 500-2000 CU (10-20% overall)

---

## 5.2 Remove debug-logs Default (QUICK WIN)

**In five-vm-mito/Cargo.toml line 31:**
```toml
default = []  # Was: default = ["debug-logs"]
```

**Savings:** Eliminates debug macro overhead

---

# IMPLEMENTATION PRIORITY

## Sprint 1: Quick Wins (< 2 hours total)
1. ✅ Cargo profile optimization (30 min) - **500-2000 CU**
2. ✅ Remove dead stack check (15 min) - **20-50 CU/call**
3. ✅ Remove debug-logs default (5 min) - **Variable**
4. ✅ Skip heap zeroing (15 min) - **100-200 CU/alloc**

## Sprint 2: Memory Optimizations (4-8 hours)
5. ResourceManager Vec→fixed array - **200-400 CU**
6. CallFrame packing - **64 bytes saved**
7. Account data pointer reuse within handlers - **10-30% speedup for multi-field ops**

## Sprint 3: Pinocchio Patterns (8-16 hours)
8. 4x u64 pubkey comparison - **20-50 CU/compare**
9. Branch hints (unlikely/cold) - **10-30 CU overall**
10. Zero-copy account data access - **50-100 CU**

## Sprint 4: Dispatch Optimization (8-16 hours)
11. Hot opcode fast path - **500-1500 CU**
12. Inline critical handlers - **50-100 CU**

## Sprint 5: Deploy-Time Work (16-32 hours)
13. Function table pre-compilation - **~10 CU/call**
14. Parameter schema validation - **20-30 CU/param**
15. Constraint pre-validation - **25-50 CU/check**

## Sprint 6: Advanced (Future)
16. ValueRef U128 fix - **512 bytes saved**
17. Import hash table - **30-100 CU**
18. Lazy account parsing - **100+ CU**

---

# FILES TO MODIFY

| File | Changes |
|------|---------|
| `/Cargo.toml` | Add release profile |
| `/five-vm-mito/Cargo.toml` | Remove debug-logs default |
| `/five-vm-mito/src/systems/resource.rs` | Fixed array, skip zeroing |
| `/five-vm-mito/src/execution.rs` | Hot opcode dispatch, pre-computed offsets |
| `/five-vm-mito/src/handlers/functions.rs` | Remove stack check, use function table |
| `/five-vm-mito/src/handlers/memory.rs` | Zero-copy patterns, data pointer reuse |
| `/five-vm-mito/src/handlers/accounts.rs` | Pubkey comparison optimization |
| `/five-vm-mito/src/handlers/constraints.rs` | Pre-validation support |
| `/five-vm-mito/src/context.rs` | Parameter schema, data pointer reuse |
| `/five-vm-mito/src/types.rs` | CallFrame packing |
| `/five-vm-mito/src/lazy_validation.rs` | Bitfield optimization |
| `/five-protocol/src/value.rs` | ValueRef U128 fix |
| `/five-dsl-compiler/src/bytecode_generator/` | Emit tables/schemas |

---

# ESTIMATED TOTAL SAVINGS

| Category | Estimated CU Savings |
|----------|---------------------|
| Quick Wins | 800-2500 |
| Memory Optimizations | 300-700 |
| Pinocchio Patterns | 100-300 |
| Dispatch Optimization | 550-1600 |
| Deploy-Time Work | 500-1500+ |
| **TOTAL** | **2250-6600+ CU** |

**Reality Check:**
- mint_to (6338 → target 4500): Need 1838 CU → **Achievable with Sprints 1-3**
- transfer (7341 → target 4600): Need 2741 CU → **Achievable with Sprints 1-4**
- approve (5968 → target 2900): Need 3068 CU → **Needs all optimizations**

---

# VERIFICATION PLAN

1. Baseline: Run e2e-token-test.mjs, record all CU
2. After each sprint: Re-run tests, track CU delta
3. Regression: `cargo test -p five-vm-mito`
4. Binary size: `cargo bloat` (ensure not growing)
5. Profile: Use `solana-program-test` with CU tracking


# Five-VM Optimization Analysis Report

## Executive Summary

This report validates the proposed "Five-VM-Mito CU Optimization Bible" against the current codebase. The analysis confirms that the proposed optimizations are largely feasible and correctly targeted. Significant Compute Unit (CU) reductions are achievable with minimal risk by focusing on configuration, memory layout, and dispatch logic.

## 1. Validated Quick Wins (Sprint 1)

These optimizations are confirmed to be low-effort and high-impact.

*   **Release Profile Optimization:**
    *   **Status:** ✅ CONFIRMED
    *   **Finding:** The workspace `Cargo.toml` lacks a `[profile.release]` section. Adding `lto = "fat"`, `codegen-units = 1`, and `panic = "abort"` will yield immediate binary size and runtime improvements.
*   **Disable Debug Logs:**
    *   **Status:** ✅ CONFIRMED
    *   **Finding:** `five-vm-mito/Cargo.toml` has `default = ["debug-logs"]`. Disabling this removes significant macro overhead from the default build. `five-solana` correctly disables it by default, but ensuring it's off in `five-vm-mito` standalone is best practice.
*   **Remove Dead Stack Check:**
    *   **Status:** ✅ CONFIRMED
    *   **Finding:** `five-vm-mito/src/handlers/functions.rs` calls `ctx.check_stack_limit()?`, but the implementation in `resource.rs` is empty/disabled. Removing the call site saves function call overhead on every `CALL`.
*   **Skip Heap Zeroing:**
    *   **Status:** ✅ CONFIRMED
    *   **Finding:** `alloc_heap_unsafe` in `five-vm-mito/src/systems/resource.rs` explicitly zeroes memory (`ptr::write_bytes`). Removing this (or making it debug-only) saves cycles for every memory allocation.

## 2. High Impact Architecture Changes (Sprint 2)

*   **Hot Opcode Dispatch:**
    *   **Status:** ✅ CONFIRMED
    *   **Finding:** `five-vm-mito/src/execution.rs` uses a nibble-based match (`opcode & 0xF0`). This adds a branch for every instruction. Refactoring to check hot opcodes (`PUSH`, `ADD`, `LOAD`, `STORE`, `EQ`) *before* the nibble dispatch will significantly reduce overhead for the most common operations.
*   **ResourceManager `Vec` Elimination:**
    *   **Status:** ✅ CONFIRMED
    *   **Finding:** `ResourceManager` uses `Vec<(*mut u8, usize, usize)>` to track heap chunks. Since the number of chunks is small (typically <4), replacing this with a fixed-size array `[...; 4]` eliminates `Vec` allocation and pointer indirection overhead.
*   **CallFrame Packing:**
    *   **Status:** ✅ CONFIRMED
    *   **Finding:** `CallFrame` uses `usize` (8 bytes) for `return_address`. Since `JUMP` instructions use `u16` offsets, the script size is effectively limited to 64KB. Packing `return_address` to `u16` and optimizing other fields reduces `CallFrame` size from ~32 bytes to ~24 bytes or less, improving stack locality and cache usage.

## 3. Advanced Pinocchio Patterns

*   **Zero-Copy Pubkey Comparison:**
    *   **Status:** ✅ CONFIRMED
    *   **Finding:** `five-vm-mito`'s `check_equality` logic (used by `EQ`, `NEQ`) currently copies 32-byte Pubkeys via `extract_pubkey` (which returns `[u8; 32]`) before comparing them.
    *   **Recommendation:** Implement a zero-copy comparison that casts the data pointers to `*const u64` and compares 4 `u64` words in place, bypassing the 32-byte copy. This matches the highly efficient Pinocchio pattern.
*   **Zero-Copy Field Access:**
    *   **Status:** ✅ CONFIRMED
    *   **Finding:** `STORE_FIELD` and `LOAD_FIELD` currently use slice copying. Using raw pointer arithmetic (carefully guarded) can bypass bounds checking overheads validated by `AccountInfo`.

## 4. Observations on Other Components

*   **five-solana:**
    *   Correctly uses `borrow_data_unchecked` (Pinocchio pattern).
    *   Delegates execution to `MitoVM`. Optimizations in `MitoVM` will directly benefit Solana transaction performance.
*   **five-dsl-compiler:**
    *   No immediate changes required for VM-internal optimizations (Dispatch, Memory).
    *   Future optimizations involving "Function Tables" or "Parameter Schemas" will require compiler updates, but are not blockers for Sprint 1/2.
*   **five-protocol:**
    *   `ValueRef` alignment issue (U128) confirmed, but fixing it requires a breaking protocol change. Recommended to defer until after VM runtime optimizations are solidified.

## Prioritized Implementation Plan

1.  **Configuration (Immediate):** Update `Cargo.toml` profiles and feature flags.
2.  **Memory Core (High Priority):** Optimize `ResourceManager` (Fixed Array) and `CallFrame` (Packing).
3.  **Execution Loop (High Priority):** Implement Hot Opcode Dispatch.
4.  **Pinocchio Optimizations (Medium Priority):** Implement zero-copy `EQ` for Pubkeys.
