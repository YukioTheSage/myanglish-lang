"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";
import CodeBlock from "../ui/CodeBlock";

export default function VisionSlide() {
  return (
    <div
      className="flex flex-col items-center text-center"
      style={{ background: "radial-gradient(ellipse at 50% 50%, rgba(167,139,250,0.06) 0%, transparent 60%)" }}
    >
      <SlideHeader number="24" title="Current Milestone — Networking + Concurrency Runtime" centered />
      <p className="text-text-secondary text-[0.95rem] mb-8 max-w-[700px]">
        Phase 3 delivers goroutine-style concurrency (<code className="bg-accent-blue/10 text-accent-blue px-1 rounded font-mono text-[0.85rem]">kyoe</code>), channels (<code className="bg-accent-blue/10 text-accent-blue px-1 rounded font-mono text-[0.85rem]">laung</code>), defer cleanup (<code className="bg-accent-blue/10 text-accent-blue px-1 rounded font-mono text-[0.85rem]">naut_sone</code>), and a full HTTP server runtime with <code className="bg-accent-blue/10 text-accent-blue px-1 rounded font-mono text-[0.85rem]">handle</code> / <code className="bg-accent-blue/10 text-accent-blue px-1 rounded font-mono text-[0.85rem]">listen</code>.
      </p>
      <motion.div
        initial={{ opacity: 0, scale: 0.95 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{ delay: 0.2, duration: 0.5 }}
        className="max-w-[700px] w-full text-left"
      >
        <CodeBlock size="large" className="shadow-[0_8px_32px_rgba(0,0,0,0.5),0_0_60px_rgba(96,165,250,0.08)]">
          <span className="kw">yu</span> <span className="str">&quot;kainn/http&quot;</span>;{"\n\n"}
          <span className="kw">loke</span> <span className="fn">handler</span>(http.Request req, http.ResponseWriter w) -&gt; <span className="kw">amhar</span> {"{"}{"\n"}
          {"    "}w.<span className="fn">write</span>(<span className="str">&quot;Mingalabar from M-Lang!&quot;</span>);{"\n"}
          {"    "}<span className="kw">pyan</span> <span className="kw">bhala</span>;{"\n"}
          {"}"}{"\n\n"}
          <span className="kw">loke</span> <span className="fn">main</span>() -&gt; <span className="kw">kain</span> {"{"}{"\n"}
          {"    "}http.<span className="fn">handle</span>(<span className="str">&quot;/&quot;</span>, handler);{"\n"}
          {"    "}<span className="fn">pya</span>(<span className="str">&quot;Server running on :8080&quot;</span>);{"\n"}
          {"    "}http.<span className="fn">listen</span>(<span className="str">&quot;:8080&quot;</span>);{"\n"}
          {"    "}<span className="kw">pyan</span> <span className="num">0</span>;{"\n"}
          {"}"}
        </CodeBlock>
      </motion.div>
    </div>
  );
}
