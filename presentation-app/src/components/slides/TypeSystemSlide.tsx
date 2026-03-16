"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";
import CodeBlock from "../ui/CodeBlock";

const coreTypes = [
  { name: "kain", target: "int64", desc: "64-bit signed integer" },
  { name: "sar", target: "string", desc: "UTF-8 string" },
  { name: "sit", target: "bool", desc: "hman (true) / hmar (false)" },
  { name: "da_tha", target: "float64", desc: "Floating-point number" },
  { name: "amhar", target: "error", desc: "Error type for tuple-style handling" },
];

const collectionTypes = [
  { name: "su<T>", target: "[]T", desc: "Homogeneous array" },
  { name: "twe<K,V>", target: "map[K]V", desc: "Key-value hashmap" },
  { name: "(A, B)", target: "tuple", desc: "Multiple return values" },
  { name: "loke(...) -> T", target: "func", desc: "First-class function type" },
  { name: "laung<T>", target: "chan T", desc: "Typed channel (send/recv/close)" },
];

function TypeCard({ name, target, desc, delay }: { name: string; target: string; desc: string; delay: number }) {
  return (
    <motion.div
      initial={{ opacity: 0, x: -10 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay }}
      className="grid grid-cols-1 sm:grid-cols-[120px_30px_96px_1fr] items-start sm:items-center gap-1.5 sm:gap-2 px-4 py-2.5 bg-bg-card border border-white/10 rounded-lg text-[0.85rem]"
    >
      <code className="font-mono font-semibold text-accent-cyan bg-accent-cyan/[0.08] px-2 py-0.5 rounded text-left sm:text-center text-[0.85rem]">{name}</code>
      <span className="hidden sm:block text-text-muted text-center">→</span>
      <span className="font-mono text-accent-green text-[0.8rem]">{target}</span>
      <span className="text-text-secondary text-[0.8rem]">{desc}</span>
    </motion.div>
  );
}

export default function TypeSystemSlide() {
  return (
    <div>
      <SlideHeader number="13" title="Type System" />
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 lg:gap-10">
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Core Types</h3>
          <div className="flex flex-col gap-2.5">
            {coreTypes.map((t, i) => (
              <TypeCard key={t.name} {...t} delay={i * 0.08} />
            ))}
          </div>
        </div>
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Collection &amp; Composite Types</h3>
          <div className="flex flex-col gap-2.5">
            {collectionTypes.map((t, i) => (
              <TypeCard key={t.name} {...t} delay={0.3 + i * 0.08} />
            ))}
          </div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4 mt-6">Type Conversions</h3>
          <CodeBlock size="small">
            <span className="kw">sar</span> num_str = <span className="fn">pyaung_sar</span>(<span className="num">42</span>);{"\n"}
            <span className="kw">kain</span> parsed = <span className="fn">pyaung_kain</span>(<span className="str">&quot;100&quot;</span>);{"\n"}
            <span className="kw">da_tha</span> f = <span className="fn">pyaung_da_tha</span>(<span className="str">&quot;3.14&quot;</span>);
          </CodeBlock>
        </div>
      </div>
    </div>
  );
}
