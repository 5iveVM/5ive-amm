# Stablecoin Template - Architecture

A minimal, modular layout for stablecoin patterns in Five DSL.

## Layout

```
stablecoin/
├── five.toml
├── src/
│   ├── types/
│   ├── stablecoin/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
