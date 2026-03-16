"use client";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useMemo, useRef, useState } from "react";
import SlideHeader from "../ui/SlideHeader";
import CodeBlock from "../ui/CodeBlock";

interface ExampleSnippet {
  id: string;
  title: string;
  description: string;
  file: string;
  tags: string[];
  rawCode: string;
  highlightedCode: string;
}

const snippets: ExampleSnippet[] = [
  {
    id: "struct-collection-mutation",
    title: "Struct + Collection Mutation",
    description: "Current phase1 pattern: struct literals, field assignment, index assignment, and array methods.",
    file: "examples/phase1/01_struct_and_collection_mutation.ml",
    tags: ["structs", "mutation", "collections"],
    rawCode: `pone Cart {
    sar customer;
    kain item_count;
}

loke main() -> kain {
    Cart cart = Cart { customer: "Aye Aye", item_count: 1 };
    cart.customer = "Ko Ko";
    su<sar> items = ["tea", "coffee"];
    items[1] = "latte";
    items.push("cake");
    pya(cart.customer);
    pyan 0;
}`,
    highlightedCode: `<span class="kw">pone</span> <span class="type">Cart</span> {
    <span class="kw">sar</span> customer;
    <span class="kw">kain</span> item_count;
}

<span class="kw">loke</span> <span class="fn">main</span>() -&gt; <span class="kw">kain</span> {
    <span class="type">Cart</span> cart = <span class="type">Cart</span> { customer: <span class="str">"Aye Aye"</span>, item_count: <span class="num">1</span> };
    cart.customer = <span class="str">"Ko Ko"</span>;
    <span class="kw">su</span>&lt;<span class="kw">sar</span>&gt; items = [<span class="str">"tea"</span>, <span class="str">"coffee"</span>];
    items[<span class="num">1</span>] = <span class="str">"latte"</span>;
    items.<span class="fn">push</span>(<span class="str">"cake"</span>);
    <span class="fn">pya</span>(cart.customer);
    <span class="kw">pyan</span> <span class="num">0</span>;
}`,
  },
  {
    id: "classic-for-loop",
    title: "Classic For Loop (`pat`)",
    description: "Phase1 now supports C-style for init/condition/post syntax.",
    file: "examples/phase1/04_classic_for_loop.ml",
    tags: ["phase1", "loops", "classic-for"],
    rawCode: `loke main() -> kain {
    pat (kain i = 0; i < 5; i = i + 1) {
        pya(i);
    }
    pyan 0;
}`,
    highlightedCode: `<span class="kw">loke</span> <span class="fn">main</span>() -&gt; <span class="kw">kain</span> {
    <span class="kw">pat</span> (<span class="kw">kain</span> i = <span class="num">0</span>; i &lt; <span class="num">5</span>; i = i + <span class="num">1</span>) {
        <span class="fn">pya</span>(i);
    }
    <span class="kw">pyan</span> <span class="num">0</span>;
}`,
  },
  {
    id: "http-json-stdlib",
    title: "HTTP + JSON Stdlib",
    description: "Phase2 stdlib usage with tuple error handling (`value, amhar err`).",
    file: "examples/phase2/06_http_json_client.ml",
    tags: ["stdlib", "http", "json"],
    rawCode: `yu "kainn/http";
yu "json";

loke main() -> kain {
    twe<sar, kain> payload_map = {"order_id": 123, "amount": 5000};
    sar payload, amhar encode_err = json.encode(payload_map);
    hlyin (encode_err != bhala) { pyan 1; }

    http.Response post_res, amhar post_err = http.post("https://httpbin.org/post", payload);
    hlyin (post_err != bhala) { pyan 1; }

    pya(post_res.status);
    pyan 0;
}`,
    highlightedCode: `<span class="kw">yu</span> <span class="str">"kainn/http"</span>;
<span class="kw">yu</span> <span class="str">"json"</span>;

<span class="kw">loke</span> <span class="fn">main</span>() -&gt; <span class="kw">kain</span> {
    <span class="kw">twe</span>&lt;<span class="kw">sar</span>, <span class="kw">kain</span>&gt; payload_map = {<span class="str">"order_id"</span>: <span class="num">123</span>, <span class="str">"amount"</span>: <span class="num">5000</span>};
    <span class="kw">sar</span> payload, <span class="kw">amhar</span> encode_err = json.<span class="fn">encode</span>(payload_map);
    <span class="kw">hlyin</span> (encode_err != <span class="kw">bhala</span>) { <span class="kw">pyan</span> <span class="num">1</span>; }

    http.Response post_res, <span class="kw">amhar</span> post_err = http.<span class="fn">post</span>(<span class="str">"https://httpbin.org/post"</span>, payload);
    <span class="kw">hlyin</span> (post_err != <span class="kw">bhala</span>) { <span class="kw">pyan</span> <span class="num">1</span>; }

    <span class="fn">pya</span>(post_res.status);
    <span class="kw">pyan</span> <span class="num">0</span>;
}`,
  },
  {
    id: "local-package-import",
    title: "Local Package Import (`atote` + `pay`)",
    description: "Phase2 core module system with relative import, package declaration, and explicit export.",
    file: "examples/phase2/07_local_package_import_main.ml + 07_local_package_import_util.ml",
    tags: ["phase2", "modules", "atote", "pay"],
    rawCode: `// main.ml
yu "./07_local_package_import_util";

loke main() -> kain {
    kain result = util.add(10, 20);
    pya(result);
    pyan 0;
}

// util.ml
atote util;

pay loke add(kain a, kain b) -> kain {
    pyan a + b;
}`,
    highlightedCode: `<span class="cm">// main.ml</span>
<span class="kw">yu</span> <span class="str">"./07_local_package_import_util"</span>;

<span class="kw">loke</span> <span class="fn">main</span>() -&gt; <span class="kw">kain</span> {
    <span class="kw">kain</span> result = util.<span class="fn">add</span>(<span class="num">10</span>, <span class="num">20</span>);
    <span class="fn">pya</span>(result);
    <span class="kw">pyan</span> <span class="num">0</span>;
}

<span class="cm">// util.ml</span>
<span class="kw">atote</span> util;

<span class="kw">pay</span> <span class="kw">loke</span> <span class="fn">add</span>(<span class="kw">kain</span> a, <span class="kw">kain</span> b) -&gt; <span class="kw">kain</span> {
    <span class="kw">pyan</span> a + b;
}`,
  },
  {
    id: "fmt-io-log-stdlib",
    title: "fmt/io/log Stdlib (`pone_set`/`in_ote`/`hmat`)",
    description: "Phase2 extended stdlib modules for formatting, stdin/stdout, and structured level logging.",
    file: "examples/phase2/08_pone_set_in_ote_hmat.ml",
    tags: ["phase2", "stdlib", "fmt", "io", "log"],
    rawCode: `yu "pone_set";
yu "in_ote";
yu "hmat";

loke main() -> kain {
    sar prompt = pone_set.pon_san("%s", "Enter your name: ");
    amhar write_prompt_err = in_ote.htote_yay(prompt);
    hlyin (write_prompt_err != bhala) { pyan 1; }

    sar name, amhar read_err = in_ote.twin_phat();
    hlyin (read_err != bhala) { pyan 1; }

    amhar info_err = hmat.mhat_chet(name);
    amhar warn_err = hmat.mhat_thati(name);
    amhar err_err = hmat.mhat_amhar(name);
    pya(info_err);
    pya(warn_err);
    pya(err_err);
    pyan 0;
}`,
    highlightedCode: `<span class="kw">yu</span> <span class="str">"pone_set"</span>;
<span class="kw">yu</span> <span class="str">"in_ote"</span>;
<span class="kw">yu</span> <span class="str">"hmat"</span>;

<span class="kw">loke</span> <span class="fn">main</span>() -&gt; <span class="kw">kain</span> {
    <span class="kw">sar</span> prompt = pone_set.<span class="fn">pon_san</span>(<span class="str">"%s"</span>, <span class="str">"Enter your name: "</span>);
    <span class="kw">amhar</span> write_prompt_err = in_ote.<span class="fn">htote_yay</span>(prompt);
    <span class="kw">hlyin</span> (write_prompt_err != <span class="kw">bhala</span>) { <span class="kw">pyan</span> <span class="num">1</span>; }

    <span class="kw">sar</span> name, <span class="kw">amhar</span> read_err = in_ote.<span class="fn">twin_phat</span>();
    <span class="kw">hlyin</span> (read_err != <span class="kw">bhala</span>) { <span class="kw">pyan</span> <span class="num">1</span>; }

    <span class="kw">amhar</span> info_err = hmat.<span class="fn">mhat_chet</span>(name);
    <span class="kw">amhar</span> warn_err = hmat.<span class="fn">mhat_thati</span>(name);
    <span class="kw">amhar</span> err_err = hmat.<span class="fn">mhat_amhar</span>(name);
    <span class="fn">pya</span>(info_err);
    <span class="fn">pya</span>(warn_err);
    <span class="fn">pya</span>(err_err);
    <span class="kw">pyan</span> <span class="num">0</span>;
}`,
  },
  {
    id: "http-server",
    title: "HTTP Server (`kainn/http`)",
    description: "Phase3 server runtime: define handlers and listen on a port with the kainn/http module.",
    file: "examples/phase3/09_http_server.ml",
    tags: ["phase3", "http", "server"],
    rawCode: `yu "kainn/http";

loke handler(http.Request req, http.ResponseWriter w) -> amhar {
    w.write("Mingalabar from M-Lang!");
    pyan bhala;
}

loke main() -> kain {
    http.handle("/", handler);
    pya("Server running on :8080");
    http.listen(":8080");
    pyan 0;
}`,
    highlightedCode: `<span class="kw">yu</span> <span class="str">"kainn/http"</span>;

<span class="kw">loke</span> <span class="fn">handler</span>(http.Request req, http.ResponseWriter w) -&gt; <span class="kw">amhar</span> {
    w.<span class="fn">write</span>(<span class="str">"Mingalabar from M-Lang!"</span>);
    <span class="kw">pyan</span> <span class="kw">bhala</span>;
}

<span class="kw">loke</span> <span class="fn">main</span>() -&gt; <span class="kw">kain</span> {
    http.<span class="fn">handle</span>(<span class="str">"/"</span>, handler);
    <span class="fn">pya</span>(<span class="str">"Server running on :8080"</span>);
    http.<span class="fn">listen</span>(<span class="str">":8080"</span>);
    <span class="kw">pyan</span> <span class="num">0</span>;
}`,
  },
  {
    id: "concurrency-channels",
    title: "Concurrency + Channels (`kyoe` + `laung`)",
    description: "Phase3 goroutine-style concurrency with channel message passing.",
    file: "examples/phase3/10_concurrency_channels.ml",
    tags: ["phase3", "concurrency", "channels"],
    rawCode: `loke worker(laung<sar> ch) {
    ch.send("hello from goroutine");
}

loke main() -> kain {
    laung<sar> ch = laung(sar);
    kyoe worker(ch);
    sar msg = ch.recv();
    pya(msg);
    ch.close();
    pyan 0;
}`,
    highlightedCode: `<span class="kw">loke</span> <span class="fn">worker</span>(<span class="kw">laung</span>&lt;<span class="kw">sar</span>&gt; ch) {
    ch.<span class="fn">send</span>(<span class="str">"hello from goroutine"</span>);
}

<span class="kw">loke</span> <span class="fn">main</span>() -&gt; <span class="kw">kain</span> {
    <span class="kw">laung</span>&lt;<span class="kw">sar</span>&gt; ch = <span class="kw">laung</span>(<span class="kw">sar</span>);
    <span class="kw">kyoe</span> <span class="fn">worker</span>(ch);
    <span class="kw">sar</span> msg = ch.<span class="fn">recv</span>();
    <span class="fn">pya</span>(msg);
    ch.<span class="fn">close</span>();
    <span class="kw">pyan</span> <span class="num">0</span>;
}`,
  },
  {
    id: "defer-cleanup",
    title: "Defer Cleanup (`naut_sone`)",
    description: "Phase3 defer-style statement for guaranteed cleanup, lowered to Go defer.",
    file: "examples/phase3/11_defer.ml",
    tags: ["phase3", "defer", "cleanup"],
    rawCode: `yu "file";

loke main() -> kain {
    sar content, amhar read_err = file.read("data.txt");
    hlyin (read_err != bhala) { pyan 1; }
    naut_sone pya("cleanup done");
    pya(content);
    pyan 0;
}`,
    highlightedCode: `<span class="kw">yu</span> <span class="str">"file"</span>;

<span class="kw">loke</span> <span class="fn">main</span>() -&gt; <span class="kw">kain</span> {
    <span class="kw">sar</span> content, <span class="kw">amhar</span> read_err = file.<span class="fn">read</span>(<span class="str">"data.txt"</span>);
    <span class="kw">hlyin</span> (read_err != <span class="kw">bhala</span>) { <span class="kw">pyan</span> <span class="num">1</span>; }
    <span class="kw">naut_sone</span> <span class="fn">pya</span>(<span class="str">"cleanup done"</span>);
    <span class="fn">pya</span>(content);
    <span class="kw">pyan</span> <span class="num">0</span>;
}`,
  },
];

export default function CodeExamplesSlide() {
  const [activeIndex, setActiveIndex] = useState(0);
  const [copyState, setCopyState] = useState<"idle" | "done" | "error">("idle");
  const copyTimerRef = useRef<number | null>(null);
  const activeSnippet = snippets[activeIndex];

  useEffect(() => {
    return () => {
      if (copyTimerRef.current !== null) {
        window.clearTimeout(copyTimerRef.current);
      }
    };
  }, []);

  const copyLabel = useMemo(() => {
    if (copyState === "done") return "Copied";
    if (copyState === "error") return "Copy failed";
    return "Copy";
  }, [copyState]);

  function scheduleCopyReset() {
    if (copyTimerRef.current !== null) {
      window.clearTimeout(copyTimerRef.current);
    }
    copyTimerRef.current = window.setTimeout(() => setCopyState("idle"), 1300);
  }

  async function handleCopy() {
    try {
      await navigator.clipboard.writeText(activeSnippet.rawCode);
      setCopyState("done");
    } catch {
      setCopyState("error");
    } finally {
      scheduleCopyReset();
    }
  }

  function handleSelect(index: number) {
    setActiveIndex(index);
    setCopyState("idle");
  }

  return (
    <div className="space-y-5">
      <SlideHeader number="14" title="Code Examples" />
      <div className="grid grid-cols-1 xl:grid-cols-[270px_minmax(0,1fr)] gap-4 lg:gap-6">
        <div data-slide-nav-lock-x className="flex xl:flex-col gap-2 overflow-x-auto touch-pan-x xl:overflow-visible pb-1 xl:pb-0 pr-1">
          {snippets.map((snippet, index) => {
            const isActive = index === activeIndex;
            return (
              <motion.button
                key={snippet.id}
                onClick={() => handleSelect(index)}
                whileTap={{ scale: 0.98 }}
                className={`shrink-0 xl:shrink w-[220px] sm:w-[250px] xl:w-full text-left rounded-xl border px-4 py-3 transition-all ${
                  isActive
                    ? "bg-accent-blue/10 border-accent-blue/35 shadow-[0_0_30px_rgba(96,165,250,0.12)]"
                    : "bg-bg-card border-white/10 hover:border-accent-blue/25 hover:bg-bg-card-hover"
                }`}
                aria-label={`Show snippet ${index + 1}: ${snippet.title}`}
                aria-pressed={isActive}
              >
                <p className={`text-[0.82rem] font-semibold ${isActive ? "text-accent-blue" : "text-text-primary"}`}>{snippet.title}</p>
                <p className="text-[0.72rem] text-text-muted mt-1">{snippet.tags.join(" · ")}</p>
              </motion.button>
            );
          })}
        </div>

        <div className="min-w-0 rounded-2xl border border-white/10 bg-bg-card/70 overflow-hidden">
          <div className="flex flex-wrap items-center justify-between gap-3 border-b border-white/10 px-4 py-3">
            <div className="min-w-0">
              <p className="text-[0.76rem] font-mono text-accent-green truncate">{activeSnippet.file}</p>
              <p className="text-[0.72rem] text-text-muted">{`${activeIndex + 1} / ${snippets.length}`}</p>
            </div>
            <div className="flex items-center gap-2">
              <button
                onClick={() => handleSelect(Math.max(0, activeIndex - 1))}
                disabled={activeIndex === 0}
                className="h-8 px-3 rounded-lg border border-white/10 bg-bg-secondary text-text-secondary text-[0.72rem] font-medium disabled:opacity-40 disabled:cursor-not-allowed hover:border-accent-blue/30 hover:text-accent-blue transition-colors"
                aria-label="Show previous code example"
              >
                Prev
              </button>
              <button
                onClick={() => handleSelect(Math.min(snippets.length - 1, activeIndex + 1))}
                disabled={activeIndex === snippets.length - 1}
                className="h-8 px-3 rounded-lg border border-white/10 bg-bg-secondary text-text-secondary text-[0.72rem] font-medium disabled:opacity-40 disabled:cursor-not-allowed hover:border-accent-blue/30 hover:text-accent-blue transition-colors"
                aria-label="Show next code example"
              >
                Next
              </button>
              <button
                onClick={handleCopy}
                className={`h-8 px-3 rounded-lg border text-[0.72rem] font-medium transition-colors ${
                  copyState === "done"
                    ? "border-accent-green/40 text-accent-green bg-accent-green/10"
                    : copyState === "error"
                      ? "border-accent-red/40 text-accent-red bg-accent-red/10"
                      : "border-white/10 bg-bg-secondary text-text-secondary hover:border-accent-blue/30 hover:text-accent-blue"
                }`}
                aria-label="Copy active code example"
              >
                {copyLabel}
              </button>
            </div>
          </div>

          <AnimatePresence mode="wait">
            <motion.div
              key={activeSnippet.id}
              initial={{ opacity: 0, y: 8 }}
              animate={{ opacity: 1, y: 0 }}
              exit={{ opacity: 0, y: -8 }}
              transition={{ duration: 0.2 }}
              className="p-3 sm:p-4"
            >
              <CodeBlock className="max-h-[46vh] sm:max-h-[52vh] overflow-auto touch-pan-x p-4 sm:p-5 text-[0.72rem] sm:text-[0.8rem] leading-[1.65]">
                <code dangerouslySetInnerHTML={{ __html: activeSnippet.highlightedCode }} />
              </CodeBlock>
              <p className="mt-3 px-1 text-[0.78rem] text-text-secondary">{activeSnippet.description}</p>
            </motion.div>
          </AnimatePresence>
        </div>
      </div>
    </div>
  );
}
