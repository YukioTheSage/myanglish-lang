# M-Lang (mlang) — Myanmar Language Compiler

**M-Lang** is a statically-typed programming language that uses **Myanglish** (romanized Burmese) keywords. It transpiles to **Go** via a multi-stage pipeline written in **Rust**, then invokes `go build` to produce a native executable. The language supports integers, strings, booleans, arrays, hashmaps, functions, control flow, and user I/O — all expressed with Myanglish keywords. A legacy C backend (`gcc`) is also available via `--target c`.

---

## Table of Contents

- [Features](#features)
- [Quick Start](#quick-start)
- [Language Reference](#language-reference)
  - [Keywords](#keywords)
  - [Types](#types)
  - [Variables](#variables)
  - [Functions](#functions)
  - [Control Flow](#control-flow)
  - [Loops](#loops)
  - [Arrays](#arrays)
  - [HashMaps](#hashmaps)
  - [Printing](#printing)
  - [Reading Input](#reading-input)
  - [Imports](#imports)
  - [Comments](#comments)
  - [Operators](#operators)
  - [Myanmar Numerals](#myanmar-numerals)
- [Example Programs](#example-programs)
- [Architecture](#architecture)
- [Building from Source](#building-from-source)
- [Project Structure](#project-structure)
- [Contributing](#contributing)
- [License](#license)

---

## Features

- **Myanglish keywords** — Write code with romanized Burmese keywords (`kain`, `sar`, `loke`, `pya`, etc.)
- **Static type system** — Type-checked at compile time with clear error messages
- **Compiles to native code** — Transpiles to Go (default) or C, then compiled to fast native executables
- **Myanmar numeral support** — Use `၀`–`၉` (U+1040–U+1049) or ASCII `0`–`9`
- **Rich data structures** — Arrays (`su`) and HashMaps (`twe`) with generic type parameters
- **First-class functions** — Named functions with typed parameters and return types
- **String concatenation** — Using `+` on strings (native in Go, `mlang_concat()` in C backend)
- **User input** — `phat("prompt")` reads a line from stdin
- **Dual backend** — Go backend (default) with goroutine-ready runtime, or C backend for minimal output
- **Line comments** — `//` single-line comments

---

## Quick Start

### Prerequisites

| Tool                           | Purpose                                |
| ------------------------------ | -------------------------------------- |
| **Rust** (2024 edition)        | Build the compiler                     |
| **Go** (1.21+)                 | Compile generated Go code (default)    |
| **gcc** (MinGW-w64 on Windows) | Only needed for `--target c` C backend |

### Install & Run

```bash
# Clone the repository
git clone <repo-url>
cd mlang

# Build the compiler
cargo build --release

# Compile a .ml source file to an executable
cargo run -- build hello.ml

# Compile and immediately run
cargo run -- run hello.ml
```

### CLI Usage

```
M-Lang (Myanmar Language) Compiler
Usage:
  mlang build <file.ml>                  # Build (default: Go backend)
  mlang build --target go <file.ml>      # Build with Go backend (explicit)
  mlang build --target c <file.ml>       # Build with legacy C backend
  mlang run <file.ml>                    # Build and run
  mlang run --target c <file.ml>         # Build and run with C backend
  mlang fmt <file.ml>                    # Format source file in place
  mlang fmt --check <file.ml>            # Check formatting without writing
```

### Code Formatting

Use the formatter to normalize spacing, indentation, line breaks, and expression layout:

```bash
# Format file in place
cargo run --bin mlang -- fmt hello.ml

# CI-friendly check mode (non-zero exit code if reformat is needed)
cargo run --bin mlang -- fmt --check hello.ml
```

Formatter guarantees:

- Stable 4-space indentation and trailing newline
- Readable line-wrapping for long parameter lists and collection literals
- Operator-precedence-safe expression formatting (keeps required parentheses)

#### VS Code Format on Save

With the M-Lang VS Code extension installed, add this to your workspace `.vscode/settings.json`:

```json
{
  "[mlang]": {
    "editor.defaultFormatter": "mlang.mlang-vscode",
    "editor.formatOnSave": true
  }
}
```

Optional formatter path override (if `mlang` is not on `PATH`):

```json
{
  "mlang.formatter.path": "C:/path/to/mlang/target/debug/mlang.exe"
}
```

The `build` command:

1. Lexes & parses the `.ml` file
2. Type-checks the AST
3. Generates an intermediate `.go` file (or `.c` with `--target c`)
4. Invokes `go build` (or `gcc` with `--target c`) to produce a `.exe` / binary
5. Reports success or errors at each stage

The `run` command does everything `build` does, then executes the resulting binary.

---

## Language Reference

### Keywords

| Keyword | Burmese Origin | Meaning  | Usage                |
| ------- | -------------- | -------- | -------------------- |
| `kain`  | ကိန်း          | integer  | Type declaration     |
| `sar`   | စာ             | string   | Type declaration     |
| `sit`   | စစ်            | boolean  | Type declaration     |
| `hman`  | မှန်           | true     | Boolean literal      |
| `hmar`  | မှား           | false    | Boolean literal      |
| `hlyin` | လျှင်          | if       | Conditional          |
| `mo`    | မို့           | else     | Conditional          |
| `pat`   | ပတ်            | while    | Loop                 |
| `loke`  | လုပ်           | function | Function declaration |
| `pyan`  | ပြန်           | return   | Return statement     |
| `pya`   | ပြ             | print    | Print statement      |
| `phat`  | ဖတ်            | read     | Read input           |
| `su`    | စု             | array    | Array type           |
| `yu`    | ယူ             | import   | Module import        |
| `twe`   | တွဲ            | hashmap  | HashMap type         |

### Types

M-Lang has five types:

| Type    | Keyword     | Go Equivalent | C Equivalent | Description                    |
| ------- | ----------- | ------------- | ------------ | ------------------------------ |
| Integer | `kain`      | `int64`       | `long long`  | 64-bit signed integer          |
| String  | `sar`       | `string`      | `char*`      | UTF-8 string                   |
| Boolean | `sit`       | `bool`        | `bool`       | `hman` (true) / `hmar` (false) |
| Array   | `su<T>`     | `[]T`         | `T*`         | Homogeneous collection         |
| HashMap | `twe<K, V>` | `map[K]V`     | _(limited)_  | Key-value map                  |

### Variables

Variables are declared with a type annotation followed by a name, `=`, and an initial value:

```
kain age = 20;            // int age = 20;
sar name = "Aung Aung";  // string name = "Aung Aung";
sit flag = hman;          // bool flag = true;
```

Reassignment (without re-declaring the type):

```
age = 21;
```

### Functions

Functions are declared with `loke`, take typed parameters, specify a return type after `->`, and have a block body:

```
loke main() -> kain {
    pyan 0;
}
```

**`main`** is the special entry-point function name. It is translated to `func main()` in generated Go (or `int main()` in C). Return statements in `main` are omitted in Go output since Go's `main` has no return value.

Functions with parameters:

```
loke add(kain a, kain b) -> kain {
    pyan a + b;
}
```

Calling a function:

```
kain result = add(10, 20);
```

### Control Flow

**If / Else:**

```
hlyin (age > 18) {
    pya("adult");
} mo {
    pya("child");
}
```

- Condition must evaluate to type `sit` (boolean).
- The `mo` (else) block is optional.

### Loops

**While loop:**

```
pat (count > 0) {
    pya(count);
    count = count - 1;
}
```

- Condition must evaluate to type `sit` (boolean).

**For-in loop:**

```
su<kain> numbers = [1, 2, 3];
pat item htae numbers {
  pya(item);
}
```

- Collection must be an array (`su<...>`).

### Arrays

Arrays are declared with `su<T>` and initialized with bracket syntax:

```
su<kain> numbers = [100, 200, 300];
```

Indexing (0-based):

```
kain first = numbers[0];
```

- All elements must be the same type.
- Index must be type `kain` (integer).

### HashMaps

HashMaps are declared with `twe<K, V>`:

```
twe<sar, kain> dict = {"a": 1, "b": 2};
```

Indexing:

```
kain value = dict["a"];
```

> **Note:** The Go backend supports full HashMap functionality via Go's native `map` type. The legacy C backend has limited HashMap support.

### Printing

Use `pya(expression)` to print any expression to stdout (with a newline):

```
pya("Hello World!");
pya(age);
pya(name);
```

The Go backend uses `fmt.Println()` which handles all types automatically. The C backend uses `printf` with type-appropriate format specifiers.

### Reading Input

Use `phat("prompt")` to read a line of text from stdin:

```
sar name = phat("Enter your name: ");
```

- The prompt argument must be a string (`sar`).
- Always returns a string (`sar`).

### Imports

Use `yu` to import a module:

```
yu "json";
yu json;
```

Both quoted and legacy forms are accepted. The formatter canonicalizes imports to the quoted style.

Built-in stdlib modules (Go backend):

- `"kainn/http"` (alias `http`)
- `"json"`
- `"file"`
- `"su_nit"`

For these stdlib modules, use the Go backend (`--target go`). The C backend rejects them with a clear error.

Example (`file` module):

```
yu "file";

loke main() -> kain {
    amhar write_err = file.write("output.txt", "Hello!");
    hlyin (write_err != bhala) {
        pya(write_err);
        pyan 1;
    }

    sar content, amhar read_err = file.read("output.txt");
    hlyin (read_err != bhala) {
        pya(read_err);
        pyan 1;
    }

    pya(content);
    pyan 0;
}
```

### Comments

Single-line comments start with `//`:

```
// this is a comment
kain x = 10; // inline comment
```

### Operators

**Arithmetic** (operate on `kain`):

| Operator | Description                                  |
| -------- | -------------------------------------------- |
| `+`      | Addition (or string concatenation for `sar`) |
| `-`      | Subtraction                                  |
| `*`      | Multiplication                               |
| `/`      | Division                                     |

**Comparison** (return `sit`):

| Operator | Description      |
| -------- | ---------------- |
| `==`     | Equal            |
| `!=`     | Not equal        |
| `>`      | Greater than     |
| `<`      | Less than        |
| `>=`     | Greater or equal |
| `<=`     | Less or equal    |

**Operator precedence** (lowest to highest):

1. `==`, `!=`
2. `<`, `>`, `<=`, `>=`
3. `+`, `-`
4. `*`, `/`
5. Function calls, index access

### Myanmar Numerals

M-Lang supports **Myanmar (Burmese) digits** alongside ASCII digits:

| Myanmar | ASCII | Value |
| ------- | ----- | ----- |
| ၀       | 0     | 0     |
| ၁       | 1     | 1     |
| ၂       | 2     | 2     |
| ၃       | 3     | 3     |
| ၄       | 4     | 4     |
| ၅       | 5     | 5     |
| ၆       | 6     | 6     |
| ၇       | 7     | 7     |
| ၈       | 8     | 8     |
| ၉       | 9     | 9     |

Both forms can be freely mixed: `kain x = ၂0;` is valid and equals `20`.

---

## Example Programs

### Hello World (`hello.ml`)

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

### String Concatenation & Arrays (`hello2.ml`)

```
loke main() -> kain {
    sar name = "Mingalabar, ";
    sar friend = "Aung Aung!";
    sar greeting = name + friend;
    pya(greeting);

    su<kain> numbers = [100, 200, 300];
    kain first = numbers[0];
    kain second = numbers[1];
    kain third = numbers[2];

    pya("Array Elements: ");
    pya(first);
    pya(second);
    pya(third);

    pyan 0;
}
```

---

## Architecture

M-Lang follows a classic **multi-pass compiler** pipeline, transpiling to **Go** (default) or **C**:

```
  .ml source
      │
      ▼
  ┌──────────┐    Lexer tokenizes Myanglish keywords + ASCII
  │  Lexer   │    into a stream of Tokens with line/column info
  └────┬─────┘
       │ Token stream
       ▼
  ┌──────────┐    Recursive-descent (Pratt) parser builds
  │  Parser  │    an Abstract Syntax Tree (AST)
  └────┬─────┘
       │ AST (Program)
       ▼
  ┌────────────┐  Static type checker validates types,
  │ TypeChecker│  scoping, and produces error messages
  └────┬───────┘
       │ Validated AST
       ├───────────────────────────┐
       ▼                           ▼
  ┌────────────┐            ┌────────────┐
  │ CodeGenGo  │ (default)  │  CodeGenC  │ (--target c)
  │ codegen_go │            │  codegen   │
  └────┬───────┘            └────┬───────┘
       │ .go file                │ .c file
       ▼                         ▼
  ┌──────────┐              ┌──────────┐
  │ go build │              │   gcc    │
  └────┬─────┘              └────┬─────┘
       │                         │
       ▼                         ▼
   Native .exe               Native .exe
```

### Pipeline Stages

#### 1. Lexer (`src/lexer.rs`)

- Converts raw source text into a stream of `Token`s.
- Recognizes **Myanmar Unicode** characters (U+1000–U+109F) as valid identifier characters.
- Recognizes **Myanmar digits** (U+1040–U+1049) as numeric literals.
- Maps Myanglish keywords (e.g. `kain`, `loke`) to dedicated `TokenKind` variants.
- Supports single-line comments (`//`), string literals (`"..."`), and all comparison/arithmetic operators.
- Tracks **line** and **column** positions for error reporting.

#### 2. Parser (`src/parser.rs`)

- **Recursive-descent** parser with **Pratt precedence climbing** for expressions.
- Produces a fully-typed AST defined in `src/ast.rs`.
- Parses:
  - Variable declarations (`Let`) with type annotations
  - Variable assignments (`Assign`)
  - Function declarations (`FunctionDecl`) with typed params and return type
  - `If`/`Else`, `While` blocks
  - `Return`, `Print`, `Import` statements
  - Expressions: binary ops, function calls, array/hash literals, index access, read input
- Collects parse errors in `parser.errors` without halting (error recovery).

#### 3. Type Checker (`src/typecheck.rs`)

- Walks the AST and enforces **static typing** rules.
- Uses a **scoped environment** (`Environment`) with `outer` pointers for lexical scoping.
- Checks:
  - Type consistency in variable declarations and assignments
  - Undeclared variables and functions
  - Boolean conditions in `if`/`while`
  - Function call arity and argument types
  - Array element homogeneity and integer indexing
  - HashMap key/value type consistency
  - String concatenation validity (`+` on two `sar` values)
  - Return type correctness within functions

#### 4. Code Generator — Go Backend (`src/codegen_go.rs`) _(Default)_

- Traverses the validated AST and emits **Go source code**.
- Emits `package main` with automatic `import` management (`"fmt"`, `"bufio"`, `"os"` as needed).
- **Unicode identifiers** are used directly — Go supports Unicode in identifiers natively, so no hex-encoding is needed.
- String concatenation uses Go's native `+` operator.
- Arrays use Go slices (`[]int64{...}`), HashMaps use Go maps (`map[string]int64{...}`).
- While loops compile to `for condition { ... }` (Go has no `while` keyword).
- For-in loops compile to `for _, item := range collection { ... }`.
- Emits `_ = varname` after declarations to satisfy Go's unused-variable rule.
- The function named `main` is mapped to `func main()` with no return value (return statements are omitted).
- Emits a `mlangReadInput()` helper function when `phat()` is used.

#### 4b. Code Generator — C Backend (`src/codegen.rs`) _(Legacy, `--target c`)_

- Traverses the validated AST and emits **C source code**.
- Automatically includes standard C headers (`stdio.h`, `stdbool.h`, `string.h`, `stdlib.h`).
- Emits two runtime helper functions:
  - `mlang_concat()` — heap-allocated string concatenation
  - `mlang_read_input()` — buffered stdin reader with prompt
- **Identifier mangling**: Non-ASCII identifiers are converted to `mlang_` + hex-encoded UTF-8 bytes for C compatibility.
- The function named `main` is mapped to `int main()`.
- Selects correct `printf` format specifiers based on expression types.
- HashMap support is limited (outputs `NULL` placeholder).

#### 5. Native Compilation

- **Go backend (default):** The generated `.go` file is compiled with `go build` to produce a native executable. Requires Go 1.21+ in `PATH`.
- **C backend (`--target c`):** The generated `.c` file is compiled with `gcc`. On Windows, MinGW-w64 `gcc` is expected in the `PATH`.

---

## Building from Source

```bash
# Requires Rust 2024 edition (nightly or appropriate stable version)
cargo build

# Run tests
cargo test

# Build optimized release
cargo build --release
```

The compiler binary is output to `target/debug/mlang` (or `target/release/mlang`).

### Running Tests

The project includes unit tests for the lexer, parser, and code generator:

```bash
cargo test
```

Test coverage includes:

- **Lexer**: Full tokenization of a program with comments, Myanmar numerals, keywords, strings, operators, arrays, hashmaps, and read expressions.
- **Parser**: `Let` statements, function declarations with parameters, array/hashmap literals, index expressions, and read input.
- **Go CodeGen**: End-to-end generation from M-Lang source to Go code, verifying `func main()`, `fmt.Println` calls, string concatenation, for-range loops, hashmaps, if/elif/else, read input, and Unicode identifiers.
- **C CodeGen**: End-to-end generation from M-Lang source to C code, verifying `int main()` and `printf` calls.

---

## Project Structure

```
mlang/
├── Cargo.toml          # Rust package manifest (edition 2024)
├── README.md           # This documentation
├── hello.ml            # Example: Hello World with if/else
├── hello2.ml           # Example: String concat + arrays
└── src/
    ├── main.rs         # CLI entry point (build/run/fmt commands, --target flag)
    ├── lib.rs          # Module exports
    ├── token.rs        # Token and TokenKind definitions
    ├── lexer.rs        # Lexer — source text → token stream
    ├── ast.rs          # AST node definitions (Expression, Statement, Type, Program)
    ├── parser.rs       # Recursive-descent Pratt parser → AST
    ├── typecheck.rs    # Static type checker with scoped environments
    ├── codegen_go.rs   # Go code generator (default backend)
    ├── codegen.rs      # C code generator (legacy backend, --target c)
    └── formatter.rs    # Source code formatter (mlang fmt)
```

Generated `.go`, `.c`, and `.exe` outputs are local build artifacts and are ignored by default.

### Module Dependency Graph

```
main.rs
  ├── lexer.rs     ← token.rs
  ├── parser.rs    ← lexer.rs, ast.rs, token.rs
  ├── typecheck.rs ← ast.rs
  ├── codegen_go.rs ← ast.rs    (default)
  ├── codegen.rs    ← ast.rs    (--target c)
  └── formatter.rs
```

---

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Make your changes and add tests
4. Run `cargo test` to ensure all tests pass
5. Submit a pull request

### Areas for Improvement

- **Structs / custom types**: Support for user-defined data structures (`pone`)
- **Methods on types**: Ability to attach functions to structs (`nee`)
- **Interfaces**: Go-style implicit interfaces for composable abstractions (`pyint`)
- **Error handling**: Error type and multiple return values (`amhar`, `bhone`)
- **Package system**: Real module system beyond basic imports (`atote`)
- **HTTP/networking stdlib**: Built-in server capabilities (`kainn/http`)
- **Concurrency**: Goroutine-style lightweight threads (`kyein`, `laung`)
- **Float type**: Floating-point numbers (`da_thin`)
- **Return type checking**: Function return types are not fully validated against all code paths.
- **Standard library**: Built-in functions for math, string manipulation, etc.

See [docs/SERVER_PIVOT_ROADMAP.md](docs/SERVER_PIVOT_ROADMAP.md) for the full roadmap.

---

## License

This project is provided as-is for educational purposes. See the repository for license details.
