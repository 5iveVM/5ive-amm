# DEVNET_CU_FULL_SWEEP

## Environment Snapshot
- Timestamp: 20260217-153854
- RPC URL: https://api.devnet.solana.com
- Wallet: 9Uu1SEs2hEWSfjXAh7JG7DuZK2exsgzMfiAPbJ8ufP9w
- VM Program ID: 3SzYVwBGUJRatFNQCTerZoReuqDHDFjM2wwCdsQ48Qu1
- Versions: 5ive 1.0.26, Node v24.10.0, npm 11.6.1, Cargo cargo 1.86.0-nightly (cecde95c1 2025-01-24), Solana solana-cli 3.1.8 (src:2717084a; feat:1620780344, client:Agave)

## Summary
- PASS: 11
- FAIL: 9
- PARTIAL: 1
- Total attempted: 21
- CU max observed: 108241
- CU min observed: 3086

## Validator Harness Scenarios
| Scenario | Deploy CU | Execute CU | Total CU | Steps |
| --- | --- | --- | --- | --- |
| token_full_e2e | 5800 | 76935 | 82735 | 15 |
| external_non_cpi | 12204 | 6782 | 18986 | 1 |
| external_interface_mapping_non_cpi | 12399 | 5441 | 17840 | 1 |
| external_burst_non_cpi | 13431 | 12710 | 26141 | 1 |
| memory_string_heavy | 5800 | 76916 | 82716 | 15 |

## CPI Suite Results
| Key | Status | Exit | CU values | Signatures | Error class |
| --- | --- | --- | --- | --- | --- |
| cpi_examples_spl_token | FAIL | 1 | n/a | n/a | Program/account ownership mismatch |
| cpi_examples_pda_invoke | FAIL | 1 | n/a | n/a | Program/account ownership mismatch |
| cpi_examples_anchor_program | FAIL | 1 | n/a | n/a | Program/account ownership mismatch |
| cpi_integration_devnet | FAIL | 1 | n/a | n/a | Program/account ownership mismatch |

## Project Suite Results
| Key | Status | Exit | CU values | Signatures | Error class |
| --- | --- | --- | --- | --- | --- |
| p_5ive_token_test | PASS | 0 | n/a | n/a |  |
| p_5ive_token_client_run | FAIL | 2 | n/a | n/a | Build/type-check failure |
| p_5ive_token_client_token | FAIL | 2 | n/a | n/a | Build/type-check failure |
| p_5ive_token2_test | PASS | 0 | n/a | n/a |  |
| p_5ive_token2_client_run | PASS | 0 | n/a | n/a |  |
| p_5ive_lending_test | PASS | 0 | n/a | n/a |  |
| p_5ive_lending2_test | PASS | 0 | n/a | n/a |  |
| p_5ive_lending3_test | PASS | 0 | n/a | n/a |  |
| p_5ive_lending4_test | PASS | 0 | n/a | n/a |  |
| p_5ive_amm_test | PASS | 0 | n/a | n/a |  |
| p_5ive_cfd_test | FAIL | 1 | n/a | n/a | Missing fixtures/accounts |
| p_5ive_cfd_client_run | FAIL | 2 | n/a | n/a | Build/type-check failure |
| p_5ive_esccrow_test | FAIL | 1 | n/a | n/a | Missing fixtures/accounts |
| p_5ive_esccrow_client_run | PASS | 0 | n/a | n/a |  |

## Top Blockers
- Program/account ownership mismatch
- Build/type-check failure
- Missing fixtures/accounts
- InvalidInstructionData/encoding mismatch

## Missing-CU Runs
- cpi_examples_spl_token (Program/account ownership mismatch)
- cpi_examples_pda_invoke (Program/account ownership mismatch)
- cpi_examples_anchor_program (Program/account ownership mismatch)
- cpi_integration_devnet (Program/account ownership mismatch)
- p_5ive_token_client_run (Build/type-check failure)
- p_5ive_token_client_token (Build/type-check failure)
- p_5ive_cfd_test (Missing fixtures/accounts)
- p_5ive_cfd_client_run (Build/type-check failure)
- p_5ive_esccrow_test (Missing fixtures/accounts)

## Artifacts
- Rollup JSON: /Users/ivmidable/Development/five-mono/.reports/devnet-cu/20260217-153854/devnet-cu-rollup.json
- Validator report: not found
- Logs dir: /Users/ivmidable/Development/five-mono/.reports/devnet-cu/20260217-153854/logs
