"use client";
import { motion } from "framer-motion";
import SlideHeader from "../ui/SlideHeader";
import CodeBlock from "../ui/CodeBlock";

const features = [
  "Converts raw .ml source text into a stream of Tokens",
  "Each token has a TokenKind, value, line number, and column",
  "Recognizes Myanmar Unicode range (U+1000–U+109F) as valid identifiers",
  "Recognizes Myanmar digits (U+1040–U+1049) as numeric literals",
  "Maps Myanglish keywords to dedicated TokenKind variants",
];

const tags = ["Keywords", "String Literals", "Number Literals", "Myanmar Numerals", "Operators", "Comments //", "Line & Column Tracking"];

export default function LexerSlide() {
  return (
    <div>
      <SlideHeader number="16" title="Stage 1 — Lexer" badge="lexer.rs · ~12 KB" badgeType="file" />
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6 lg:gap-10">
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">What it Does</h3>
          <ul className="list-none space-y-1.5">
            {features.map((f, i) => (
              <motion.li
                key={i}
                initial={{ opacity: 0, x: -10 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: i * 0.08 }}
                className="text-[0.85rem] text-text-secondary flex items-start gap-2"
              >
                <span className="text-accent-blue font-bold shrink-0">→</span>
                <span dangerouslySetInnerHTML={{ __html: f.replace(/`([^`]+)`/g, '<code class="bg-accent-blue/10 text-accent-blue px-1 rounded font-mono text-[0.8rem]">$1</code>') }} />
              </motion.li>
            ))}
          </ul>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4 mt-6">Handles</h3>
          <div className="flex flex-wrap gap-1.5">
            {tags.map((tag) => (
              <span key={tag} className="text-[0.7rem] font-medium px-2.5 py-1 bg-bg-card border border-white/10 rounded-full text-text-secondary">{tag}</span>
            ))}
          </div>
        </div>
        <div>
          <h3 className="text-[0.7rem] font-bold uppercase tracking-[0.15em] text-accent-blue mb-4">Example Transformation</h3>
          <CodeBlock size="small">
            <span className="cm">{"// Input"}</span>{"\n"}
            <span className="kw">kain</span> age = <span className="num">၂0</span>;
          </CodeBlock>
          <div className="text-center text-accent-blue font-bold py-2 text-[0.85rem]">⬇ Lexer ⬇</div>
          <CodeBlock size="small">
            <span className="text-[0.7rem] leading-[1.8]">
{`Token { kind: `}<span className="type">Kain</span>{`,    line: 1, col: 1 }
Token { kind: `}<span className="type">Ident</span>{`,   value: "age" }
Token { kind: `}<span className="type">Assign</span>{`,  value: "=" }
Token { kind: `}<span className="type">Number</span>{`,  value: "20" }
Token { kind: `}<span className="type">Semi</span>{`,    value: ";" }`}
            </span>
          </CodeBlock>
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            transition={{ delay: 0.5 }}
            className="flex items-start gap-2 p-3 bg-accent-yellow/5 border border-accent-yellow/15 rounded-lg mt-4 text-[0.8rem] text-text-secondary"
          >
            <span>💡</span>
            <span>Myanmar digit <code className="bg-accent-blue/10 text-accent-blue px-1 rounded font-mono text-[0.75rem]">၂</code> (U+1042) is converted to ASCII <code className="bg-accent-blue/10 text-accent-blue px-1 rounded font-mono text-[0.75rem]">2</code> during lexing</span>
          </motion.div>
        </div>
      </div>
    </div>
  );
}
