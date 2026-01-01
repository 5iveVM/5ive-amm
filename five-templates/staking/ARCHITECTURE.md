# Staking Template - Architecture

A minimal, modular layout for staking patterns in Five DSL.

## Layout

```
staking/
├── five.toml
├── src/
│   ├── types/
│   ├── staking/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
