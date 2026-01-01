# Amm Template - Architecture

A minimal, modular layout for amm patterns in Five DSL.

## Layout

```
amm/
├── five.toml
├── src/
│   ├── types/
│   ├── amm/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
