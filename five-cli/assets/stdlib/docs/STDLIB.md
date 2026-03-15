# 5IVE Standard Library (Bundled v1)

The compiler provides stdlib modules from a bundled source registry.
Declare stdlib explicitly in `five.toml`:

```toml
schema_version = 1

[dependencies]
std = { package = "@5ive/std", version = "0.1.0", source = "bundled", link = "inline" }
```
When bundled stdlib examples and installed CLI behavior diverge from the pinned monorepo toolchain, treat the pinned compiler/runtime as authoritative.

## Included modules

1. `std::prelude`
2. `std::builtins`
3. `std::types`
4. `std::interfaces::spl_token`
5. `std::interfaces::system_program`

## Import style (explicit)

```v
use std::builtins::{now_seconds};
use std::interfaces::spl_token;
use std::interfaces::system_program;

pub transfer_tokens(
  source: account @mut,
  destination: account @mut,
  authority: account @signer,
) {
  spl_token::SPLToken::transfer(source, destination, authority, 1);
}
```

Also supported:

```v
use std::builtins;
let now = builtins::now_seconds();
```

Authoring guidance:
1. Prefer lowercase authored DSL types like `account`, `pubkey`, and `string<N>`.
2. Some bundled stdlib sources or generated ABI artifacts may still display `Account`; that does not change the recommended authored source style.

Documented import forms:

1. `use std::builtins::{now_seconds};` then call `now_seconds()`
2. `use std::builtins;` then call `builtins::now_seconds()`
3. `use std::interfaces::spl_token;` then call `spl_token::SPLToken::...`
4. `use std::interfaces::system_program;` then call `system_program::SystemProgram::...`
5. `use std::interfaces::spl_token::SPLToken;` then call `SPLToken::...`
6. Full path calls are also supported, e.g. `std::interfaces::spl_token::SPLToken::transfer(...)`

Imported stdlib interfaces use explicit interface paths like `spl_token::SPLToken::transfer(...)`.
Locally declared interfaces should prefer `ExampleProgram::do_thing(...)`.
Legacy in-file dot-call syntax like `ExampleProgram.do_thing(...)` may still compile for compatibility.
Authority-aware CPI guidance:
1. mark interface authority account params with `@authority`
2. declare PDA authorities on caller params with `account @pda(seeds=[...])`
3. let interface calls choose `INVOKE` vs signed CPI automatically; do not pass signer-seed arrays in normal interface call sites

Use these forms as canonical stdlib module paths.

## Builtins crypto support

Bundled `std::builtins` supports explicit-output hash and verification flows:

1. `sha256(input, out32)` and wrapper `hash_sha256_into(input, out32)`
2. `keccak256(input, out32)` and wrapper `hash_keccak256_into(input, out32)`
3. `blake3(input, out32)` and wrapper `hash_blake3_into(input, out32)`
4. `bytes_concat(left, right)` for deterministic byte preimage construction
5. `verify_ed25519_instruction(instruction_sysvar, expected_pubkey, message, signature) -> bool`
6. Large fixed `[u8; N]` literals are supported and are suitable for static signatures, messages, and known vectors

Recommended practice:
1. build preimages explicitly with `bytes_concat`
2. hash into a fixed `[u8; 32]` output buffer
3. gate entropy/auth-sensitive logic on `verify_ed25519_instruction(...) == true`
4. treat `bytes_concat(...)` output as a bytes-compatible buffer for further concat/hash calls

## Anchor porting guidance

When using bundled stdlib/interfaces to port Anchor programs:
1. preserve upstream byte layouts exactly when hashing or verifying proofs
2. keep signer/account parameters as account-like values for CPI, not raw pubkeys, when account metas are required
3. use explicit `instruction_sysvar: account` inputs for Ed25519 instruction-sysvar verification
4. do not replace upstream proof validation with counters or simplified placeholders

## Migration path

Current mode is bundled/inlined stdlib.
Future mode may support external dependency linkage.

## Troubleshooting

If your globally installed `5ive` binary behaves differently from the local monorepo code, run the local CLI dist directly:

```bash
node ./five-cli/dist/index.js <command>
```
