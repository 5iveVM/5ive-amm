# Namespace Manager Template

Privileged 5NS management template for:

- Per-symbol top-level registration pricing (`! @ # $ %`)
- TLD ownership records
- Subprogram binding (`@domain/subprogram -> script account`)
- Mutable updates with append-only history entries

Intended usage:

1. Deploy this program with the special symbol permission gate enabled.
2. Initialize config with `init_manager`.
3. Register TLDs with `register_tld`.
4. Bind/update subprograms with `bind_subprogram` and `update_subprogram`.
5. Resolve active mapping through `resolve`.

Fee enforcement:

- `register_tld` now enforces symbol-priced lamport payment on-chain.
- It debits `owner.lamports` and credits `treasury_account.lamports`.
- `treasury_account.key` must match configured `cfg.treasury`.

PDA-backed state accounts:

- Config: seeds `["5ns_config"]`
- TLD: seeds `["5ns_tld", symbol, domain]`
- Binding: seeds `["5ns_binding", symbol, domain, subprogram]`
- History: seeds `["5ns_history", symbol, domain, subprogram, version]`

This keeps namespace policy in upgradeable Five bytecode rather than hardcoding behavior in VM Rust code.
