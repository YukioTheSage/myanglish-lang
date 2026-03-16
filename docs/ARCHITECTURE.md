# M-Lang Architecture

This document describes the current compiler/runtime architecture for `mlang` v0.1.0.

## 1. End-to-End Flow

```text
.ml source
  -> lexer
  -> parser (AST)
  -> type checker
  -> codegen_go (default) or codegen (legacy C)
  -> go build / gcc
  -> native executable
```

`src/main.rs` handles CLI parsing (`build`, `run`, `fmt`) and orchestrates this flow.

## 2. Front-End (Lexer + Parser)

### Lexer (`src/lexer.rs`)

- Unicode-aware identifiers (including Myanmar script characters).
- ASCII and Myanmar numerals (`၀`..`၉`) for integers.
- Float literal scanning (`da_tha` ecosystem) via `123.45`.
- Keywords include:
  - Core: `kain`, `sar`, `sit`, `da_tha`, `amhar`, `bhala`
  - Control: `hlyin`, `mo`, `pat`, `htae`, `yut`, `shar`, `kyoe`, `naut_sone`
  - Declarations: `loke`, `pone`, `nee`, `myat`, `yu`, `atote`, `pay`
  - Concurrency type keyword: `laung`
- Emits comment tokens (`//...`) for formatter tooling.

### Parser (`src/parser.rs`)

Recursive-descent parser with Pratt expression parsing.

Key statement forms:

- Typed declarations: `Type name = expr;`
- Destructured declarations: `Type a, Type b = expr;`
- Assignment targets:
  - variable (`x = ...`)
  - field (`obj.field = ...`)
  - index (`arr[i] = ...`, `map[k] = ...`)
- Loops:
  - while: `pat (cond) { ... }`
  - classic for: `pat (init; cond; post) { ... }`
  - for-in: `pat item htae xs { ... }`
  - for-in with index: `pat (idx, item) htae xs { ... }`
- Function / struct / method / interface declarations.
- Concurrency statements:
  - `kyoe fn_call();`
  - `naut_sone fn_call();`

Key expression forms:

- Literals: int/float/string/bool/nil
- Binary ops and comparisons
- Function calls and method calls
- Struct literals (`Cart { ... }`) and field access
- Index + slice (`a[i]`, `a[low:high]`, `a[:high]`, `a[low:]`)
- Closures
- Error construction (`amhar("msg")`)
- Type conversion helpers (`pyaung_kain`, `pyaung_sar`, `pyaung_da_tha`)
- Channel constructor expression: `laung<T>()` / `laung<T>(capacity)`

Type grammar supports:

- Primitive types + `amhar`
- Arrays/maps
- Channels
- Tuple types: `(kain, amhar)`
- Function types: `loke(kain, sar) -> sit`
- Qualified struct-like names: `http.Response`

## 3. Core AST (`src/ast.rs`)

### Type Nodes

- `Kain`, `Sar`, `Sit`, `DaTha`, `Nil`, `Error`
- `Array`, `Map`
- `Channel`
- `Struct`, `Interface`
- `Tuple`
- `Function { params, return_type }`

### Statement Nodes

- `Let`, `LetDestructured`
- `Assign`, `FieldAssign`, `IndexAssign`
- `If`, `While`, `ForIn`, `ForClassic`, `Break`, `Continue`
- `Go`, `Defer`
- `FunctionDecl`, `MethodDecl`, `StructDecl`, `InterfaceDecl`
- `Return`, `Print`, `Import`, `ExpressionStatement`

### Expression Nodes

- Literals + identifiers
- `Binary`, `FunctionCall`, `MethodCall`, `FieldAccess`
- `ArrayLiteral`, `HashLiteral`, `IndexExpression`, `SliceExpression`
- `TypeConversion`, `StructLiteral`, `ClosureLiteral`
- `ErrorCreate`, `TupleLiteral`, `ReadInput`
- `ChannelMake`

## 4. Type Checker (`src/typecheck.rs`)

The type checker uses scoped symbol environments and registries:

- `struct_registry`: struct fields
- `method_registry`: methods + module functions
- `interface_registry`: interface declarations

Behavior highlights:

- Enforces declaration/assignment type compatibility.
- Supports `nil` assignability (`bhala`) in typed flows.
- Validates tuple destructuring arity/types.
- Validates break/continue usage with loop depth tracking.
- Validates for-in source is array type.
- Registers supported stdlib imports from `src/stdlib.rs`:
  - `kainn/http`
  - `kainn`
  - `json`
  - `file`
  - `su_nit`
  - `pone_set`
  - `in_ote`
  - `hmat`
- Validates built-in helpers:
  - functions: `htae`, `ashay`
  - methods: `push`, `remove`, `len`, `khwae`, `swal`
- Validates Phase 3 semantics:
  - channel type/method assignability (`send/recv/close`)
  - `kyoe`/`naut_sone` call-only operand checks
  - callable-scope enforcement for `naut_sone`

## 5. Code Generation

### Go Backend (`src/codegen_go.rs`) - Primary

- Emits idiomatic Go (`package main`, managed imports).
- Type mapping:
  - `kain -> int64`
  - `sar -> string`
  - `sit -> bool`
  - `da_tha -> float64`
  - `amhar -> error`
  - tuples -> Go multiple returns
- Handles:
  - structs, methods, interfaces
  - closures (`func(...) ... { ... }`)
  - tuple returns + destructured assignment
  - `kyoe`/`naut_sone` lowering (`go`/`defer`)
  - channel lowering (`make(chan ...)`, send/recv/close)
  - slice expressions
  - array/map mutation
  - `yut`/`shar` as `break`/`continue`
- Built-in lowering:
  - `htae(arr, x)` -> `append(arr, x)`
  - `ashay(v)` -> `int64(len(v))`
  - `arr.push(x)` / `arr.remove(i)` rewrites
- `text.khwae(sep)` -> `strings.Split`
- `text.swal(sub)` -> `strings.Contains`
- `text.ayaik()` -> `strings.ToLower`
- Stdlib import lowering:
  - `http.get/post/handle/listen`, `json.encode/decode`, `file.read/write`, `su_nit.env/args`
  - `pone_set.pon_san`
  - `in_ote.twin_phat/htote_yay`
  - `hmat.mhat_chet/mhat_thati/mhat_amhar`
  - `kainn.tcp_listen/tcp_dial/udp_bind`
  - stdlib struct methods (`http.Request`, `http.ResponseWriter`, TCP/UDP wrappers)
  - helper functions are emitted only when modules are imported.

### C Backend (`src/codegen.rs`) - Legacy

- Retained for backward compatibility and basic programs.
- Supports older subset well (core scalar types, while/for-in, basic arrays, print/input).
- Newer language constructs are not fully emitted (many nodes are no-op in C backend match arms).
- CLI guard rejects Phase 3 constructs on `--target c` with explicit diagnostics.
- Stdlib modules are blocked by CLI on C target.

## 6. Formatter (`src/formatter.rs`)

`format_source` pipeline:

1. Parse source to AST (fail if syntax invalid).
2. Tokenize with comments to preserve comment/digit style.
3. Pretty-print statements/expressions/types from AST.

Current formatter behavior:

- Canonical import style: always quoted (`yu "json";`).
- 4-space indentation + single trailing newline.
- Formats modern syntax including tuple types/returns, closures, struct/method/interface declarations, slices, and destructured declarations.
- Preserves Myanmar-vs-ASCII number style where source tokens allow.

## 7. LSP (`src/lsp/*`)

- Full-document sync analysis cache.
- Diagnostics from parser/typechecker.
- Hover/completion/definition/semantic tokens.
- Document formatting by calling `formatter::format_source`.

See `docs/LSP.md` for editor integration details.

## 8. Module Map

```text
main.rs
  -> lexer.rs -> token.rs
  -> module_loader.rs
  -> parser.rs -> ast.rs
  -> typecheck.rs -> ast.rs + stdlib.rs
  -> codegen_go.rs / codegen.rs
  -> formatter.rs (fmt command)

lsp/main.rs
  -> lsp/analysis.rs (reuses lexer/parser/typecheck)
  -> lsp/hover.rs
  -> lsp/completion.rs
  -> lsp/semantic_tokens.rs
```

## 9. Validation

`cargo test` currently covers lexer, parser, type checker, formatter, Go codegen, C codegen, and stdlib usage paths.
