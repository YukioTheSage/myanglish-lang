"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

const astNodeTypes = [
  "Let / LetDestructured",
  "Assign / FieldAssign / IndexAssign",
  "FunctionDecl",
  "If / Else",
  "While / ForIn / ForClassic",
  "Break / Continue",
  "PackageDecl / Import / Export",
  "Return / Print",
  "StructDecl / MethodDecl / InterfaceDecl",
  "Goroutine (kyoe)",
  "ChannelMake (laung)",
  "Defer (naut_sone)",
];

const precedence = [
  { level: "5", desc: "Calls, index, slices, field/method access" },
  { level: "4", ops: "* /" },
  { level: "3", ops: "+ -" },
  { level: "2", ops: "< > <= >=" },
  { level: "1", ops: "== !=" },
];

export default function ParserSlide() {
  return (
    <div>
      <SlideHeader number="17" title="Stage 2 — Parser" badge="parser.rs · ~77 KB" badgeType="file" />
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 lg:gap-10">
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Design</h3>
          <ul className="list-none space-y-1.5 mb-6">
            {[
              "<strong>Recursive-descent</strong> parser with <strong>Pratt precedence climbing</strong>",
              "Produces a fully-typed AST defined in <code>ast.rs</code>",
              "<strong>Error recovery</strong>: collects all errors without halting",
            ].map((f, i) => (
              <motion.li key={i} initial={{ opacity: 0, x: -10 }} animate={{ opacity: 1, x: 0 }} transition={{ delay: i * 0.1 }} className="text-[0.85rem] text-text-secondary flex items-start gap-2">
                <span className="text-accent-blue font-bold shrink-0">→</span>
                <span dangerouslySetInnerHTML={{ __html: f }} />
              </motion.li>
            ))}
          </ul>

          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">AST Node Types</h3>
          <div className="flex flex-wrap gap-1.5 mb-6">
            {astNodeTypes.map((t) => (
              <span key={t} className="text-[0.7rem] font-medium px-2.5 py-1 bg-bg-card border border-white/10 rounded-full text-text-secondary">{t}</span>
            ))}
          </div>

          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Operator Precedence</h3>
          <div className="flex flex-col gap-1">
            {precedence.map((p, i) => (
              <motion.div key={i} initial={{ opacity: 0, x: -10 }} animate={{ opacity: 1, x: 0 }} transition={{ delay: 0.3 + i * 0.05 }} className="flex items-center gap-2.5 text-[0.8rem] text-text-secondary">
                <span className="w-6 h-6 flex items-center justify-center bg-bg-card border border-white/10 rounded-full text-[0.7rem] font-bold text-accent-orange font-mono shrink-0">{p.level}</span>
                {p.ops ? <code className="font-mono text-accent-blue text-[0.8rem]">{p.ops}</code> : <span>{p.desc}</span>}
              </motion.div>
            ))}
          </div>
        </div>
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Example AST Structure</h3>
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.2 }} className="font-mono text-[0.75rem]">
            <div className="px-3 py-1.5 rounded-md mb-1 border-l-[3px] bg-accent-purple/10 border-l-accent-purple text-accent-purple font-bold">Program</div>
            <div className="pl-5 border-l border-dashed border-white/15 ml-2.5">
              <div className="px-3 py-1.5 rounded-md mb-1 border-l-[3px] bg-accent-blue/10 border-l-accent-blue text-accent-blue font-semibold">FunctionDecl &quot;main&quot;</div>
              <div className="pl-5 border-l border-dashed border-white/15 ml-2.5">
                <div className="px-3 py-1.5 rounded-md mb-1 border-l-[3px] bg-accent-cyan/[0.06] border-l-accent-cyan text-accent-cyan">{'Let { name: "age", type: Kain }'}</div>
                <div className="px-3 py-1.5 rounded-md mb-1 border-l-[3px] bg-accent-cyan/[0.06] border-l-accent-cyan text-accent-cyan">{'Let { name: "name", type: Sar }'}</div>
                <div className="px-3 py-1.5 rounded-md mb-1 border-l-[3px] bg-accent-pink/[0.08] border-l-accent-pink text-accent-pink font-semibold">If</div>
                <div className="pl-5 border-l border-dashed border-white/15 ml-2.5">
                  <div className="px-3 py-1.5 rounded-md mb-1 border-l-[3px] bg-accent-orange/[0.08] border-l-accent-orange text-accent-orange">{'BinaryOp { op: >, left: "age", right: 18 }'}</div>
                  <div className="px-3 py-1.5 rounded-md mb-1 border-l-[3px] bg-accent-cyan/[0.06] border-l-accent-cyan text-accent-cyan">{'Print { expr: "Hello World!" }'}</div>
                </div>
                <div className="px-3 py-1.5 rounded-md mb-1 border-l-[3px] bg-accent-cyan/[0.06] border-l-accent-cyan text-accent-cyan">{'Return { expr: 0 }'}</div>
              </div>
            </div>
          </motion.div>
        </div>
      </div>
    </div>
  );
}
