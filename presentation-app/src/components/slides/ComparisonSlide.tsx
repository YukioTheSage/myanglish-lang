"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

const languages = [
  { name: "M-Lang 🇲🇲", keywords: "Myanglish", target: "Native (Go/C)", typing: "Static", typingColor: "green", tooling: "LSP, Formatter, VS Code", highlight: true },
  { name: "Qalb (قلب)", keywords: "Arabic", target: "Interpreter", typing: "Dynamic", typingColor: "yellow", tooling: "Minimal" },
  { name: "Wenyan (文言)", keywords: "Classical Chinese", target: "JavaScript", typing: "Dynamic", typingColor: "yellow", tooling: "Online IDE" },
  { name: "Rapira", keywords: "Russian", target: "Interpreter", typing: "Dynamic", typingColor: "yellow", tooling: "Minimal" },
  { name: "Robik", keywords: "Russian", target: "Bytecode", typing: "Static", typingColor: "green", tooling: "Basic" },
];

const diffs = [
  { icon: "🏗️", text: "<strong>Static type system</strong> — Most non-English languages are dynamically typed" },
  { icon: "⚡", text: "<strong>Native code compilation</strong> — Via Go/C backends, not interpreted" },
  { icon: "🛠️", text: "<strong>Full developer tooling</strong> — LSP, formatter, VS Code extension" },
  { icon: "🔀", text: "<strong>Dual compilation backends</strong> — Shared AST with Go and C targets" },
  { icon: "🧵", text: "<strong>Built-in concurrency</strong> — Goroutines, channels, and defer with Myanglish keywords" },
];

export default function ComparisonSlide() {
  return (
    <div>
      <SlideHeader number="25" title="Related Work & Comparison" />
      <div data-slide-nav-lock-x className="rounded-xl border border-white/10 overflow-x-auto overflow-y-hidden touch-pan-x mb-8">
        <table className="w-full min-w-[700px] border-collapse text-[0.8rem] sm:text-[0.85rem]">
          <thead>
            <tr>
              {["Language", "Keywords", "Target", "Typing", "Tooling"].map((h) => (
                <th key={h} className="bg-bg-secondary text-left px-4 py-3 text-[0.7rem] font-bold uppercase tracking-wider text-text-muted border-b border-white/10">{h}</th>
              ))}
            </tr>
          </thead>
          <tbody>
            {languages.map((lang, i) => (
              <motion.tr
                key={i}
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                transition={{ delay: i * 0.08 }}
                className={lang.highlight ? "bg-accent-blue/[0.04]" : ""}
              >
                <td className={`px-4 py-2.5 border-b border-white/10 ${lang.highlight ? "font-medium text-text-primary" : "text-text-secondary"}`}>
                  {lang.highlight ? <strong>{lang.name}</strong> : lang.name}
                </td>
                <td className={`px-4 py-2.5 border-b border-white/10 ${lang.highlight ? "text-text-primary" : "text-text-secondary"}`}>{lang.keywords}</td>
                <td className={`px-4 py-2.5 border-b border-white/10 ${lang.highlight ? "text-text-primary" : "text-text-secondary"}`}>{lang.target}</td>
                <td className="px-4 py-2.5 border-b border-white/10">
                  <span className={`text-[0.65rem] font-semibold px-2 py-0.5 rounded-full ${
                    lang.typingColor === "green" ? "bg-accent-green/15 text-accent-green" : "bg-accent-yellow/15 text-accent-yellow"
                  }`}>
                    {lang.typing}
                  </span>
                </td>
                <td className={`px-4 py-2.5 border-b border-white/10 ${lang.highlight ? "text-text-primary" : "text-text-secondary"}`}>{lang.tooling}</td>
              </motion.tr>
            ))}
          </tbody>
        </table>
      </div>

      <div className="mt-2">
        <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">M-Lang Differentiators</h3>
        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
          {diffs.map((d, i) => (
            <motion.div
              key={i}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.4 + i * 0.1 }}
              className="flex items-start gap-2.5 text-[0.85rem] text-text-secondary"
            >
              <span className="text-lg shrink-0">{d.icon}</span>
              <span dangerouslySetInnerHTML={{ __html: d.text }} />
            </motion.div>
          ))}
        </div>
      </div>
    </div>
  );
}
