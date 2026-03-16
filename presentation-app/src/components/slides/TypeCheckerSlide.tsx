"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";
import CodeBlock from "../ui/CodeBlock";

const checks = [
  "Type consistency in declarations",
  "Undeclared variables & functions",
  "Boolean conditions in if/while/for",
  "Function call arity & arg types",
  "Array element homogeneity",
  "Integer-only array indexing",
  "HashMap key/value consistency",
  "String concatenation validity",
  "Return type correctness",
  "Struct field validation",
  "Method receiver types",
  "C-style for loop init/cond/post typing",
  "Unknown imports are explicit errors",
  "Cross-package visibility via pay exports",
  "Qualified symbol resolution (pkg.symbol)",
];

export default function TypeCheckerSlide() {
  return (
    <div>
      <SlideHeader number="18" title="Stage 3 — Static Type Checker" badge="typecheck.rs · ~66 KB" badgeType="file" />
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 lg:gap-10">
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Approach</h3>
          <ul className="list-none space-y-1.5 mb-4">
            <li className="text-[0.85rem] text-text-secondary flex items-start gap-2">
              <span className="text-accent-blue font-bold shrink-0">→</span>
              <span>AST-walking type checker with <strong className="text-text-primary">scoped environments</strong></span>
            </li>
            <li className="text-[0.85rem] text-text-secondary flex items-start gap-2">
              <span className="text-accent-blue font-bold shrink-0">→</span>
              <span>Environment uses <code className="bg-accent-blue/10 text-accent-blue px-1 rounded font-mono text-[0.8rem]">outer</code> pointers for <strong className="text-text-primary">lexical scoping</strong></span>
            </li>
          </ul>

          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Checks Performed</h3>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-x-3 gap-y-1">
            {checks.map((c, i) => (
              <motion.div key={i} initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: i * 0.04 }} className="text-[0.8rem] text-text-secondary py-0.5">
                <span className="text-accent-green font-bold mr-1.5">✓</span>{c}
              </motion.div>
            ))}
          </div>
        </div>
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Error Reporting Example</h3>
          <CodeBlock size="small">
            <span className="cm">{"// Source code with type error"}</span>{"\n"}
            <span className="kw">kain</span> x = <span className="str">&quot;hello&quot;</span>;  <span className="err">← ERROR</span>
          </CodeBlock>
          <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.3 }}
            className="bg-accent-red/[0.06] border border-accent-red/20 rounded-lg p-4 mt-2"
          >
            <div className="flex items-center gap-1.5 text-[0.85rem] mb-1.5">
              <span className="text-accent-red font-bold">✗</span>
              <span>Type error at <strong>line 1, col 10</strong>:</span>
            </div>
            <div className="text-[0.8rem] text-accent-red font-mono pl-5">
              Cannot assign value of type <code className="bg-accent-red/10 px-1 rounded">Sar</code> to variable of type <code className="bg-accent-red/10 px-1 rounded">Kain</code>
            </div>
          </motion.div>

          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4 mt-6">Scoping Model</h3>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.4 }}
            className="text-[0.8rem]"
          >
            <div className="border border-dashed border-accent-purple/30 bg-accent-purple/[0.03] rounded-lg p-3">
              <span className="text-[0.7rem] font-semibold text-text-muted block mb-1.5">Global Scope</span>
              <div className="border border-dashed border-accent-blue/30 bg-accent-blue/[0.03] rounded-lg p-3">
                <span className="text-[0.7rem] font-semibold text-text-muted block mb-1.5">Function &quot;main&quot;</span>
                <div className="border border-dashed border-accent-cyan/30 bg-accent-cyan/[0.03] rounded-lg p-3">
                  <span className="text-[0.7rem] font-semibold text-text-muted block mb-1">If Block</span>
                  <span className="text-[0.65rem] text-accent-orange italic">← outer pointer</span>
                </div>
                <span className="text-[0.65rem] text-accent-orange italic mt-1 block">← outer pointer</span>
              </div>
            </div>
          </motion.div>
        </div>
      </div>
    </div>
  );
}
