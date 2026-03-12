# 5ive VS Code Extension

Native VS Code extension for the 5ive DSL.

## Packaging

This extension expects a bundled native `five-lsp` binary under:

- `server/aarch64-apple-darwin/five-lsp`
- `server/x86_64-apple-darwin/five-lsp`
- `server/aarch64-unknown-linux-gnu/five-lsp`
- `server/x86_64-unknown-linux-gnu/five-lsp`
- `server/aarch64-pc-windows-msvc/five-lsp.exe`
- `server/x86_64-pc-windows-msvc/five-lsp.exe`

Use:

```bash
npm ci
npm run build
node scripts/package-target.mjs --vscode-target darwin-arm64 --rust-target aarch64-apple-darwin --binary /path/to/five-lsp
```

## Settings

- `five.languageServer.path`: optional absolute override path.
- `five.languageServer.trace`: `off | messages | verbose`.
