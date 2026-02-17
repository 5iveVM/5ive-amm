# Adversarial Security Audit Report

Date: 2026-02-17
Target scope: `five-solana` runtime + `five-vm-mito` VM
Attacker model: unprivileged tx composer, malicious deployer, malicious CPI callee, malicious external bytecode account

## Method
- Static review of instruction handlers, CPI sites, account remap/dispatch logic, parameter decoding, and validation subsystems.
- Dynamic PoC tests added for exploitability evidence where feasible.
- Findings rated `P0..P3` by attacker impact to funds, authority, and trust boundaries.

## Findings Register

### F-01: Arbitrary CPI target in fee/deploy/vault-init paths via weak system-program identity checks
- Severity: P0
- Confidence: High
- Evidence type: Dynamic PoC + static
- Impact: Attacker-supplied account can be used as CPI program target in fee/vault-init flows when payer is system-owned, violating trusted program boundary.
- Preconditions: Caller can pass arbitrary account metas for system program slot.
- Blast radius: Fee transfer path, deploy fee collection, and fee-vault init call path.
- Anchors:
  - `five-solana/src/instructions/fees.rs:20`
  - `five-solana/src/instructions/fees.rs:153`
  - `five-solana/src/instructions/fees.rs:189`
  - `five-solana/src/instructions/deploy.rs:197`
- PoC tests:
  - `five-solana/src/instructions/fees.rs:481`
  - `five-solana/src/instructions/fees.rs:528`

### F-02: Permissionless first-initializer authority capture
- Severity: P0
- Confidence: High
- Evidence type: Dynamic PoC + static
- Impact: First signer to initialize canonical VM state becomes authority.
- Preconditions: Fresh or otherwise uninitialized canonical VM state account.
- Blast radius: Full administrative control of runtime fees and privileged deploy semantics.
- Anchors:
  - `five-solana/src/instructions/deploy.rs:28`
  - `five-solana/src/instructions/deploy.rs:109`
- PoC test:
  - `five-solana/src/instructions/deploy.rs:568`

### F-03: CALL_EXTERNAL remap clobber (validated with one remap, execution uses another)
- Severity: P0
- Confidence: High
- Evidence type: Dynamic PoC + static
- Impact: Constraints can be validated against call-arg remap but runtime context uses stale prior remap.
- Preconditions: External call with account args and existing remap state.
- Blast radius: Cross-program call isolation and constraint soundness.
- Anchors:
  - `five-vm-mito/src/handlers/functions.rs:863`
  - `five-vm-mito/src/handlers/functions.rs:880`
  - `five-vm-mito/src/handlers/functions.rs:897`
- PoC test:
  - `five-vm-mito/src/handlers/functions.rs:1286`

### F-04: External visibility bypass via legacy absolute-offset selector fallback
- Severity: P1
- Confidence: High
- Evidence type: Dynamic PoC + static
- Impact: Selector can target non-public code offsets directly, bypassing public entry table/function visibility intent.
- Preconditions: External bytecode length includes attacker-selected offset.
- Blast radius: Private/internal function reachability and invariant breaks.
- Anchor:
  - `five-vm-mito/src/handlers/functions.rs:520`
- PoC test:
  - `five-vm-mito/src/handlers/functions.rs:1242`

### F-05: Parameter account-index truncation (`u32 -> u8`)
- Severity: P1
- Confidence: High
- Evidence type: Dynamic PoC + static
- Impact: High account indices are silently wrapped, enabling mis-binding to unintended accounts.
- Preconditions: Crafted execute envelope with `types::ACCOUNT` index > 255.
- Blast radius: Account routing and permission checks.
- Anchor:
  - `five-vm-mito/src/context.rs:1262`
- PoC test:
  - `five-vm-mito/src/context.rs:1459`

### F-06: Lazy validator bitmap width mismatch (`u64` bitmap vs account index space up to 254)
- Severity: P1
- Confidence: High
- Evidence type: Dynamic PoC + static
- Impact: Accessing index >= 64 can panic (`shift right with overflow`) or mis-track validation state depending build/runtime.
- Preconditions: Transactions with >= 65 account indices reachable by VM paths.
- Blast radius: Availability (abort) and validation correctness.
- Anchors:
  - `five-vm-mito/src/lazy_validation.rs:19`
  - `five-vm-mito/src/lazy_validation.rs:47`
  - `five-solana/src/lib.rs:38`
- PoC test:
  - `five-vm-mito/src/lazy_validation.rs:225`

### F-07: Double execution path behind `PERMISSION_POST_BYTECODE`
- Severity: P1
- Confidence: High
- Evidence type: Static
- Impact: Bytecode executes twice when post permission flag set; side-effectful scripts can mutate state twice or duplicate transfers.
- Preconditions: Script deployed with `PERMISSION_POST_BYTECODE` and stateful opcode flow.
- Blast radius: Any side-effectful execution path.
- Anchor:
  - `five-solana/src/instructions/execute.rs:108`

### F-08: INVOKE_SIGNED TempRef panic concern (status update)
- Severity: P3
- Confidence: Medium
- Evidence type: Dynamic negative test + static
- Impact: Previously suspected unchecked-slice panic did not reproduce under current constraints because `TempRef(offset,len)` uses `u8` and temp buffer is 512 bytes (max end 510).
- Preconditions: N/A for panic under current bounds.
- Blast radius: Low in current implementation; keep as defense-in-depth check if representation changes.
- Anchors:
  - `five-vm-mito/src/handlers/system/invoke.rs:351`
  - `five-vm-mito/src/handlers/system/invoke.rs:387`
- Negative tests:
  - `five-vm-mito/src/handlers/system/invoke.rs:502`
  - `five-vm-mito/src/handlers/system/invoke.rs:551`

## Reproduction Bundle

### Direct commands
- `cargo test -p five fee_validation_accepts_non_system_program_key -- --nocapture`
- `cargo test -p five init_fee_vault_accepts_non_system_program_identity_when_idempotent -- --nocapture`
- `cargo test -p five initialize_allows_first_signer_to_capture_authority -- --nocapture`
- `cargo test -p five-vm-mito call_external_clobbers_computed_account_remap -- --nocapture`
- `cargo test -p five-vm-mito resolve_external_target_allows_legacy_absolute_offset_into_non_public_code -- --nocapture`
- `cargo test -p five-vm-mito parse_parameters_truncates_account_index_to_u8 -- --nocapture`
- `cargo test -p five-vm-mito ensure_validated_panics_for_indices_above_bitmap_width -- --nocapture`

### Executed during audit
- All commands above were executed in this session and passed (except intentionally superseded INVOKE_SIGNED panic assumptions, now covered by non-panic checks).

## Remediation Map

### R-01 (F-01): Enforce exact System Program identity everywhere
- Patch: In `fees.rs` and deploy-fee/vault-init call sites, require `system_program.key() == Pubkey::default()` and optionally `owner == executable system program` invariant.
- Guard tests:
  - Reject fake system key in `collect_deploy_fee_with_state`.
  - Reject fake system key in `init_fee_vault` (including idempotent path).

### R-02 (F-02): Lock bootstrap authority semantics
- Patch options:
  - Hardcoded bootstrap authority signer required for first init.
  - Configurable bootstrap PDA/signature gate with one-time consume flag.
- Guard tests:
  - Unauthorized first initializer rejected.
  - Authorized initializer succeeds once.

### R-03 (F-03): Preserve computed remap through frame prep and callee setup
- Patch: Rename variables to avoid shadowing; pass computed remap into `prepare_callee_frame` and `ctx.set_external_account_remap`.
- Guard tests:
  - Existing PoC should assert new remap is retained and used.

### R-04 (F-04): Remove or gate legacy absolute-offset fallback
- Patch options:
  - Remove fallback entirely when public table/function names exist.
  - Restrict fallback behind explicit compatibility feature and verify visibility.
- Guard tests:
  - Non-public offset selector must fail.
  - Public selector/hash/index still works.

### R-05 (F-05): Reject out-of-range account indices in envelope parsing
- Patch: Replace cast with checked conversion: `u8::try_from(idx).map_err(...)`.
- Guard tests:
  - `idx <= 255` accepted.
  - `idx > 255` rejected with deterministic VM error.

### R-06 (F-06): Replace fixed-width bitmap with scalable tracking
- Patch options:
  - Dynamic bitset sized to account_count.
  - Explicit hard guard `account_count <= 64` with early error.
- Guard tests:
  - Index 64+ handling must not panic.
  - Validation bookkeeping remains correct for high account counts.

### R-07 (F-07): Make post hook semantically distinct or single-execution safe
- Patch options:
  - Call dedicated post-hook entrypoint instead of re-running main bytecode.
  - Disable duplicate full execution for stateful scripts.
- Guard tests:
  - Side-effect counter increments once unless explicitly intended.

### R-08 (F-08): Keep non-panic guarantees explicit
- Patch: Add explicit bounds checks before TempRef slicing for future-proofing.
- Guard tests:
  - Current non-panic tests remain green even if buffer/ValueRef representation changes.

## Residual Risk
- If F-01 and F-02 remain unpatched, attacker-controlled authority and CPI trust-boundary breaks remain system-critical.
- F-03 and F-04 can silently bypass intended external-call constraints/visibility and are high-risk for cross-contract composition.
- F-06 remains an availability hazard for high-account transactions.
- F-08 currently appears non-exploitable under current type/size bounds but should remain covered by regression tests.
