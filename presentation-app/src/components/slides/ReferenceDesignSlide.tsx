"use client";
import SlideHeader from "../ui/SlideHeader";
import CodeBlock from "../ui/CodeBlock";

const designPoints = [
  "Programs start from a package declaration and explicit imports.",
  "Functions declare typed parameters and typed return values.",
  "Control flow is structured around if, for, return, and function calls.",
  "Errors are commonly propagated explicitly rather than hidden.",
  "Concurrency is a language-level concept through go, chan, and defer.",
];

export default function ReferenceDesignSlide() {
  return (
    <div>
      <SlideHeader number="05" title="Reference Language Design" />
      <div className="grid grid-cols-1 xl:grid-cols-[0.9fr_1.1fr] gap-6">
        <div className="bg-bg-card border border-white/10 rounded-2xl p-6">
          <p className="text-[0.72rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Design Characteristics</p>
          <div className="space-y-3">
            {designPoints.map((point) => (
              <div key={point} className="flex items-start gap-2 text-[0.83rem] text-text-secondary">
                <span className="text-accent-green font-bold shrink-0">+</span>
                <span>{point}</span>
              </div>
            ))}
          </div>
        </div>

        <div>
          <p className="text-[0.72rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Reference Example (Go)</p>
          <CodeBlock size="small">
            <span className="kw">package</span> main{"\n\n"}
            <span className="kw">import</span> <span className="str">&quot;fmt&quot;</span>{"\n\n"}
            <span className="kw">func</span> <span className="fn">add</span>(a <span className="kw">int</span>, b <span className="kw">int</span>) <span className="kw">int</span> {"{"}{"\n"}
            {"    "}<span className="kw">return</span> a + b{"\n"}
            {"}"}{"\n\n"}
            <span className="kw">func</span> <span className="fn">main</span>() {"{"}{"\n"}
            {"    "}result := <span className="fn">add</span>(10, 20){"\n"}
            {"    "}fmt.<span className="fn">Println</span>(result){"\n"}
            {"}"}
          </CodeBlock>
          <p className="mt-3 text-[0.8rem] text-text-secondary leading-relaxed">
            This example is small, but it already shows the key reference ideas used in the M-Lang design review: package structure, imports, typed functions,
            return values, and procedural program flow.
          </p>
        </div>
      </div>
    </div>
  );
}
