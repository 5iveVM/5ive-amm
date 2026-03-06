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
- `cfg.treasury` should be set to the VM state authority account (same destination used for VM fee flow).
- Supported symbol defaults at init:
  - `@`: `1_000_000_000` lamports (1 SOL)
  - `!`: `2_000_000_000` lamports (2 SOL)
  - `#`: `1_500_000_000` lamports (1.5 SOL)
  - `$`: `10_000_000_000` lamports (10 SOL)
  - `%`: `1_250_000_000` lamports (1.25 SOL)
- Existing deployments keep their stored values; run admin `set_symbol_price` to migrate `$` to `10_000_000_000` lamports.

PDA-backed state accounts:

- Config: seeds `["5ns_config"]`
- TLD: seeds `["5ns_tld", symbol, domain]`
- Binding: seeds `["5ns_binding", symbol, domain, subprogram]`
- History: seeds `["5ns_history", symbol, domain, subprogram, version]`

This keeps namespace policy in upgradeable Five bytecode rather than hardcoding behavior in VM Rust code.
