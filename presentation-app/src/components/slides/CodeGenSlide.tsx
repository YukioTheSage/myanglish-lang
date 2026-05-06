"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";
import CodeBlock from "../ui/CodeBlock";

export default function CodeGenSlide() {
  return (
    <div>
      <SlideHeader number="19" title="Stage 4 — Code Generation" />
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6 xl:gap-10">
        {/* LLVM Backend */}
        <div>
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex items-center gap-3 p-4 rounded-xl mb-4 bg-accent-cyan/[0.06] border border-accent-cyan/15"
          >
            <span className="text-2xl">🟣</span>
            <div>
              <h3 className="text-lg font-bold">LLVM Backend <span className="text-[0.6rem] font-semibold px-2 py-0.5 rounded-full bg-accent-green/15 text-accent-green border border-accent-green/25 align-middle ml-1">Default Native</span></h3>
              <span className="text-[0.7rem] font-mono text-text-muted">codegen_llvm.rs · native compiler path</span>
            </div>
          </motion.div>
          <ul className="list-none space-y-1">
            {[
              "Default path: <code>.ml → AST → LLVM IR → object → executable</code>",
              "Emits inspectable <code>.ll</code> files for compiler review",
              "Compiles IR with <code>llc</code> or <code>clang</code>",
              "Links a small native runtime from <code>runtime_llvm.c</code>",
              "Phase 1 examples compile and run through native e2e tests",
              "Clear diagnostics redirect Go-only stdlib/server features to <code>--target go</code>",
            ].map((f, i) => (
              <motion.li key={i} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: i * 0.05 }} className="text-[0.8rem] text-text-secondary flex items-start gap-2 py-0.5">
                <span className="text-accent-blue font-bold shrink-0">→</span>
                <span dangerouslySetInnerHTML={{ __html: f }} />
              </motion.li>
            ))}
          </ul>
          <CodeBlock size="small" className="mt-3">
            <span className="cm">{"; Generated LLVM IR"}</span>{"\n"}
            <span className="kw">define</span> i64 <span className="fn">@main</span>() {"{"}{"\n"}
            entry:{"\n"}
            {"  "}%i.addr = <span className="fn">alloca</span> i64{"\n"}
            {"  "}<span className="fn">store</span> i64 <span className="num">0</span>, i64* %i.addr{"\n"}
            {"  "}<span className="fn">ret</span> i64 <span className="num">0</span>{"\n"}
            {"}"}
          </CodeBlock>
        </div>

        {/* Go Interop Backend */}
        <div>
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="flex items-center gap-3 p-4 rounded-xl mb-4 bg-accent-cyan/[0.06] border border-accent-cyan/15"
          >
            <span className="text-2xl">🔵</span>
            <div>
              <h3 className="text-lg font-bold">Go Interop Backend <span className="text-[0.6rem] font-semibold px-2 py-0.5 rounded-full bg-accent-cyan/15 text-accent-cyan border border-accent-cyan/25 align-middle ml-1">Stdlib</span></h3>
              <span className="text-[0.7rem] font-mono text-text-muted">codegen_go.rs · full server surface</span>
            </div>
          </motion.div>
          <ul className="list-none space-y-1">
            {[
              "Maintains the full Phase 2/3/4 stdlib and server runtime",
              "Module graph is flattened with stable <code>pkg__symbol</code> mangling",
              "Supports <code>atote</code> package declarations and <code>pay</code> exports",
              "Unicode identifiers used directly",
              "Arrays → Go slices; HashMaps → Go maps",
              "<code>kyoe</code> → <code>go</code>, <code>laung</code> → <code>chan</code>, <code>naut_sone</code> → <code>defer</code>",
              "HTTP server shims: <code>handle</code>, <code>listen</code>, <code>Request</code>, <code>ResponseWriter</code>",
              "Legacy C backend remains available for the older subset",
            ].map((f, i) => (
              <motion.li key={i} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.1 + i * 0.05 }} className="text-[0.8rem] text-text-secondary flex items-start gap-2 py-0.5">
                <span className="text-accent-blue font-bold shrink-0">→</span>
                <span dangerouslySetInnerHTML={{ __html: f }} />
              </motion.li>
            ))}
          </ul>
          <CodeBlock size="small" className="mt-3">
            <span className="cm">{"// Generated Go interop code"}</span>{"\n"}
            <span className="kw">package</span> main{"\n"}
            <span className="kw">import</span> <span className="str">&quot;fmt&quot;</span>{"\n\n"}
            <span className="kw">func</span> <span className="fn">main</span>() {"{"}{"\n"}
            {"    "}age := <span className="kw">int64</span>(<span className="num">20</span>){"\n"}
            {"    "}_ = age{"\n"}
            {"    "}<span className="fn">fmt.Println</span>(<span className="str">&quot;Hello!&quot;</span>){"\n"}
            {"}"}
          </CodeBlock>
        </div>
      </div>
    </div>
  );
}
