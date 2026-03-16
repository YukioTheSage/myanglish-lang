"use client";
import SlideHeader from "../ui/SlideHeader";
import CodeBlock from "../ui/CodeBlock";

export default function ReferenceBNFSlide() {
  return (
    <div>
      <SlideHeader number="06" title="Reference Language BNF" />
      <p className="text-text-secondary text-[0.92rem] max-w-[780px] mb-5">
        Representative grammar summary for Go, adapted from the official specification. Go uses EBNF notation, which is also convenient for presenting the
        preliminary grammar of M-Lang.
      </p>

      <CodeBlock size="small" className="mb-4">
        <span className="type">Program</span> ::= <span className="type">PackageDecl</span> <span className="type">ImportDecl</span>* <span className="type">TopLevelDecl</span>*{"\n"}
        <span className="type">PackageDecl</span> ::= <span className="str">&quot;package&quot;</span> Identifier{"\n"}
        <span className="type">ImportDecl</span> ::= <span className="str">&quot;import&quot;</span> StringLit{"\n"}
        <span className="type">TopLevelDecl</span> ::= <span className="type">VarDecl</span> | <span className="type">TypeDecl</span> | <span className="type">FuncDecl</span>{"\n"}
        <span className="type">VarDecl</span> ::= <span className="str">&quot;var&quot;</span> Identifier <span className="type">Type</span> [<span className="str">&quot;=&quot;</span> <span className="type">Expr</span>]{"\n"}
        <span className="type">FuncDecl</span> ::= <span className="str">&quot;func&quot;</span> Identifier <span className="type">Signature</span> <span className="type">Block</span>{"\n"}
        <span className="type">Stmt</span> ::= <span className="type">IfStmt</span> | <span className="type">ForStmt</span> | <span className="type">ReturnStmt</span> | <span className="type">ExprStmt</span>{"\n"}
        <span className="type">ForStmt</span> ::= <span className="str">&quot;for&quot;</span> ( <span className="type">Condition</span> | <span className="type">ForClause</span> | <span className="type">RangeClause</span> ) <span className="type">Block</span>
      </CodeBlock>

      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        <div className="rounded-xl border border-white/10 bg-bg-card p-4">
          <h3 className="text-[0.85rem] font-bold mb-2">What this review extracts from the reference grammar</h3>
          <p className="text-[0.8rem] text-text-secondary leading-relaxed">
            A program structure, declaration forms, block-based statements, and typed function signatures. Those are the parts most directly reused and localized
            in the M-Lang preliminary design.
          </p>
        </div>
        <div className="rounded-xl border border-accent-blue/20 bg-accent-blue/[0.05] p-4">
          <h3 className="text-[0.85rem] font-bold mb-2 text-accent-blue">Official source</h3>
          <p className="text-[0.8rem] text-text-secondary leading-relaxed">
            Go Language Specification: <span className="font-mono text-accent-cyan">go.dev/ref/spec</span>
          </p>
        </div>
      </div>
    </div>
  );
}
