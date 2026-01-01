# Perps Template - Architecture

A minimal, modular layout for perps patterns in Five DSL.

## Layout

```
perps/
├── five.toml
├── src/
│   ├── types/
│   ├── perps/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
