# Governance Template - Architecture

A minimal, modular layout for governance patterns in Five DSL.

## Layout

```
governance/
├── five.toml
├── src/
│   ├── types/
│   ├── governance/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
