# Lending Template - Architecture

A minimal, modular layout for lending patterns in Five DSL.

## Layout

```
lending/
├── five.toml
├── src/
│   ├── types/
│   ├── lending/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
