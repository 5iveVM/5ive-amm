# Namespace Manager Template

Privileged 5NS management template for:

- Per-symbol top-level registration pricing (`! @ # $ %`)
- TLD ownership records
- Subprogram binding (`@domain/subprogram -> script account`)
- Mutable updates with append-only history entries

Intended usage:

1. Deploy this program with the special symbol permission gate enabled.
2. Register TLDs with `register_tld`.
3. Bind/update subprograms with `bind_subprogram` and `update_subprogram`.
4. Resolve active mapping through `resolve`.

This keeps namespace policy in upgradeable Five bytecode rather than hardcoding behavior in VM Rust code.

