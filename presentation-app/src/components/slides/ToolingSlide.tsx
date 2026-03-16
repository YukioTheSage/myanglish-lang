"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

const tools = [
  {
    icon: "📐",
    name: "Code Formatter",
    cmd: "mlang fmt file.ml",
    features: [
      "Stable 4-space indentation",
      "Readable line-wrapping",
      "Canonicalizes imports to quoted style",
      "Formats atote/pay declarations",
      "CI check mode: --check",
    ],
    file: "formatter.rs",
  },
  {
    icon: "🧠",
    name: "LSP Server",
    cmd: "Language Server Protocol",
    features: [
      "Semantic token highlighting",
      "Hover, completion, definition",
      "Package/export diagnostics (atote/pay)",
      "Live parse/type diagnostics",
      "textDocument/formatting support",
    ],
    file: "tower-lsp backend (Rust)",
  },
  {
    icon: "💻",
    name: "VS Code Extension",
    cmd: "mlang-vscode",
    features: [
      "TextMate syntax grammar",
      "Language configuration",
      "LSP integration",
      "LSP-first + CLI fallback formatting",
    ],
    file: "TypeScript client + grammar",
  },
];

export default function ToolingSlide() {
  return (
    <div>
      <SlideHeader number="20" title="Developer Tooling Ecosystem" />
      <div className="grid grid-cols-1 md:grid-cols-2 xl:grid-cols-3 gap-4 sm:gap-6 mt-4">
        {tools.map((tool, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: i * 0.15 }}
            whileHover={{ backgroundColor: "rgba(31,43,61,1)", borderColor: "rgba(96,165,250,0.3)", y: -4, boxShadow: "0 0 30px rgba(96,165,250,0.15)" }}
            className="bg-bg-card border border-white/10 rounded-2xl p-6 sm:p-8 text-center transition-all"
          >
            <div className="text-4xl mb-4">{tool.icon}</div>
            <h3 className="text-xl font-bold mb-2">{tool.name}</h3>
            <p className="font-mono text-[0.8rem] text-accent-green mb-4"><code>{tool.cmd}</code></p>
            <ul className="list-none text-left px-2 space-y-1">
              {tool.features.map((f, j) => (
                <li key={j} className="text-[0.8rem] text-text-secondary py-0.5">{f}</li>
              ))}
            </ul>
            <div className="mt-4 pt-3 border-t border-white/10 text-[0.7rem] font-mono text-text-muted">{tool.file}</div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
