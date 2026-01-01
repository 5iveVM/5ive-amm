# Repository Guidelines

## Project Structure & Module Organization
- `src/`: Rust cdylib compiled to WebAssembly; `src/tests.rs` drives DSL fixtures in `test-scripts/` with optional localnet mode (`FIVE_TEST_MODE=localnet`).
- `wrapper/`: TypeScript bindings exporting WASM symbols; `pkg*/` build outputs (do not edit manually).
- `app/`: TypeScript services, CLIs, and UI pieces (e.g., `wasm-compiler.ts`, deployment helpers, components/hooks/styles).
- `tests/`: Jest integration/validation suites; `scripts/` hold benchmarking and size tooling; `examples/` contains runnable demos; `reports/` captures build artifacts.

## Build, Test, and Development Commands
- `npm install` (Node >=16) to fetch TS deps; ensure `rustup` and `wasm-pack` are installed for Rust/WASM builds.
- `npm run dev` builds debug WASM to `pkg/`; `npm run build`, `build:nodejs`, and `build:bundler` target specific runtimes; `npm run build:all` or `./build.sh` produces release bundles plus size reports.
- `npm test` runs wasm-pack tests headless in Firefox; `npm run test:node` executes the Node.js target; `npm run test:ts` limits to Jest suites.
- `npm run lint` and `npm run type-check` keep TypeScript style consistent; use `npm run benchmark` or `npm run size-analysis` when tuning performance.

## Coding Style & Naming Conventions
- TypeScript: 4-space indentation, single quotes, explicit interfaces for WASM-bound data, PascalCase types, camelCase functions/vars; keep wrappers typed and re-exported from `wrapper/index.ts`.
- Rust: run `cargo fmt`/`cargo clippy` before pushes; modules/files stay snake_case; favor descriptive structs/enums and `Result`-based errors.
- Generated outputs in `pkg*` are build products—regenerate rather than hand-edit.

## Testing Guidelines
- Jest matches `**/*.test.ts` under `tests/` and `app/`; coverage is written to `coverage/` per `jest.config.js`—keep new code covered.
- Rust integration tests in `src/tests.rs` honor `FIVE_TEST_MODE=wasm|localnet`; `localnet` requires `five-cli` (`node ../five-cli/dist/index.js`) and a running validator.
- Keep DSL fixtures in `test-scripts/` organized by category; add params via `// @test-params` lines when extending fixtures.

## Commit & Pull Request Guidelines
- Commits use short, imperative subjects (e.g., `Add multi-file compile entrypoint`); avoid trailing punctuation.
- PRs should state scope, highlight which WASM target(s) are affected, list commands run (build/lint/tests), and link issues. Include screenshots or logs for UI/CLI output when relevant.
