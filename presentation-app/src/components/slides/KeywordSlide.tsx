"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

const keywords = [
  { ml: "kain", burmese: "ကိန်း", english: "integer", usage: "Type declaration" },
  { ml: "sar", burmese: "စာ", english: "string", usage: "Type declaration" },
  { ml: "sit", burmese: "စစ်", english: "boolean", usage: "Type declaration" },
  { ml: "da_tha", burmese: "ဒဿမ", english: "float", usage: "Type declaration" },
  { ml: "loke", burmese: "လုပ်", english: "function", usage: "Function declaration" },
  { ml: "hlyin", burmese: "လျှင်", english: "if", usage: "Conditional" },
  { ml: "mo", burmese: "မို့", english: "else", usage: "Conditional" },
  { ml: "pat", burmese: "ပတ်", english: "while / for-in / C-for", usage: "Loop" },
  { ml: "htae", burmese: "ထည့်", english: "in / range", usage: "For-in iteration" },
  { ml: "pyan", burmese: "ပြန်", english: "return", usage: "Return statement" },
  { ml: "pya", burmese: "ပြ", english: "print", usage: "Output" },
  { ml: "phat", burmese: "ဖတ်", english: "read", usage: "Input" },
  { ml: "su", burmese: "စု", english: "array", usage: "Collection" },
  { ml: "twe", burmese: "တွဲ", english: "hashmap", usage: "Key-value store" },
  { ml: "pone", burmese: "ပုံ", english: "struct", usage: "Custom types" },
  { ml: "nee", burmese: "နည်း", english: "method", usage: "Methods on types" },
  { ml: "myat", burmese: "—", english: "interface", usage: "Abstractions" },
  { ml: "atote", burmese: "အထုပ်", english: "package", usage: "Module declaration" },
  { ml: "pay", burmese: "ပေး", english: "export", usage: "Public top-level symbol" },
  { ml: "kyoe", burmese: "ကြိုး", english: "goroutine", usage: "Concurrent execution" },
  { ml: "laung", burmese: "လောင်း", english: "channel", usage: "Channel type / make" },
  { ml: "naut_sone", burmese: "နောက်ဆုံး", english: "defer", usage: "Deferred cleanup" },
];

export default function KeywordSlide() {
  return (
    <div>
      <SlideHeader number="12" title="Language Design — Keyword Mapping" />
      <p className="text-text-secondary text-[0.95rem] mb-8 max-w-[700px]">
        Every keyword is derived from romanized Burmese (Myanglish), preserving semantic meaning
      </p>
      <div data-slide-nav-lock-x className="overflow-x-auto overflow-y-auto touch-pan-x max-h-[calc(100vh-220px)] rounded-xl border border-white/10">
        <table className="w-full min-w-[620px] border-collapse text-[0.82rem] sm:text-[0.9rem]">
          <thead>
            <tr>
              {["M-Lang", "Burmese", "English", "Usage"].map((h) => (
                <th key={h} className="sticky top-0 bg-bg-secondary text-left px-5 py-3 text-[0.7rem] font-bold uppercase tracking-wider text-text-muted border-b border-white/10">
                  {h}
                </th>
              ))}
            </tr>
          </thead>
          <tbody>
            {keywords.map((kw, i) => (
              <motion.tr
                key={kw.ml}
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: i * 0.03 }}
                className="hover:bg-accent-blue/[0.03]"
              >
                <td className="px-5 py-2.5 border-b border-white/10">
                  <code className="bg-accent-blue/10 text-accent-cyan px-2 py-0.5 rounded font-mono font-semibold text-[0.9rem]">{kw.ml}</code>
                </td>
                <td className="px-5 py-2.5 border-b border-white/10 font-[var(--font-myanmar)] text-base text-accent-orange">{kw.burmese}</td>
                <td className="px-5 py-2.5 border-b border-white/10">{kw.english}</td>
                <td className="px-5 py-2.5 border-b border-white/10 text-text-secondary">{kw.usage}</td>
              </motion.tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
}
