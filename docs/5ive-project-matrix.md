# 5IVE Project Matrix

## Scope
All active `5ive-*` projects in `/Users/ivmidable/Development/five-mono`.

## Canonical Lending Decision
- Canonical: `5ive-lending-2`
- Active matrix excludes deprecated lending variants.

## Canonical Token Decision
- Canonical: `5ive-token`
- Active matrix excludes `5ive-token-2` (legacy/informational variant).

## Matrix
| Project | Compile Entry | Unit Tests | Client | Local On-chain | Devnet On-chain | Known Blockers |
|---|---|---|---|---|---|---|
| `5ive-amm` | `npm run build` | `npm test` | No | `npm run test:onchain:local` | `npm run test:onchain:devnet` | Requires local validator + 5ive CLI target setup |
| `5ive-cfd` | `npm run build` | `npm test` | `client/main.ts` | `npm run test:onchain:local`, `npm run client:run:local` | `npm run test:onchain:devnet`, `npm run client:run:devnet` | Needs payer funding and script-account overrides for stateful funcs |
| `5ive-esccrow` | `npm run build` | `npm test` | `client/main.ts` | `npm run test:onchain:local`, `npm run client:run:local` | `npm run test:onchain:devnet`, `npm run client:run:devnet` | Account ownership/setup for escrow state in integration paths |
| `5ive-lending-2` (canonical) | `npm run build` | `npm test` | No | `npm run test:onchain:local` | `npm run test:onchain:devnet` | Oracle account plumbing needed for full liquidation e2e |
| `5ive-token` | `npm run build` | `npm test` | `client/main.ts`, `client/token.ts` | `npm run test:onchain:local`, `npm run client:run:local` | `npm run test:onchain:devnet`, `npm run client:run:devnet` | Requires token account/mint authority setup |

## Two-Phase Gate
1. **Required**: local compile + tests + local execution matrix (`scripts/verify-5ive-projects.sh`).
2. **Tracked**: devnet execution matrix with blocker accounting (`scripts/verify-5ive-devnet.sh`).
