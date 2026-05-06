"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";

interface PhaseItem {
  text: string;
  done?: boolean;
}

interface Phase {
  title: string;
  status?: string;
  done?: boolean;
  items: PhaseItem[];
  last?: boolean;
}

const phases: Phase[] = [
  {
    title: "Phase 1 — Language Foundations",
    status: "Completed",
    done: true,
    items: [
      { text: "Structs (pone)", done: true },
      { text: "Methods (nee)", done: true },
      { text: "Interfaces (myat)", done: true },
      { text: "Float (da_tha)", done: true },
      { text: "Nil (bhala)", done: true },
      { text: "Type conversions", done: true },
      { text: "Slices & String methods", done: true },
      { text: "Tuple error handling (amhar + destructuring)", done: true },
      { text: "Break/continue with for-in index loops", done: true },
      { text: "C-style for loops: pat (init; cond; post)", done: true },
      { text: "String lowercase method: ayaik()", done: true },
    ],
  },
  {
    title: "Phase 2 — Core Modules + Stdlib Baseline",
    status: "Completed",
    done: true,
    items: [
      { text: "json.encode / json.decode", done: true },
      { text: "file.read / file.write", done: true },
      { text: "su_nit.env / su_nit.args", done: true },
      { text: "kainn/http get/post client", done: true },
      { text: "VS Code + LSP + formatter integration", done: true },
      { text: "Package declaration + export visibility (atote + pay)", done: true },
      { text: "Relative local imports + module graph checks", done: true },
      { text: "Extended stdlib (pone_set / in_ote / hmat)", done: true },
    ],
  },
  {
    title: "Phase 3 — Concurrency & Networking Runtime",
    status: "Completed",
    done: true,
    items: [
      { text: "Goroutine-style primitives (kyoe)", done: true },
      { text: "Channels / message passing (laung<T>)", done: true },
      { text: "HTTP server runtime (handle/listen)", done: true },
      { text: "Defer-style cleanup (naut_sone)", done: true },
      { text: "TCP/UDP sockets (kainn)", done: true },
    ],
  },
  {
    title: "Phase 4 — Production Readiness",
    status: "Completed",
    done: true,
    items: [
      { text: "Testing framework (set_sae)", done: true },
      { text: "Context/timeout (baung)", done: true },
      { text: "Database connectors / adapters", done: true },
      { text: "Dependency manager (mlang get)", done: true },
      { text: "Cross-compilation", done: true },
    ],
  },
  {
    title: "Phase 5 — Native Compiler Migration",
    status: "MVP",
    items: [
      { text: "LLVM default target", done: true },
      { text: "runtime_llvm.c native linking", done: true },
      { text: "Phase 1 native e2e tests", done: true },
      { text: "Phase 2/3/4 LLVM parity", done: false },
    ],
    last: true,
  },
];

export default function RoadmapSlide() {
  return (
    <div>
      <SlideHeader number="23" title="Roadmap & Current Status" />
      <div className="flex flex-col pl-4">
        {phases.map((phase, i) => (
          <motion.div
            key={i}
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ delay: i * 0.15 }}
            className="flex gap-6"
          >
            <div className="flex flex-col items-center shrink-0">
              <div
                className={`w-4 h-4 rounded-full border-2 z-10 shrink-0 ${
                  phase.done ? "bg-accent-green border-accent-green" : "bg-bg-card border-accent-blue"
                }`}
              />
              {!phase.last && <div className="w-0.5 flex-grow bg-white/10 min-h-[20px]" />}
            </div>
            <div className="pb-8">
              <h4 className="text-base font-bold mb-3">
                {phase.title}
                {phase.status && (
                  <span className="text-[0.65rem] font-semibold px-2 py-0.5 rounded-full bg-accent-green/15 text-accent-green ml-2 align-middle">
                    {phase.status}
                  </span>
                )}
              </h4>
              <div className="flex flex-wrap gap-1.5">
                {phase.items.map((item, j) => (
                  <span
                    key={j}
                    className={`text-[0.78rem] px-3 py-1 rounded-full border ${
                      item.done === true
                        ? "border-accent-green/20 text-accent-green"
                        : item.done === false
                        ? "border-accent-yellow/20 text-accent-yellow"
                        : "bg-bg-card border-white/10 text-text-secondary"
                    }`}
                  >
                    {item.done === true ? "✅ " : item.done === false ? "⬜ " : ""}{item.text}
                  </span>
                ))}
              </div>
            </div>
          </motion.div>
        ))}
      </div>
    </div>
  );
}
