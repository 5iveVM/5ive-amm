# Insurance Template - Architecture

A minimal, modular layout for insurance patterns in Five DSL.

## Layout

```
insurance/
├── five.toml
├── src/
│   ├── types/
│   ├── insurance/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
