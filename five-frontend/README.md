# Five Frontend

## Development

```bash
pnpm dev
```

Open http://localhost:3000.

## Native SOL Fees (On-Chain)

Deploys and executions on the Five VM program may include native SOL fees when the
program admin configures fee basis points on-chain. These fees are collected by
the program and are separate from normal network fees, so the UI should surface
them alongside rent/transaction costs.

