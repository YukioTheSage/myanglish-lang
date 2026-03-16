# M-Lang Chatbot & Commerce Platform — Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make M-Lang the go-to language for building chatbots, payment integrations, and social commerce automation in Myanmar — "20 lines to a working shop bot."

**Architecture:** Layered approach. First fix language gaps that block all higher features (struct field assignment, closures, error handling, mutation). Then build stdlib modules (HTTP client, JSON, file I/O) that transpile to Go's stdlib. Finally, build thin SDK layers for Telegram, Viber, Wave Money that compose the stdlib modules.

**Tech Stack:** Rust compiler → Go transpilation → Go stdlib (`net/http`, `encoding/json`, `os`, `io`)

---

## Current State (What Already Works)

| Feature | Status |
|---------|--------|
| Structs (`pone`), field access, struct literals | ✅ Working |
| Methods (`nee`) on structs | ✅ Working |
| Interfaces (`myat`) | ✅ Working |
| Float (`da_tha`), Nil (`bhala`) | ✅ Working |
| Type conversions (`pyaung_kain`, `pyaung_sar`, `pyaung_da_tha`) | ✅ Working |
| Slice expressions (`numbers[1:3]`) | ✅ Working |
| String methods (`khwae`, `swal`) | ✅ Working |
| Error creation (`amhar("msg")`) | ✅ Working |
| Tuple returns & destructuring | ⚠️ Partial (vars forced unused) |
| Imports (`yu`) | ⚠️ Stub (outputs as comment) |

## What's Broken / Missing (Blocking Everything)

| Gap | Why It Blocks |
|-----|---------------|
| No struct field assignment (`p.name = "x"`) | Can't mutate state — bots need to update user data |
| No array/map mutation (`arr.push(x)`, `map[k] = v`) | Can't build order lists, track conversations |
| No closures / first-class functions | Can't do `bot.on_message(callback)` |
| Error destructuring vars forced unused | Can't check errors — Go-style `if err != nil` impossible |
| No break/continue in loops | Can't control iteration flow |
| No for-loop with index | Can't iterate with position |
| Module system is a stub | Can't import stdlib packages |

---

## Phase 1: Language Fixes (Weeks 1-3)
*Fix the gaps that block everything else.*

### Task 1: Struct Field Assignment

**Files:**
- Modify: `src/ast.rs` — add `FieldAssign` variant to Statement
- Modify: `src/parser.rs` — detect `identifier.field = expr` in assignment parsing
- Modify: `src/typecheck.rs` — validate struct type, field exists, type matches
- Modify: `src/codegen_go.rs` — generate `object.Field = value`
- Test: `src/parser.rs` (parser tests), `src/codegen_go.rs` (codegen tests)

**Step 1: Write failing test in parser**
```rust
#[test]
fn test_struct_field_assignment() {
    let input = r#"
pone Person { sar name; kain age; }
loke main() -> kain {
    Person p = Person { name: "Aung", age: 20 };
    p.name = "Ko Ko";
    p.age = 25;
    pya(p.name);
    pyan 0;
}
"#;
    let tokens = Lexer::new(input).tokenize();
    let (program, errors) = Parser::new(tokens).parse();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
}
```

**Step 2: Run test, verify it fails**
Run: `cargo test test_struct_field_assignment -v`
Expected: FAIL — parser doesn't recognize `p.name = "Ko Ko"`

**Step 3: Add FieldAssign to AST**
```rust
// In Statement enum, add:
FieldAssign {
    object: String,
    field: String,
    value: Expression,
    name_span: Span,
},
```

**Step 4: Implement parser support**
In `parse_statement()`, when we see `Identifier` followed by `.`, check if it's a field assignment (has `=` after the field name) vs method call or field access expression.

**Step 5: Add type checking**
Validate: object is a struct type, field exists on that struct, value type matches field type.

**Step 6: Add Go codegen**
Generate: `object.FieldName = value` (capitalize field name for Go export).

**Step 7: Write end-to-end test**
Create test `.ml` file, compile, verify generated Go is correct.

**Step 8: Run all tests, verify nothing broke**
Run: `cargo test`
Expected: All 34+ tests pass

**Step 9: Commit**
```bash
git add src/ast.rs src/parser.rs src/typecheck.rs src/codegen_go.rs
git commit -m "feat: add struct field assignment (p.name = value)"
```

---

### Task 2: Array & Map Mutation

**Files:**
- Modify: `src/parser.rs` — detect `array[index] = expr` and `map[key] = expr`
- Modify: `src/ast.rs` — add `IndexAssign` variant to Statement
- Modify: `src/typecheck.rs` — validate collection type, index/key type, value type
- Modify: `src/codegen_go.rs` — generate `array[i] = val` / `mapVar[key] = val`
- Modify: `src/codegen_go.rs` — add built-in method codegen for `push`, `remove`, `len`

**Step 1: Write failing test**
```rust
#[test]
fn test_array_index_assignment() {
    let input = r#"
loke main() -> kain {
    su<kain> nums = [1, 2, 3];
    nums[0] = 10;
    twe<sar, kain> prices = {"tea": 500};
    prices["coffee"] = 800;
    pyan 0;
}
"#;
    let tokens = Lexer::new(input).tokenize();
    let (program, errors) = Parser::new(tokens).parse();
    assert!(errors.is_empty(), "Parse errors: {:?}", errors);
}
```

**Step 2: Run test, verify it fails**

**Step 3: Add IndexAssign to AST**
```rust
IndexAssign {
    object: Expression,
    index: Expression,
    value: Expression,
    name_span: Span,
},
```

**Step 4: Implement parser — detect `expr[index] = value`**

**Step 5: Type check — validate types match**

**Step 6: Codegen — generate Go index assignment**

**Step 7: Add array `push` method support in codegen**
When `MethodCall` on array type with method name `push`:
```go
// M-Lang: nums.push(4)
// Go: nums = append(nums, 4)
```

**Step 8: Run all tests**
Run: `cargo test`

**Step 9: Commit**
```bash
git add src/ast.rs src/parser.rs src/typecheck.rs src/codegen_go.rs
git commit -m "feat: add array/map index assignment and push method"
```

---

### Task 3: Fix Error Handling (Tuple Destructuring)

**Files:**
- Modify: `src/codegen_go.rs` — stop generating `_ = varname` for destructured vars
- Modify: `src/typecheck.rs` — ensure error type comparisons work (`err != bhala`)

**Step 1: Write failing test**
```rust
#[test]
fn test_error_handling_pattern() {
    let input = r#"
loke divide(kain a, kain b) -> (kain, amhar) {
    hlyin (b == 0) {
        pyan (0, amhar("division by zero"));
    }
    pyan (a / b, bhala);
}
loke main() -> kain {
    kain result, amhar err = divide(10, 0);
    hlyin (err != bhala) {
        pya("Error occurred");
    }
    pyan 0;
}
"#;
    // Should compile and run without error
}
```

**Step 2: Fix codegen — remove forced unused suppression for destructured vars**
In `codegen_go.rs`, find the `LetDestructured` handler. Remove the `_ = varname` lines.

**Step 3: Ensure nil comparison works**
`err != bhala` should generate `err != nil` in Go.

**Step 4: Test end-to-end**

**Step 5: Commit**
```bash
git add src/codegen_go.rs src/typecheck.rs
git commit -m "fix: error handling — allow using destructured variables"
```

---

### Task 4: Break & Continue

**Files:**
- Modify: `src/token.rs` — add `Break` and `Continue` token kinds
- Modify: `src/lexer.rs` — recognize `yut` (break) and `shar` (continue) keywords
- Modify: `src/ast.rs` — add `Break` and `Continue` to Statement enum
- Modify: `src/parser.rs` — parse break/continue statements
- Modify: `src/typecheck.rs` — validate they're inside loops
- Modify: `src/codegen_go.rs` — generate `break` / `continue`

**Keywords:**
- `yut` (ရပ် / stop) → break
- `shar` (ဆက် / continue) → continue

**Step 1: Add tokens and lexer recognition**

**Step 2: Add AST variants**
```rust
Break,
Continue,
```

**Step 3: Parse in statement parser**

**Step 4: Type check — track loop depth, error if outside loop**

**Step 5: Codegen — emit `break` / `continue`**

**Step 6: Test**

**Step 7: Commit**
```bash
git commit -m "feat: add break (yut) and continue (shar) statements"
```

---

### Task 5: For Loop with Index

**Files:**
- Modify: `src/ast.rs` — add optional `index` field to `ForIn`
- Modify: `src/parser.rs` — parse `pat (kain i, kain item) htae collection`
- Modify: `src/typecheck.rs` — type check index as Kain
- Modify: `src/codegen_go.rs` — generate `for i, item := range collection`

**Step 1: Extend ForIn AST node**
```rust
ForIn {
    index: Option<String>,    // NEW
    iterator: String,
    collection: Expression,
    body: BlockStatement,
    name_span: Span,
},
```

**Step 2: Update parser — detect two-variable form**
`pat (kain i, sar item) htae collection { }` → index=Some("i"), iterator="item"

**Step 3: Update codegen**
If index is Some: `for i, item := range collection`
If index is None: `for _, item := range collection` (current behavior)

**Step 4: Test, commit**

---

### Task 6: First-Class Functions & Closures

**Files:**
- Modify: `src/ast.rs` — add `Function` type variant, `ClosureLiteral` expression
- Modify: `src/token.rs` — (may need no changes if reusing `loke`)
- Modify: `src/parser.rs` — parse closure syntax
- Modify: `src/typecheck.rs` — type check closures, function types
- Modify: `src/codegen_go.rs` — generate Go anonymous functions

**This is the most critical feature for the chatbot SDK.** Without callbacks, `bot.on_message(handler)` is impossible.

**Syntax:**
```mlang
// Function type
loke(sar, sar) -> kain

// Closure literal (anonymous function)
loke(sar msg, sar sender) -> kain {
    pya(msg);
    pyan 0;
}

// Passing as argument
bot.on_message(loke(sar msg) {
    pya(msg);
});
```

**Step 1: Add Function type**
```rust
// In Type enum:
Function {
    params: Vec<Type>,
    return_type: Box<Type>,
},
```

**Step 2: Add ClosureLiteral expression**
```rust
// In Expression enum:
ClosureLiteral {
    parameters: Vec<(String, Type, Span)>,
    return_type: Type,
    body: BlockStatement,
},
```

**Step 3: Parse closure syntax**
When `loke` appears in expression position (not statement), parse as closure.

**Step 4: Type check closures**
Infer closure type from parameters and return type. Check it matches expected function type at call sites.

**Step 5: Generate Go anonymous functions**
```go
func(msg string) {
    fmt.Println(msg)
}
```

**Step 6: Test with callback patterns**

**Step 7: Commit**
```bash
git commit -m "feat: add first-class functions and closures"
```

---

## Phase 2: Standard Library (Weeks 4-6)
*Build stdlib modules that transpile to Go's stdlib.*

### Task 7: Module System

**Files:**
- Modify: `src/parser.rs` — parse `yu "module/path"` with string paths
- Modify: `src/typecheck.rs` — register known stdlib modules and their exports
- Modify: `src/codegen_go.rs` — map M-Lang modules to Go imports
- Create: `src/stdlib.rs` — registry of built-in module types and functions

**Approach:** No file-based module loading yet. Hard-code a stdlib registry that maps M-Lang module names to Go imports and provides type information.

```rust
// stdlib.rs — maps M-Lang modules to Go types/imports
pub struct StdlibModule {
    pub mlang_name: &'static str,     // "kainn/http"
    pub go_import: &'static str,      // "net/http"
    pub types: Vec<StdlibType>,
    pub functions: Vec<StdlibFunc>,
}
```

When `yu "kainn/http"` is parsed:
1. Type checker loads the module's type definitions
2. Codegen adds the Go import
3. Module functions/types become available

**Step 1: Create stdlib registry with one module (`json`)**

**Step 2: Wire parser to resolve `yu` against registry**

**Step 3: Wire codegen to add Go imports**

**Step 4: Test with `yu "json"`**

**Step 5: Commit**

---

### Task 8: HTTP Client (`kainn/http`)

**Files:**
- Modify: `src/stdlib.rs` — add HTTP client module definition
- Modify: `src/codegen_go.rs` — generate Go net/http client code

**M-Lang API:**
```mlang
yu "kainn/http";

loke main() -> kain {
    http.Response res, amhar err = http.get("https://api.example.com/data");
    hlyin (err != bhala) {
        pya("Error: " + err);
        pyan 1;
    }
    pya(res.body);
    pyan 0;
}
```

**Generates Go:**
```go
import "net/http"
import "io"

resp, err := http.Get("https://api.example.com/data")
if err != nil {
    fmt.Println("Error: " + err.Error())
    return 1
}
defer resp.Body.Close()
body, _ := io.ReadAll(resp.Body)
```

**Types to register:**
- `http.Response { kain status; sar body; twe<sar,sar> headers; }`
- `http.get(sar url) -> (http.Response, amhar)`
- `http.post(sar url, sar body) -> (http.Response, amhar)`

---

### Task 9: JSON Module (`json`)

**Files:**
- Modify: `src/stdlib.rs` — add JSON module
- Modify: `src/codegen_go.rs` — generate encoding/json code

**M-Lang API:**
```mlang
yu "json";

loke main() -> kain {
    twe<sar, kain> data = {"price": 5000, "quantity": 2};
    sar json_str, amhar err = json.encode(data);
    pya(json_str);

    twe<sar, kain> parsed, amhar err2 = json.decode(json_str);
    pyan 0;
}
```

---

### Task 10: File I/O (`file`)

**M-Lang API:**
```mlang
yu "file";

loke main() -> kain {
    sar content, amhar err = file.read("data.csv");
    file.write("output.txt", "Hello!");
    pyan 0;
}
```

---

### Task 11: Environment Variables & OS

**M-Lang API:**
```mlang
yu "su_nit";

loke main() -> kain {
    sar token = su_nit.env("BOT_TOKEN");
    su<sar> args = su_nit.args();
    pyan 0;
}
```

---

## Phase 3: Telegram Bot SDK (Weeks 7-8)
*First platform SDK. Telegram is chosen because its Bot API is the simplest (HTTP-based, no webhooks required for polling mode).*

### Task 12: Telegram SDK (`telegram`)

**Files:**
- Modify: `src/stdlib.rs` — add telegram module
- Modify: `src/codegen_go.rs` — generate Telegram bot code

**Approach:** Generate Go code that uses raw `net/http` to call Telegram Bot API. No external Go dependencies needed.

**M-Lang API:**
```mlang
yu "telegram";

loke main() -> kain {
    telegram.Bot bot = telegram.connect(su_nit.env("BOT_TOKEN"));

    bot.on_message(loke(telegram.Message msg) {
        hlyin (msg.text == "/start") {
            bot.reply(msg, "Mingalabar! Welcome!");
        } mo hlyin (msg.text == "menu") {
            bot.reply(msg, "🍜 ခေါက်ဆွဲ - 2000ks");
        } mo {
            bot.reply(msg, "Type 'menu' to see options");
        }
    });

    bot.start();
    pyan 0;
}
```

**Types to register:**
```
telegram.Bot { }
telegram.Message { sar text; kain chat_id; sar sender_name; }
telegram.connect(sar token) -> telegram.Bot
Bot.on_message(loke(telegram.Message))
Bot.reply(telegram.Message, sar text)
Bot.start()
```

**Generated Go code uses:**
- `net/http` for Telegram API calls
- `encoding/json` for parsing updates
- Long polling via `getUpdates` endpoint
- Simple goroutine for the polling loop

**Step 1: Define telegram module in stdlib registry**

**Step 2: Generate Go Telegram bot scaffolding**
The codegen generates a complete Go file with:
- Telegram API helper functions (sendMessage, getUpdates)
- Polling loop
- Message dispatch to user's callback

**Step 3: Test with a real bot token**

**Step 4: Commit**

---

## Phase 4: Viber & Messenger SDKs (Weeks 9-10)

### Task 13: Viber SDK (`viber`)

Similar to Telegram but requires webhook (HTTP server). Depends on HTTP server support.

**M-Lang API:**
```mlang
yu "viber";

loke main() -> kain {
    viber.Bot bot = viber.connect(su_nit.env("VIBER_TOKEN"));
    bot.on_message(loke(viber.Message msg) {
        bot.reply(msg, "Hello from Viber!");
    });
    bot.listen(8080);
    pyan 0;
}
```

**Generates Go:**
- HTTP server on specified port
- Webhook handler for Viber callback URL
- Viber REST API calls for sending messages

### Task 14: Messenger SDK (`messenger`)

Same pattern, Facebook Messenger Platform API.

---

## Phase 5: Wave Money Payment SDK (Weeks 11-12)

### Task 15: Wave Money Integration (`wave`)

**M-Lang API:**
```mlang
yu "wave";

loke main() -> kain {
    wave.Client w = wave.connect(su_nit.env("WAVE_API_KEY"));

    wave.Payment p, amhar err = w.request_payment(5000, "Order #123");
    hlyin (err != bhala) {
        pya("Payment error: " + err);
        pyan 1;
    }

    pya("Payment link: " + p.link);
    pyan 0;
}
```

**Types:**
```
wave.Client { }
wave.Payment { sar link; sar status; kain amount; }
wave.connect(sar api_key) -> wave.Client
Client.request_payment(kain amount, sar description) -> (wave.Payment, amhar)
Client.check_status(sar payment_id) -> (wave.Payment, amhar)
```

---

## Phase 6: Templates & CLI (Weeks 13-14)

### Task 16: Project Templates

Add `mlang new <template>` CLI command.

```bash
mlang new telegram-bot      # Scaffolds a Telegram bot project
mlang new shop-bot           # Telegram + Wave Money shop bot
mlang new viber-bot          # Viber bot with webhook
mlang new gold-price-bot     # Fetches gold price, posts to group
```

Each template generates:
- `main.ml` with working bot code
- `.env.example` with required tokens
- `README.md` in Burmese with setup instructions

### Task 17: `mlang deploy` (Stretch Goal)

One-command deploy to a free-tier server.

```bash
mlang deploy main.ml  # Builds, packages, deploys to Railway/Fly.io
```

---

## North Star: The 25-Line Shop Bot

After all phases, this works:

```mlang
yu "telegram";
yu "wave";
yu "json";

pone Order { sar customer; sar item; kain price; }
su<Order> orders = [];

loke main() -> kain {
    telegram.Bot bot = telegram.connect(su_nit.env("BOT_TOKEN"));
    wave.Client wallet = wave.connect(su_nit.env("WAVE_KEY"));

    bot.on_message(loke(telegram.Message msg) {
        hlyin (msg.text == "menu") {
            bot.reply(msg, "🍜 ခေါက်ဆွဲ - 2000ks\n🍛 ထမင်းကြော် - 2500ks");
        } mo hlyin (msg.text == "order") {
            Order o = Order { customer: msg.sender_name, item: "ခေါက်ဆွဲ", price: 2000 };
            orders.push(o);
            wave.Payment p, amhar err = wallet.request_payment(o.price, "Order for " + o.customer);
            hlyin (err == bhala) {
                bot.reply(msg, "Payment link: " + p.link);
            } mo {
                bot.reply(msg, "Payment error, try again");
            }
        } mo {
            bot.reply(msg, "'menu' လို့ ရိုက်ပါ!");
        }
    });

    bot.start();
    pyan 0;
}
```

---

## Dependency Graph

```
Phase 1 (Language Fixes)
  ├── Task 1: Field Assignment ──────────────────────┐
  ├── Task 2: Array/Map Mutation ────────────────────┤
  ├── Task 3: Fix Error Handling ────────────────────┤
  ├── Task 4: Break/Continue ────────────────────────┤
  ├── Task 5: For Loop with Index ───────────────────┤
  └── Task 6: Closures ─────────────────────────────┤ ← CRITICAL PATH
                                                      │
Phase 2 (Stdlib)                                      │
  ├── Task 7: Module System ◄─── depends on all ─────┘
  ├── Task 8: HTTP Client ◄─── depends on 7
  ├── Task 9: JSON ◄─── depends on 7
  ├── Task 10: File I/O ◄─── depends on 7
  └── Task 11: Env/OS ◄─── depends on 7
                │
Phase 3         │
  └── Task 12: Telegram SDK ◄─── depends on 6,7,8,9,11
                │
Phase 4         │
  ├── Task 13: Viber SDK ◄─── depends on 12 + HTTP server
  └── Task 14: Messenger SDK ◄─── depends on 12
                │
Phase 5         │
  └── Task 15: Wave Money ◄─── depends on 8,9
                │
Phase 6         │
  ├── Task 16: Templates ◄─── depends on 12-15
  └── Task 17: Deploy ◄─── stretch goal
```

---

## Timeline Summary

| Phase | Weeks | What Ships |
|-------|-------|-----------|
| 1 — Language Fixes | 1-3 | M-Lang handles real programs (mutation, callbacks, errors) |
| 2 — Stdlib | 4-6 | HTTP requests, JSON, file I/O work |
| 3 — Telegram SDK | 7-8 | **First working chatbot in M-Lang** ← demo moment |
| 4 — Viber & Messenger | 9-10 | All major Myanmar platforms covered |
| 5 — Wave Money | 11-12 | Payment integration works |
| 6 — Templates & DX | 13-14 | `mlang new shop-bot` → working project in 30 seconds |

**First usable demo (Telegram bot): ~8 weeks**
**Full platform (all SDKs + payments): ~14 weeks**
