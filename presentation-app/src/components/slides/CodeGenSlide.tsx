"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";
import CodeBlock from "../ui/CodeBlock";

export default function CodeGenSlide() {
  return (
    <div>
      <SlideHeader number="19" title="Stage 4 — Code Generation" />
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6 xl:gap-10">
        {/* Go Backend */}
        <div>
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            className="flex items-center gap-3 p-4 rounded-xl mb-4 bg-accent-cyan/[0.06] border border-accent-cyan/15"
          >
            <span className="text-2xl">🔵</span>
            <div>
              <h3 className="text-lg font-bold">Go Backend <span className="text-[0.6rem] font-semibold px-2 py-0.5 rounded-full bg-accent-green/15 text-accent-green border border-accent-green/25 align-middle ml-1">Default</span></h3>
              <span className="text-[0.7rem] font-mono text-text-muted">codegen_go.rs · ~110 KB</span>
            </div>
          </motion.div>
          <ul className="list-none space-y-1">
            {[
              "Emits <code>package main</code> with auto imports",
              "Module graph is flattened with stable <code>pkg__symbol</code> mangling",
              "Supports <code>atote</code> package declarations and <code>pay</code> exports",
              "Unicode identifiers used directly",
              "String concat → native <code>+</code>",
              "Arrays → Go slices",
              "HashMaps → Go maps",
              "While / for-in / classic for all lower to Go <code>for</code>",
              "<code>kyoe</code> → <code>go</code>, <code>laung</code> → <code>chan</code>, <code>naut_sone</code> → <code>defer</code>",
              "HTTP server shims: <code>handle</code>, <code>listen</code>, <code>Request</code>, <code>ResponseWriter</code>",
              "Emits <code>_ = var</code> for Go's unused rule",
            ].map((f, i) => (
              <motion.li key={i} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: i * 0.05 }} className="text-[0.8rem] text-text-secondary flex items-start gap-2 py-0.5">
                <span className="text-accent-blue font-bold shrink-0">→</span>
                <span dangerouslySetInnerHTML={{ __html: f }} />
              </motion.li>
            ))}
          </ul>
          <CodeBlock size="small" className="mt-3">
            <span className="cm">{"// Generated Go code"}</span>{"\n"}
            <span className="kw">package</span> main{"\n"}
            <span className="kw">import</span> <span className="str">&quot;fmt&quot;</span>{"\n\n"}
            <span className="kw">func</span> <span className="fn">main</span>() {"{"}{"\n"}
            {"    "}age := <span className="kw">int64</span>(<span className="num">20</span>){"\n"}
            {"    "}_ = age{"\n"}
            {"    "}<span className="fn">fmt.Println</span>(<span className="str">&quot;Hello!&quot;</span>){"\n"}
            {"}"}
          </CodeBlock>
        </div>

        {/* C Backend */}
        <div>
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
            className="flex items-center gap-3 p-4 rounded-xl mb-4 bg-accent-orange/[0.06] border border-accent-orange/15"
          >
            <span className="text-2xl">🟠</span>
            <div>
              <h3 className="text-lg font-bold">C Backend <span className="text-[0.6rem] font-semibold px-2 py-0.5 rounded-full bg-accent-orange/15 text-accent-orange border border-accent-orange/25 align-middle ml-1">Legacy</span></h3>
              <span className="text-[0.7rem] font-mono text-text-muted">codegen.rs · ~25 KB</span>
            </div>
          </motion.div>
          <ul className="list-none space-y-1">
            {[
              "Auto-includes stdio, stdbool, string, stdlib",
              "Runtime: <code>mlang_concat()</code>",
              "Runtime: <code>mlang_read_input()</code>",
              "<strong>Identifier mangling</strong> for non-ASCII",
              "Type-appropriate <code>printf</code> specifiers",
              "HashMap support limited / placeholder",
              "<strong>Rejects local module system</strong> (<code>atote</code>, <code>pay</code>, relative <code>yu</code>)",
              "Use Go backend for multi-file package compilation",
            ].map((f, i) => (
              <motion.li key={i} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.1 + i * 0.05 }} className="text-[0.8rem] text-text-secondary flex items-start gap-2 py-0.5">
                <span className="text-accent-blue font-bold shrink-0">→</span>
                <span dangerouslySetInnerHTML={{ __html: f }} />
              </motion.li>
            ))}
          </ul>
          <CodeBlock size="small" className="mt-3">
            <span className="cm">{"// Generated C code"}</span>{"\n"}
            <span className="pp">#include</span> <span className="str">&lt;stdio.h&gt;</span>{"\n"}
            <span className="pp">#include</span> <span className="str">&lt;stdbool.h&gt;</span>{"\n\n"}
            <span className="kw">int</span> <span className="fn">main</span>() {"{"}{"\n"}
            {"    "}<span className="kw">long long</span> age = <span className="num">20</span>;{"\n"}
            {"    "}<span className="fn">printf</span>(<span className="str">&quot;%s\n&quot;</span>, <span className="str">&quot;Hello!&quot;</span>);{"\n"}
            {"    "}<span className="kw">return</span> <span className="num">0</span>;{"\n"}
            {"}"}
          </CodeBlock>
        </div>
      </div>
    </div>
  );
}
