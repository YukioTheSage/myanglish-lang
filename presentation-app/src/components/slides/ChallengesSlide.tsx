"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

const challenges = [
  { icon: "🔤", title: "Myanmar Unicode in Identifiers", solution: "Lexer recognizes U+1000–U+109F; Go uses them natively; C backend hex-encodes to mlang_ prefix" },
  { icon: "🔢", title: "Myanmar Numeral Parsing", solution: "Lexer maps ၀–၉ (U+1040–U+1049) to 0–9; allows mixing: kain x = ၂0; → 20" },
  { icon: "⚖️", title: "Expression Precedence", solution: "Pratt precedence climbing algorithm ensures mathematically correct parse order" },
  { icon: "⚠️", title: "Go's Unused Variable Rule", solution: "CodeGen emits _ = varname after every declaration to satisfy Go compiler" },
  { icon: "🔗", title: "String Concat Across Backends", solution: "Go: native + operator; C: custom mlang_concat() with heap allocation" },
  { icon: "🔀", title: "Dual Backend Maintenance", solution: "Shared AST definition; each backend independently walks the same validated tree" },
  { icon: "🧵", title: "Concurrency Keyword Lowering", solution: "kyoe/laung/naut_sone map 1:1 to Go's go/chan/defer — zero abstraction cost with Myanglish syntax" },
  { icon: "🌐", title: "HTTP Server Shim Design", solution: "Stdlib shim bridges M-Lang handler signatures to Go net/http; Request/ResponseWriter wrap Go types" },
];

export default function ChallengesSlide() {
  return (
    <div>
      <SlideHeader number="22" title="Technical Challenges & Solutions" />
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
        {challenges.map((ch, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 15 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.08 }}
            whileHover={{ backgroundColor: "rgba(31,43,61,1)", borderColor: "rgba(96,165,250,0.3)" }}
            className="bg-bg-card border border-white/10 rounded-xl p-5 transition-all"
          >
            <div className="flex items-center gap-2.5 mb-3">
              <span className="text-xl">{ch.icon}</span>
              <h4 className="text-[0.9rem] font-bold">{ch.title}</h4>
            </div>
            <div className="text-[0.8rem] text-text-secondary leading-relaxed pl-8">
              <span className="text-accent-green font-semibold text-[0.7rem] uppercase tracking-wide">Solution: </span>
              {ch.solution}
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
