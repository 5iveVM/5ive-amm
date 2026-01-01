# Streaming Template - Architecture

A minimal, modular layout for streaming patterns in Five DSL.

## Layout

```
streaming/
├── five.toml
├── src/
│   ├── types/
│   ├── streaming/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
