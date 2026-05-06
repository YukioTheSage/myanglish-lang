"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

const goals = [
  { num: "01", title: "Myanglish Language", desc: "Design a statically-typed language with romanized Burmese keywords (kain, sar, loke, pya)" },
  { num: "02", title: "Multi-Pass Compiler", desc: "Build a full pipeline in Rust: Lexer → Parser → Type Checker → Code Generator" },
  { num: "03", title: "Native Compiler Backend", desc: "Compile to LLVM IR by default, then produce object code and native executables" },
  { num: "04", title: "Developer Tooling", desc: "Code formatter (mlang fmt), LSP server, and VS Code extension with syntax highlighting" },
  { num: "05", title: "Myanmar Numerals", desc: "Support Myanmar digits ၀–၉ (U+1040–U+1049) alongside ASCII 0–9, freely mixable" },
  { num: "06", title: "Go Interop Runtime", desc: "Goroutines, channels, defer, HTTP server, and TCP/UDP sockets remain available on --target go" },
];

export default function GoalsSlide() {
  return (
    <div>
      <SlideHeader number="11" title="Project Goals & Scope" />
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4 sm:gap-5">
        {goals.map((goal, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.08 }}
            whileHover={{ backgroundColor: "rgba(31,43,61,1)", borderColor: "rgba(96,165,250,0.3)", y: -2, boxShadow: "0 0 30px rgba(96,165,250,0.15)" }}
            className="bg-bg-card border border-white/10 rounded-xl p-5 sm:p-6 relative overflow-hidden group"
          >
            <div className="absolute top-0 left-0 w-full h-[3px] opacity-0 group-hover:opacity-100 transition-opacity" style={{ background: "linear-gradient(135deg, #60a5fa, #a78bfa, #f472b6)" }} />
            <div className="text-[2rem] font-black font-mono gradient-text opacity-40 mb-3">{goal.num}</div>
            <h4 className="text-base font-bold mb-2">{goal.title}</h4>
            <p className="text-[0.85rem] text-text-secondary leading-relaxed">{goal.desc}</p>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
