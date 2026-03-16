"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";
import Card from "../ui/Card";

const concepts = [
  {
    title: "Compiled and native-oriented",
    desc: "The reference model compiles source code ahead of time and produces machine-executable programs through a standard toolchain.",
  },
  {
    title: "Statically typed",
    desc: "Types are checked before execution, which catches mismatches early and simplifies reasoning about program behavior.",
  },
  {
    title: "Imperative / procedural",
    desc: "Programs are organized around statements, variables, functions, control flow, and explicit effects on state.",
  },
  {
    title: "Concurrency-aware",
    desc: "The reference model treats concurrent execution and communication as first-class programming concepts rather than add-on libraries.",
  },
];

const whyReference = [
  "Go is the best practical reference language for this project because M-Lang transpiles to Go by default.",
  "M-Lang borrows the ideas of explicit typing, multiple return values, error-oriented flows, package organization, and concurrency semantics from Go.",
  "Using Go as the reference language keeps the academic review connected to the real implementation choices in this repository.",
];

export default function ReferenceParadigmSlide() {
  return (
    <div>
      <SlideHeader number="03" title="Reference Programming Paradigm Review" />
      <div className="grid grid-cols-1 xl:grid-cols-[1.25fr_0.95fr] gap-6">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {concepts.map((concept, index) => (
            <Card key={concept.title} delay={index}>
              <h3 className="text-[0.92rem] font-bold mb-2">{concept.title}</h3>
              <p className="text-[0.82rem] text-text-secondary leading-relaxed">{concept.desc}</p>
            </Card>
          ))}
        </div>

        <div className="bg-bg-card border border-white/10 rounded-2xl p-6">
          <p className="text-[0.72rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Why This Reference Model</p>
          <div className="space-y-3">
            {whyReference.map((item, index) => (
              <motion.div
                key={item}
                initial={{ opacity: 0, x: 12 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.15 + index * 0.08 }}
                className="flex items-start gap-2"
              >
                <span className="text-accent-green font-bold shrink-0">+</span>
                <p className="text-[0.83rem] text-text-secondary leading-relaxed">{item}</p>
              </motion.div>
            ))}
          </div>

          <div className="mt-5 pt-4 border-t border-white/10">
            <p className="text-[0.72rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-3">Reference Language Used in This Review</p>
            <div className="inline-flex items-center px-4 py-2 rounded-lg border border-accent-cyan/25 bg-accent-cyan/[0.06] text-accent-cyan font-semibold">
              Go
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
