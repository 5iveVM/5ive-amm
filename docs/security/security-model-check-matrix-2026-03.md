# Five VM Security Model Check Matrix (2026-03)

## Purpose

Map each major security control to the right enforcement stage:

- Compile-time (developer ergonomics, static mistakes)
- Deploy-time (one-time bytecode/account admission)
- Runtime (transaction/account/context dependent)
- SDK/client-side (ergonomics only, never trust boundary)

## Trust Boundary Rule

Only on-chain runtime checks are a hard trust boundary. Compile/deploy/SDK checks are defense-in-depth and should reduce risk/cost, but must not be the sole enforcement for account authorization.

## Control Matrix

| Control | Threat Prevented | Compile | Deploy | Runtime | SDK/Client | Current Implementation |
|---|---|---|---|---|---|---|
| Script-scoped PDA signer domain (`[active_script_key] + user_seeds`) for `INVOKE_SIGNED` | Cross-script PDA signer collision/drain under shared VM `program_id` | Informational docs/comments only | No | **Yes (required)** | `findAddress` mirrors runtime | `five-vm-mito/src/handlers/system/invoke.rs` |
| Script-scoped PDA derivation for `INIT_PDA_ACCOUNT` | Cross-script PDA create/claim collision | Informational docs/comments only | No | **Yes (required)** | `findAddress` mirrors runtime | `five-vm-mito/src/handlers/system/init.rs` |
| Require `active_script_key` for signed PDA ops | Anonymous/no-context signer derivation | No | No | **Yes (required)** | No | `five-vm-mito/src/handlers/system/invoke.rs`, `five-vm-mito/src/handlers/system/init.rs` |
| Root/callee `active_script_key` set/switch/restore | Wrong signer domain after `CALL_EXTERNAL`/return | No | No | **Yes (required)** | No | `five-vm-mito/src/execution.rs`, `five-vm-mito/src/handlers/functions.rs`, `five-vm-mito/src/handlers/control_flow.rs` |
| External account remap enforcement (bound indices only) | External callee reading/writing unbound tx accounts | No | No | **Yes (required)** | No | `five-vm-mito/src/context.rs`, `five-vm-mito/src/handlers/functions.rs`, `five-vm-mito/src/handlers/memory.rs`, `five-vm-mito/src/handlers/system/invoke.rs`, `five-vm-mito/src/handlers/arrays.rs` |
| Program-owned state isolation via owner metadata trailer (`5SAO` + script key) | Script A mutating Script B state accounts | Compiler should stay unaware of trailer bytes | No | **Yes (required)** | No | `five-vm-mito/src/context.rs`, `five-vm-mito/src/systems/accounts.rs` |
| Script write authorization checks (`SAVE_ACCOUNT`, `SET_LAMPORTS`, `CLOSE_ACCOUNT`, etc.) | Unauthorized writes/lamport mutation of VM-owned accounts | No | No | **Yes (required)** | No | `five-vm-mito/src/handlers/accounts.rs`, `five-vm-mito/src/systems/accounts.rs` |
| Reserved fee-vault seed namespace block | Script forging signer in VM fee-vault namespace | Optional literal-seed lint possible | Optional static scan possible | **Yes (required)** | No | `five-vm-mito/src/handlers/system/invoke.rs`, `five-vm-mito/src/handlers/system/init.rs` |
| `CALL_EXTERNAL` import verification metadata | Calling unauthorized external bytecode account | Compiler emits metadata | Optional stronger admission gate (recommended) | **Yes (required for strict allowlist policy)** | No | `five-vm-mito/src/handlers/functions.rs`, `five-vm-mito/src/metadata.rs` |
| Bytecode structural validation (header/opcodes/jump/call bounds) | Malformed/truncated/out-of-bounds bytecode | Compiler catches most source errors | **Yes (primary)** | No | No | `five-solana/src/instructions/verify.rs` |
| Deployment account/authority checks (canonical vm_state, ownership, permissions/admin signer, no overwrite) | Unauthorized deploy/replace or invalid permissions | No | **Yes (primary)** | No | Optional preflight checks | `five-solana/src/instructions/deploy.rs`, `five-solana/src/common.rs` |
| Distinct account identity (`script_account != vm_state_account`) | Canonical vm_state overwrite via deploy/upload aliasing | No | **Yes (required)** | No | Optional preflight checks | `five-solana/src/common.rs`, `five-solana/src/instructions/deploy.rs` |
| Execution entry checks (script/vm_state ownership, canonical fee path accounts) | Executing against spoofed core accounts/fee sink | No | No | **Yes (entry gate)** | Optional instruction builder checks | `five-solana/src/instructions/execute.rs`, `five-solana/src/instructions/fees.rs` |
| SDK PDA derivation for script accounts | Client deriving wrong authority PDA | No | No | Runtime still authoritative | **Yes (ergonomic mirror)** | `five-sdk/src/program/FiveProgram.ts`, `five-sdk/src/program/AccountResolver.ts`, `five-sdk/src/program/FunctionBuilder.ts` |

## Stage Placement Guidance (What Should Be One-Time vs Per-Invocation)

### Best enforced once at deploy-time

- Bytecode structure and control-flow bounds validity.
- Permission bitmask validity and privileged permission signer requirements.
- Script account overwrite prevention / upload ownership progression.

### Must remain runtime checks

- Any check depending on live accounts/signers/writable flags in the current transaction.
- External context account remap/binding checks.
- Script ownership/isolation for VM-owned mutable accounts.
- Signed PDA derivation domain (`active_script_key`) and signer-meta matching.
- External call target authorization using runtime-provided account list.

### Compile-time only (defense-in-depth, not trust boundary)

- DSL misuse prevention (type checks, account annotation shape, unsafe patterns).
- Optional lints for static seed literals in reserved namespaces.

## Findings From This Pass

- Added one more runtime hardening: `ARRAY_CONCAT` raw-byte serialization now resolves `AccountRef` through context-bound account resolution, preventing external-context remap bypass in this path.
  - File: `five-vm-mito/src/handlers/arrays.rs`
  - Tests: unbound external `AccountRef` now fails; bound remap resolves to mapped account key.
- Added deploy-time hardening: reject `script_account` aliasing canonical `vm_state` key in deploy/upload validation.
  - Files: `five-solana/src/common.rs`, `five-solana/src/instructions/deploy.rs`
  - Tests: deploy and init-large-program alias regressions now fail with `InvalidArgument`.

## Open Hardening Opportunities

1. Strict import policy toggle:
- Current behavior is backward-compatible when import metadata is absent.
- If we want strict allowlist-by-default, add deploy/runtime policy to reject `CALL_EXTERNAL` when metadata is missing for scripts that use external calls.

2. Reserved namespace preflight lints:
- Add compiler/deploy warnings/errors for literal seeds matching reserved VM namespaces.
- Runtime block must remain final authority.

3. Publish-time SDK consistency guard:
- Keep `five-sdk` derivation tests in CI (already present for `FiveProgram.findAddress`) and ensure release pipeline blocks if those tests fail.

## Current Assessment

With script-scoped PDA signing/creation, external remap hardening, state-owner metadata isolation, and signed-op `active_script_key` requirements, the cross-script PDA drain class is closed under the intended VM model. Remaining risk is mostly policy tightening (strict import metadata requirements), not a known active signer-domain bypass.
