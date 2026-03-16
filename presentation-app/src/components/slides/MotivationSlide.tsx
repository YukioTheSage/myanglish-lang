"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";
import Card from "../ui/Card";

export default function MotivationSlide() {
  return (
    <div>
      <SlideHeader number="10" title="Motivation & Problem Statement" />
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 lg:gap-10">
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">The Problem</h3>
          {[
            { icon: "🌏", title: "Language Barrier", desc: "All mainstream programming languages use English keywords — creating cognitive overhead for non-English speakers" },
            { icon: "🇲🇲", title: "Zero Representation", desc: "No existing programming language uses Burmese or Myanglish (romanized Burmese) syntax" },
            { icon: "📚", title: "Educational Opportunity", desc: "Building a compiler covers the full pipeline: lexing, parsing, type-checking, and code generation" },
          ].map((item, i) => (
            <Card key={i} delay={i} className="mb-4">
              <div className="text-2xl mb-2">{item.icon}</div>
              <h4 className="text-base font-bold mb-1">{item.title}</h4>
              <p className="text-[0.85rem] text-text-secondary leading-relaxed">{item.desc}</p>
            </Card>
          ))}
        </div>
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Research Question</h3>
          <motion.div
            initial={{ opacity: 0, x: 20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: 0.3 }}
            className="bg-accent-blue/5 border border-accent-blue/20 border-l-[3px] border-l-accent-blue rounded-xl p-6 mb-6"
          >
            <p className="text-[1.05rem] italic text-text-primary leading-relaxed">
              &ldquo;Can a statically-typed language with romanized Burmese keywords compile to efficient native code while maintaining developer-friendly tooling?&rdquo;
            </p>
          </motion.div>
          <div className="grid grid-cols-1 sm:grid-cols-3 gap-4">
            {[
              { number: "~410", unit: "KB", label: "Rust Compiler Code" },
              { number: "2", unit: "", label: "Code Gen Backends" },
              { number: "22", unit: "", label: "Myanglish Keywords" },
            ].map((stat, i) => (
              <motion.div
                key={i}
                initial={{ opacity: 0, scale: 0.9 }}
                animate={{ opacity: 1, scale: 1 }}
                transition={{ delay: 0.4 + i * 0.1 }}
                className="text-center p-4 bg-bg-card border border-white/10 rounded-xl"
              >
                <span className="block text-[2rem] font-extrabold" style={{ background: "linear-gradient(135deg, #22d3ee, #60a5fa)", WebkitBackgroundClip: "text", WebkitTextFillColor: "transparent" }}>
                  {stat.number}<span className="text-[0.9rem]">{stat.unit}</span>
                </span>
                <span className="block text-[0.7rem] text-text-muted mt-1 uppercase tracking-wide">{stat.label}</span>
              </motion.div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
