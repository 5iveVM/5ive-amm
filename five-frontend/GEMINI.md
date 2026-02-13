# Project Context

Short notes for the Five frontend.

## Commands
- `pnpm dev`
- `pnpm lint`
- `pnpm build`
- `pnpm run deploy`

## Key Files
- IDE page: `src/app/ide/page.tsx`
- LSP bridge: `src/lib/monaco-lsp.ts`
- On-chain client: `src/lib/onchain-client.ts`
- State store: `src/stores/ide-store.ts`

## Prereqs
- Run `npm run sync:deps` before `dev/build` if dependencies changed.
- `five-vm-wasm` and `five-sdk` are local deps and are synced by `sync:deps`.
