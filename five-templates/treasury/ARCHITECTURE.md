# Treasury Template - Architecture

A minimal, modular layout for treasury patterns in Five DSL.

## Layout

```
treasury/
├── five.toml
├── src/
│   ├── types/
│   ├── treasury/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
