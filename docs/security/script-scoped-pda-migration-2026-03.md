# Script-Scoped PDA Migration (2026-03)

## Summary

The VM now enforces script-scoped signed PDA domains for `INVOKE_SIGNED` and `INIT_PDA_ACCOUNT`.

Effective derivation changed from:

`effective_seeds = user_seeds (+ bump)`

To:

`effective_seeds = [active_script_key] + user_seeds (+ bump)`

## Security Impact

This prevents one bytecode script from reusing another script's PDA signer domain under the shared VM program id.

## Migration Requirement

Recompute any precomputed PDA addresses used for signed CPI authority or PDA initialization using the new effective seed model.

## Pseudocode

```text
program_id = VM_PROGRAM_ID
script_key = <current bytecode account pubkey>
user_seeds = [...]

pda = find_program_address([script_key] + user_seeds, program_id)
```

## Release Classification

Security hardening release. This is an intentional breaking change for legacy seed derivations that did not include script identity.
