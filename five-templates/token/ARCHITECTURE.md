# Token Template - Architecture

A minimal, modular layout for token patterns in Five DSL.

## Layout

```
token/
├── five.toml
├── src/
│   ├── types/
│   ├── token/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
