# M-Lang Language Server (mlang-lsp)

A full-featured language server for M-Lang (Myanmar Language), similar to `gopls` for Go.

## Features

| Feature              | Description                                            |
| -------------------- | ------------------------------------------------------ |
| **Diagnostics**      | Real-time syntax and type errors as you type           |
| **Hover**            | Hover over keywords/variables for docs and type info   |
| **Go to Definition** | Jump to variable/function declarations                 |
| **Completion**       | Auto-complete keywords, snippets, variables, functions |
| **Semantic Tokens**  | Rich syntax highlighting via LSP semantic tokens       |

## Building

```bash
# From the mlang project root:
cargo build --release --bin mlang-lsp

# The binary will be at:
# target/release/mlang-lsp.exe  (Windows)
# target/release/mlang-lsp      (Linux/macOS)
```

## VS Code Extension Setup

### Quick Setup (Development)

1. **Build the language server:**

   ```bash
   cargo build --release --bin mlang-lsp
   ```

2. **Install extension dependencies:**

   ```bash
   cd editors/vscode
   npm install
   npm run compile
   ```

3. **Add `mlang-lsp` to your PATH** or set the path in VS Code settings:

   ```json
   {
     "mlang.server.path": "C:/path/to/mlang/target/release/mlang-lsp.exe"
   }
   ```

4. **Launch extension in dev mode:**
   - Open `editors/vscode` in VS Code
   - Press `F5` to launch the Extension Development Host
   - Open any `.ml` file — the language server starts automatically

### Enable Format on Save (Recommended)

Add this to your workspace `.vscode/settings.json`:

```json
{
  "[mlang]": {
    "editor.defaultFormatter": "mlang.mlang-vscode",
    "editor.formatOnSave": true
  }
}
```

Optional executable overrides:

```json
{
  "mlang.server.path": "C:/path/to/mlang/target/release/mlang-lsp.exe",
  "mlang.formatter.path": "C:/path/to/mlang/target/debug/mlang.exe"
}
```

### Install as VSIX

```bash
cd editors/vscode
npm install -g @vscode/vsce
vsce package
# Install the generated .vsix file in VS Code
```

## Architecture

```
mlang-lsp (binary)
  ├── main.rs          — Tower-LSP server setup, request routing
  ├── analysis.rs      — Lexer → Parser → TypeChecker pipeline, symbol collection
  ├── hover.rs         — Hover documentation for keywords & symbols
  ├── completion.rs    — Auto-completion items (keywords, snippets, symbols)
  └── semantic_tokens.rs — Token-level semantic highlighting
```

The server reuses the core `mlang` library (lexer, parser, type-checker) as a Rust crate.
On every document change, it re-runs the full analysis pipeline and publishes diagnostics.

## Supported LSP Methods

| Method                             | Status |
| ---------------------------------- | ------ |
| `initialize`                       | ✅     |
| `textDocument/didOpen`             | ✅     |
| `textDocument/didChange`           | ✅     |
| `textDocument/didClose`            | ✅     |
| `textDocument/publishDiagnostics`  | ✅     |
| `textDocument/hover`               | ✅     |
| `textDocument/completion`          | ✅     |
| `textDocument/definition`          | ✅     |
| `textDocument/semanticTokens/full` | ✅     |
| `shutdown`                         | ✅     |

## Example

Open `hello.ml` in VS Code with the extension active:

```
loke main() -> kain {
    kain age = 20;
    sar name = "Aung Aung";

    hlyin (age > 18) {
        pya("Hello World! ");
        pya(name);
    } mo {
        pya("Too young!");
    }

    pyan 0;
}
```

You'll get:

- **Syntax highlighting** via semantic tokens and TextMate grammar
- **Red squiggles** under any syntax or type errors
- **Hover** on `kain` shows "Integer type declaration"
- **Hover** on `age` shows "variable `age` — Type: `kain (int)`"
- **Auto-complete** for all M-Lang keywords with snippets
- **Go to definition** on variable references jumps to their declaration
