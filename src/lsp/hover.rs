use ropey::Rope;
use tower_lsp::lsp_types::*;

use super::analysis::{AnalysisResult, SymbolKindInfo, format_type, get_word_at_position};

/// Keyword documentation for hover.
fn keyword_docs(word: &str) -> Option<&'static str> {
    match word {
        "kain" => Some(
            "**kain** — Integer type declaration\n\nDeclares a variable of type `int`.\n\n```\nkain age = 20;\n```",
        ),
        "sar" => Some(
            "**sar** — String type declaration\n\nDeclares a variable of type `string`.\n\n```\nsar name = \"Aung\";\n```",
        ),
        "sit" => Some(
            "**sit** — Boolean type declaration\n\nDeclares a variable of type `bool`.\n\n```\nsit flag = hman;\n```",
        ),
        "hman" => Some("**hman** — Boolean literal `true`"),
        "hmar" => Some("**hmar** — Boolean literal `false`"),
        "hlyin" => Some(
            "**hlyin** — If statement\n\nConditional branching.\n\n```\nhlyin (condition) {\n    // ...\n} mo {\n    // ...\n}\n```",
        ),
        "mo" => Some(
            "**mo** — Else clause\n\nUsed after `hlyin` for the else branch.\n\n```\n} mo {\n    // ...\n}\n```",
        ),
        "pat" => Some(
            "**pat** — Loop statement\n\nSupports both while and for-in styles.\n\n```\npat (condition) {\n    // ...\n}\n\npat item htae numbers {\n    // ...\n}\n```",
        ),
        "kyoe" => Some(
            "**kyoe** — Concurrent call\n\nRuns a function or method call concurrently.\n\n```\nkyoe worker(ch);\n```",
        ),
        "naut_sone" => Some(
            "**naut_sone** — Defer cleanup\n\nSchedules a call to run when the current callable returns.\n\n```\nnaut_sone conn.close();\n```",
        ),
        "set_sae" => Some(
            "**set_sae** — Test declaration\n\nDeclares a top-level test case.\n\n```\nset_sae smoke_test {\n    pyan bhala;\n}\n```",
        ),
        "htae" => Some(
            "**htae** — For-in connector\n\nUsed with `pat` to iterate arrays.\n\n```\npat item htae numbers {\n    pya(item);\n}\n```",
        ),
        "loke" => Some(
            "**loke** — Function declaration\n\nDefine a new function.\n\n```\nloke name(params) -> return_type {\n    // ...\n}\n```",
        ),
        "pyan" => Some(
            "**pyan** — Return statement\n\nReturn a value from a function.\n\n```\npyan 0;\n```",
        ),
        "pya" => Some(
            "**pya** — Print statement\n\nPrint a value to stdout.\n\n```\npya(\"Hello\");\npya(variable);\n```",
        ),
        "phat" => Some(
            "**phat** — Read input\n\nRead a string from stdin.\n\n```\nsar input = phat(\"prompt\");\n```",
        ),
        "su" => Some(
            "**su** — Array type\n\nDeclare an array.\n\n```\nsu<kain> numbers = [1, 2, 3];\n```",
        ),
        "laung" => Some(
            "**laung** — Channel type/make\n\nDeclare or construct a channel.\n\n```\nlaung<kain> ch = laung<kain>(10);\n```",
        ),
        "baung" => Some(
            "**baung** — Context type/make\n\nCreate a timeout-bound context value.\n\n```\nbaung ctx = baung(5000);\n```",
        ),
        "yu" => Some("**yu** — Import statement\n\nImport a module.\n\n```\nyu module_name;\n```"),
        "atote" => Some(
            "**atote** — Package declaration\n\nDeclare file package.\n\n```\natote main;\n```",
        ),
        "pay" => Some(
            "**pay** — Export declaration\n\nExport a top-level declaration from a package.\n\n```\npay loke add(kain a, kain b) -> kain { pyan a + b; }\n```",
        ),
        "twe" => Some(
            "**twe** — HashMap type\n\nDeclare a hash map.\n\n```\ntwe<sar, kain> dict = {\"key\": 1};\n```",
        ),
        "da_tha" => Some(
            "**da_tha** — Float type declaration\n\nDeclares a variable of type `float64`.\n\n```\nda_tha pi = 3.14;\n```",
        ),
        "bhala" => Some(
            "**bhala** — Nil / null value\n\nRepresents the absence of a value.\n\n```\nhlyin (x == bhala) { ... }\n```",
        ),
        "pone" => Some(
            "**pone** — Struct declaration\n\nDefine a new struct type.\n\n```\npone Person {\n    sar name;\n    kain age;\n}\n```",
        ),
        "nee" => Some(
            "**nee** — Method declaration\n\nDefine a method on a struct.\n\n```\nnee (Person p) greet() -> sar {\n    pyan p.name;\n}\n```",
        ),
        "myat" => Some(
            "**myat** — Interface declaration\n\nDefine an interface (trait).\n\n```\nmyat Greeter {\n    loke greet() -> sar;\n}\n```",
        ),
        "amhar" => Some(
            "**amhar** — Error type / create error\n\nCreate an error value.\n\n```\namhar err = amhar(\"something went wrong\");\n```",
        ),
        "pyaung_kain" => Some(
            "**pyaung_kain** — Convert to integer\n\nCast a value to `kain` (int).\n\n```\nkain n = pyaung_kain(\"42\");\n```",
        ),
        "pyaung_sar" => Some(
            "**pyaung_sar** — Convert to string\n\nCast a value to `sar` (string).\n\n```\nsar s = pyaung_sar(42);\n```",
        ),
        "pyaung_da_tha" => Some(
            "**pyaung_da_tha** — Convert to float\n\nCast a value to `da_tha` (float64).\n\n```\nda_tha f = pyaung_da_tha(\"3.14\");\n```",
        ),
        "ashay" => Some(
            "**ashay** — Get length\n\nReturns the length of an array or string.\n\n```\nkain len = ashay(my_array);\n```",
        ),
        "khwae" => Some(
            "**khwae** — String split method\n\nSplit a string by delimiter.\n\n```\nsu<sar> parts = text.khwae(\",\");\n```",
        ),
        "swal" => Some(
            "**swal** — String contains method\n\nCheck if a string contains a substring.\n\n```\nsit has = text.swal(\"hello\");\n```",
        ),
        "ayaik" => Some(
            "**ayaik** — String lowercase method\n\nLowercase a string.\n\n```\nsar lower = text.ayaik();\n```",
        ),
        _ => None,
    }
}

pub fn get_hover(rope: &Rope, pos: Position, analysis: Option<&AnalysisResult>) -> Option<Hover> {
    let word = get_word_at_position(rope, pos)?;

    // Check for keyword documentation first
    if let Some(docs) = keyword_docs(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: docs.to_string(),
            }),
            range: None,
        });
    }

    // Check for symbol info
    if let Some(analysis) = analysis {
        for sym in &analysis.symbols {
            if sym.name == word {
                let type_str = sym
                    .ty
                    .as_ref()
                    .map(|t| format_type(t))
                    .unwrap_or_else(|| "unknown".to_string());

                let detail = match sym.kind {
                    SymbolKindInfo::Variable => {
                        format!("**variable** `{}`\n\nType: `{}`", sym.name, type_str)
                    }
                    SymbolKindInfo::Function => {
                        let params: Vec<String> = sym
                            .parameters
                            .iter()
                            .map(|(n, t)| format!("{} {}", format_type(t), n))
                            .collect();
                        format!(
                            "**function** `{}`\n\n```\nloke {}({}) -> {}\n```",
                            sym.name,
                            sym.name,
                            params.join(", "),
                            type_str
                        )
                    }
                    SymbolKindInfo::Parameter => {
                        format!("**parameter** `{}`\n\nType: `{}`", sym.name, type_str)
                    }
                    SymbolKindInfo::Struct => {
                        let fields: Vec<String> = sym
                            .parameters
                            .iter()
                            .map(|(n, t)| format!("  {} {};", format_type(t), n))
                            .collect();
                        format!(
                            "**struct** `{}`\n\n```\npone {} {{\n{}\n}}\n```",
                            sym.name,
                            sym.name,
                            fields.join("\n")
                        )
                    }
                    SymbolKindInfo::Method => {
                        let params: Vec<String> = sym
                            .parameters
                            .iter()
                            .map(|(n, t)| format!("{} {}", format_type(t), n))
                            .collect();
                        format!(
                            "**method** `{}`\n\n```\nnee {}({}) -> {}\n```",
                            sym.name,
                            sym.name,
                            params.join(", "),
                            type_str
                        )
                    }
                    SymbolKindInfo::Interface => {
                        format!(
                            "**interface** `{}`\n\n```\nmyat {} {{ ... }}\n```",
                            sym.name, sym.name
                        )
                    }
                };

                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: detail,
                    }),
                    range: None,
                });
            }
        }
    }

    None
}
