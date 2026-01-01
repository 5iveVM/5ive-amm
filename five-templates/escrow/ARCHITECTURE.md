# Escrow Template - Architecture

A minimal, modular layout for escrow patterns in Five DSL.

## Layout

```
escrow/
├── five.toml
├── src/
│   ├── types/
│   ├── escrow/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
