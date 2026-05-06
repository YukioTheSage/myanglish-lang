"use client";
import { motion } from "framer-motion";

const stagger = {
  animate: { transition: { staggerChildren: 0.1 } },
};
const fadeUp = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
};

export default function TitleSlide() {
  return (
    <motion.div
      variants={stagger}
      initial="initial"
      animate="animate"
      className="flex flex-col items-center text-center min-h-[80vh] justify-center relative px-2"
    >
      {/* Background pattern */}
      <div
        className="absolute inset-0 pointer-events-none"
        style={{
          background: `radial-gradient(ellipse at 30% 50%, rgba(96,165,250,0.08) 0%, transparent 60%),
                       radial-gradient(ellipse at 70% 40%, rgba(167,139,250,0.06) 0%, transparent 50%)`,
        }}
      />

      <motion.div
        variants={fadeUp}
        className="text-[0.64rem] sm:text-[0.75rem] font-semibold uppercase tracking-[0.18em] sm:tracking-[0.2em] text-accent-cyan border border-accent-cyan/30 px-4 sm:px-5 py-1.5 rounded-full mb-6 sm:mb-8 bg-accent-cyan/5 relative"
      >
        Programming Language Design &amp; Implementation
      </motion.div>

      <motion.h1
        variants={fadeUp}
        className="text-[3.2rem] sm:text-[4.8rem] lg:text-[6rem] font-black tracking-tighter leading-none mb-4 gradient-text relative"
      >
        M-Lang
      </motion.h1>

      <motion.p variants={fadeUp} className="text-[1.15rem] sm:text-2xl font-light text-text-secondary leading-relaxed mb-4 relative">
        A Statically-Typed Compiler Using<br />Romanized Burmese Keywords
      </motion.p>

      <motion.div
        variants={fadeUp}
        className="w-20 h-[3px] rounded-full my-6 relative"
        style={{ background: "linear-gradient(135deg, #60a5fa, #a78bfa, #f472b6)" }}
      />

      <motion.p variants={fadeUp} className="text-[0.95rem] sm:text-lg text-text-muted mb-8 relative">
        Compiling <strong className="text-text-primary">Myanglish</strong> to Native Executables via an LLVM Backend
      </motion.p>

      <motion.div variants={fadeUp} className="flex flex-col sm:flex-row items-center gap-3 sm:gap-8 mb-8 relative">
        <div className="flex items-center gap-2 text-text-secondary">
          <span className="text-xl">📅</span>
          <span>March 2026</span>
        </div>
        <div className="flex items-center gap-2 text-text-secondary">
          <span className="text-xl">🏫</span>
          <span>Rangsit University</span>
        </div>
      </motion.div>

      <motion.div variants={fadeUp} className="flex flex-wrap justify-center gap-2.5 sm:gap-3 relative">
        {[
          { name: "Rust", color: "#fb923c" },
          { name: "LLVM", color: "#a78bfa" },
          { name: "Go Interop", color: "#22d3ee" },
          { name: "VS Code", color: "#a78bfa" },
        ].map((tag) => (
          <span
            key={tag.name}
            className="flex items-center gap-1.5 text-[0.8rem] font-medium text-text-secondary bg-bg-card border border-white/10 px-3.5 py-1.5 rounded-full"
          >
            <span className="w-2 h-2 rounded-full" style={{ background: tag.color }} />
            {tag.name}
          </span>
        ))}
      </motion.div>
    </motion.div>
  );
}
