# Vault Template - Architecture

A minimal, modular layout for vault patterns in Five DSL.

## Layout

```
vault/
├── five.toml
├── src/
│   ├── types/
│   ├── vault/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
