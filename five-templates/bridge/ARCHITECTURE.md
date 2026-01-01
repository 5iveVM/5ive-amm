# Bridge Template - Architecture

A minimal, modular layout for bridge patterns in Five DSL.

## Layout

```
bridge/
├── five.toml
├── src/
│   ├── types/
│   ├── bridge/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
