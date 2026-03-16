# M-Lang VS Code Extension

VS Code language tooling for M-Lang (`.ml`) with:

- Language Server (`mlang-lsp`) integration
- Semantic highlighting, diagnostics, hover, completion, definition
- Document formatting (LSP-first, CLI fallback)

## Requirements

- VS Code `^1.75.0`
- Rust toolchain (to build binaries locally)

Recommended binaries:

- `mlang-lsp` for language features
- `mlang` for formatter fallback

## Development Setup

From the `mlang` root:

```bash
cargo build --release --bin mlang-lsp
cargo build --bin mlang
```

From `mlang/editors/vscode`:

```bash
npm install
npm run compile
```

Then open `editors/vscode` in VS Code and press `F5` to launch an Extension Development Host.

## User Setup

Add settings in your workspace (or user) `settings.json`:

```json
{
  "mlang.server.path": "C:/path/to/mlang/target/release/mlang-lsp.exe",
  "mlang.formatter.path": "C:/path/to/mlang/target/debug/mlang.exe",
  "[mlang]": {
    "editor.defaultFormatter": "mlang.mlang-vscode",
    "editor.formatOnSave": true,
    "editor.formatOnSaveMode": "file"
  }
}
```

If `mlang.server.path` / `mlang.formatter.path` are empty, the extension attempts workspace-local `target/*` binaries and then falls back to PATH.

## Extension Settings

- `mlang.server.path`
- `mlang.formatter.path`
- `mlang.formatter.args`
- `mlang.formatter.enabled` (default `true`)
- `mlang.trace.server` (`off`, `messages`, `verbose`)

## Commands

- `M-Lang: Format Document` (`mlang.formatDocument`)
- `M-Lang: Restart Language Server` (`mlang.restartServer`)

## Formatting Behavior

Formatting is handled as:

1. Try LSP `textDocument/formatting` if server is running.
2. Fallback to CLI command `<mlang> fmt <temp_file>` when LSP formatting fails/unavailable.

This supports stable format-on-save even when the language server is unavailable.

## Packaging as VSIX

```bash
npm install -g @vscode/vsce
vsce package
```

Install the generated `.vsix` via VS Code command:
`Extensions: Install from VSIX...`

## Troubleshooting

- LSP fails to start:
  - Ensure `mlang-lsp` exists at configured path or is available on PATH.
  - Rebuild: `cargo build --release --bin mlang-lsp`
- Formatting does nothing:
  - Verify `mlang.formatter.enabled` is `true`.
  - Verify `mlang` CLI path (`mlang.formatter.path`) or PATH fallback.
- View logs:
  - Open output panel: `M-Lang Language Server`.
