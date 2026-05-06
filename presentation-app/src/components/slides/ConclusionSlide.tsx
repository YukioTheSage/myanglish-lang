"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

const items = [
  { num: 1, title: "Language Design", desc: "A complete statically-typed language with Myanglish keywords and Myanmar numeral support" },
  { num: 2, title: "Multi-Pass Compiler", desc: "Full pipeline (Lexer → Parser → Type Checker → Module Loader → Code Gen) implemented in Rust" },
  { num: 3, title: "Native Backend", desc: "LLVM is the default compiler path, with Go retained for stdlib/server interop and C frozen as legacy" },
  { num: 4, title: "Tooling Ecosystem", desc: "Formatter, LSP server, and VS Code extension with format-on-save workflow" },
  { num: 5, title: "Server-Side Runtime", desc: "Goroutines (kyoe), channels (laung), defer (naut_sone), HTTP server, and TCP/UDP sockets — all production-ready" },
];

export default function ConclusionSlide() {
  return (
    <div style={{ background: "radial-gradient(ellipse at 50% 50%, rgba(96,165,250,0.05) 0%, transparent 60%)" }}>
      <SlideHeader number="26" title="Conclusion" centered />
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4 my-8">
        {items.map((item, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.1 }}
            whileHover={{ y: -4, boxShadow: "0 0 30px rgba(96,165,250,0.15)" }}
            className="bg-bg-card border border-white/10 rounded-xl p-6 text-center relative overflow-hidden"
          >
            <div className="absolute top-0 left-0 w-full h-[3px]" style={{ background: "linear-gradient(135deg, #60a5fa, #a78bfa, #f472b6)" }} />
            <span className="inline-flex items-center justify-center w-8 h-8 bg-accent-blue/10 rounded-full text-[0.85rem] font-bold text-accent-blue font-mono mb-3">
              {item.num}
            </span>
            <h4 className="text-[0.9rem] font-bold mb-2">{item.title}</h4>
            <p className="text-[0.78rem] text-text-secondary leading-relaxed">{item.desc}</p>
          </motion.div>
        ))}
      </div>
      <motion.div
        initial={{ opacity: 0, y: 10 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.6 }}
        className="max-w-[800px] mx-auto p-6 bg-accent-blue/[0.04] border border-accent-blue/15 border-l-[3px] border-l-accent-blue rounded-xl text-[0.95rem] leading-relaxed text-text-secondary"
      >
        M-Lang demonstrates that a programming language with non-English keywords can achieve the same rigor — <em className="text-text-primary italic">static typing, native compilation, and professional tooling</em> — as mainstream languages, while making programming more accessible to Burmese-speaking communities.
      </motion.div>
    </div>
  );
}
