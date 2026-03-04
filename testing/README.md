# DSL Coverage Contracts

The `testing/` directory now holds three related coverage contracts:

- `dsl-feature-matrix.json`: representative scenario-oriented coverage used by the existing end-to-end runners.
- `dsl-builtin-matrix.json`: exhaustive builtin-level inventory with explicit per-layer expectations.
- `dsl-feature-inventory.json`: tracked classification of uncataloged fixtures and feature families.

## Source Of Truth

- Canonical DSL fixtures live under `five-cli/test-scripts/`.
- Numbered category directories are the shared corpus.
- `five-wasm/test-scripts/bin`, `dex_project`, and `temp_check` remain WASM-local and are intentionally outside this matrix.
- Loose root-level `five-cli/test-scripts/*.v` fixtures are reported through the feature inventory until they are promoted into the feature matrix.

## `dsl-feature-matrix.json`

`dsl-feature-matrix.json` is the shared coverage contract for representative DSL feature testing. It is intentionally not a complete inventory of every canonical fixture under `five-cli/test-scripts/`.

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

## `dsl-builtin-matrix.json`

`dsl-builtin-matrix.json` makes stdlib builtin coverage explicit at the wrapper level.

Top-level fields:

- `version`
- `builtin_groups`
- `builtins`

Builtin group fields:

- `id`: builtin category key.
- `description`: short human description.

Builtin fields:

- `id`: stable builtin inventory key.
- `name`: wrapper name exported by `std::builtins`.
- `module`: usually `std::builtins`.
- `category`: builtin group id.
- `wrapper_of`: underlying compiler builtin or syscall.
- `arity`: wrapper parameter count.
- `requires_accounts`
- `requires_runtime_buffers`
- `requires_runtime_sysvar`
- `requires_signature_material`
- `runtime_applicable`
- `validator_applicable`
- `layers`: builtin-level coverage expectations.
- `unit_suites`: owned test files or modules.
- `matrix_scenario`: representative feature-matrix scenario, when one exists.
- `expected_limitations`: explicit gap or phase note, when applicable.

Builtin layer names:

- `compiler`
- `bytecode_unit`
- `vm_unit`
- `runtime_unit`
- `cli_matrix`
- `wasm_matrix`
- `lsp_matrix`
- `runtime_matrix`
- `validator_localnet`

## `dsl-feature-inventory.json`

`dsl-feature-inventory.json` tracks the larger fixture surface so fixtures outside the representative matrix do not silently disappear behind a green representative report.

Top-level fields:

- `version`
- `feature_families`
- `fixtures`

Feature family fields:

- `id`
- `description`
- `owned_by_category`
- `priority`: `A` or `B`
- `required_layers`
- `phase`

Fixture fields:

- `path`: relative to `five-cli/test-scripts/`, or `__root__/...` for loose root-level fixtures.
- `family`
- `status`: `uncataloged`, `matrix_candidate`, `unit_only`, or `covered`
- `coverage_notes`
- `preferred_runner`

## Intended Usage Across All Three Contracts

- Representative end-to-end suites continue to use `dsl-feature-matrix.json`.
- Exhaustive reporting and builtin/fixture expansion use `dsl-builtin-matrix.json` and `dsl-feature-inventory.json`.
- Validation scripts should fail fast if a builtin, promoted fixture, or inventory entry references a missing DSL file or invalid layer name.
- Validation also fails if a canonical `.v` fixture exists in `five-cli/test-scripts/` but is present in neither `dsl-feature-matrix.json` nor `dsl-feature-inventory.json`.
- Reports should distinguish “representative category health” from “builtin completeness” and “fixture inventory classification”.
- The feature parity markdown report's “Outside Matrix” count means “not in the representative matrix,” not “unclassified.” The true unclassified set lives in `target/feature-parity/feature-inventory.json`.
