# M-Lang Server-Side Pivot Roadmap

## Comparison: Go vs M-Lang for Server-Side Development

This document analyzes what features M-Lang needs to become a viable **server-side development language** comparable to Go.

---

## Current M-Lang Feature Inventory

| Feature                                                 | Status                                   |
| ------------------------------------------------------- | ---------------------------------------- |
| Integer (`kain`), String (`sar`), Boolean (`sit`) types | Done                                     |
| Arrays (`su<T>`), HashMaps (`twe<K,V>`)                 | Done                                     |
| Functions with typed params & return types              | Done                                     |
| If/else-if/else, while loops, for-in loops              | Done                                     |
| Print (`pya`), Read input (`phat`)                      | Done                                     |
| String concatenation                                    | Done                                     |
| Myanmar numeral support                                 | Done                                     |
| Static type checking                                    | Done                                     |
| **Go backend (`go build`)**                             | **Done** (default since Feb 2026)        |
| C backend (`gcc`)                                       | Done (legacy, via `--target c`)          |
| LSP + VS Code extension                                 | Done                                     |
| Formatter (`mlang fmt`)                                 | Done                                     |
| Imports (`yu`)                                          | Partial (comment in Go, `#include` in C) |

---

## Feature Gap Analysis: What Go Has That M-Lang Needs

### TIER 1 — Critical (Must-Have for Any Server-Side Work)

#### 1. Goroutine-style Concurrency (`kyein` — ကြိုး / thread/rope)

**Go equivalent:** `go func()`, goroutines, channels  
**Why critical:** Server-side = handling many concurrent requests. This is Go's #1 killer feature.

**What to build:**

- Lightweight green threads or async tasks
- Keyword: `kyein` (thread) — `kyein myFunction();`
- Channels for communication: `laung` (လောင်း / channel/pipe)
  - `laung<kain> ch = laung_thit();` — create channel
  - `po(ch, value);` — send to channel (ပို့)
  - `kain val = yu_laung(ch);` — receive from channel
- Select/multiplexing over channels

**Implementation approach:**

- Short-term: Compile to C with pthreads or libuv
- Long-term: Compile to LLVM IR or generate Go/Rust and leverage their runtimes

---

#### 2. Error Handling (`amhar` — အမှား / error)

**Go equivalent:** `error` interface, `if err != nil`, multiple return values  
**Why critical:** Every I/O operation, every network call can fail. No error handling = no serious server code.

**What to build:**

- Error type: `amhar`
- Multiple return values: `loke readFile(sar path) -> (sar, amhar) { ... }`
- Or Result type: `yin<sar, amhar>` (ရင်း / result/outcome)
- Error creation: `amhar_thit("message")` — create new error
- Nil/null concept: `bhone` (ဘုန်း / empty/void) for checking errors

**Suggested syntax:**

```
loke divide(kain a, kain b) -> (kain, amhar) {
    hlyin (b == 0) {
        pyan (0, amhar_thit("cannot divide by zero"));
    }
    pyan (a / b, bhone);
}

// Caller
kain result, amhar err = divide(10, 0);
hlyin (err != bhone) {
    pya("Error: " + err);
}
```

---

#### 3. Structs / Custom Types (`pone` — ပုံ / shape/form)

**Go equivalent:** `type MyStruct struct { ... }`  
**Why critical:** You cannot model HTTP requests, database rows, JSON payloads, configs, etc. without structured data.

**What to build:**

- Struct definition keyword: `pone`
- Field access with dot notation
- Struct literals

**Suggested syntax:**

```
pone User {
    sar name;
    kain age;
    sar email;
}

loke main() -> kain {
    User u = User{name: "Aung", age: 25, email: "aung@example.com"};
    pya(u.name);
    pyan 0;
}
```

---

#### 4. Methods on Types (`nee` — နည်း / method/way)

**Go equivalent:** `func (u *User) GetName() string { ... }`  
**Why critical:** Methods are the bridge between data and behavior. Required for interfaces, HTTP handlers, etc.

**Suggested syntax:**

```
nee (User u) getName() -> sar {
    pyan u.name;
}
```

---

#### 5. Interfaces (`pyint` — ပြင့် / open/exposed)

**Go equivalent:** `type Handler interface { ServeHTTP(...) }`  
**Why critical:** Go's implicit interface satisfaction is what makes its stdlib so composable. Needed for HTTP handlers, readers/writers, etc.

**Suggested syntax:**

```
pyint Reader {
    loke read(su<kain> buf) -> (kain, amhar);
}

pyint Writer {
    loke write(su<kain> data) -> (kain, amhar);
}
```

---

#### 6. Package/Module System (`atote` — အထုပ် / package)

**Go equivalent:** `package main`, `import "net/http"`  
**Why critical:** The current `yu` import is just a C `#include`. Real server code needs proper module boundaries, namespaces, dependency resolution.

**What to build:**

- Package declaration: `atote server;` at top of file
- Full module resolution with paths: `yu "net/http";`
- Public/private visibility (capitalization like Go, or explicit keyword `pya_thi` / ပြသည် for exported)
- Standard library packages as separate `.ml` files

---

#### 7. Standard Library — Network & HTTP

**Go equivalent:** `net/http`, `net`, `encoding/json`  
**Why critical:** This IS server-side development.

**Must-have stdlib packages:**
| Package | Myanmar Name | Purpose |
|---------|-------------|---------|
| `net/http` | `kainn/http` (ကိုင်း) | HTTP server & client |
| `net` | `kainn` | TCP/UDP sockets |
| `encoding/json` | `json` | JSON encode/decode |
| `io` | `in_ote` (အင်အုပ်) | Reader/Writer interfaces |
| `fmt` | `pone_set` (ပုံစံ) | Formatted I/O (beyond `pya`) |
| `os` | `su_nit` (စနစ်) | OS interaction, env vars, file ops |
| `log` | `hmat` (မှတ်) | Logging |

**Example — HTTP server in M-Lang:**

```
yu "kainn/http";

loke handler(http.Request req, http.ResponseWriter w) -> amhar {
    w.write("Mingalabar from M-Lang server!");
    pyan bhone;
}

loke main() -> kain {
    http.handle("/", handler);
    http.listen(":8080");
    pyan 0;
}
```

---

### TIER 2 — Important (Needed for Production Server Code)

#### 8. For Loop / Range Iteration (`yu_set` — ယူစဉ် / take-in-order)

**Go equivalent:** `for i, v := range slice { ... }`

**Currently only `pat` (while) exists. Need:**

```
// C-style for
pat (kain i = 0; i < 10; i = i + 1) {
    pya(i);
}

// Range over array
pat item yu_set numbers {
    pya(item);
}

// Range with index
pat (kain i, kain item) yu_set numbers {
    pya(i);
    pya(item);
}
```

---

#### 9. Floating-Point Numbers (`da_thin` — ဒဿမ / decimal)

**Go equivalent:** `float64`

Servers need floating-point for:

- Timestamps, durations
- Financial calculations
- Metrics/monitoring

```
da_thin price = 19.99;
da_thin pi = 3.14159;
```

---

#### 10. Nil/Null Type (`bhone` — ဘုန် / void/empty)

**Go equivalent:** `nil`

Needed for:

- Optional values
- Uninitialized pointers
- Error checking (`if err != nil`)

---

#### 11. Type Conversions (`pyaung` — ပြောင်း / convert/change)

**Go equivalent:** `strconv.Atoi()`, `string(bytes)`, type casting

```
kain num = pyaung_kain("42");     // string -> int
sar text = pyaung_sar(42);        // int -> string
da_thin f = pyaung_da_thin(42);   // int -> float
```

---

#### 12. Defer Statement (`naut_sone` — နောက်ဆုံး / finally/at-the-end)

**Go equivalent:** `defer file.Close()`

Critical for resource cleanup in servers:

```
loke readFile(sar path) -> (sar, amhar) {
    File f = file.open(path);
    naut_sone f.close();
    // ... read contents ...
}
```

---

#### 13. Slice Operations

**Go equivalent:** `slice[1:3]`, `append(slice, elem)`

```
su<kain> nums = [1, 2, 3, 4, 5];
su<kain> sub = nums[1:3];           // [2, 3]
nums = htae(nums, 6);               // append: ထည့် (htae / insert)
kain length = ashay(nums);           // len: အရေ (ashay / count)
```

---

#### 14. Multiple Return Values

**Go equivalent:** `func divide(a, b int) (int, error)`

Fundamental to Go's error handling pattern:

```
loke divide(kain a, kain b) -> (kain, amhar) {
    pyan (a / b, bhone);
}
```

---

#### 15. String Operations (Built-in Methods)

**Go equivalent:** `strings.Split()`, `strings.Contains()`, etc.

Needed for URL parsing, header manipulation, body processing:

```
su<sar> parts = sar.split(url, "/");
sit found = sar.pal(text, "search");    // ပါ (pal / contains)
kain length = ashay(text);
sar lower = sar.ayaik(text);            // အရိုက် (lowercase)
```

---

### TIER 3 — Nice-to-Have (Ecosystem & DX Maturity)

#### 16. Testing Framework (`set_sae` — စစ်ဆေး / verify/test)

**Go equivalent:** `go test`, `testing.T`

```
set_sae "addition works" {
    kain result = add(2, 3);
    mhann(result == 5);  // assert: မှန် (mhann / assertTrue)
}
```

CLI: `mlang test file_test.ml`

---

#### 17. JSON Support

**Go equivalent:** `encoding/json`, struct tags

```
pone User {
    sar name   `json:"name"`;
    kain age   `json:"age"`;
}

sar json_str = json.encode(user);
User u = json.decode(json_str, User);
```

---

#### 18. Context & Timeout (`atae_anay` — အတွေ့အနေ / context/situation)

**Go equivalent:** `context.Context`, `context.WithTimeout()`

Critical for HTTP request lifecycle management.

---

#### 19. Middleware Pattern

Should be composable function chaining:

```
loke logging(http.Handler next) -> http.Handler {
    // log the request
    pyan next;
}

http.handle("/", logging(handler));
```

---

#### 20. Database Drivers

**Go equivalent:** `database/sql`

Standard interface for MySQL, PostgreSQL, SQLite.

---

## Implementation Priority & Roadmap

### Phase 1: Language Foundations (Weeks 1-8)

| #   | Feature                       | Effort | Keyword           |
| --- | ----------------------------- | ------ | ----------------- |
| 1   | Structs                       | Medium | `pone`            |
| 2   | Methods                       | Medium | `nee`             |
| 3   | Interfaces                    | Medium | `pyint`           |
| 4   | Error type + multiple returns | Hard   | `amhar`, `bhone`  |
| 5   | For loop / range              | Easy   | `pat ... yu_set`  |
| 6   | Float type                    | Easy   | `da_thin`         |
| 7   | Nil type                      | Easy   | `bhone`           |
| 8   | Type conversions              | Easy   | `pyaung`          |
| 9   | Slice operations              | Medium | `htae`, `ashay`   |
| 10  | String methods                | Medium | `sar.split`, etc. |

### Phase 2: Module System & Stdlib (Weeks 9-16)

| #   | Feature                           | Effort | Keyword                    |
| --- | --------------------------------- | ------ | -------------------------- |
| 11  | Real package system               | Hard   | `atote`                    |
| 12  | Visibility (public/private)       | Easy   | Capitalization or `export` |
| 13  | `os` package — file I/O, env vars | Medium | `su_nit`                   |
| 14  | `fmt` — sprintf, formatted output | Easy   | `pone_set`                 |
| 15  | `io` — Reader/Writer interfaces   | Medium | `in_ote`                   |
| 16  | `log` — structured logging        | Easy   | `hmat`                     |

### Phase 3: Networking & Concurrency (Weeks 17-24)

| #   | Feature                     | Effort    | Keyword      |
| --- | --------------------------- | --------- | ------------ |
| 17  | TCP/UDP sockets             | Hard      | `kainn`      |
| 18  | HTTP server & client        | Hard      | `kainn/http` |
| 19  | JSON encode/decode          | Medium    | `json`       |
| 20  | Goroutine-style concurrency | Very Hard | `kyein`      |
| 21  | Channels                    | Very Hard | `laung`      |
| 22  | Defer                       | Easy      | `naut_sone`  |

### Phase 4: Production Readiness (Weeks 25-32)

| #   | Feature                        | Effort | Keyword                |
| --- | ------------------------------ | ------ | ---------------------- |
| 23  | Testing framework              | Medium | `set_sae`              |
| 24  | Context/timeout                | Medium | `atae_anay`            |
| 25  | Middleware pattern             | Easy   | (composable functions) |
| 26  | Database interface             | Hard   | `database`             |
| 27  | `mlang get` dependency manager | Hard   | —                      |
| 28  | Cross-compilation              | Medium | —                      |

---

## Compiler Backend Decision

> **RESOLVED (February 2026):** The Go backend is now implemented and is the **default** transpilation target. The C backend remains available via `--target c`.

| Backend          | Status             | Pros                                                        | Cons                                                  |
| ---------------- | ------------------ | ----------------------------------------------------------- | ----------------------------------------------------- |
| **Go (default)** | **✅ Implemented** | Goroutines/channels for free, GC, maps, Unicode identifiers | Requires Go toolchain                                 |
| **C (legacy)**   | ✅ Implemented     | Simple, fast compile, portable                              | No goroutines, manual memory, hex-encoded identifiers |
| **LLVM IR**      | Future             | Full optimization, native code                              | Complex to implement                                  |

The Go backend gives us the foundation for Phase 3 features (concurrency, HTTP, channels) without building a runtime from scratch.

---

## Vision: What "Hello World HTTP Server" Looks Like

```mlang
atote main;

yu "kainn/http";

loke handler(http.Request req, http.ResponseWriter w) -> amhar {
    w.write("မင်္ဂလာပါ! Mingalabar from M-Lang! 🇲🇲");
    pyan bhone;
}

loke main() -> kain {
    http.handle("/", handler);
    pya("Server running on :8080");
    http.listen(":8080");
    pyan 0;
}
```

This is the north star. Every feature in this roadmap serves getting to this point.

---

## Summary: The 10 Most Critical Features to Add

1. **Structs** (`pone`) — Can't model data without them
2. **Methods** (`nee`) — Can't attach behavior to data
3. **Interfaces** (`pyint`) — Can't write composable code
4. **Error handling** (`amhar` + multiple returns) — Can't do I/O safely
5. **Package system** (`atote` + proper `yu`) — Can't organize real projects
6. **HTTP server stdlib** — The actual deliverable
7. **JSON support** — Every API needs it
8. **Concurrency** (`kyein` + `laung`) — Handle multiple requests
9. **For/range loops** — Basic iteration is missing
10. **Float type** (`da_thin`) — Timestamps, durations, metrics
