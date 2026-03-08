# M-Lang Architecture & Internals

This document provides an in-depth technical reference for each compiler stage in M-Lang.

---

## 1. Token System (`src/token.rs`)

### TokenKind Enum

Every element in the source is classified into one of the `TokenKind` variants:

| Category        | Variants                                                                                                                    | Description                                        |
| --------------- | --------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------- |
| **Keywords**    | `Kain`, `Sar`, `Sit`, `Hman`, `Hmar`, `Hlyin`, `Mo`, `Pat`, `Htae`, `Loke`, `Pyan`, `Pya`, `Phat`, `Su`, `Yu`, `Twe`        | Myanglish keywords mapped to language constructs   |
| **Literals**    | `Identifier(String)`, `Number(i64)`, `StringLiteral(String)`                                                                | User-defined names, numeric values, quoted strings |
| **Operators**   | `Plus`, `Minus`, `Star`, `Slash`, `Assign`, `Equals`, `NotEquals`, `GreaterThan`, `LessThan`, `GreaterEquals`, `LessEquals` | Arithmetic, assignment, and comparison             |
| **Punctuation** | `LParen`, `RParen`, `LBrace`, `RBrace`, `LBracket`, `RBracket`, `Comma`, `Semicolon`, `Colon`, `Arrow`                      | Delimiters and separators                          |
| **Special**     | `Eof`, `Illegal`                                                                                                            | End-of-file marker and unrecognized characters     |

### Token Struct

```rust
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,    // 1-based line number
    pub column: usize,  // 1-based column number
}
```

Tokens carry position information for error reporting throughout the pipeline.

---

## 2. Lexer (`src/lexer.rs`)

### Design

The lexer is a **single-pass, character-by-character** scanner. It maintains:

- `input: Vec<char>` — source text decomposed into Unicode code points
- `position` / `read_position` — current and lookahead cursor indices
- `ch` — current character under the cursor
- `line` / `column` — position tracking for diagnostics

### Character Classification

```
is_letter_or_myanmar(ch) → ch.is_alphabetic() || ch == '_' || U+1000..=U+109F
is_myanmar_digit(ch)     → U+1040..=U+1049
```

Myanmar digits (၀–၉) are converted to their integer values: `ch as i64 - '\u{1040}' as i64`.

### Tokenization Rules

| Input Pattern          | Produced Token                                                    |
| ---------------------- | ----------------------------------------------------------------- |
| Whitespace             | Skipped (newlines update `line`/`column`)                         |
| `//...`                | Comment — consumed until newline, then recursively get next token |
| `"..."`                | `StringLiteral(content)`                                          |
| ASCII/Myanmar digits   | `Number(value)` — Myanmar digits normalized to 0–9                |
| Myanmar/Latin letters  | Identifier or Keyword (via `lookup_ident`)                        |
| `->`                   | `Arrow`                                                           |
| `==`, `!=`, `>=`, `<=` | Two-char comparison operators                                     |
| Single symbols         | Corresponding punctuation/operator token                          |

### Keyword Lookup

The `lookup_ident` function maps Myanglish strings to keyword tokens:

```
"kain" → Kain    "sar" → Sar      "sit" → Sit
"hman" → Hman    "hmar" → Hmar    "hlyin" → Hlyin
"mo" → Mo        "pat" → Pat      "htae" → Htae
"loke" → Loke
"pyan" → Pyan    "pya" → Pya      "phat" → Phat
"su" → Su        "yu" → Yu        "twe" → Twe
```

Anything not matching a keyword becomes `Identifier(name)`.

---

## 3. Abstract Syntax Tree (`src/ast.rs`)

### Type System

```rust
pub enum Type {
    Kain,                          // int
    Sar,                           // string
    Sit,                           // bool
    Array(Box<Type>),              // e.g., Array(Kain) = su<kain>
    Map(Box<Type>, Box<Type>),     // e.g., Map(Sar, Kain) = twe<sar, kain>
}
```

### Expression Nodes

| Variant           | Fields                      | Description                             |
| ----------------- | --------------------------- | --------------------------------------- |
| `IntegerLiteral`  | `i64`                       | Numeric constant                        |
| `StringLiteral`   | `String`                    | String constant                         |
| `BooleanLiteral`  | `bool`                      | `hman` or `hmar`                        |
| `Identifier`      | `String`                    | Variable / function name reference      |
| `Binary`          | `left`, `operator`, `right` | Infix expression (`+`, `-`, `==`, etc.) |
| `FunctionCall`    | `function`, `arguments`     | Function invocation                     |
| `ArrayLiteral`    | `elements`                  | `[expr, expr, ...]`                     |
| `HashLiteral`     | `pairs`                     | `{key: val, key: val}`                  |
| `IndexExpression` | `left`, `index`             | `collection[index]`                     |
| `ReadInput`       | `prompt`                    | `phat("...")` user input                |

### Statement Nodes

| Variant               | Fields                                      | Description                         |
| --------------------- | ------------------------------------------- | ----------------------------------- |
| `Let`                 | `name`, `value`, `ty`                       | Variable declaration with type      |
| `Assign`              | `name`, `value`                             | Variable reassignment               |
| `If`                  | `condition`, `consequence`, `alternative`   | Conditional branch (with elif/else) |
| `While`               | `condition`, `body`                         | While loop                          |
| `ForIn`               | `iterator`, `collection`, `body`            | For-in loop (`pat item htae arr`)   |
| `FunctionDecl`        | `name`, `parameters`, `return_type`, `body` | Function definition                 |
| `Return`              | `value`                                     | Return from function                |
| `Print`               | `value`                                     | Print expression                    |
| `Import`              | `module`                                    | Module import                       |
| `ExpressionStatement` | `Expression`                                | Bare expression as a statement      |

### Program Structure

```rust
pub struct Program {
    pub statements: Vec<Statement>,  // Top-level statements
}

pub struct BlockStatement {
    pub statements: Vec<Statement>,  // Block body (function, if, while)
}
```

---

## 4. Parser (`src/parser.rs`)

### Design

**Recursive-descent** parser with **Pratt precedence climbing** for expressions. Uses a two-token lookahead (`current_token` and `peek_token`).

### Precedence Levels

```
Lowest      = 1  (default)
Equals      = 2  (==, !=)
LessGreater = 3  (<, >, <=, >=)
Sum         = 4  (+, -)
Product     = 5  (*, /)
Call        = 6  (function calls, index access)
```

### Statement Parsing Dispatch

The parser dispatches on the current token's kind:

| Token                             | Parser Method                | Resulting AST                           |
| --------------------------------- | ---------------------------- | --------------------------------------- |
| `Kain`, `Sar`, `Sit`, `Su`, `Twe` | `parse_let_statement`        | `Statement::Let`                        |
| `Hlyin`                           | `parse_if_statement`         | `Statement::If`                         |
| `Pyan`                            | `parse_return_statement`     | `Statement::Return`                     |
| `Pat`                             | `parse_pat_statement`        | `Statement::While` / `Statement::ForIn` |
| `Yu`                              | `parse_import_statement`     | `Statement::Import`                     |
| `Loke`                            | `parse_function_declaration` | `Statement::FunctionDecl`               |
| `Pya`                             | `parse_print_statement`      | `Statement::Print`                      |
| `Identifier` + peek `=`           | `parse_assign_statement`     | `Statement::Assign`                     |
| Other                             | `parse_expression_statement` | `Statement::ExpressionStatement`        |

### Expression Parsing (Pratt)

**Prefix** parse functions (based on current token):

| Token                | Result                       |
| -------------------- | ---------------------------- |
| `Identifier(name)`   | `Expression::Identifier`     |
| `Number(val)`        | `Expression::IntegerLiteral` |
| `StringLiteral(val)` | `Expression::StringLiteral`  |
| `Hman` / `Hmar`      | `Expression::BooleanLiteral` |
| `LParen`             | Grouped expression           |
| `LBracket`           | `Expression::ArrayLiteral`   |
| `LBrace`             | `Expression::HashLiteral`    |
| `Phat`               | `Expression::ReadInput`      |

**Infix** parse functions (based on peek token, if higher precedence):

| Token                                                | Result                        |
| ---------------------------------------------------- | ----------------------------- |
| `+`, `-`, `*`, `/`, `==`, `!=`, `<`, `>`, `<=`, `>=` | `Expression::Binary`          |
| `LParen`                                             | `Expression::FunctionCall`    |
| `LBracket`                                           | `Expression::IndexExpression` |

### Type Parsing

Type annotations are parsed by `parse_type()`:

| Token            | Result            |
| ---------------- | ----------------- |
| `Kain`           | `Type::Kain`      |
| `Sar`            | `Type::Sar`       |
| `Sit`            | `Type::Sit`       |
| `Su` + `<T>`     | `Type::Array(T)`  |
| `Twe` + `<K, V>` | `Type::Map(K, V)` |

### Error Handling

Parse errors are collected in `parser.errors: Vec<String>` without halting. The compiler reports all syntax errors before aborting compilation.

---

## 5. Type Checker (`src/typecheck.rs`)

### Environment (Symbol Table)

```rust
pub struct Symbol {
    pub ty: Type,
    pub is_function: bool,
    pub parameters: Vec<Type>,  // Only for functions
}

pub struct Environment {
    pub store: HashMap<String, Symbol>,
    pub outer: Option<Box<Environment>>,  // Lexical scope chain
}
```

Supports nested scoping — `get()` walks the outer chain to resolve identifiers.

### Type Rules Enforced

| Rule                                                | Error Message                                                             |
| --------------------------------------------------- | ------------------------------------------------------------------------- |
| Let declaration type must match value type          | "Type mismatch: cannot assign `{vt}` to variable `{name}` of type `{ty}`" |
| Assignment target must be declared                  | "Undeclared variable `{name}`"                                            |
| Assignment value must match variable type           | "Type mismatch: cannot assign..."                                         |
| If/While conditions must be `sit` (bool)            | "If/While condition must be a boolean (sit)"                              |
| For-in collection must be array `su<...>`           | "For-in collection must be an array (su<...>)"                            |
| Arithmetic (`+`, `-`, `*`, `/`) requires two `kain` | "Operator `{op}` requires two integers (kain)"                            |
| String concatenation (`+`) requires two `sar`       | _(same as above, with special-case for Sar+Sar)_                          |
| Comparison (`==`, `!=`) requires matching types     | "Cannot compare differing types"                                          |
| Ordering (`>`, `<`, `>=`, `<=`) requires two `kain` | "Operator `{op}` requires two integers (kain)"                            |
| Function calls must match arity and param types     | "Function `{name}` expects {n} arguments, got {m}"                        |
| Array elements must be homogeneous                  | "Array elements must have the same type"                                  |
| Array index must be `kain`                          | "Array index must be an integer (kain)"                                   |
| HashMap keys/values must be consistent              | "HashMap elements must have consistent types"                             |
| `phat` prompt must be `sar`, returns `sar`          | "Prompt must be a string (sar)"                                           |
| Identifier must be declared                         | "Undeclared identifier `{name}`"                                          |
| Callee must be a function                           | "`{name}` is not a function"                                              |

---

## 6. Code Generator — Go Backend (`src/codegen_go.rs`) _(Default)_

The Go backend is the **primary code generator** as of February 2026. It produces clean, idiomatic Go code.

### Advantages over C Backend

- **Native Unicode identifiers** — Go supports Unicode in identifiers, so no hex-encoding is needed
- **Native string concatenation** — Go's `+` handles strings, no helper function required
- **Real HashMap support** — Go's `map[K]V` gives full hashmap functionality
- **Automatic memory management** — Go's garbage collector handles allocations
- **Server-ready runtime** — Foundation for goroutines, channels, `net/http` in future phases

### Type Mapping

| M-Lang Type      | Go Type   |
| ---------------- | --------- |
| `kain` (Kain)    | `int64`   |
| `sar` (Sar)      | `string`  |
| `sit` (Sit)      | `bool`    |
| `su<T>` (Array)  | `[]T`     |
| `twe<K,V>` (Map) | `map[K]V` |

### Identifier Handling

Go supports Unicode letters in identifiers natively. The only transformation is checking for Go keyword collisions (e.g., `map`, `range`, `type` → prefixed with `ml_`).

### Special Cases

- **Entry point**: Function named `main` is emitted as `func main()` with no return type. Return statements inside `main` are omitted.
- **String `+`**: Binary `+` on string expressions uses Go's native `+` operator directly.
- **Print**: `pya(expr)` compiles to `fmt.Println(expr)` — Go handles type formatting automatically.
- **While loops**: `pat (cond)` compiles to `for cond { ... }` (Go has no `while` keyword).
- **For-in loops**: `pat item htae arr` compiles to `for _, item := range arr { ... }`.
- **Import**: `yu module` emits a comment (full module system planned).
- **Arrays**: Array literals use Go slices: `[]int64{1, 2, 3}`.
- **HashMaps**: Emit Go map literals: `map[string]int64{"a": 1, "b": 2}`.
- **Unused variables**: `_ = varname` is emitted after every declaration to satisfy Go's unused-variable rule.

### Import Management

The generator scans the AST before emitting code to determine which Go imports are needed:

- `"fmt"` — always included (for `fmt.Println`)
- `"bufio"` + `"os"` — included when `phat()` (read input) is used

### Generated Go Structure

```go
package main

import (
	"fmt"
)

func main() {
	var age int64 = 20
	_ = age
	if (age > 18) {
		fmt.Println("adult")
	}
}
```

---

## 6b. Code Generator — C Backend (`src/codegen.rs`) _(Legacy, `--target c`)_

The C backend is the original code generator, retained for cases where Go is not available or minimal output is desired.

### C Runtime Helpers

Two helper functions are emitted at the top of every generated C file:

```c
// String concatenation (heap-allocated)
char* mlang_concat(const char* s1, const char* s2);

// Read a line from stdin with optional prompt
char* mlang_read_input(const char* prompt);
```

### Type Mapping

| M-Lang Type      | C Type                |
| ---------------- | --------------------- |
| `kain` (Kain)    | `long long`           |
| `sar` (Sar)      | `char*`               |
| `sit` (Sit)      | `bool`                |
| `su<T>` (Array)  | `T*`                  |
| `twe<K,V>` (Map) | `void* /* HashMap */` |

### Identifier Mangling

Non-ASCII identifiers are not valid C identifiers in most compilers, so they are hex-encoded:

```
name → "mlang_" + hex(UTF-8 bytes)

Example: "myVar" → "mlang_6d79566172"
```

### Special Cases

- **Entry point**: Function named `main` is emitted as `int main()`.
- **String `+`**: Binary `+` on string expressions emits `mlang_concat(left, right)` instead of C `+`.
- **Print**: `pya(expr)` dispatches to `printf` with the correct format specifier:
  - Integer → `"%lld\n"`
  - String → `"%s\n"`
  - Boolean → `"%d\n"`
- **Import**: `yu module` → `#include "module.c"`
- **Arrays**: Array literals use C compound literals: `(long long[]){1, 2, 3}`
- **HashMaps**: Currently emit `NULL /* true HashMap requires complex C runtime */`

### Generated C Structure

```c
#include <stdio.h>
#include <stdbool.h>
#include <string.h>
#include <stdlib.h>

char* mlang_concat(const char* s1, const char* s2) { ... }
char* mlang_read_input(const char* prompt) { ... }

// User code follows (functions, main, etc.)
int main() {
    long long mlang_... = 20;
    ...
    return 0;
}
```

---

## 7. Compilation Pipeline (`src/main.rs`)

### CLI Flow

```
mlang build <file.ml>                  # Go backend (default)
mlang build --target c <file.ml>       # C backend
mlang run <file.ml>                    # Build + run (Go)
mlang run --target c <file.ml>         # Build + run (C)
```

### `compile_file()` Steps

1. **Read** source file from disk
2. **Lex & Parse** → `Program` AST (abort on syntax errors)
3. **Type Check** → validate AST (abort on type errors)
4. **Backend dispatch** based on `--target` flag (default: `go`)
5. **Code Generate** → produce Go or C source string
6. **Write** intermediate file (`.go` or `.c`, same stem as input)
7. **Invoke compiler** → `go build` or `gcc` to produce `.exe` / binary
8. **Report** success or compiler errors

For `run`, the resulting executable is immediately launched with `Command::new().status()`.

---

## 8. Test Suite

### Lexer Tests (`lexer.rs`)

- Tokenizes a full program with comments, keywords, Myanmar numerals, strings, operators, array/map syntax, read expressions, imports, and if/else blocks.
- Verifies exact token sequence matches expected output.

### Parser Tests (`parser.rs`)

- **`test_let_statements`**: Parses `kain` and `sar` variable declarations.
- **`test_function_declaration`**: Parses `loke` with two typed parameters and return type.
- **`test_arrays_and_hashmaps`**: Parses `su<kain>` array and `twe<sar, kain>` hashmap declarations with indexing.
- **`test_read_input`**: Parses `phat("prompt")` expression in a let statement.

### Go CodeGen Tests (`codegen_go.rs`)

- **`test_go_codegen_basic`**: Verifies `package main`, `func main()`, `var age int64`, `fmt.Println`.
- **`test_go_codegen_string_concat`**: Verifies native `+` for strings (no `mlang_concat`).
- **`test_go_codegen_for_in_loop`**: Verifies `for _, item := range` pattern.
- **`test_go_codegen_while_loop`**: Verifies `for condition { ... }` (Go has no `while`).
- **`test_go_codegen_hashmap`**: Verifies `map[string]int64{...}` literals.
- **`test_go_codegen_function_with_params`**: Verifies `func add(a int64, b int64) int64`.
- **`test_go_codegen_unicode_identifiers`**: Verifies no hex-encoding (Go supports Unicode).
- **`test_go_codegen_read_input`**: Verifies `mlangReadInput` helper and `bufio`/`os` imports.
- **`test_go_codegen_if_elif_else`**: Verifies `if`/`else if`/`else` chain generation.

### C CodeGen Tests (`codegen.rs`) _(Legacy)_

- End-to-end: parses M-Lang source → generates C → asserts `int main()` and `printf` calls are present.
