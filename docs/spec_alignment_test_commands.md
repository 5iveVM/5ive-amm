# Spec Alignment Regression Commands

Run these targeted suites to validate cross-crate bytecode and execute-payload alignment.

## Rust crates

```bash
cargo test -p five-protocol --features test-fixtures execute_payload_fixtures
cargo test -p five-vm-mito execute_payload_alignment_tests
cargo test -p five deploy_verification_tests::verifier_and_parser_align_on_shared_fixtures
cargo test -p five parameter_indexing_tests
cargo test -p five-dsl-compiler protocol_alignment_tests
```

## SDK

```bash
cd five-sdk
npm run test:jest -- src/__tests__/unit/bytecode-encoder-execute.test.ts src/__tests__/unit/parameter-encoder.test.ts src/__tests__/unit/execute-wire-format.test.ts
```
