# M-Lang (mlang)

M-Lang is a statically typed language using Myanglish keywords. The compiler is written in Rust and targets Go by default (`--target go`), with a legacy C backend (`--target c`).

Current crate version: `0.1.0`.

## What Is Implemented

- Lexer/parser/typechecker/codegen pipeline with Myanmar identifier + numeral support.
- Core types: `kain`, `sar`, `sit`, `da_tha`, `amhar`, `bhala`.
- Composite/function types: `su<T>`, `twe<K, V>`, tuples, function types.
- Control flow: `hlyin`/`mo`, `pat` while loops, `pat ... htae ...` for-in loops (optional index), `yut`, `shar`.
- Functions and closures (`loke(...) -> ... { ... }`), including function-typed parameters.
- Structs/methods/interfaces (`pone`, `nee`, `myat`) and struct literals/field access.
- Destructured declarations for tuple returns (`kain x, amhar err = ...;`).
- Built-ins:
  - Functions: `htae(arr, item)`, `ashay(value)`.
  - Methods: `arr.push(x)`, `arr.remove(i)`, `arr.len()`, `text.khwae(sep)`, `text.swal(sub)`, `text.ashay()`.
  - Type conversion helpers: `pyaung_kain(...)`, `pyaung_sar(...)`, `pyaung_da_tha(...)`.
- Go stdlib module shims: `"kainn/http"`, `"json"`, `"file"`, `"su_nit"`, `"pone_set"`, `"in_ote"`, `"hmat"`.
- CLI formatter (`mlang fmt`) and LSP server (`mlang-lsp`).

## Quick Start

### Requirements

- Rust toolchain (edition 2024 crate)
- Go (for default backend)
- GCC (only if using `--target c`)

### Build + Run

```bash
cargo build
cargo run -- build hello.ml
cargo run -- run hello.ml
```

### CLI

```text
mlang build <file.ml>
mlang build --target go <file.ml>
mlang build --target c <file.ml>
mlang run <file.ml>
mlang run --target c <file.ml>
mlang fmt <file.ml>
mlang fmt --check <file.ml>
```

### Formatter

```bash
cargo run --bin mlang -- fmt examples/phase1/03_error_handling_and_closure.ml
cargo run --bin mlang -- fmt --check examples/phase2/06_http_json_client.ml
```

Notes:

- Canonicalizes imports to quoted form (`yu "json";`).
- Preserves Myanmar digits when possible.
- Supports new syntax including structs/methods/interfaces, tuple returns, closures, and slices.

## Language Snapshot

### Keywords

| Keyword | Meaning |
| --- | --- |
| `kain`, `sar`, `sit`, `da_tha`, `amhar` | Primitive types |
| `hman`, `hmar`, `bhala` | `true`, `false`, `nil` |
| `hlyin`, `mo` | `if`, `else` |
| `pat`, `htae`, `yut`, `shar` | loops, for-in separator, break, continue |
| `loke`, `pyan` | function, return |
| `pya`, `phat` | print, read input |
| `su`, `twe` | array, map types |
| `yu` | import |
| `pone`, `nee`, `myat` | struct, method, interface |

### Common Forms

```ml
// tuple-return + destructuring
loke safe_div(kain a, kain b) -> (kain, amhar) {
    hlyin (b == 0) {
        pyan (0, amhar("division by zero"));
    }
    pyan (a / b, bhala);
}

loke main() -> kain {
    kain q, amhar err = safe_div(10, 2);
    hlyin (err != bhala) {
        pya(err);
        pyan 1;
    }
    pya(q);
    pyan 0;
}
```

```ml
// struct + field mutation + array for-in with index
pone Cart {
    sar customer;
    kain item_count;
}

loke main() -> kain {
    Cart c = Cart { customer: "Aye Aye", item_count: 1 };
    c.item_count = 2;

    su<kain> costs = [1200, 2500, 3900];
    kain total = 0;
    pat (idx, item) htae costs {
        hlyin (idx == 0) { shar; }
        total = total + item;
    }
    pya(total);
    pyan 0;
}
```

### Imports + Stdlib

Both forms parse, formatter outputs quoted style:

```ml
yu "json";
yu json;
```

Available stdlib modules (Go backend):

- `"kainn/http"` as `http`
- `"json"`
- `"file"`
- `"su_nit"`
- `"pone_set"` (`pon_san(sar format, sar value) -> sar`)
- `"in_ote"` (`twin_phat() -> (sar, amhar)`, `htote_yay(sar text) -> amhar`)
- `"hmat"` (`mhat_chet/mhat_thati/mhat_amhar`)

## Backend Status

| Area | Go backend (`--target go`) | C backend (`--target c`) |
| --- | --- | --- |
| Core language (phase-1/2 syntax) | Supported | Partial |
| Struct/method/interface/closure/tuple/error features | Supported | Not fully supported |
| Stdlib modules (`http/json/file/su_nit/pone_set/in_ote/hmat`) | Supported | Rejected with error |
| HashMap behavior | Native Go map support | Placeholder runtime only |

The C backend is intentionally legacy and does not implement the full modern surface.

## Tooling

### Tests

```bash
cargo test
```

Current suite covers lexer, parser, typechecker, formatter, Go codegen, and legacy C codegen.

### LSP

Build the language server:

```bash
cargo build --release --bin mlang-lsp
```

See `docs/LSP.md` for VS Code setup and formatter/LSP settings.

## Repository Layout

```text
mlang/
  src/
    main.rs          # CLI (build/run/fmt)
    lexer.rs         # tokenizer
    parser.rs        # recursive descent + Pratt parser
    typecheck.rs     # semantic checks and type rules
    codegen_go.rs    # primary backend
    codegen.rs       # legacy C backend
    formatter.rs     # source formatter
    stdlib.rs        # stdlib module signatures
    lsp/             # language server implementation
  examples/
    phase1/
    phase2/
  docs/
```

## Known Gaps

- C backend support lags behind Go backend capabilities.
- Interface conformance checks are currently minimal (declaration-oriented).
- Unknown imports are treated as plain identifiers until used; only built-in stdlib modules are pre-registered.

## Additional Docs

- [docs/CHEATSHEET.md](docs/CHEATSHEET.md)
- [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- [docs/LSP.md](docs/LSP.md)
- [editors/vscode/README.md](editors/vscode/README.md)
- [docs/SERVER_PIVOT_ROADMAP.md](docs/SERVER_PIVOT_ROADMAP.md)

## License

Educational / project-use repository. Add or update explicit license text as needed.
