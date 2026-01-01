# Repository Guidelines

## Project Structure & Modules
- Core library code lives in `src/`, organized by compiler stages: `parser/`, `tokenizer.rs`, `ast.rs`, `type_checker/`, `bytecode_generator/`, `module_resolver.rs`, and `security_rules.rs`.
- CLI entrypoints sit in `src/bin/` (e.g., `five`, `compile_script`, `debug_compile`, `extract_abi`, demos). Run them with `cargo run --bin <name>`.
- Integration tests are under `tests/`; unit tests live next to their modules in `src/`. Example DSL inputs and fixtures are under `examples/`.
- Build artifacts land in `target/`. Avoid committing anything from there.

## Build, Test, and Development
- `cargo build` — compile the library and all binaries with default features (`abi-pack` enabled).
- `cargo test` — run unit and integration tests; pass `-- --nocapture` to see DSL compiler output.
- `cargo run --bin five -- <args>` — main compiler CLI. Use `--features call-metadata` or `--features security-audit` to exercise optional code paths.
- `cargo fmt` and `cargo clippy --all-targets --all-features` — format and lint before opening a PR.

## Coding Style & Naming
- Rust 2021 edition, 4-space indentation, `snake_case` for modules/functions, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for consts.
- Keep functions small and composable per compiler stage; prefer explicit types around parsing/type-checking boundaries.
- Align with `rustfmt` defaults; let `clippy` guide idioms and avoid unnecessary allocations in hot paths (tokenizer/parser).

## Testing Guidelines
- Favor table-driven tests in `tests/` for end-to-end compilation scenarios; add unit tests beside new modules.
- Name tests after the behavior under check (e.g., `resolves_nested_imports`, `rejects_invalid_visibility`).
- When adding features, include samples in `examples/` and wire them into integration tests to prevent regressions.

## Commit & Pull Request Practices
- Follow the existing log style: short, imperative summaries (e.g., “Add TOML metrics export”, “Fix dispatcher HALT logic”).
- Commits should be scoped and buildable; include test updates with logic changes.
- Pull requests: describe the behavior change, note affected binaries/flags, and list tests or reproduction steps. Link issues when present; attach snippets or logs if touching error reporting or diagnostics.

## Security & Configuration Notes
- The compiler supports feature flags: `call-metadata`, `security-audit`, and `abi-pack` (default). Document why a flag is enabled when adding tests or examples.
- Be cautious with filesystem access in new CLI options—keep paths relative and validate input to avoid executing untrusted DSL payloads.

## Current Code Review Notes (2025-12-13)
- `src/five_file.rs:314-325` maps ABI types using capitalized strings ("String"/"Pubkey"), but the compiler emits lowercase type names from `account_utils::type_node_to_string`, causing `FiveFile::to_bytes` to reject valid ABIs that include strings or pubkeys.
- `src/bytecode_parser.rs:154-165` treats `PUSH_STRING` as a fixed 1-byte operand; the parser skips over the length prefix but not the string payload, so `parse_function_calls` mis-aligns offsets when a string literal precedes a CALL and can surface spurious `IncompleteFunctionName` errors.
- `src/security_rules.rs:176-199` increments `current_context.call_depth` for each external call without ever decrementing, so any script with more than ~10 external calls is flagged for "Excessive call depth" even when calls are sequential rather than recursive.
