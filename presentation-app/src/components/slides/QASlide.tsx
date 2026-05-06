"use client";
import { motion } from "framer-motion";

const qas = [
  { q: "Does M-Lang compile directly now?", a: "Yes. The default target emits LLVM IR, compiles it to an object file, links runtime_llvm.c, and produces a native executable" },
  { q: "Why Rust for the compiler?", a: "Memory safety, pattern matching, strong type system, and excellent ecosystem for compiler construction" },
  { q: "How does this compare to just using Go directly?", a: "M-Lang is an educational & cultural tool — it lowers the language barrier to programming for Burmese speakers" },
  { q: "Why keep the Go backend?", a: "Go remains the interop backend for the full stdlib, HTTP server, sockets, database, and concurrency runtime while LLVM parity grows" },
  { q: "How does concurrency work in M-Lang?", a: "Today kyoe maps to Go goroutines, laung<T> to typed channels, and naut_sone to defer on --target go" },
];

export default function QASlide() {
  return (
    <div
      className="flex flex-col items-center text-center min-h-[80vh] justify-center"
      style={{ background: "radial-gradient(ellipse at 50% 30%, rgba(167,139,250,0.06) 0%, transparent 60%)" }}
    >
      <motion.h2
        initial={{ opacity: 0, scale: 0.9 }}
        animate={{ opacity: 1, scale: 1 }}
        className="text-[2.6rem] sm:text-[4rem] font-black gradient-text mb-2"
      >
        Q &amp; A
      </motion.h2>
      <motion.p
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        transition={{ delay: 0.1 }}
        className="text-[0.95rem] sm:text-lg text-text-muted mb-8 sm:mb-10"
      >
        Thank you for your attention
      </motion.p>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 max-w-[800px] w-full mb-8 sm:mb-10">
        {qas.map((qa, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 15 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.2 + i * 0.1 }}
            whileHover={{ backgroundColor: "rgba(31,43,61,1)", borderColor: "rgba(96,165,250,0.3)" }}
            className="bg-bg-card border border-white/10 rounded-xl p-5 text-left transition-all"
          >
            <div className="text-[0.85rem] font-semibold text-accent-blue mb-2">{qa.q}</div>
            <div className="text-[0.8rem] text-text-secondary leading-relaxed">{qa.a}</div>
          </motion.div>
        ))}
      </div>

      <motion.div
        initial={{ opacity: 0, y: 20 }}
        animate={{ opacity: 1, y: 0 }}
        transition={{ delay: 0.7 }}
        className="mt-4"
      >
        <span className="block text-2xl sm:text-3xl font-bold gradient-warm" style={{ fontFamily: "var(--font-myanmar)" }}>
          ကျေးဇူးတင်ပါသည်
        </span>
        <span className="text-[0.85rem] text-text-muted mt-2 block">Kyay Zu Tin Par Tae — Thank You</span>
      </motion.div>
    </div>
  );
}
