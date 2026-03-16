"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

function ConceptNode({ label, detail, delay }: { label: string; detail: string; delay: number }) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.94 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ delay }}
      className="w-full rounded-xl border border-white/10 bg-bg-card px-4 py-4 text-center"
    >
      <span className="block text-[0.9rem] font-bold mb-1">{label}</span>
      <span className="block text-[0.75rem] text-text-muted">{detail}</span>
    </motion.div>
  );
}

export default function ReferenceContextSlide() {
  return (
    <div>
      <SlideHeader number="04" title="Reference Context & Concept View" />
      <p className="text-text-secondary text-[0.93rem] max-w-[760px] mb-6">
        For this presentation, Go is used as the reference language because it matches the main implementation direction of M-Lang: a statically typed,
        compiled, package-oriented language with clear concurrency and networking semantics.
      </p>

      <div className="rounded-2xl border border-white/10 bg-bg-secondary/30 p-5 sm:p-6 mb-6">
        <div className="grid grid-cols-1 md:grid-cols-5 gap-3 items-center">
          <ConceptNode label="Source File" detail="syntax and declarations" delay={0.05} />
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.1 }} className="text-center text-text-muted font-mono">
            -&gt;
          </motion.div>
          <ConceptNode label="Package + Imports" detail="modular program structure" delay={0.15} />
          <motion.div initial={{ opacity: 0 }} animate={{ opacity: 1 }} transition={{ delay: 0.2 }} className="text-center text-text-muted font-mono">
            -&gt;
          </motion.div>
          <ConceptNode label="Compiled Binary" detail="native execution target" delay={0.25} />
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-3 items-center mt-4">
          <ConceptNode label="Static Types" detail="checked before run time" delay={0.3} />
          <ConceptNode label="Functions + Control Flow" detail="procedural core model" delay={0.35} />
          <ConceptNode label="Concurrency Primitives" detail="goroutines, channels, defer" delay={0.4} />
        </div>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        {[
          {
            title: "Context description",
            desc: "The academic review starts from a mainstream compiled language so the new language can be justified against a concrete design baseline.",
          },
          {
            title: "Concept view",
            desc: "The diagram shows the reference model as source code plus packages, then compilation, then execution with structured control flow and concurrency.",
          },
          {
            title: "Why it matters",
            desc: "This lets the presentation explain both what M-Lang changes and what core semantics it intentionally keeps stable.",
          },
        ].map((item, index) => (
          <motion.div
            key={item.title}
            initial={{ opacity: 0, y: 12 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 + index * 0.08 }}
            className="rounded-xl border border-white/10 bg-bg-card p-4"
          >
            <h3 className="text-[0.88rem] font-bold mb-2">{item.title}</h3>
            <p className="text-[0.8rem] text-text-secondary leading-relaxed">{item.desc}</p>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
