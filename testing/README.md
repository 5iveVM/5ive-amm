# DSL Feature Matrix

`dsl-feature-matrix.json` is the shared coverage contract for DSL feature testing.

## Source Of Truth

- Canonical DSL fixtures live under `five-cli/test-scripts/`.
- Numbered category directories are the shared corpus.
- `five-wasm/test-scripts/bin`, `dex_project`, and `temp_check` remain WASM-local and are intentionally outside this matrix.
- Loose root-level `five-cli/test-scripts/*.v` fixtures are reported as uncataloged until added here.

## Top-Level Shape

- `version`: manifest version.
- `categories`: category metadata and required layer coverage.
- `scenarios`: concrete testable feature scenarios.

## Category Fields

- `id`: category directory name.
- `description`: short human label.
- `required_layers`: layers that must be represented by at least one scenario in the category.

Valid layer names:

- `compiler`
- `vm`
- `cli`
- `wasm`
- `lsp`
- `solana_runtime`
- `validator_localnet`
- `validator_devnet_tracked`

## Scenario Fields

- `id`: stable scenario key used in reports and validator runs.
- `category`: category id.
- `source`: path relative to `five-cli/test-scripts/`.
- `kind`: `positive` or `negative`.
- `function`: function index for execution-oriented runners.
- `layers`: per-layer participation flags.
- `params_source`: `inline` or `test-params-comment`.
- `params`: explicit parameters when `params_source` is `inline`.
- `expected_result`: scalar expected result for local execution layers.
- `expected_error_contains`: substring that must appear in compiler/LSP diagnostics for negative cases.
- `bytecode_assertions`: optional compiler assertions.
- `runtime_mode`: `none`, `generic`, or `template_fixture`.
- `runtime_fixture`: fixture path relative to repo root when `runtime_mode` is `template_fixture`.
- `validator_mode`: `none`, `localnet_generic`, or `sdk_suite`.
- `validator_scenario`: existing SDK validator scenario name when `validator_mode` is `sdk_suite`.
- `requires_accounts`: whether the scenario requires account setup.
- `requires_cpi`: whether the scenario requires CPI or external program interaction.

## Intended Usage

- Rust matrix suites read this file directly for compiler, VM, WASM, LSP, and `five-solana` runtime coverage.
- Node runners use it for CLI execution and validator orchestration.
- The parity report derives category status from this manifest rather than hard-coded category heuristics.
- When a DSL construct is supported in compiler/local execution layers but not yet deployable on the Solana runtime, keep that limitation explicit by splitting the category across a direct DSL scenario and a separate runtime/validator bridge scenario.
