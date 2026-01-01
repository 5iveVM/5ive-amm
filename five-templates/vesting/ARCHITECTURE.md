# Vesting Template - Architecture

A minimal, modular layout for vesting patterns in Five DSL.

## Layout

```
vesting/
├── five.toml
├── src/
│   ├── types/
│   ├── vesting/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
