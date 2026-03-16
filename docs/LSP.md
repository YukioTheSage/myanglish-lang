# M-Lang Language Server (`mlang-lsp`)

Current state for crate version `0.1.0`.

VS Code extension-specific notes are in `editors/vscode/README.md`.

## Features

| Feature | Status | Notes |
| --- | --- | --- |
| Diagnostics | Supported | Parse + type errors published on open/change |
| Hover | Supported | Keywords + symbols |
| Completion | Supported | Keywords/snippets/symbols |
| Go to Definition | Supported | Symbol definition lookup |
| Semantic Tokens | Supported | Token-level semantic highlighting |
| Document Formatting | Supported | Uses `mlang` formatter logic |

## Build

```bash
cargo build --release --bin mlang-lsp
```

Binary output:

- Windows: `target/release/mlang-lsp.exe`
- Linux/macOS: `target/release/mlang-lsp`

## VS Code Extension Setup

From `editors/vscode`:

```bash
npm install
npm run compile
```

Recommended workspace settings:

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

If path settings are empty, the extension auto-detects local targets or falls back to PATH binaries.

## Extension Commands

- `M-Lang: Format Document`
- `M-Lang: Restart Language Server`

## Extension Config Keys

- `mlang.server.path`
- `mlang.formatter.path`
- `mlang.formatter.args`
- `mlang.formatter.enabled`
- `mlang.trace.server` (`off`, `messages`, `verbose`)

## LSP Capabilities (`src/lsp/main.rs`)

- `textDocument/didOpen`
- `textDocument/didChange`
- `textDocument/didClose`
- `textDocument/publishDiagnostics`
- `textDocument/hover`
- `textDocument/completion`
- `textDocument/definition`
- `textDocument/semanticTokens/full`
- `textDocument/formatting`

Sync mode: full document (`TextDocumentSyncKind::FULL`).

## Internal Layout

```text
src/lsp/main.rs             # server wiring + request handlers
src/lsp/analysis.rs         # parse/typecheck + symbol extraction
src/lsp/hover.rs            # hover content
src/lsp/completion.rs       # completion items
src/lsp/semantic_tokens.rs  # semantic token emission
```

The server reuses core compiler modules (`lexer`, `parser`, `typecheck`, `formatter`) from the same crate.

## Notes

- Type errors are currently reported without precise source spans (fallback diagnostic range near file start).
- Formatting requests are delegated to `formatter::format_source`; syntax-invalid files will not format until parse errors are resolved.
