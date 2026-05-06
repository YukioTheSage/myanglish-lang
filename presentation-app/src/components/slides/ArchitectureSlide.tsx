"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

interface StageBoxProps {
  label: string;
  desc?: string;
  colorClass: string;
  delay: number;
}

function StageBox({ label, desc, colorClass, delay }: StageBoxProps) {
  return (
    <motion.div
      initial={{ opacity: 0, scale: 0.9 }}
      animate={{ opacity: 1, scale: 1 }}
      transition={{ delay }}
      className={`w-full max-w-[320px] sm:max-w-none sm:min-w-[200px] px-6 sm:px-8 py-3 rounded-lg font-semibold text-[0.82rem] sm:text-[0.85rem] text-center border ${colorClass}`}
    >
      <span className="block text-[0.95rem]">{label}</span>
      {desc && <span className="block text-[0.65rem] text-text-muted font-mono mt-0.5">{desc}</span>}
    </motion.div>
  );
}

function Arrow({ delay, split }: { delay: number; split?: boolean }) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ delay }}
      className={`text-text-muted text-[0.9rem] py-1 ${split ? "flex justify-center gap-10 sm:gap-40" : "text-center"}`}
    >
      {split ? <span>▼</span> : null}
      {split ? <span>▼</span> : null}
      {!split && "▼"}
    </motion.div>
  );
}

function StageLabel({ text, delay }: { text: string; delay: number }) {
  return (
    <motion.div
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ delay }}
      className="text-[0.65rem] text-text-muted font-mono py-0.5 tracking-wide text-center"
    >
      {text}
    </motion.div>
  );
}

export default function ArchitectureSlide() {
  return (
    <div style={{ background: "radial-gradient(ellipse at 50% 30%, rgba(96,165,250,0.06) 0%, transparent 60%)" }}>
      <SlideHeader number="15" title="Compiler Architecture" badge="Core Contribution" />
      <p className="text-text-secondary text-[0.95rem] mb-6 max-w-[700px]">
        M-Lang follows a classic multi-pass compiler pipeline, compiling to LLVM/native executables by default with Go kept as an interop backend
      </p>
      <div className="flex flex-col items-center gap-0 mt-4 px-1">
        <StageBox label=".ml source" colorClass="bg-accent-purple/10 border-accent-purple/30 text-accent-purple" delay={0.1} />
        <Arrow delay={0.15} />
        <StageBox label="Lexer" desc="lexer.rs · ~12 KB" colorClass="bg-accent-cyan/[0.08] border-accent-cyan/25 text-accent-cyan" delay={0.2} />
        <StageLabel text="Token Stream" delay={0.25} />
        <Arrow delay={0.3} />
        <StageBox label="Parser" desc="parser.rs · ~77 KB" colorClass="bg-accent-blue/[0.08] border-accent-blue/25 text-accent-blue" delay={0.35} />
        <StageLabel text="AST (Program)" delay={0.4} />
        <Arrow delay={0.45} />
        <StageBox label="Type Checker" desc="typecheck.rs · ~66 KB" colorClass="bg-accent-pink/[0.08] border-accent-pink/25 text-accent-pink" delay={0.5} />
        <StageLabel text="Validated AST" delay={0.55} />
        <Arrow delay={0.6} split />

        <div className="flex flex-col sm:flex-row gap-4 sm:gap-8 w-full items-center justify-center">
          <div className="flex flex-col items-center">
            <StageBox label="CodeGen LLVM" desc="codegen_llvm.rs" colorClass="bg-accent-purple/[0.08] border-accent-purple/25 text-accent-purple" delay={0.65} />
            <StageLabel text=".ll IR + runtime_llvm.c" delay={0.7} />
            <Arrow delay={0.75} />
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.8 }}
              className="w-full max-w-[220px] px-6 sm:px-8 py-3 bg-bg-card border border-white/10 rounded-lg font-mono text-text-secondary text-[0.85rem] sm:min-w-[140px] text-center"
            >
              clang / gcc link
            </motion.div>
          </div>
          <div className="flex flex-col items-center">
            <StageBox label="CodeGen Go" desc="codegen_go.rs · interop" colorClass="bg-accent-cyan/[0.08] border-accent-cyan/25 text-accent-cyan" delay={0.65} />
            <StageLabel text=".go file for stdlib/server features" delay={0.7} />
            <Arrow delay={0.75} />
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ delay: 0.8 }}
              className="w-full max-w-[220px] px-6 sm:px-8 py-3 bg-bg-card border border-white/10 rounded-lg font-mono text-text-secondary text-[0.85rem] sm:min-w-[140px] text-center"
            >
              go build
            </motion.div>
          </div>
        </div>

        <Arrow delay={0.85} />
        <StageBox label="Native Executable" colorClass="bg-accent-green/[0.08] border-accent-green/25 text-accent-green text-base px-10 py-3.5" delay={0.9} />
      </div>
    </div>
  );
}
