# 5IVE DSL Feature Parity

This workflow verifies that DSL features are supported consistently across:

1. Tokenizer / parser
2. Type checker
3. Bytecode generation
4. Runtime behavior (representative fixture runs)

## Generate Matrix Report

```bash
cargo run -p five-dsl-compiler --bin feature_parity_report
```

Outputs:

- `target/feature-parity/matrix.json`
- `target/feature-parity/matrix.md`

## Full Audit Command

```bash
./scripts/run_feature_parity_audit.sh
```

This command:

1. Generates the matrix report.
2. Runs discrepancy-focused compiler suites (casts, diagnostics).
3. Runs protocol/compiler/VM alignment suites.
4. Runs representative runtime fixture tests.

Alignment execution is strict:

1. `five-dsl-compiler` protocol alignment test binary
2. `five-protocol` execute payload fixture test binary
3. `five-vm-mito` execute payload alignment test binary
4. `five` shared deploy/parser fixture alignment test

## CI Gate Recommendation

Treat a parity audit as failed when:

1. Any discrepancy suite fails.
2. Any alignment suite fails.
3. Runtime fixture verification fails.
4. The generated matrix reports `red > 0` without an approved deferral.
