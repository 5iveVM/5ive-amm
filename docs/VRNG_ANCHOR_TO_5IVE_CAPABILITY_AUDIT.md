# Anchor -> 5IVE VRNG Capability Audit (Corrected)

## Scope
This audit validates the 15 reported feature gaps using compiler/tests as source of truth first, then `five-cli init` template docs.

Primary evidence roots:
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/tests`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-cli/templates/AGENTS.md`
- `/Users/amberjackson/Documents/Development/five-org/five-mono/five-cli/templates/AGENTS_REFERENCE.md`

## Executive Result
The original report overstates several hard blockers. `CPI`, `clock/sysvar access`, and `SHA256` are present in compiler codepaths. Remaining VRNG-critical risk is primarily around explicit `ed25519` verification workflow clarity and incomplete high-level array/runtime ergonomics.

## 15-Feature Classification Matrix

### 1) Ed25519 Signature Verification
- Original claim: unsupported.
- Status: `Partial` (no direct evidence of a first-class `ed25519_verify` builtin).
- Evidence:
  - Crypto syscalls include `sha256/keccak256/blake3/secp256k1_recover`, but no `ed25519` symbol: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/bytecode_generator/ast_generator/functions.rs`
- VRNG impact: authenticity checks for signed house entropy may still be blocked or require a different pattern.
- Action: `Language/runtime feature` + `Compiler diagnostic`.
- Confidence: `medium`.

### 2) SHA256 Hash Function
- Original claim: unsupported.
- Status: `Supported`.
- Evidence:
  - `sha256` syscall emission: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/bytecode_generator/ast_generator/functions.rs`
- VRNG impact: multi-source entropy hashing is feasible.
- Action: `Doc update`.
- Confidence: `high`.

### 3) Sysvar Clock Access
- Original claim: unsupported.
- Status: `Supported`.
- Evidence:
  - `get_clock` typed in checker: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/type_checker/expressions.rs`
  - `get_clock_sysvar` syscall emission: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/bytecode_generator/ast_generator/functions.rs`
- VRNG impact: slot/timestamp-like entropy sources are available.
- Action: `Doc update`.
- Confidence: `high`.

### 4) Binary Struct Serialization
- Original claim: unsupported.
- Status: `Partial`.
- Evidence:
  - Interface serializer supports binary argument encoding paths: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/interface_serializer.rs`
- VRNG impact: some binary encoding exists, but Anchor-like custom struct pack/unpack ergonomics are not 1:1.
- Action: `Compiler diagnostic` + `Doc update`.
- Confidence: `medium`.

### 5) Vector / Dynamic Arrays
- Original claim: unsupported.
- Status: `Partial`.
- Evidence:
  - Parser supports dynamic array type syntax (`type[]`): `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/parser/types.rs`
  - Codegen rejects runtime `Value::Array` as not implemented: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/bytecode_generator/ast_generator/expressions.rs`
- VRNG impact: cannot assume flexible `Vec`-like runtime behavior.
- Action: `Compiler diagnostic` + `Language/runtime feature`.
- Confidence: `high`.

### 6) Error Codes with Messages
- Original claim: unsupported.
- Status: `Partial`.
- Evidence:
  - Enhanced error framework exists: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/error`
- VRNG impact: better diagnostics are possible, but not Anchor-equivalent user-defined error-code workflow.
- Action: `Compiler diagnostic`.
- Confidence: `medium`.

### 7) Advanced Type Casting / byte-to-int helpers
- Original claim: unsupported.
- Status: `Partial`.
- Evidence:
  - Casting paths tested (parity tests): `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/tests/feature_parity_matrix.rs`
  - No direct DSL evidence for ergonomic `from_le_bytes`/`try_into` helpers.
- VRNG impact: manual byte parsing ergonomics still weaker than Rust/Anchor.
- Action: `Compiler diagnostic` + `Doc update`.
- Confidence: `medium`.

### 8) Option & Result Types
- Original claim: unsupported.
- Status: `Supported but constrained`.
- Evidence:
  - Tokens/types include Option/Result/Some/Ok/Err: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/tokenizer/tokens.rs`
  - Typechecker constructors for Some/Ok/Err: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/type_checker/expressions.rs`
- VRNG impact: optional/result modeling is possible, but not full Rust trait ecosystem.
- Action: `Doc update`.
- Confidence: `high`.

### 9) Generic Type Parameters
- Original claim: unsupported.
- Status: `Partial`.
- Evidence:
  - Generic parsing exists in type parser: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/parser/types.rs`
- VRNG impact: limited generic support exists; not equivalent to Rust generic/lifetime system.
- Action: `Doc update`.
- Confidence: `medium`.

### 10) Constraint Composition
- Original claim: only runtime require.
- Status: `Partial`.
- Evidence:
  - Parser/bytecode contain explicit constraint components: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/bytecode_generator/constraint_enforcer.rs`
- VRNG impact: declarative anchor-style ergonomics differ, but constraint tooling is not absent.
- Action: `Doc update` + `Compiler diagnostic`.
- Confidence: `medium`.

### 11) Immutable References/Borrows
- Original claim: missing.
- Status: `Unsupported (Rust-equivalent model)`.
- Evidence:
  - DSL model is value/opcode-oriented; no borrow checker semantics surfaced in parser/typechecker public flow.
- VRNG impact: requires explicit mutation/ownership style in DSL patterns.
- Action: `No change` (expected language model).
- Confidence: `high`.

### 12) Declarative Constraints (inline complex Anchor constraints)
- Original claim: missing.
- Status: `Partial`.
- Evidence:
  - Constraint parsing/enforcement infrastructure exists, but not Anchor attribute parity.
- VRNG impact: more runtime explicitness needed.
- Action: `Doc update`.
- Confidence: `medium`.

### 13) Checked Arithmetic
- Original claim: missing.
- Status: `Supported`.
- Evidence:
  - Token/operator support: `PlusChecked/MinusChecked/MultiplyChecked`: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/tokenizer/tokens.rs`
  - Bytecode ops emitted for `+? -? *?`: `/Users/amberjackson/Documents/Development/five-org/five-mono/five-dsl-compiler/src/bytecode_generator/ast_generator/expressions.rs`
- VRNG impact: overflow-safe math can be expressed.
- Action: `Doc update`.
- Confidence: `high`.

### 14) Iterators & Collection Methods
- Original claim: missing.
- Status: `Mostly unsupported at Rust parity`.
- Evidence:
  - No clear DSL-level iterator combinator surface (`map/collect`) in compiler public syntax paths.
- VRNG impact: imperative loops/manual transformations needed.
- Action: `No change` + optional roadmap.
- Confidence: `medium`.

### 15) Lifetime Management
- Original claim: missing.
- Status: `Unsupported (Rust-equivalent model)`.
- Evidence:
  - No lifetime semantics in DSL token/type surface.
- VRNG impact: not applicable in same way as Anchor Rust.
- Action: `No change`.
- Confidence: `high`.

## Misdocumentation Findings (init templates)
1. Templates under-communicate available crypto/sysvar builtins (caused false “unsupported” conclusions).
2. Templates do not clearly distinguish parser-level support vs codegen/runtime support (notably arrays).
3. Templates were overly rigid about `pubkey(0)` despite compatibility support in checker paths.

## VRNG Feasibility Map

### 1:1 Portable
- State/accounts and authorization checks.
- CPI interface calls (program/discriminator-driven).
- Hash-based entropy mixing using `sha256`.
- Time/sysvar-derived entropy collection via `get_clock`/sysvar builtins.
- Checked arithmetic using `+?/-?/*?`.

### Adaptation Required
- Ed25519 entropy authenticity checks: requires explicit validation of currently supported path.
- Dynamic entropy vector assembly: prefer fixed-layout fields/tuples over Vec-like accumulation.
- Byte parsing helpers: use supported serialization patterns instead of Rust trait helpers.

### Hard Blockers (Current)
- No proven first-class ed25519 verification builtin in current evidence set.

### Security Consequences
- Without strong signature verification, house-provided entropy can be spoofed or replayed.
- Without robust array/runtime ergonomics, entropy composition logic can become brittle unless fixed-size schemas are used.

## Prioritized Backlog
- P0: Document true support for CPI/sysvar/sha256 in init templates.
- P0: Clarify ed25519 status and add explicit unsupported diagnostic if absent.
- P1: Add diagnostics for parser-supported but codegen-unsupported constructs (arrays).
- P1: Add doc snippets that compile in CI to prevent capability drift.
- P2: Expand high-level helpers for byte parsing and richer collections.
