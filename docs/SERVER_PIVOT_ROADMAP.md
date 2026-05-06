# M-Lang Server-Side Pivot Roadmap

## Goal

This roadmap tracks what M-Lang already supports, what remains Go-backed for server-side work, and the current migration from Go-target-first positioning to a native LLVM compiler path.

Date baseline: **March 2026**.

---

## Current Implementation Snapshot

### Language Core

| Capability | Status | Current Keyword/Syntax |
| --- | --- | --- |
| Structs, methods, interfaces | Done | `pone`, `nee`, `myat` |
| Error type + tuple returns + nil | Done | `amhar`, `(a, b)` returns, `bhala` |
| For-in, indexed for-in, break/continue, C-style for | Done | `pat ... htae`, `yut`, `shar`, `pat (init; cond; post)` |
| Float + conversions | Done | `da_tha`, `pyaung_kain/sar/da_tha` |
| Slices + array/string helpers | Done | `arr[low:high]`, `htae`, `ashay`, `khwae/swal/ayaik` |
| Package declaration + explicit export | Done | `atote`, `pay` |

### Modules and Stdlib (Go Interop Backend)

| Capability | Status | Notes |
| --- | --- | --- |
| Local module imports | Done | Relative `yu "./util"` with cycle detection |
| HTTP module (`kainn/http`) | Done | Client + server runtime (`get/post/handle/listen`, request/writer helpers) |
| Socket module (`kainn`) | Done | TCP/UDP listen/dial/bind plus connection methods |
| JSON module (`json`) | Done | `encode/decode` baseline workflows |
| File module (`file`) | Done | `read/write` |
| System module (`su_nit`) | Done | `env/args` |
| `fmt`/`io`/`log` modules | Done | `pone_set.pon_san`, `in_ote.twin_phat/htote_yay`, `hmat.mhat_chet/mhat_thati/mhat_amhar` |

### Tooling and Backends

| Capability | Status | Notes |
| --- | --- | --- |
| LLVM backend | MVP | Default native compiler target (`--target llvm`) for Phase 1 examples |
| Go backend | Done | Interop/bootstrap target (`--target go`) for stdlib/server features |
| C backend | Legacy | Limited surface, no local modules/stdlib shims |
| LSP + VS Code extension | Done | Semantic + diagnostics support |
| Formatter (`mlang fmt`) | Done | Formats modern syntax and imports |

---

## Example That Works Today

```mlang
yu "kainn/http";
yu "json";

loke main() -> kain {
    twe<sar, kain> payload = {"order_id": 123, "amount": 5000};
    sar body, amhar enc_err = json.encode(payload);
    hlyin (enc_err != bhala) {
        pyan 1;
    }

    http.Response res, amhar req_err = http.post("https://httpbin.org/post", body);
    hlyin (req_err != bhala) {
        pyan 1;
    }

    pya(res.status);
    pyan 0;
}
```

---

## Gap Analysis (Remaining Work)

### Tier 1 - Critical for Serious Server Work

1. **Concurrency primitive** (`kyoe`) - Done  
   Implemented as keyword-native statement lowering to Go `go`.

2. **Channels/message passing** (`laung`) - Done  
   `laung<T>` type and `send/recv/close` runtime lowering are implemented.

3. **HTTP server runtime API** - Done  
   `kainn/http` now includes `handle/listen`, `Request`, and `ResponseWriter` APIs.

4. **Defer-style cleanup** (`naut_sone`) - Done  
   Implemented as keyword-native statement lowering to Go `defer`.

5. **Socket-level networking** (`kainn`) - Done  
   TCP/UDP functions and methods are exposed through stdlib shims.

### Tier 2 - Production Readiness

1. **Extended stdlib modules** - Done  
   `pone_set` (fmt), `in_ote` (io), `hmat` (log) are implemented in the Go interop stdlib shim.

2. **Language-level test framework** (`set_sae`) - Done  
   `mlang test <file.ml>` executes language-level tests with pass/fail reporting and non-zero exit on failure.

3. **Context/timeout model** (`baung`) - Done  
   `baung ctx = baung(timeout_ms);` and `ctx.close()` are implemented with Go `context.WithTimeout`.

4. **Database abstraction** (`database`) - Done  
   Postgres-first `database.open/conn.exec/query_one/query_all/close` APIs are available on the Go backend.

5. **Dependency manager command** (`mlang get`) - Done  
   Git ref pinning to commit SHA with deterministic `mlang.lock` and `.mlang/deps/<commit>/` cache.

6. **Cross-compilation UX** - Done  
   `mlang build --goos/--goarch` is supported; `mlang run` rejects non-host targets explicitly.

---

## Phase Status

| Phase | Scope | Status |
| --- | --- | --- |
| Phase 1: Language Foundations | Structs/methods/interfaces, tuples/errors/nil, loops, float/conversions, slices/string ops | **Completed** |
| Phase 2: Modules + Stdlib Baseline | `atote/pay`, local import graph, `json/file/su_nit/pone_set/in_ote/hmat`, HTTP client baseline | **Completed** |
| Phase 3: Networking + Concurrency Runtime | sockets, server-side HTTP runtime, `kyoe`, `laung`, `naut_sone` | **Completed** |
| Phase 4: Production Readiness | testing DSL, context, database, dependency manager, cross-compilation | **Completed** |
| Phase 5: Native Compiler Migration | LLVM default target, native runtime linking, Phase 1 native e2e coverage | **MVP In Progress** |

### Detailed Backlog by Phase

#### Phase 2 (Remaining)

No remaining items in Phase 2 baseline scope.

#### Phase 3

| Item | Status |
| --- | --- |
| `kainn` TCP/UDP | Done |
| `kainn/http` server APIs | Done |
| `json` encode/decode | Done |
| `kyoe` concurrency | Done |
| `laung` channels | Done |
| `naut_sone` defer | Done |

#### Phase 4

| Item | Status |
| --- | --- |
| `set_sae` testing | Done |
| `baung` context/timeout | Done |
| Middleware pattern helpers | Done |
| `database` interface | Done |
| `mlang get` | Done |
| Cross-compilation | Done |

---

## Compiler Backend Decision

> **Updated (May 2026):** LLVM is the default native compiler target. Go remains available via `--target go` as the interop/backend path for modules, stdlib, and server features. C remains available via `--target c` as a legacy path.

| Backend | Status | Pros | Tradeoffs |
| --- | --- | --- | --- |
| LLVM IR (default) | MVP implemented | Professor-facing native compiler path, object-code/linker pipeline, Phase 1 e2e coverage | Phase 2/3/4 stdlib/server parity still pending |
| Go (interop) | Implemented | Mature runtime, strong stdlib interop, Unicode-friendly identifiers | Source-to-Go bootstrap target, requires Go toolchain |
| C (legacy) | Implemented (limited) | Portable and simple for older subset | No local module graph support, no Go stdlib shim support |

---

## North Star (Target, Not Fully Implemented Yet)

```mlang
atote main;
yu "kainn/http";

loke handler(http.Request req, http.ResponseWriter w) -> amhar {
    w.write("Mingalabar from M-Lang!");
    pyan bhala;
}

loke main() -> kain {
    http.handle("/", handler);
    pya("Server running on :8080");
    http.listen(":8080");
    pyan 0;
}
```

This server-side target syntax is available on the Go interop backend. The LLVM backend intentionally rejects these Go-backed server/runtime features until native runtime parity is added.

---

## Next High-Value Items

1. Expand LLVM backend parity beyond Phase 1 examples while keeping clear Go-only diagnostics
2. Add a small M-Lang IR or typed-AST lowering layer before LLVM if the backend grows complex
3. Package registry and semver-aware dependency resolution (beyond git+lockfile v1)
4. Broader database adapters and typed row mapping beyond `twe<sar, sar>`
