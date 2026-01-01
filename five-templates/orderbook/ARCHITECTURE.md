# Orderbook Template - Architecture

A minimal, modular layout for orderbook patterns in Five DSL.

## Layout

```
orderbook/
├── five.toml
├── src/
│   ├── types/
│   ├── orderbook/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
