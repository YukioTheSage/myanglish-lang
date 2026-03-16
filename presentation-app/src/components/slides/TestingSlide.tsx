"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

const testItems = [
  { name: "Lexer Tests", desc: "Keywords, Myanmar numerals, comments, operators, arrays/maps, and token stream correctness" },
  { name: "Parser Tests", desc: "Destructuring, break/continue, for-in index loops, C-style for loops, atote/pay, imports, and assignment variants" },
  { name: "Module Loader Tests", desc: "Relative local import resolution, cycle detection, missing file errors, duplicate package rejection, and export visibility" },
  { name: "Type Checker Tests", desc: "Tuple/error flows, loop control validation, unknown import failures, package visibility, and mismatch diagnostics" },
  { name: "Go CodeGen Tests", desc: "End-to-end M-Lang → Go including multi-file module flattening, pkg__symbol mangling, structs, stdlib helpers, and control flow" },
  { name: "C CodeGen Tests", desc: "Legacy backend coverage, plus clear rejection of local package/module features" },
];

const metrics = [
  { component: "Lexer", size: "covered" },
  { component: "Parser", size: "covered" },
  { component: "Type Checker", size: "covered" },
  { component: "Module Loader", size: "covered" },
  { component: "Go Code Generator", size: "covered" },
  { component: "C Code Generator", size: "covered" },
  { component: "Formatter", size: "covered" },
  { component: "LSP Analysis", size: "covered" },
];

export default function TestingSlide() {
  return (
    <div>
      <SlideHeader number="21" title="Testing Strategy" />
      <div className="grid grid-cols-1 xl:grid-cols-2 gap-6 xl:gap-10">
        <div>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            className="bg-bg-code border border-white/10 rounded-xl px-6 py-4 font-mono text-[0.9rem] text-accent-green mb-6"
          >
            <span className="text-accent-blue mr-2">$</span> cargo test <span className="text-text-muted"># 85 tests passing</span>
          </motion.div>

          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Test Coverage</h3>
          <div className="flex flex-col gap-3">
            {testItems.map((item, i) => (
              <motion.div
                key={i}
                initial={{ opacity: 0, y: 10 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: i * 0.1 }}
                className="bg-bg-card border border-white/10 rounded-lg p-4"
              >
                <div className="flex items-center gap-2 mb-1.5">
                  <span className="text-accent-green font-bold">✓</span>
                  <h4 className="text-[0.9rem] font-bold">{item.name}</h4>
                </div>
                <p className="text-[0.78rem] text-text-secondary leading-relaxed">{item.desc}</p>
              </motion.div>
            ))}
          </div>
        </div>
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Implementation Metrics</h3>
          <div className="border border-white/10 rounded-xl overflow-hidden">
            <div className="flex justify-between px-4 py-2.5 bg-bg-secondary text-[0.7rem] font-bold uppercase tracking-wider text-text-muted border-b border-white/10">
              <span>Component</span><span>Status</span>
            </div>
            {metrics.map((m, i) => (
              <motion.div
                key={i}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: 0.2 + i * 0.05 }}
                className="flex justify-between px-4 py-2.5 text-[0.85rem] border-b border-white/10"
              >
                <span>{m.component}</span>
                <span className="font-mono text-accent-cyan">{m.size}</span>
              </motion.div>
            ))}
            <div className="flex justify-between px-4 py-2.5 bg-accent-blue/[0.06] font-bold text-accent-blue text-[0.85rem]">
              <span>Tested Surface</span><span>Frontend + Backend</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
