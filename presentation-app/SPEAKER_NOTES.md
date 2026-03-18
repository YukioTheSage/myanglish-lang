# M-Lang Speaker Notes

This file matches the current 27-slide deck in `presentation-app/`.

## Presenter Safety Notes

- The deck now includes 27 slides.
- Slide 2 now includes the five member names and student IDs. Add more detailed member roles only if your course requires them.
- In this local workspace, `cargo test` does not currently pass because recent AST additions are not fully wired into `src/codegen_go.rs`.
- Because of that, do not promise a live compiler build from this exact checkout unless you first switch to a clean presentation branch.
- For the testing slide, the safest phrasing is: "The project has broad automated coverage across lexer, parser, module loading, type checking, code generation, formatter, and LSP analysis."

## Framing Note

For the academic review section, this presentation uses Go as the reference language. That is the most defensible choice for this repo because M-Lang transpiles to Go by default and several language/runtime ideas in M-Lang are intentionally Go-aligned.

## One-Sentence Thesis

M-Lang is a statically typed programming language with Myanglish keywords, implemented as a Rust compiler that transpiles mainly to Go while still providing tooling such as formatting, LSP support, and a VS Code extension.

## Slide 1 - Title

- Main point: introduce the project as both a language-design project and a compiler project.
- Say: "This project is called M-Lang. It is a statically typed programming language with Myanglish keywords, and the compiler is written in Rust. The language targets native executables through a Go-first backend, with C kept as a legacy backend."
- If asked "is this only a syntax experiment?": "No. The contribution includes lexing, parsing, type checking, code generation, formatting, language-server support, and editor tooling."

## Slide 2 - Group Introduction

- Main point: identify the team and frame the presentation goal.
- Say: "This slide introduces the group and clarifies the project scope: language design, compiler construction, and tooling."
- The current names and student IDs are filled in, and `Nyan Lin Htet` is marked as the presenter.
- Add finer role labels only if your instructor expects them on the slide.

## Slide 3 - Reference Programming Paradigm Review

- Main point: explain the paradigm baseline before introducing M-Lang.
- Say: "The reference paradigm is a compiled, statically typed, imperative or procedural model with explicit functions, control flow, and concurrency support. This gives us a strong baseline for comparing the new language design."
- Emphasize: compiled programs, static types, function-based structure, and first-class concurrency ideas.
- If asked why this paradigm was chosen: "Because it maps directly to the actual implementation path of this project and supports serious systems-style language design."

## Slide 4 - Reference Context & Concept View

- Main point: show the concept view visually.
- Say: "This slide shows the reference model as source code plus packages and imports, then compilation, then binary execution with structured types, functions, and concurrency."
- Point at the diagram as the "picture concept view" required by the presentation format.
- If asked why Go is the reference language: "Because M-Lang already targets Go by default and mirrors Go's package, error-handling, and concurrency ideas."

## Slide 5 - Reference Language Design

- Main point: describe the reference language design in plain terms.
- Say: "In the reference language, programs begin with package and import declarations, functions use typed parameters and results, control flow is block-based, and concurrency is part of the language model rather than an afterthought."
- Use the Go example to point out: package, import, typed function, return value, and procedural execution.
- If asked "is M-Lang just Go renamed?": "No. Go is the reference model here, but M-Lang has its own localized keyword system, syntax surface, compiler pipeline, and project goals."

## Slide 6 - Reference Language BNF

- Main point: show the reference language in grammar form.
- Say: "This is a simplified presentation-level grammar summary, adapted from the official Go specification. The goal is to show the shape of the language design, not reproduce the full spec."
- Important phrasing: "adapted summary" or "representative productions."
- If asked why BNF matters: "Because grammar makes the language design explicit and provides a bridge from syntax discussion to parser implementation."
- If asked whether this is the complete Go grammar: "No. It is an academic summary for presentation use."

## Slide 7 - Target New Language Outline

- Main point: explain how M-Lang changes the surface syntax while preserving strong semantics.
- Say: "M-Lang localizes the programmer-facing syntax into Myanglish while preserving static typing, a structured compiler pipeline, and a Go-aligned runtime model."
- Point at the mapping table: `package/import -> atote/yu`, `func/return -> loke/pyan`, `go/chan/defer -> kyoe/laung/naut_sone`.
- If asked what is genuinely new: "The keyword system, the localization design, the compiler implementation, and the combined educational plus tooling focus."

## Slide 8 - Preliminary M-Lang BNF

- Main point: show the target language in preliminary grammar form.
- Say: "This is the preliminary M-Lang grammar used for presentation. It summarizes imports, declarations, functions, statements, loops, concurrency constructs, and types."
- Explain that this grammar is intentionally simplified compared to the full parser implementation.
- If asked how faithful it is to the implementation: "It is a presentation-level summary derived from the implemented syntax in the repo."

## Slide 9 - Document Review Sources

- Main point: satisfy the academic requirement for sources and show that the review is grounded in official references.
- Say: "These are the main references used for the document review section: the official Go language specification, A Tour of Go, the Go documentation site, Effective Go, and the public Myanglish Lang repository used for implementation reference."
- External links to mention:
- `https://go.dev/ref/spec`
- `https://go.dev/tour/`
- `https://go.dev/doc`
- `https://go.dev/doc/effective_go`
- `https://github.com/YukioTheSage/myanglish-lang`
- Internal design sources to mention:
- `README.md`
- `docs/ARCHITECTURE.md`
- `docs/CHEATSHEET.md`
- If asked why repo docs are listed too: "Because the presentation includes both literature review and the implemented target-language design, and the GitHub repository shows the public implementation context."

## Slide 10 - Motivation & Problem Statement

- Main point: explain why the language exists.
- Say: "The motivation is both accessibility and engineering. English-centric syntax creates additional cognitive overhead for non-English speakers, and compiler construction is a strong way to study the full language pipeline."
- If asked whether this is an accessibility claim or a compiler claim: "It is both."

## Slide 11 - Project Goals & Scope

- Main point: show the project has clear boundaries.
- Say: "The goals were to design a Myanglish language, build a multi-pass compiler in Rust, support backends, add tooling, support Myanmar numerals, and extend the language toward server-side programming."
- If asked about scope creep: "The project expanded in phases: foundations, modules and stdlib, then concurrency and networking."

## Slide 12 - Language Design: Keyword Mapping

- Main point: show the keywords are systematic and meaningful.
- Say: "Each keyword is derived from Burmese meaning but written in Myanglish for practical typing. The goal is semantic familiarity with lower typing friction."
- Point at examples like `kain`, `sar`, `loke`, `hlyin`, `pyan`, `kyoe`.
- If asked why not full Myanmar-script keywords: "Myanglish is more practical on common keyboards, while the compiler still supports Myanmar identifiers and numerals."

## Slide 13 - Type System

- Main point: show that M-Lang is a real statically typed language.
- Say: "M-Lang supports explicit primitive types, collection and composite types, tuple-style returns, function types, and typed channels."
- Mention explicit conversion helpers as a deliberate design choice.
- If asked about error handling: "Errors are represented explicitly through `amhar`, often in tuple-style return values similar to Go."

## Slide 14 - Code Examples

- Main point: prove the language can express real constructs.
- Say: "This slide samples practical features rather than toy syntax: structs, mutation, loops, modules, stdlib, HTTP, concurrency, and cleanup."
- Best presentation strategy: do not read every line. Explain what each snippet proves.
- Good mini-lines:
- Struct snippet proves mutation and methods.
- Classic `pat` proves flexible loop syntax.
- HTTP/JSON snippet proves tuple-style error handling and stdlib integration.
- Local package import proves `atote` and `pay`.
- HTTP server and channel snippets prove server-side capability.

## Slide 15 - Compiler Architecture

- Main point: explain the pipeline from source to executable.
- Say: "M-Lang follows a classic multi-pass compiler architecture: lexing, parsing, type checking, then backend code generation to Go or C, then the target toolchain produces the executable."
- Strong phrase: "one front end, multiple backends."
- If asked why multi-pass: "It keeps stages simpler, makes testing easier, and supports multiple targets from a shared validated representation."

## Slide 16 - Stage 1: Lexer

- Main point: explain tokenization and normalization.
- Say: "The lexer turns raw source into tokens with kind, value, line, and column. It also handles Myanglish keywords, Myanmar identifiers, and Myanmar numerals."
- Key example: mixed numeral `၂0` becomes the numeric value `20` during lexing.
- If asked why numeral normalization happens early: "It simplifies later stages because the parser and type checker can treat numeric literals uniformly."

## Slide 17 - Stage 2: Parser

- Main point: explain syntax-to-structure conversion.
- Say: "The parser is a recursive-descent parser with Pratt precedence handling for expressions. It builds the AST, which becomes the structural contract for later stages."
- Point out that the AST includes declarations, loops, imports, package constructs, concurrency statements, and expressions.
- If asked why Pratt parsing is used: "It is a clean way to handle operator precedence inside a recursive-descent parser."

## Slide 18 - Stage 3: Static Type Checker

- Main point: explain semantic validation.
- Say: "The type checker walks the AST with scoped environments and validates declarations, assignments, function calls, collections, package visibility, and import resolution."
- Use the `kain x = "hello";` example to explain early rejection of incorrect programs.
- If asked about scope handling: "The checker uses lexical environments with outer links, so nested scopes resolve naturally."

## Slide 19 - Stage 4: Code Generation

- Main point: show how valid programs become backend code.
- Say: "After semantic validation, the compiler emits backend source code. Go is the main backend because it provides the best runtime fit for strings, maps, Unicode, concurrency, and networking. C remains a legacy backend."
- Important nuance: the Go backend is the real modern target in this repo.
- If asked about performance: "Generated Go or C is compiled by the native toolchain, so this is not an interpreted runtime."

## Slide 20 - Developer Tooling Ecosystem

- Main point: show the language is usable in practice.
- Say: "M-Lang includes a formatter, a language server, and a VS Code extension. That means the compiler internals are reusable beyond the CLI."
- Explain the formatter briefly: parse plus pretty-print, not just text replacement.
- If asked what the LSP supports: "Diagnostics, hover, completion, definition, semantic tokens, and formatting."

## Slide 21 - Testing Strategy

- Main point: show validation breadth across the toolchain.
- Say: "The project tests the language in layers: lexer, parser, module loader, type checker, code generation, formatter, and LSP analysis."
- Use safe wording if the audience asks about exact counts: "The important point is broad subsystem coverage."
- If pressed on the exact number shown on the slide: say it reflects a presentation snapshot rather than the only thing that matters.

## Slide 22 - Technical Challenges & Solutions

- Main point: show the real engineering work behind the language.
- Say: "The main challenges were Unicode identifiers, Myanmar numeral parsing, expression precedence, backend differences, Go-specific constraints, and concurrency keyword lowering."
- Good short answers:
- Unicode identifiers: supported directly in Go, mangled in C.
- Numerals: normalized at lexing time.
- Unused variables in Go: handled by emitted `_ = var`.
- Dual backend complexity: both backends walk the same validated AST.

## Slide 23 - Roadmap & Current Status

- Main point: show phased progress and remaining work.
- Say: "Phase 1 built the core language, Phase 2 added modules and stdlib, Phase 3 added concurrency and networking, and Phase 4 is about production readiness."
- If asked what is still missing for production: "Dependency management, database integrations, broader ecosystem packaging, and more mature runtime-facing tooling."

## Slide 24 - Current Milestone: Networking + Concurrency Runtime

- Main point: prove the language goes beyond syntax experiments.
- Say: "This milestone is important because it shows that M-Lang can express server-side logic, not just basic academic programs."
- Point at `http.handle`, `http.listen`, `kyoe`, `laung`, and `naut_sone`.
- If asked whether this is just Go under another syntax: "The runtime model intentionally reuses Go semantics, but the source language, compiler, and tooling are all part of the research and implementation contribution."

## Slide 25 - Related Work & Comparison

- Main point: place M-Lang among other non-English or localized languages.
- Say: "Other localized languages exist, but many are interpreted, dynamically typed, or limited in tooling. M-Lang differentiates itself by combining localized syntax with static typing, native compilation, and a fuller developer workflow."
- Avoid overclaiming. Say "unusual" or "distinctive," not "the only one."

## Slide 26 - Conclusion

- Main point: restate the project contributions clearly.
- Say: "M-Lang combines localized syntax, static typing, a Rust compiler, Go-first native compilation, and practical tooling. The broader result is that non-English syntax does not have to mean weaker engineering quality."
- Strong closing line: "Accessibility-oriented language design can coexist with type safety, native compilation, and professional tooling."

## Slide 27 - Q&A

- Main point: invite both language-design and compiler questions.
- Good opener: "I am happy to answer questions about either the academic language design section or the implementation details."
- Good fallback line: "I can separate the answer into syntax design, semantics, and backend/runtime behavior."

## General Q&A Cheat Sheet

### Why Rust for the compiler?

- memory safety without garbage collection
- strong enums and pattern matching for AST and tokens
- good performance
- good fit for compiler architecture

Short answer:

"Rust is a strong fit for compilers because it gives memory safety, expressive data types for AST design, and strong performance without needing a runtime GC."

### Why is Go the reference language in this presentation?

Short answer:

"Because M-Lang targets Go by default and several core ideas in the language, especially packages, explicit errors, concurrency, and defer-style cleanup, align most closely with Go."

### Why transpile to Go instead of building a direct machine-code backend?

Short answer:

"Go gives a mature runtime and toolchain immediately. That let the project focus on language design, semantics, modules, networking, concurrency, and tooling."

### Do we need both Rust and Go to use this language?

Short answer:

"If you build the compiler from this repo, yes: Rust builds the compiler and Go builds M-Lang programs on the default backend. If you already have a prebuilt `mlang` compiler binary, then you can skip Rust but still need Go for the default target."

### Is M-Lang interpreted?

Short answer:

"No. It is transpiled to Go or C and then compiled by the target toolchain into a native executable."

### How are Myanmar numerals supported?

Short answer:

"The lexer recognizes Myanmar digits and normalizes them into standard numeric token values, so later compiler stages do not need special-case number logic."

### How are Myanmar identifiers supported?

Short answer:

"The lexer accepts Myanmar Unicode ranges in identifiers. The Go backend can preserve them directly, while the C backend needs name mangling."

### How does the module system work?

Short answer:

"`yu` imports a module, `atote` declares the package name, and `pay` marks exported top-level symbols. The compiler then resolves a module graph and checks visibility rules."

### How does error handling work?

Short answer:

"Errors are explicit values through `amhar`, often returned in tuples, so the type checker can validate the error-aware flow."

### How does concurrency work?

Short answer:

"`kyoe` lowers to a Go goroutine, `laung<T>` lowers to a typed channel, and `naut_sone` lowers to `defer`, so the concurrency model is intentionally direct."

### What are the current limitations?

- the Go backend is clearly stronger than the C backend
- some tooling behavior still depends on ongoing repo work
- production ecosystem completeness is still a roadmap item

Short answer:

"The biggest limitation is that the Go backend is the real modern target while the C backend is legacy. The ecosystem is still evolving beyond the core language and compiler."

### If the professor asks about current local repo status

Use this only if needed:

"The presentation describes the implemented architecture and language design. This local workspace currently contains in-progress compiler changes, so I would present the design decisions and validated deck rather than promise a live build from this exact checkout."
