"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

const externalSources = [
  {
    title: "Go Language Specification",
    url: "https://go.dev/ref/spec",
    note: "Used for the reference-language design review and EBNF notation framing.",
  },
  {
    title: "A Tour of Go",
    url: "https://go.dev/tour/",
    note: "Used for introductory reference examples covering syntax, data structures, and concurrency concepts.",
  },
  {
    title: "Go Documentation",
    url: "https://go.dev/doc",
    note: "Used for ecosystem-level reference context and official learning/documentation links.",
  },
  {
    title: "Effective Go",
    url: "https://go.dev/doc/effective_go",
    note: "Used for idiomatic reference patterns such as explicit errors, functions, and channels.",
  },
  {
    title: "Myanglish Lang GitHub Repository",
    url: "https://github.com/YukioTheSage/myanglish-lang",
    note: "Used as the public implementation reference for the compiler, examples, and tooling shown in the presentation.",
  },
];

const internalSources = [
  "README.md",
  "docs/ARCHITECTURE.md",
  "docs/CHEATSHEET.md",
  "presentation-app/src/components/slides/*",
];

export default function SourcesSlide() {
  return (
    <div>
      <SlideHeader number="09" title="Document Review Sources" />
      <div className="grid grid-cols-1 xl:grid-cols-[1.2fr_0.8fr] gap-6">
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {externalSources.map((source, index) => (
            <motion.a
              key={source.url}
              href={source.url}
              target="_blank"
              rel="noreferrer"
              initial={{ opacity: 0, y: 12 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.08 }}
              className="bg-bg-card border border-white/10 rounded-xl p-5 hover:border-accent-blue/30 hover:bg-bg-card-hover transition-colors"
            >
              <h3 className="text-[0.9rem] font-bold mb-2">{source.title}</h3>
              <p className="text-[0.76rem] font-mono text-accent-cyan break-all mb-3">{source.url}</p>
              <p className="text-[0.8rem] text-text-secondary leading-relaxed">{source.note}</p>
            </motion.a>
          ))}
        </div>

        <div className="bg-bg-card border border-white/10 rounded-2xl p-6">
          <p className="text-[0.72rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Internal Design Sources</p>
          <div className="space-y-3 mb-5">
            {internalSources.map((source, index) => (
              <motion.div
                key={source}
                initial={{ opacity: 0, x: 10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: 0.2 + index * 0.08 }}
                className="rounded-lg border border-white/10 bg-bg-secondary/40 px-4 py-3 font-mono text-[0.76rem] text-text-secondary"
              >
                {source}
              </motion.div>
            ))}
          </div>
          <p className="text-[0.8rem] text-text-secondary leading-relaxed">
            The external sources support both the academic review section and the public implementation reference, while the internal repository documents
            support the target-language and implementation section of this presentation.
          </p>
        </div>
      </div>
    </div>
  );
}
