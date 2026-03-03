# CLI Compensation Inventory

This inventory tracks repo-local scripts and wrappers that existed because the CLI either did not emit correct packaged artifacts or was being consumed through unreleased local builds.

## Artifact Generation Workarounds

| Path | Classification | Notes |
| --- | --- | --- |
| `5ive-amm/package.json` | `true_cli_gap` | Previously bypassed `5ive build` and called the Rust compiler directly. Now uses local CLI build. |
| `5ive-cfd/package.json` | `true_cli_gap` | Previously emitted `build/5ive-cfd.five` directly from the Rust compiler. Now uses local CLI build. |
| `5ive-esccrow/package.json` | `true_cli_gap` | Previously used a local artifact rewrite script. Now uses local CLI build. |
| `5ive-lending/package.json` | `true_cli_gap` | Previously bypassed project builds. Now uses local CLI build. |
| `5ive-lending-2/package.json` | `true_cli_gap` | Previously bypassed project builds. Now uses local CLI build. |
| `5ive-lending-3/package.json` | `true_cli_gap` | Previously bypassed project builds. Now uses local CLI build. |
| `5ive-lending-4/package.json` | `true_cli_gap` | Previously bypassed project builds. Now uses local CLI build. |
| `5ive-token/package.json` | `true_cli_gap` | Previously compiled a raw deployment artifact from `src/token.v`. Now uses project build output. |
| `5ive-token-2/package.json` | `true_cli_gap` | Previously bypassed project builds. Now uses local CLI build. |

## Artifact Packaging Workarounds

| Path | Classification | Notes |
| --- | --- | --- |
| `5ive-esccrow/scripts/build-artifacts.mjs` | `script_should_be_deleted` | Temporary escrow-only artifact rewriter. Deleted after fixing CLI packaging. |
| `five-templates/counter/create-artifact.js` | `legacy_template_compat` | Manual `.five` packer replaced by `5ive artifact pack`. Deleted. |
| `five-templates/token/create-artifact.js` | `legacy_template_compat` | Manual `.five` packer replaced by `5ive artifact pack`. Deleted. |

## Local CLI Resolution Wrappers

| Path | Classification | Notes |
| --- | --- | --- |
| `scripts/verify-5ive-devnet.sh` | `temporary_local_dev_wrapper` | Intentionally keeps `.codex-bin/5ive` first in `PATH` while npm remains behind. |
| `five-wasm/test-account-system.sh` | `temporary_local_dev_wrapper` | Still points at local CLI dist, but no longer rebuilds it implicitly. |

## Deployment Script Reimplementations

| Path | Classification | Notes |
| --- | --- | --- |
| `five-cli/deploy-token.mjs` | `script_should_be_deleted` | Hardcoded one-off deploy helper with direct write fallback. Deleted. |
