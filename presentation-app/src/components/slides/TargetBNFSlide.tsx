"use client";
import SlideHeader from "../ui/SlideHeader";
import CodeBlock from "../ui/CodeBlock";

export default function TargetBNFSlide() {
  return (
    <div>
      <SlideHeader number="08" title="Preliminary M-Lang BNF" />
      <p className="text-text-secondary text-[0.92rem] max-w-[800px] mb-5">
        Preliminary grammar summary for the new language design. This is intentionally simplified for presentation use; the full implementation details live in
        the compiler parser and AST definitions.
      </p>

      <div className="grid grid-cols-1 xl:grid-cols-2 gap-4">
        <CodeBlock size="small">
          <span className="type">Program</span> ::= <span className="type">ImportDecl</span>* <span className="type">TopLevelDecl</span>*{"\n"}
          <span className="type">ImportDecl</span> ::= <span className="str">&quot;yu&quot;</span> StringLit <span className="str">&quot;;&quot;</span>{"\n"}
          <span className="type">TopLevelDecl</span> ::= <span className="type">PackageDecl</span> | <span className="type">ExportDecl</span> | <span className="type">FunctionDecl</span> | <span className="type">StructDecl</span> | <span className="type">InterfaceDecl</span>{"\n"}
          <span className="type">PackageDecl</span> ::= <span className="str">&quot;atote&quot;</span> Identifier <span className="str">&quot;;&quot;</span>{"\n"}
          <span className="type">ExportDecl</span> ::= <span className="str">&quot;pay&quot;</span> <span className="type">TopLevelDecl</span>{"\n"}
          <span className="type">FunctionDecl</span> ::= <span className="str">&quot;loke&quot;</span> Identifier <span className="str">&quot;(&quot;</span> <span className="type">Params</span>? <span className="str">&quot;)&quot;</span> [ <span className="str">&quot;-&gt;&quot;</span> <span className="type">Type</span> ] <span className="type">Block</span>{"\n"}
          <span className="type">VarDecl</span> ::= <span className="type">Type</span> Identifier [ <span className="str">&quot;=&quot;</span> <span className="type">Expr</span> ] <span className="str">&quot;;&quot;</span>
        </CodeBlock>

        <CodeBlock size="small">
          <span className="type">Stmt</span> ::= <span className="type">VarDecl</span> | <span className="type">Assign</span> | <span className="type">IfStmt</span> | <span className="type">LoopStmt</span> | <span className="type">ReturnStmt</span> | <span className="type">PrintStmt</span> | <span className="type">GoStmt</span> | <span className="type">DeferStmt</span>{"\n"}
          <span className="type">IfStmt</span> ::= <span className="str">&quot;hlyin&quot;</span> <span className="str">&quot;(&quot;</span> <span className="type">Expr</span> <span className="str">&quot;)&quot;</span> <span className="type">Block</span> [ <span className="str">&quot;mo&quot;</span> <span className="type">Block</span> ]{"\n"}
          <span className="type">LoopStmt</span> ::= <span className="str">&quot;pat&quot;</span> <span className="str">&quot;(&quot;</span> <span className="type">Expr</span> <span className="str">&quot;)&quot;</span> <span className="type">Block</span>{"\n"}
          {"              "} | <span className="str">&quot;pat&quot;</span> <span className="str">&quot;(&quot;</span> <span className="type">VarDecl</span> <span className="str">&quot;;&quot;</span> <span className="type">Expr</span> <span className="str">&quot;;&quot;</span> <span className="type">Assign</span> <span className="str">&quot;)&quot;</span> <span className="type">Block</span>{"\n"}
          {"              "} | <span className="str">&quot;pat&quot;</span> Identifier <span className="str">&quot;htae&quot;</span> <span className="type">Expr</span> <span className="type">Block</span>{"\n"}
          <span className="type">GoStmt</span> ::= <span className="str">&quot;kyoe&quot;</span> <span className="type">CallExpr</span> <span className="str">&quot;;&quot;</span>{"\n"}
          <span className="type">DeferStmt</span> ::= <span className="str">&quot;naut_sone&quot;</span> <span className="type">CallExpr</span> <span className="str">&quot;;&quot;</span>{"\n"}
          <span className="type">Type</span> ::= <span className="str">&quot;kain&quot;</span> | <span className="str">&quot;sar&quot;</span> | <span className="str">&quot;sit&quot;</span> | <span className="str">&quot;da_tha&quot;</span> | <span className="str">&quot;amhar&quot;</span> | <span className="str">&quot;su&quot;</span> <span className="str">&quot;&lt;&quot;</span> <span className="type">Type</span> <span className="str">&quot;&gt;&quot;</span> | <span className="str">&quot;twe&quot;</span> <span className="str">&quot;&lt;&quot;</span> <span className="type">Type</span> <span className="str">&quot;,&quot;</span> <span className="type">Type</span> <span className="str">&quot;&gt;&quot;</span> | <span className="str">&quot;laung&quot;</span> <span className="str">&quot;&lt;&quot;</span> <span className="type">Type</span> <span className="str">&quot;&gt;&quot;</span>
        </CodeBlock>
      </div>
    </div>
  );
}
