# Nft Template - Architecture

A minimal, modular layout for nft patterns in Five DSL.

## Layout

```
nft/
├── five.toml
├── src/
│   ├── types/
│   ├── nft/
│   └── main.v
└── PROJECT_SUMMARY.md
```

## Design Goals

- Keep types isolated for reuse
- Keep core logic in focused modules
- Provide a simple main entry point
