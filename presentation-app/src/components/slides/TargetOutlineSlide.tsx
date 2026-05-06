"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";
import Card from "../ui/Card";

const mappings = [
  { reference: "package / import", target: "atote / yu", note: "Module identity and import syntax are localized." },
  { reference: "func / return", target: "loke / pyan", note: "Function structure is preserved but keywords become Myanglish." },
  { reference: "error return", target: "amhar", note: "Explicit error-aware programming remains part of the design." },
  { reference: "go / chan / defer", target: "kyoe / laung / naut_sone", note: "Concurrency semantics are preserved with localized keywords." },
];

export default function TargetOutlineSlide() {
  return (
    <div>
      <SlideHeader number="07" title="Target New Language Outline" />
      <p className="text-text-secondary text-[0.93rem] max-w-[780px] mb-6">
        M-Lang keeps the rigor of the reference language model while changing the surface syntax into Myanglish. The design goal is not to invent a completely
        unrelated language, but to localize the programmer-facing syntax while preserving strong static semantics and practical backend behavior.
      </p>

      <div className="grid grid-cols-1 xl:grid-cols-[1.05fr_1fr] gap-6">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {[
            {
              title: "Localized keywords",
              desc: "Core constructs are renamed into Myanglish keywords such as kain, sar, loke, pya, hlyin, and pyan.",
            },
            {
              title: "Static type system",
              desc: "The language still uses explicit declared types, typed collections, typed functions, tuples, and typed channels.",
            },
            {
              title: "Multi-pass compiler",
              desc: "The syntax is implemented through lexer, parser, type checker, and backend code generation rather than simple text substitution.",
            },
            {
              title: "Native-first backend model",
              desc: "The default backend emits LLVM IR for native executables, while Go remains available for packages, networking, and concurrency interop.",
            },
          ].map((item, index) => (
            <Card key={item.title} delay={index}>
              <h3 className="text-[0.9rem] font-bold mb-2">{item.title}</h3>
              <p className="text-[0.82rem] text-text-secondary leading-relaxed">{item.desc}</p>
            </Card>
          ))}
        </div>

        <div className="rounded-2xl border border-white/10 bg-bg-card p-5">
          <p className="text-[0.72rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Reference to Target Mapping</p>
          <div className="space-y-3">
            {mappings.map((mapping, index) => (
              <motion.div
                key={mapping.reference}
                initial={{ opacity: 0, x: 10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.08 }}
                className="rounded-xl border border-white/10 bg-bg-secondary/40 px-4 py-3"
              >
                <div className="grid grid-cols-[1fr_24px_1fr] items-center gap-2 text-[0.83rem] font-mono">
                  <span className="text-accent-cyan">{mapping.reference}</span>
                  <span className="text-center text-text-muted">-&gt;</span>
                  <span className="text-accent-green">{mapping.target}</span>
                </div>
                <p className="mt-2 text-[0.76rem] text-text-secondary leading-relaxed">{mapping.note}</p>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
